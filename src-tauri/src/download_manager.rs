use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

static DOWNLOADING: AtomicBool = AtomicBool::new(false);

use serde::{Deserialize, Serialize};
use tauri::{Emitter, Manager};
use tracing::info;

use crate::setup::KataGoStatus;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum DownloadState {
    NotInstalled,
    Downloading { progress: f64, phase: String },
    Ready,
    Error { message: String },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DownloadStatus {
    pub katago: DownloadState,
    pub llm: DownloadState,
}

fn global_status() -> &'static Arc<Mutex<DownloadStatus>> {
    static INSTANCE: OnceLock<Arc<Mutex<DownloadStatus>>> = OnceLock::new();
    INSTANCE.get_or_init(|| {
        Arc::new(Mutex::new(DownloadStatus {
            katago: DownloadState::NotInstalled,
            llm: DownloadState::NotInstalled,
        }))
    })
}

pub fn get_status() -> DownloadStatus {
    global_status()
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone()
}

fn emit_status(app_handle: &tauri::AppHandle) {
    let status = get_status();
    let _ = app_handle.emit("download-progress", &status);
}

pub async fn run_initial_downloads(app_handle: tauri::AppHandle) {
    if DOWNLOADING.swap(true, Ordering::SeqCst) {
        tracing::info!("Downloads already in progress, skipping");
        return;
    }

    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .expect("failed to resolve app data dir");

    let katago_dir = app_data_dir.join("katago");

    // --- KataGo ---
    let katago_status = crate::setup::setup_status(&katago_dir);
    if katago_status != KataGoStatus::Ready {
        info!("KataGo not installed, starting download…");
        {
            let mut s = global_status().lock().unwrap_or_else(|e| e.into_inner());
            s.katago = DownloadState::Downloading {
                progress: 0.0,
                phase: "starting".into(),
            };
        }
        emit_status(&app_handle);

        let handle = app_handle.clone();
        let dir = katago_dir.clone();
        let result = tokio::task::spawn_blocking(move || {
            crate::setup::ensure_katago(&dir, |sp| {
                let progress = if sp.total > 0 {
                    (sp.downloaded as f64 / sp.total as f64) * 100.0
                } else {
                    0.0
                };
                let mut s = global_status().lock().unwrap_or_else(|e| e.into_inner());
                s.katago = DownloadState::Downloading {
                    progress,
                    phase: sp.phase.clone(),
                };
                drop(s);
                let status = get_status();
                let _ = handle.emit("download-progress", &status);
            })
        })
        .await;

        match result {
            Ok(Ok(())) => {
                info!("KataGo download complete");
                let mut s = global_status().lock().unwrap_or_else(|e| e.into_inner());
                s.katago = DownloadState::Ready;
            }
            Ok(Err(e)) => {
                tracing::error!("KataGo download failed: {e}");
                let mut s = global_status().lock().unwrap_or_else(|e| e.into_inner());
                s.katago = DownloadState::Error {
                    message: e.to_string(),
                };
            }
            Err(e) => {
                tracing::error!("KataGo download task panicked: {e}");
                let mut s = global_status().lock().unwrap_or_else(|e| e.into_inner());
                s.katago = DownloadState::Error {
                    message: format!("task panicked: {e}"),
                };
            }
        }
        emit_status(&app_handle);
    } else {
        info!("KataGo already installed");
        let mut s = global_status().lock().unwrap_or_else(|e| e.into_inner());
        s.katago = DownloadState::Ready;
        drop(s);
        emit_status(&app_handle);
    }

    // --- LLM ---
    #[cfg(feature = "llm")]
    {
        let model_dir = app_data_dir.join("llm");
        let model_path = model_dir.join(gosensei_llm::download::DEFAULT_MODEL_FILENAME);
        if !model_path.exists() {
            info!("LLM model not found, starting download…");
            {
                let mut s = global_status().lock().unwrap_or_else(|e| e.into_inner());
                s.llm = DownloadState::Downloading {
                    progress: 0.0,
                    phase: "model".into(),
                };
            }
            emit_status(&app_handle);

            let handle = app_handle.clone();
            let dir = model_dir.clone();
            let result = tokio::task::spawn_blocking(move || {
                gosensei_llm::download::ensure_model(&dir, |downloaded, total| {
                    let progress = if total > 0 {
                        (downloaded as f64 / total as f64) * 100.0
                    } else {
                        0.0
                    };
                    let mut s = global_status().lock().unwrap_or_else(|e| e.into_inner());
                    s.llm = DownloadState::Downloading {
                        progress,
                        phase: "model".into(),
                    };
                    drop(s);
                    let status = get_status();
                    let _ = handle.emit("download-progress", &status);
                })
            })
            .await;

            match result {
                Ok(Ok(_)) => {
                    info!("LLM model download complete");
                    let mut s = global_status().lock().unwrap_or_else(|e| e.into_inner());
                    s.llm = DownloadState::Ready;
                }
                Ok(Err(e)) => {
                    tracing::error!("LLM download failed: {e}");
                    let mut s = global_status().lock().unwrap_or_else(|e| e.into_inner());
                    s.llm = DownloadState::Error {
                        message: e.to_string(),
                    };
                }
                Err(e) => {
                    tracing::error!("LLM download task panicked: {e}");
                    let mut s = global_status().lock().unwrap_or_else(|e| e.into_inner());
                    s.llm = DownloadState::Error {
                        message: format!("task panicked: {e}"),
                    };
                }
            }
            emit_status(&app_handle);
        } else {
            info!("LLM model already present");
            let mut s = global_status().lock().unwrap_or_else(|e| e.into_inner());
            s.llm = DownloadState::Ready;
            drop(s);
            emit_status(&app_handle);
        }
    }

    #[cfg(not(feature = "llm"))]
    {
        let mut s = global_status().lock().unwrap_or_else(|e| e.into_inner());
        s.llm = DownloadState::Ready;
        drop(s);
        emit_status(&app_handle);
    }

    DOWNLOADING.store(false, Ordering::SeqCst);
}

#[tauri::command]
pub fn get_download_status() -> DownloadStatus {
    get_status()
}

#[tauri::command]
pub async fn retry_downloads(app_handle: tauri::AppHandle) {
    // Reset error states so UI shows "starting" immediately
    {
        let mut s = global_status().lock().unwrap_or_else(|e| e.into_inner());
        if matches!(s.katago, DownloadState::Error { .. }) {
            s.katago = DownloadState::NotInstalled;
        }
        if matches!(s.llm, DownloadState::Error { .. }) {
            s.llm = DownloadState::NotInstalled;
        }
    }
    emit_status(&app_handle);
    run_initial_downloads(app_handle).await;
}
