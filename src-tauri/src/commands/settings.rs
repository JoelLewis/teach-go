use serde::{Deserialize, Serialize};
use tauri::State;

use crate::error::AppError;
use crate::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub board_size: u8,
    pub komi: f32,
    pub show_coordinates: bool,
    pub show_move_numbers: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            board_size: 9,
            komi: 6.5,
            show_coordinates: true,
            show_move_numbers: false,
        }
    }
}

#[tauri::command]
pub fn get_settings(_state: State<'_, AppState>) -> Result<Settings, AppError> {
    // Stub — returns defaults, will read from DB in Step 8
    Ok(Settings::default())
}

#[tauri::command]
pub fn update_settings(
    _state: State<'_, AppState>,
    _settings: Settings,
) -> Result<Settings, AppError> {
    // Stub — will persist to DB in Step 8
    Ok(Settings::default())
}
