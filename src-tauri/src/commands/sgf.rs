use gosensei_core::game::{Game, GameState};
use serde::Serialize;
use specta::Type;
use tauri::State;

use crate::error::AppError;
use crate::state::AppState;

#[derive(Debug, Clone, Serialize, Type)]
pub struct SgfLoadWarning {
    pub loaded_moves: u16,
    pub total_moves: u16,
    pub move_number: u16,
    pub coordinate: String,
    pub reason: String,
    pub dropped_moves: u16,
}

#[derive(Debug, Clone, Serialize, Type)]
pub struct SgfLoadResult {
    pub game_state: GameState,
    pub warning: Option<SgfLoadWarning>,
}

#[tauri::command]
#[specta::specta]
pub async fn save_game_sgf(
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<Option<String>, AppError> {
    let sgf = {
        let game_lock = state.game.lock().unwrap();
        let game = game_lock
            .as_ref()
            .ok_or(AppError::Other("No active game".into()))?;
        game.to_sgf()
    };

    use tauri_plugin_dialog::DialogExt;
    let path = app
        .dialog()
        .file()
        .add_filter("SGF Files", &["sgf"])
        .set_file_name("game.sgf")
        .blocking_save_file();

    if let Some(file_path) = path {
        let path_buf = file_path
            .as_path()
            .ok_or(AppError::Other("Invalid file path".into()))?
            .to_path_buf();
        std::fs::write(&path_buf, &sgf)?;
        Ok(Some(path_buf.to_string_lossy().into_owned()))
    } else {
        Ok(None) // User cancelled
    }
}

#[tauri::command]
#[specta::specta]
pub async fn load_game_sgf(
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<Option<SgfLoadResult>, AppError> {
    use tauri_plugin_dialog::DialogExt;
    let path = app
        .dialog()
        .file()
        .add_filter("SGF Files", &["sgf"])
        .blocking_pick_file();

    if let Some(file_path) = path {
        let path_buf = file_path
            .as_path()
            .ok_or(AppError::Other("Invalid file path".into()))?
            .to_path_buf();
        let sgf_content = std::fs::read_to_string(&path_buf)?;
        let replay = Game::from_sgf_with_report(&sgf_content).map_err(AppError::Other)?;
        let game_state = replay.game.to_state();
        let total_moves = replay.game.move_history().len() as u16 + replay.dropped_moves;

        if let Some(error) = replay.errors.first() {
            let loaded_moves = game_state.move_number;
            *state.pending_sgf_game.lock().unwrap() = Some(replay.game);
            return Ok(Some(SgfLoadResult {
                game_state,
                warning: Some(SgfLoadWarning {
                    loaded_moves,
                    total_moves,
                    move_number: error.move_number,
                    coordinate: error.coordinate.clone(),
                    reason: error.reason.clone(),
                    dropped_moves: replay.dropped_moves,
                }),
            }));
        }

        *state.game.lock().unwrap() = Some(replay.game);
        // Loaded games have no AI opponent — clear any stale AI color.
        *state.ai_color.lock().unwrap() = None;
        Ok(Some(SgfLoadResult {
            game_state,
            warning: None,
        }))
    } else {
        Ok(None) // User cancelled
    }
}

#[tauri::command]
#[specta::specta]
pub fn confirm_load_game_sgf(state: State<'_, AppState>) -> Result<GameState, AppError> {
    let game = state
        .pending_sgf_game
        .lock()
        .unwrap()
        .take()
        .ok_or(AppError::Other("No pending SGF load".into()))?;
    let game_state = game.to_state();
    *state.game.lock().unwrap() = Some(game);
    *state.ai_color.lock().unwrap() = None;
    Ok(game_state)
}

#[tauri::command]
#[specta::specta]
pub fn cancel_load_game_sgf(state: State<'_, AppState>) -> Result<(), AppError> {
    *state.pending_sgf_game.lock().unwrap() = None;
    Ok(())
}
