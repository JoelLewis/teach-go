use tauri::State;

use crate::error::AppError;
use crate::state::AppState;

#[tauri::command]
pub fn start_engine(_state: State<'_, AppState>) -> Result<String, AppError> {
    // Stub — will be implemented in Step 6
    Ok("engine_not_configured".into())
}

#[tauri::command]
pub fn stop_engine(_state: State<'_, AppState>) -> Result<(), AppError> {
    // Stub — will be implemented in Step 6
    Ok(())
}
