use std::path::{Path, PathBuf};

use tracing::{info, warn};

use crate::error::LlmError;

pub const DEFAULT_HF_REPO: &str = "unsloth/gemma-4-E2B-it-GGUF";
pub const DEFAULT_MODEL_FILENAME: &str = "gemma-4-E2B-it-Q4_K_M.gguf";

/// Size in bytes of `DEFAULT_MODEL_FILENAME` as published on HuggingFace.
/// Used to report a download total before the server responds and to sanity
/// check the downloaded file.
pub const EXPECTED_MODEL_SIZE_BYTES: u64 = 3_106_736_256;

/// Report at most every 8 MiB to avoid flooding progress event channels.
const PROGRESS_GRANULARITY_BYTES: u64 = 8 * 1024 * 1024;

/// Forwards hf-hub download progress to a `(downloaded, total)` callback,
/// throttled to `PROGRESS_GRANULARITY_BYTES`.
struct ProgressForwarder<F: Fn(u64, u64)> {
    on_progress: F,
    downloaded: u64,
    total: u64,
    last_reported: u64,
}

impl<F: Fn(u64, u64)> hf_hub::api::Progress for ProgressForwarder<F> {
    fn init(&mut self, size: usize, _filename: &str) {
        self.total = size as u64;
        (self.on_progress)(0, self.total);
    }

    fn update(&mut self, size: usize) {
        self.downloaded += size as u64;
        if self.downloaded - self.last_reported >= PROGRESS_GRANULARITY_BYTES {
            self.last_reported = self.downloaded;
            (self.on_progress)(self.downloaded, self.total);
        }
    }

    fn finish(&mut self) {
        (self.on_progress)(self.total.max(self.downloaded), self.total);
    }
}

/// Ensure the model file exists locally, downloading from HuggingFace if needed.
///
/// Returns the path to the local GGUF file. Progress is reported via callback
/// (bytes_downloaded, total_bytes).
///
/// This is a blocking operation — call from `spawn_blocking`.
pub fn ensure_model(model_dir: &Path, on_progress: impl Fn(u64, u64)) -> Result<PathBuf, LlmError> {
    let model_path = model_dir.join(DEFAULT_MODEL_FILENAME);

    if model_path.exists() {
        info!("Model already cached at {}", model_path.display());
        return Ok(model_path);
    }

    info!(
        "Downloading model from {}/{}",
        DEFAULT_HF_REPO, DEFAULT_MODEL_FILENAME
    );

    std::fs::create_dir_all(model_dir)
        .map_err(|e| LlmError::DownloadFailed(format!("create model dir: {e}")))?;

    // Use hf-hub sync API for download
    let api = hf_hub::api::sync::ApiBuilder::new()
        .with_cache_dir(model_dir.to_path_buf())
        .build()
        .map_err(|e| LlmError::DownloadFailed(format!("HF API init: {e}")))?;

    let repo = api.model(DEFAULT_HF_REPO.to_string());

    // Signal download start with the published size until the server tells us more
    on_progress(0, EXPECTED_MODEL_SIZE_BYTES);

    let forwarder = ProgressForwarder {
        on_progress: &on_progress,
        downloaded: 0,
        total: EXPECTED_MODEL_SIZE_BYTES,
        last_reported: 0,
    };
    let downloaded_path = repo
        .download_with_progress(DEFAULT_MODEL_FILENAME, forwarder)
        .map_err(|e| LlmError::DownloadFailed(format!("download: {e}")))?;

    info!("Model downloaded to {}", downloaded_path.display());

    // Sanity check: llama.cpp validates GGUF integrity at load time, so a size
    // mismatch (e.g. upstream requantization) is only worth a warning here.
    if let Ok(meta) = std::fs::metadata(&downloaded_path)
        && meta.len() != EXPECTED_MODEL_SIZE_BYTES
    {
        warn!(
            "Downloaded model size {} differs from expected {}",
            meta.len(),
            EXPECTED_MODEL_SIZE_BYTES
        );
    }

    // hf-hub caches files in its own structure; copy or symlink to our expected path
    if downloaded_path != model_path {
        // If hf-hub stored it somewhere else, create a symlink
        if !model_path.exists() {
            std::os::unix::fs::symlink(&downloaded_path, &model_path)
                .map_err(|e| LlmError::DownloadFailed(format!("symlink: {e}")))?;
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

    #[test]
    fn default_model_is_gemma_4_e2b_q4() {
        assert_eq!(DEFAULT_HF_REPO, "unsloth/gemma-4-E2B-it-GGUF");
        assert!(DEFAULT_MODEL_FILENAME.contains("gemma-4-E2B-it"));
        assert!(DEFAULT_MODEL_FILENAME.ends_with("Q4_K_M.gguf"));
    }
}
