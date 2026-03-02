use tauri::State;

use crate::error::AppError;
use crate::skill::SkillProfile;
use crate::state::AppState;

#[tauri::command]
pub fn get_skill_profile(state: State<'_, AppState>) -> Result<SkillProfile, AppError> {
    let db = state.db.lock().unwrap();
    crate::skill::get_skill_profile(&db)
}
