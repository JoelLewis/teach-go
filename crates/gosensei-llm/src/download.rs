use std::path::{Path, PathBuf};

use tracing::info;

use crate::error::LlmError;

pub const DEFAULT_HF_REPO: &str = "ggml-org/gemma-3-1b-it-GGUF";
pub const DEFAULT_MODEL_FILENAME: &str = "gemma-3-1b-it-Q4_K_M.gguf";

/// Ensure the model file exists locally, downloading from HuggingFace if needed.
///
/// Returns the path to the local GGUF file. Progress is reported via callback
/// (bytes_downloaded, total_bytes).
///
/// This is a blocking operation — call from `spawn_blocking`.
pub fn ensure_model(
    model_dir: &Path,
    on_progress: impl Fn(u64, u64),
) -> Result<PathBuf, LlmError> {
    let model_path = model_dir.join(DEFAULT_MODEL_FILENAME);

    if model_path.exists() {
        info!("Model already cached at {}", model_path.display());
        return Ok(model_path);
    }

    info!(
        "Downloading model from {}/{}",
        DEFAULT_HF_REPO, DEFAULT_MODEL_FILENAME
    );

    std::fs::create_dir_all(model_dir).map_err(|e| {
        LlmError::DownloadFailed(format!("create model dir: {e}"))
    })?;

    // Use hf-hub sync API for download
    let api = hf_hub::api::sync::ApiBuilder::new()
        .with_cache_dir(model_dir.to_path_buf())
        .build()
        .map_err(|e| LlmError::DownloadFailed(format!("HF API init: {e}")))?;

    let repo = api.model(DEFAULT_HF_REPO.to_string());

    // Signal download start (0/0 means "starting")
    on_progress(0, 0);

    let downloaded_path = repo
        .get(DEFAULT_MODEL_FILENAME)
        .map_err(|e| LlmError::DownloadFailed(format!("download: {e}")))?;

    info!("Model downloaded to {}", downloaded_path.display());

    // hf-hub caches files in its own structure; copy or symlink to our expected path
    if downloaded_path != model_path {
        // If hf-hub stored it somewhere else, create a symlink
        if !model_path.exists() {
            #[cfg(unix)]
            std::os::unix::fs::symlink(&downloaded_path, &model_path).map_err(|e| {
                LlmError::DownloadFailed(format!("symlink: {e}"))
            })?;

            #[cfg(not(unix))]
            std::fs::copy(&downloaded_path, &model_path).map_err(|e| {
                LlmError::DownloadFailed(format!("copy: {e}"))
            })?;
        }
    }

    // Signal completion
    if let Ok(meta) = std::fs::metadata(&model_path) {
        on_progress(meta.len(), meta.len());
    }

    Ok(model_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ensure_model_returns_cached_path() {
        let dir = std::env::temp_dir().join("gosensei-llm-test-cache");
        let _ = std::fs::create_dir_all(&dir);
        let model_path = dir.join(DEFAULT_MODEL_FILENAME);

        // Create a fake model file
        std::fs::write(&model_path, b"fake gguf data").unwrap();

        let result = ensure_model(&dir, |_, _| {});
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), model_path);

        // Cleanup
        let _ = std::fs::remove_dir_all(&dir);
    }
}
