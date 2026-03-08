use std::path::PathBuf;

use gosensei_core::game::GameState;
use gosensei_katago::client::KataGoClient;
use gosensei_katago::process::KataGoProcess;
use tauri::{AppHandle, Emitter, Manager, State};
use tracing::{info, warn};

use crate::convert;
use crate::error::AppError;
use crate::setup;
use crate::state::AppState;

const MAX_VISITS: u32 = 200;

fn katago_dir(app: &AppHandle) -> Result<PathBuf, AppError> {
    Ok(app
        .path()
        .app_data_dir()
        .map_err(|e| AppError::KataGo(format!("app data dir: {e}")))?
        .join("katago"))
}

fn resolve_binary_path(app: &AppHandle) -> Result<PathBuf, AppError> {
    // 1. Downloaded location
    if let Ok(dir) = katago_dir(app)
        && let Some(p) = setup::binary_path(&dir)
    {
        return Ok(p);
    }

    // 2. Bundled inside .app/Contents/Resources/katago/ (macOS App Store)
    if let Ok(resource_dir) = app.path().resource_dir() {
        let bundled_path = resource_dir.join("katago").join("katago");
        if bundled_path.exists() {
            return Ok(bundled_path);
        }
    }

    // 3. Env var override
    if let Ok(path) = std::env::var("KATAGO_BINARY") {
        let p = PathBuf::from(path);
        if p.exists() {
            return Ok(p);
        }
    }

    // 4. Dev path
    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let dev_path = PathBuf::from(manifest_dir).join("binaries").join("katago");
        if dev_path.exists() {
            return Ok(dev_path);
        }
    }

    // 5. System PATH
    if let Ok(path) = which::which("katago") {
        return Ok(path);
    }

    Err(AppError::KataGo(
        "KataGo binary not found. Use Setup to download, or set KATAGO_BINARY env var.".into(),
    ))
}

fn resolve_config_path(app: &AppHandle) -> Option<PathBuf> {
    // Downloaded config
    if let Ok(dir) = katago_dir(app)
        && let Some(p) = setup::config_path(&dir)
    {
        return Some(p);
    }

    // Bundled inside .app/Contents/Resources/katago/ (macOS App Store)
    if let Ok(resource_dir) = app.path().resource_dir() {
        let bundled_config = resource_dir.join("katago").join("analysis.cfg");
        if bundled_config.exists() {
            return Some(bundled_config);
        }
    }

    // Dev path
    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let config_path = PathBuf::from(manifest_dir)
            .join("binaries")
            .join("analysis.cfg");
        if config_path.exists() {
            return Some(config_path);
        }
    }
    None
}

fn resolve_model_path(app: &AppHandle) -> Result<PathBuf, AppError> {
    // Downloaded model
    if let Ok(dir) = katago_dir(app)
        && let Some(p) = setup::model_path(&dir)
    {
        return Ok(p);
    }

    // Bundled inside .app/Contents/Resources/katago/ (macOS App Store)
    if let Ok(resource_dir) = app.path().resource_dir() {
        let katago_res = resource_dir.join("katago");
        if let Ok(entries) = std::fs::read_dir(&katago_res) {
            for entry in entries.flatten() {
                let name = entry.file_name();
                let name_str = name.to_string_lossy();
                if name_str.ends_with(".bin.gz") || name_str.ends_with(".gz") {
                    return Ok(entry.path());
                }
            }
        }
    }

    // Env var override
    if let Ok(path) = std::env::var("KATAGO_MODEL") {
        let p = PathBuf::from(path);
        if p.exists() {
            return Ok(p);
        }
    }

    // Dev path — look for any .bin.gz model file
    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let binaries_dir = PathBuf::from(manifest_dir).join("binaries");
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
        "KataGo model not found. Use Setup to download, or set KATAGO_MODEL env var.".into(),
    ))
}

#[tauri::command]
pub fn get_katago_status(app: AppHandle) -> Result<setup::KataGoStatus, AppError> {
    // If we can resolve both binary and model from any source, it's ready
    if resolve_binary_path(&app).is_ok() && resolve_model_path(&app).is_ok() {
        return Ok(setup::KataGoStatus::Ready);
    }
    let dir = katago_dir(&app)?;
    Ok(setup::setup_status(&dir))
}

#[tauri::command]
pub async fn setup_katago(app: AppHandle) -> Result<setup::KataGoStatus, AppError> {
    let dir = katago_dir(&app)?;
    let app_clone = app.clone();

    tokio::task::spawn_blocking(move || {
        setup::ensure_katago(&dir, |progress| {
            let _ = app_clone.emit("katago-setup-progress", progress);
        })
    })
    .await
    .map_err(|e| AppError::KataGo(format!("task join: {e}")))?
    .map_err(AppError::KataGo)?;

    Ok(setup::KataGoStatus::Ready)
}

#[tauri::command]
pub async fn start_engine(state: State<'_, AppState>, app: AppHandle) -> Result<setup::KataGoStatus, AppError> {
    let mut katago = state.katago.lock().await;

    if katago.is_some() {
        return Ok(setup::KataGoStatus::Ready);
    }

    let _ = app.emit("engine-status", "starting");
    info!("Starting KataGo engine...");

    let binary_path = resolve_binary_path(&app)?;
    let model_path = resolve_model_path(&app)?;
    let config_path = resolve_config_path(&app);

    info!("KataGo binary: {}", binary_path.display());
    info!("KataGo model: {}", model_path.display());

    match KataGoProcess::spawn(binary_path, model_path, config_path).await {
        Ok(process) => {
            let client = KataGoClient::new(process);
            *katago = Some(client);
            let _ = app.emit("engine-status", "ready");
            info!("KataGo engine ready");
            Ok(setup::KataGoStatus::Ready)
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
pub async fn stop_engine(state: State<'_, AppState>, app: AppHandle) -> Result<(), AppError> {
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
        let katago = state.katago.lock().await;
        if katago.is_none() {
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
    let query = convert::build_query(
        query_id, &history, board_size, komi, MAX_VISITS, profile, None,
    );

    let response = {
        let rx = {
            let katago = state.katago.lock().await;
            let client = katago
                .as_ref()
                .ok_or(AppError::KataGo("Engine not available".into()))?;
            client.query_fire(query).await?
        };
        // Lock released — await response without holding state.katago
        rx.await.map_err(|_| AppError::KataGo("Engine response cancelled".into()))?
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
