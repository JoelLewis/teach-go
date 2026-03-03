use tauri::State;

use crate::error::AppError;
use crate::state::AppState;

#[cfg(feature = "llm")]
use tauri::{AppHandle, Emitter, Manager};
#[cfg(feature = "llm")]
use tracing::info;

#[cfg(feature = "llm")]
#[derive(Clone, serde::Serialize)]
struct DownloadProgress {
    downloaded: u64,
    total: u64,
}

/// Initialize the LLM model — downloads if needed, then loads into memory.
#[cfg(feature = "llm")]
#[tauri::command]
pub async fn init_llm_model(
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<String, AppError> {
    // Guard: already loaded
    if state.llm.try_lock().map(|g| g.is_some()).unwrap_or(false) {
        return Ok("ready".to_string());
    }

    // Determine model directory
    let model_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| AppError::Llm(format!("app data dir: {e}")))?
        .join("models");

    let app_clone = app.clone();

    // Download + load in a blocking task (both are CPU/IO heavy)
    let manager = tokio::task::spawn_blocking(move || {
        // Ensure model file exists
        let model_path = gosensei_llm::download::ensure_model(&model_dir, |downloaded, total| {
            let _ = app_clone.emit(
                "llm-download-progress",
                DownloadProgress { downloaded, total },
            );
        })
        .map_err(|e| AppError::Llm(e.to_string()))?;

        // Load the model
        gosensei_llm::model::ModelManager::load(&model_path)
            .map_err(|e| AppError::Llm(e.to_string()))
    })
    .await
    .map_err(|e| AppError::Llm(format!("task join: {e}")))??;

    // Store in state
    let mut llm_lock = state.llm.lock().await;
    *llm_lock = Some(manager);

    info!("LLM model initialized successfully");
    Ok("ready".to_string())
}

/// Check the current LLM status.
#[cfg(feature = "llm")]
#[tauri::command]
pub fn get_llm_status(state: State<'_, AppState>) -> Result<String, AppError> {
    match state.llm.try_lock() {
        Ok(guard) => {
            if guard.is_some() {
                Ok("ready".to_string())
            } else {
                Ok("not_installed".to_string())
            }
        }
        Err(_) => Ok("loading".to_string()),
    }
}

// Stubs when LLM feature is not compiled in

#[cfg(not(feature = "llm"))]
#[tauri::command]
pub async fn init_llm_model(
    _state: State<'_, AppState>,
    _app: tauri::AppHandle,
) -> Result<String, AppError> {
    Err(AppError::Llm("LLM feature not enabled".into()))
}

#[cfg(not(feature = "llm"))]
#[tauri::command]
pub fn get_llm_status(_state: State<'_, AppState>) -> Result<String, AppError> {
    Ok("disabled".to_string())
}
