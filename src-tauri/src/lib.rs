mod commands;
mod db;
mod error;
mod state;

use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            commands::game::new_game,
            commands::game::play_move,
            commands::game::pass_turn,
            commands::game::resign,
            commands::game::undo_move,
            commands::katago::start_engine,
            commands::katago::stop_engine,
            commands::coaching::get_coaching_feedback,
            commands::settings::get_settings,
            commands::settings::update_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running GoSensei");
}
