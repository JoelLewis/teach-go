use tauri::State;

use crate::error::AppError;
use crate::state::AppState;

#[tauri::command]
pub fn get_coaching_feedback(
    _state: State<'_, AppState>,
    _move_number: u16,
) -> Result<Option<String>, AppError> {
    // Stub — will be implemented in Step 7
    Ok(None)
}
