use gosensei_core::game::{Game, GameState};
use gosensei_core::types::{BoardSize, Color, GameResult, Point};
use tauri::State;

use crate::error::AppError;
use crate::state::AppState;

#[tauri::command]
pub fn new_game(
    state: State<'_, AppState>,
    board_size: u8,
    komi: Option<f32>,
    player_color: Option<String>,
) -> Result<GameState, AppError> {
    let size = BoardSize::try_from(board_size).map_err(|e| AppError::Other(e))?;
    let game = Game::new(size, komi.unwrap_or(6.5));
    let game_state = game.to_state();
    *state.game.lock().unwrap() = Some(game);

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
    Ok(game.to_state())
}

#[tauri::command]
pub fn pass_turn(state: State<'_, AppState>) -> Result<GameState, AppError> {
    let mut game_lock = state.game.lock().unwrap();
    let game = game_lock.as_mut().ok_or(AppError::Other("No active game".into()))?;
    game.pass()?;
    Ok(game.to_state())
}

#[tauri::command]
pub fn resign(state: State<'_, AppState>) -> Result<(GameState, GameResult), AppError> {
    let mut game_lock = state.game.lock().unwrap();
    let game = game_lock.as_mut().ok_or(AppError::Other("No active game".into()))?;
    let result = game.resign()?;
    Ok((game.to_state(), result))
}

#[tauri::command]
pub fn undo_move(state: State<'_, AppState>) -> Result<GameState, AppError> {
    let mut game_lock = state.game.lock().unwrap();
    let game = game_lock.as_mut().ok_or(AppError::Other("No active game".into()))?;
    game.undo()?;
    Ok(game.to_state())
}
