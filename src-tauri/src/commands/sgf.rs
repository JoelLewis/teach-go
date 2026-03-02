use gosensei_core::game::{Game, GameState};
use tauri::State;

use crate::error::AppError;
use crate::state::AppState;

#[tauri::command]
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
pub async fn load_game_sgf(
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<Option<GameState>, AppError> {
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
        let game = Game::from_sgf(&sgf_content).map_err(AppError::Other)?;
        let game_state = game.to_state();
        *state.game.lock().unwrap() = Some(game);
        Ok(Some(game_state))
    } else {
        Ok(None) // User cancelled
    }
}
