use gosensei_core::game::{Game, GamePhase, GameState};
use gosensei_core::types::{BoardSize, Color, GameResult, Point};
use serde::{Deserialize, Serialize};
use tauri::State;

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

    let db = state.db.lock().unwrap();
    let _ = db.execute(
        "INSERT INTO games (board_size, sgf, result) VALUES (?1, ?2, ?3)",
        rusqlite::params![board_size, sgf, result_str],
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
    let size = BoardSize::try_from(board_size).map_err(AppError::Other)?;
    let game = Game::new(size, komi.unwrap_or(6.5));
    let game_state = game.to_state();
    *state.game.lock().unwrap() = Some(game);
    state.game_errors.lock().unwrap().clear();

    let ai_color = player_color.and_then(|c| match c.as_str() {
        "black" => Some(Color::White),
        "white" => Some(Color::Black),
        _ => None,
    });
    *state.ai_color.lock().unwrap() = ai_color;

    Ok(game_state)
}

#[tauri::command]
pub fn play_move(
    state: State<'_, AppState>,
    row: u8,
    col: u8,
) -> Result<GameState, AppError> {
    let mut game_lock = state.game.lock().unwrap();
    let game = game_lock.as_mut().ok_or(AppError::Other("No active game".into()))?;
    game.play(Point::new(row, col))?;
    let game_state = game.to_state();
    auto_save_if_finished(&state, game);
    Ok(game_state)
}

#[tauri::command]
pub fn pass_turn(state: State<'_, AppState>) -> Result<GameState, AppError> {
    let mut game_lock = state.game.lock().unwrap();
    let game = game_lock.as_mut().ok_or(AppError::Other("No active game".into()))?;
    game.pass()?;
    let game_state = game.to_state();
    auto_save_if_finished(&state, game);
    Ok(game_state)
}

#[tauri::command]
pub fn resign(state: State<'_, AppState>) -> Result<(GameState, GameResult), AppError> {
    let mut game_lock = state.game.lock().unwrap();
    let game = game_lock.as_mut().ok_or(AppError::Other("No active game".into()))?;
    let result = game.resign()?;
    let game_state = game.to_state();
    auto_save_if_finished(&state, game);
    Ok((game_state, result))
}

#[tauri::command]
pub fn undo_move(state: State<'_, AppState>) -> Result<GameState, AppError> {
    let mut game_lock = state.game.lock().unwrap();
    let game = game_lock.as_mut().ok_or(AppError::Other("No active game".into()))?;
    game.undo()?;
    Ok(game.to_state())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
pub fn load_saved_game(
    state: State<'_, AppState>,
    game_id: i64,
) -> Result<GameState, AppError> {
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
