mod commands;
mod convert;
mod db;
mod error;
mod state;

use state::AppState;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&data_dir)?;
            let db_path = data_dir.join("gosensei.db");
            let conn = db::init_db(&db_path.to_string_lossy())?;
            app.manage(AppState::with_db(conn));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::game::new_game,
            commands::game::play_move,
            commands::game::pass_turn,
            commands::game::resign,
            commands::game::undo_move,
            commands::game::list_games,
            commands::game::load_saved_game,
            commands::katago::start_engine,
            commands::katago::stop_engine,
            commands::katago::request_ai_move,
            commands::coaching::get_coaching_feedback,
            commands::settings::get_settings,
            commands::settings::update_settings,
            commands::sgf::save_game_sgf,
            commands::sgf::load_game_sgf,
        ])
        .run(tauri::generate_context!())
        .expect("error while running GoSensei");
}
