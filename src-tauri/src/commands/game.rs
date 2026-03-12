use gosensei_core::game::{Game, GamePhase, GameState};
use gosensei_core::types::{BoardSize, Color, GameResult, Point};
use serde::{Deserialize, Serialize};
use tauri::State;
use tracing::info;

use crate::error::AppError;
use crate::state::AppState;

fn auto_save_if_finished(state: &AppState, game: &Game) {
    if *game.phase() != GamePhase::Finished {
        return;
    }
    let sgf = game.to_sgf();
    let result_str = match game.result() {
        Some(GameResult::Score { winner, margin }) => {
            let w = match winner {
                Color::Black => "B",
                Color::White => "W",
            };
            format!("{w}+{margin}")
        }
        Some(GameResult::Resignation { winner }) => {
            let w = match winner {
                Color::Black => "B",
                Color::White => "W",
            };
            format!("{w}+R")
        }
        Some(GameResult::Draw) => "0".to_string(),
        None => "?".to_string(),
    };
    let board_size = game.board().dimension();
    let player_color = match *state.ai_color.lock().unwrap() {
        Some(Color::Black) => "white",
        _ => "black", // Default: player is Black
    };

    let db = state.db.lock().unwrap();
    let _ = db.execute(
        "INSERT INTO games (board_size, sgf, result, player_color) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![board_size, sgf, result_str, player_color],
    );

    // Update skill profile from accumulated coaching errors
    let errors: Vec<_> = std::mem::take(&mut *state.game_errors.lock().unwrap());
    let _ = crate::skill::update_skill_after_game(&db, &errors);
}

#[tauri::command]
pub fn new_game(
    state: State<'_, AppState>,
    board_size: u8,
    komi: Option<f32>,
    player_color: Option<String>,
) -> Result<GameState, AppError> {
    tracing::info!("new_game: board_size={board_size}, komi={komi:?}, player_color={player_color:?}");
    let size = BoardSize::try_from(board_size).map_err(AppError::Other)?;
    let game = Game::new(size, komi.unwrap_or(6.5));
    let game_state = game.to_state();
    *state.game.lock().unwrap() = Some(game);
    state.game_errors.lock().unwrap().clear();

    // Clear coaching session context for the new game
    let db = state.db.lock().unwrap();
    let _ = crate::coaching_db::clear_session(&db);
    drop(db);

    let ai_color = player_color.and_then(|c| match c.as_str() {
        "black" => Some(Color::White),
        "white" => Some(Color::Black),
        _ => None,
    });
    *state.ai_color.lock().unwrap() = ai_color;

    Ok(game_state)
}

#[tauri::command]
pub fn play_move(state: State<'_, AppState>, row: u8, col: u8) -> Result<GameState, AppError> {
    tracing::info!("play_move: row={row}, col={col}");
    let mut game_lock = state.game.lock().unwrap();
    let game = game_lock
        .as_mut()
        .ok_or(AppError::Other("No active game".into()))?;
    game.play(Point::new(row, col))?;
    let game_state = game.to_state();
    auto_save_if_finished(&state, game);
    Ok(game_state)
}

#[tauri::command]
pub fn pass_turn(state: State<'_, AppState>) -> Result<GameState, AppError> {
    let mut game_lock = state.game.lock().unwrap();
    let game = game_lock
        .as_mut()
        .ok_or(AppError::Other("No active game".into()))?;
    game.pass()?;
    let game_state = game.to_state();
    auto_save_if_finished(&state, game);
    Ok(game_state)
}

#[tauri::command]
pub fn resign(state: State<'_, AppState>) -> Result<(GameState, GameResult), AppError> {
    let mut game_lock = state.game.lock().unwrap();
    let game = game_lock
        .as_mut()
        .ok_or(AppError::Other("No active game".into()))?;
    let result = game.resign()?;
    let game_state = game.to_state();
    auto_save_if_finished(&state, game);
    Ok((game_state, result))
}

#[tauri::command]
pub fn undo_move(state: State<'_, AppState>) -> Result<GameState, AppError> {
    let mut game_lock = state.game.lock().unwrap();
    let game = game_lock
        .as_mut()
        .ok_or(AppError::Other("No active game".into()))?;
    game.undo()?;
    Ok(game.to_state())
}

#[tauri::command]
pub fn get_game_position(
    state: State<'_, AppState>,
    move_number: u16,
) -> Result<GameState, AppError> {
    let game_lock = state.game.lock().unwrap();
    let game = game_lock
        .as_ref()
        .ok_or(AppError::Other("No active game".into()))?;

    game.state_at_move(move_number)
        .ok_or_else(|| AppError::Other(format!("Move {move_number} out of range")))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SavedGame {
    pub id: i64,
    pub board_size: u8,
    pub result: String,
    pub played_at: String,
}

#[tauri::command]
pub fn list_games(state: State<'_, AppState>) -> Result<Vec<SavedGame>, AppError> {
    let db = state.db.lock().unwrap();
    let mut stmt = db.prepare(
        "SELECT id, board_size, result, played_at FROM games ORDER BY played_at DESC LIMIT 50",
    )?;
    let games = stmt
        .query_map([], |row| {
            Ok(SavedGame {
                id: row.get(0)?,
                board_size: row.get(1)?,
                result: row.get(2)?,
                played_at: row.get(3)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(games)
}

#[tauri::command]
pub fn load_saved_game(state: State<'_, AppState>, game_id: i64) -> Result<GameState, AppError> {
    let sgf: String = {
        let db = state.db.lock().unwrap();
        db.query_row("SELECT sgf FROM games WHERE id = ?1", [game_id], |row| {
            row.get(0)
        })?
    };
    let game = Game::from_sgf(&sgf).map_err(AppError::Other)?;
    let game_state = game.to_state();
    *state.game.lock().unwrap() = Some(game);
    Ok(game_state)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct DifficultySuggestion {
    pub direction: String, // "up" or "down"
    pub message: String,
}

const STREAK_THRESHOLD: usize = 3;
const RECENT_GAMES_QUERY: usize = 10;

/// Pure function: detect win/loss streaks from recent game results.
/// Each entry is (result_string, player_color) — e.g., ("B+R", "black").
fn detect_streak(results: &[(String, String)]) -> Option<DifficultySuggestion> {
    if results.len() < STREAK_THRESHOLD {
        return None;
    }

    let recent = &results[..STREAK_THRESHOLD.min(results.len())];
    let wins = recent
        .iter()
        .filter(|&(result, color)| {
            let winner_prefix = if color == "black" { "B+" } else { "W+" };
            result.starts_with(winner_prefix)
        })
        .count();
    let losses = recent
        .iter()
        .filter(|&(result, color)| {
            let loser_prefix = if color == "black" { "W+" } else { "B+" };
            result.starts_with(loser_prefix)
        })
        .count();

    if wins >= STREAK_THRESHOLD {
        Some(DifficultySuggestion {
            direction: "up".to_string(),
            message: "You're playing well! Want to try a tougher opponent?".to_string(),
        })
    } else if losses >= STREAK_THRESHOLD {
        Some(DifficultySuggestion {
            direction: "down".to_string(),
            message: "Let's shore up the fundamentals with an easier opponent.".to_string(),
        })
    } else {
        None
    }
}

#[tauri::command]
pub fn check_difficulty_suggestion(
    state: State<'_, AppState>,
) -> Result<Option<DifficultySuggestion>, AppError> {
    let db = state.db.lock().unwrap();
    let mut stmt =
        db.prepare("SELECT result, player_color FROM games ORDER BY played_at DESC LIMIT ?1")?;
    let results: Vec<(String, String)> = stmt
        .query_map([RECENT_GAMES_QUERY as i64], |row| {
            Ok((
                row.get(0)?,
                row.get::<_, String>(1)
                    .unwrap_or_else(|_| "black".to_string()),
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;

    let suggestion = detect_streak(&results);
    if let Some(ref s) = suggestion {
        info!("Difficulty suggestion: {}", s.direction);
    }

    Ok(suggestion)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn streak_3_wins_as_black_suggests_up() {
        let results: Vec<(String, String)> = vec![
            ("B+R".into(), "black".into()),
            ("B+5.5".into(), "black".into()),
            ("B+3.0".into(), "black".into()),
        ];
        let suggestion = detect_streak(&results);
        assert!(suggestion.is_some());
        assert_eq!(suggestion.unwrap().direction, "up");
    }

    #[test]
    fn streak_3_wins_as_white_suggests_up() {
        let results: Vec<(String, String)> = vec![
            ("W+R".into(), "white".into()),
            ("W+10.5".into(), "white".into()),
            ("W+2.0".into(), "white".into()),
        ];
        let suggestion = detect_streak(&results);
        assert!(suggestion.is_some());
        assert_eq!(suggestion.unwrap().direction, "up");
    }

    #[test]
    fn streak_3_losses_as_black_suggests_down() {
        let results: Vec<(String, String)> = vec![
            ("W+R".into(), "black".into()),
            ("W+10.5".into(), "black".into()),
            ("W+2.0".into(), "black".into()),
        ];
        let suggestion = detect_streak(&results);
        assert!(suggestion.is_some());
        assert_eq!(suggestion.unwrap().direction, "down");
    }

    #[test]
    fn mixed_results_no_suggestion() {
        let results: Vec<(String, String)> = vec![
            ("B+R".into(), "black".into()),
            ("W+5.5".into(), "black".into()),
            ("B+3.0".into(), "black".into()),
        ];
        assert!(detect_streak(&results).is_none());
    }

    #[test]
    fn too_few_games_no_suggestion() {
        let results: Vec<(String, String)> = vec![
            ("B+R".into(), "black".into()),
            ("B+5.5".into(), "black".into()),
        ];
        assert!(detect_streak(&results).is_none());
    }

    #[test]
    fn empty_results_no_suggestion() {
        let results: Vec<(String, String)> = vec![];
        assert!(detect_streak(&results).is_none());
    }
}
