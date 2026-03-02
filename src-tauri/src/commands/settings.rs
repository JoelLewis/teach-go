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
    pub ai_strength: String,
    pub sound_enabled: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            board_size: 9,
            komi: 6.5,
            show_coordinates: true,
            show_move_numbers: false,
            ai_strength: "beginner".to_string(),
            sound_enabled: true,
        }
    }
}

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> Result<Settings, AppError> {
    let db = state.db.lock().unwrap();
    let mut settings = Settings::default();

    let mut stmt = db.prepare("SELECT key, value FROM settings")?;
    let rows = stmt.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    })?;

    for row in rows {
        let (key, value) = row?;
        match key.as_str() {
            "board_size" => {
                if let Ok(v) = value.parse() {
                    settings.board_size = v;
                }
            }
            "komi" => {
                if let Ok(v) = value.parse() {
                    settings.komi = v;
                }
            }
            "show_coordinates" => settings.show_coordinates = value == "true",
            "show_move_numbers" => settings.show_move_numbers = value == "true",
            "ai_strength" => settings.ai_strength = value,
            "sound_enabled" => settings.sound_enabled = value == "true",
            _ => {}
        }
    }

    Ok(settings)
}

#[tauri::command]
pub fn update_settings(
    state: State<'_, AppState>,
    settings: Settings,
) -> Result<Settings, AppError> {
    let db = state.db.lock().unwrap();

    let pairs: Vec<(&str, String)> = vec![
        ("board_size", settings.board_size.to_string()),
        ("komi", settings.komi.to_string()),
        ("show_coordinates", settings.show_coordinates.to_string()),
        ("show_move_numbers", settings.show_move_numbers.to_string()),
        ("ai_strength", settings.ai_strength.clone()),
        ("sound_enabled", settings.sound_enabled.to_string()),
    ];

    for (key, value) in &pairs {
        db.execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = ?2",
            rusqlite::params![key, value],
        )?;
    }

    Ok(settings)
}
