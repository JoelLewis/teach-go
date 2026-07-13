mod coaching_db;
mod commands;
mod convert;
mod db;
mod download_manager;
mod error;
mod generate;
mod import;
#[cfg(feature = "llm")]
pub mod llm_support;
mod problem;
mod review;
mod seed_content;
mod setup;
mod skill;
mod solver;
mod srs;
mod state;

use state::AppState;
use tauri::Manager;

/// Collects every registered command with its Specta types.
///
/// Single source of truth for both the invoke handler and the generated
/// TypeScript bindings (`src/lib/api/bindings.ts`).
fn specta_builder() -> tauri_specta::Builder<tauri::Wry> {
    tauri_specta::Builder::<tauri::Wry>::new()
        .error_handling(tauri_specta::ErrorHandlingMode::Result)
        // i64/u64/usize cross the IPC boundary as JSON numbers, so export them
        // as `number` (matching the previous hand-written types).
        .dangerously_cast_bigints_to_number()
        // Export f32/f64 as plain `number` instead of `number | null`
        // (the null only occurs for NaN/Infinity, which these APIs never produce).
        .semantic_types(
            specta_typescript::semantic::Configuration::default().enable_lossless_floats(),
        )
        .commands(tauri_specta::collect_commands![
            commands::game::new_game,
            commands::game::play_move,
            commands::game::pass_turn,
            commands::game::resign,
            commands::game::undo_move,
            commands::game::list_games,
            commands::game::load_saved_game,
            commands::game::get_game_position,
            commands::game::check_difficulty_suggestion,
            commands::game::get_ai_engine,
            commands::katago::start_engine,
            commands::katago::stop_engine,
            commands::katago::request_ai_move,
            commands::coaching::get_coaching_feedback,
            commands::settings::get_settings,
            commands::settings::update_settings,
            commands::sgf::save_game_sgf,
            commands::sgf::load_game_sgf,
            commands::sgf::confirm_load_game_sgf,
            commands::sgf::cancel_load_game_sgf,
            commands::review::start_review,
            commands::review::get_review_progress,
            commands::review::get_review_data,
            commands::review::get_review_position,
            commands::review::get_ownership_at,
            commands::review::get_review_variations,
            commands::skill::get_skill_profile,
            commands::skill::get_skill_history,
            commands::problem::list_problems,
            commands::problem::start_problem,
            commands::problem::solve_move,
            commands::problem::get_hint,
            commands::problem::skip_problem,
            commands::problem::get_problem_state,
            commands::problem::get_recommended_problem,
            commands::problem::get_problem_stats,
            commands::problem::generate_problems_from_game,
            commands::problem::import_problems_from_sgf,
            commands::llm::init_llm_model,
            commands::llm::get_llm_status,
            download_manager::get_download_status,
            download_manager::retry_downloads,
        ])
}

#[cfg(any(debug_assertions, test))]
fn export_typescript_bindings(builder: &tauri_specta::Builder<tauri::Wry>) {
    builder
        .export(
            specta_typescript::Typescript::default(),
            "../src/lib/api/bindings.ts",
        )
        .expect("failed to export typescript bindings");
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let specta_builder = specta_builder();

    #[cfg(debug_assertions)]
    export_typescript_bindings(&specta_builder);

    #[allow(unused_mut)]
    let mut builder = tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init());

    #[cfg(feature = "mcp")]
    {
        builder = builder.plugin(tauri_plugin_mcp::init_with_config(
            tauri_plugin_mcp::PluginConfig::new("GoSensei".to_string())
                .start_socket_server(true)
                .socket_path("/tmp/gosensei-mcp.sock".into()),
        ));
    }

    builder
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&data_dir)?;
            let db_path = data_dir.join("gosensei.db");
            let conn = db::init_db(&db_path.to_string_lossy())?;
            problem::seed_problems_if_empty(&conn)?;
            app.manage(AppState::with_db(conn));

            // Spawn background downloads for KataGo + LLM
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                download_manager::run_initial_downloads(handle).await;
            });

            Ok(())
        })
        .invoke_handler(specta_builder.invoke_handler())
        .run(tauri::generate_context!())
        .expect("error while running GoSensei");
}

#[cfg(test)]
mod tests {
    /// Regenerates `src/lib/api/bindings.ts`. Run via `cargo test export_bindings`.
    #[test]
    fn export_bindings() {
        super::export_typescript_bindings(&super::specta_builder());
    }
}
