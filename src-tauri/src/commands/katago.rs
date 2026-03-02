use std::path::PathBuf;

use gosensei_core::game::GameState;
use gosensei_katago::client::KataGoClient;
use gosensei_katago::process::KataGoProcess;
use tauri::{AppHandle, Emitter, State};
use tracing::{info, warn};

use crate::convert;
use crate::error::AppError;
use crate::state::AppState;

const MAX_VISITS: u32 = 200;

fn resolve_binary_path() -> Result<PathBuf, AppError> {
    if let Ok(path) = std::env::var("KATAGO_BINARY") {
        let p = PathBuf::from(path);
        if p.exists() {
            return Ok(p);
        }
    }

    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let dev_path = PathBuf::from(manifest_dir).join("binaries").join("katago");
        if dev_path.exists() {
            return Ok(dev_path);
        }
    }

    // Check PATH via `which`
    if let Ok(output) = std::process::Command::new("which").arg("katago").output()
        && output.status.success()
    {
        let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !path.is_empty() {
            return Ok(PathBuf::from(path));
        }
    }

    Err(AppError::KataGo(
        "KataGo binary not found. Set KATAGO_BINARY env var or place binary in src-tauri/binaries/"
            .into(),
    ))
}

fn resolve_config_path() -> Option<PathBuf> {
    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let config_path = PathBuf::from(manifest_dir).join("binaries").join("analysis.cfg");
        if config_path.exists() {
            return Some(config_path);
        }
    }
    None
}

fn resolve_model_path() -> Result<PathBuf, AppError> {
    if let Ok(path) = std::env::var("KATAGO_MODEL") {
        let p = PathBuf::from(path);
        if p.exists() {
            return Ok(p);
        }
    }

    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let binaries_dir = PathBuf::from(manifest_dir).join("binaries");
        // Look for any .bin.gz model file
        if let Ok(entries) = std::fs::read_dir(&binaries_dir) {
            for entry in entries.flatten() {
                let name = entry.file_name();
                let name_str = name.to_string_lossy();
                if name_str.ends_with(".bin.gz") || name_str.ends_with(".gz") {
                    return Ok(entry.path());
                }
            }
        }
    }

    Err(AppError::KataGo(
        "KataGo model not found. Set KATAGO_MODEL env var or place model in src-tauri/binaries/"
            .into(),
    ))
}

#[tauri::command]
pub async fn start_engine(
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<String, AppError> {
    let mut katago = state.katago.lock().await;

    if katago.is_some() {
        return Ok("ready".into());
    }

    let _ = app.emit("engine-status", "starting");
    info!("Starting KataGo engine...");

    let binary_path = resolve_binary_path()?;
    let model_path = resolve_model_path()?;
    let config_path = resolve_config_path();

    info!("KataGo binary: {}", binary_path.display());
    info!("KataGo model: {}", model_path.display());

    match KataGoProcess::spawn(binary_path, model_path, config_path).await {
        Ok(process) => {
            let client = KataGoClient::new(process);
            *katago = Some(client);
            let _ = app.emit("engine-status", "ready");
            info!("KataGo engine ready");
            Ok("ready".into())
        }
        Err(e) => {
            let msg = format!("Failed to start KataGo: {e}");
            warn!("{msg}");
            let _ = app.emit("engine-status", "error");
            Err(AppError::KataGo(msg))
        }
    }
}

#[tauri::command]
pub async fn stop_engine(
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<(), AppError> {
    let mut katago = state.katago.lock().await;
    *katago = None;
    let _ = app.emit("engine-status", "stopped");
    info!("KataGo engine stopped");
    Ok(())
}

#[tauri::command]
pub async fn request_ai_move(
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<GameState, AppError> {
    let _ = app.emit("ai-thinking", true);

    // Ensure engine is started (lazy init) and healthy
    {
        let mut katago = state.katago.lock().await;
        if katago.is_none() {
            *katago = None;
            drop(katago);
            start_engine(state.clone(), app.clone()).await?;
        }
    }

    // Extract game data (brief std mutex lock)
    let (history, board_size, komi) = {
        let game_lock = state.game.lock().unwrap();
        let game = game_lock
            .as_ref()
            .ok_or(AppError::Other("No active game".into()))?;
        (
            game.move_history().to_vec(),
            game.board().dimension(),
            game.komi(),
        )
    };

    // Read AI strength from settings
    let profile = {
        let db = state.db.lock().unwrap();
        let strength: String = db
            .query_row(
                "SELECT value FROM settings WHERE key = 'ai_strength'",
                [],
                |row| row.get(0),
            )
            .unwrap_or_else(|_| "beginner".to_string());
        convert::strength_to_profile(&strength)
    };

    // Build and send query (async — no std mutex held)
    let query_id = format!("ai-move-{}", history.len());
    let query = convert::build_query(query_id, &history, board_size, komi, MAX_VISITS, profile);

    let response = {
        let katago = state.katago.lock().await;
        let client = katago
            .as_ref()
            .ok_or(AppError::KataGo("Engine not available".into()))?;
        client.query(query).await?
    };

    // Parse best move
    let best = response
        .move_infos
        .first()
        .ok_or(AppError::KataGo("No move returned by KataGo".into()))?;

    info!(
        "KataGo suggests: {} (winrate: {:.1}%, visits: {})",
        best.mv,
        best.winrate * 100.0,
        best.visits
    );

    // Apply the AI's move to game state
    let game_state = {
        let mut game_lock = state.game.lock().unwrap();
        let game = game_lock
            .as_mut()
            .ok_or(AppError::Other("No active game".into()))?;

        if let Some(point) = convert::gtp_to_point(&best.mv, board_size) {
            game.play(point)?;
        } else {
            // "pass" or unrecognized — treat as pass
            game.pass()?;
        }

        game.to_state()
    };

    let _ = app.emit("ai-thinking", false);
    Ok(game_state)
}
