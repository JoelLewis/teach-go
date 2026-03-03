use tauri::State;

use crate::error::AppError;
use crate::skill::{SkillProfile, SkillSnapshot};
use crate::state::AppState;

#[tauri::command]
pub fn get_skill_profile(state: State<'_, AppState>) -> Result<SkillProfile, AppError> {
    let db = state.db.lock().unwrap();
    crate::skill::get_skill_profile(&db)
}

#[tauri::command]
pub fn get_skill_history(
    state: State<'_, AppState>,
    window_days: Option<u32>,
) -> Result<Vec<SkillSnapshot>, AppError> {
    let db = state.db.lock().unwrap();
    crate::skill::get_skill_history(&db, window_days, 500)
}
