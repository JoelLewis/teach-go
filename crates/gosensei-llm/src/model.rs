#![allow(
    clippy::cast_possible_wrap,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]

use std::num::NonZeroU32;
use std::path::Path;
use std::sync::Arc;

use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::llama_batch::LlamaBatch;
use llama_cpp_2::model::params::LlamaModelParams;
use llama_cpp_2::model::{AddBos, LlamaChatMessage, LlamaModel};
use llama_cpp_2::sampling::LlamaSampler;
use tracing::{debug, info, warn};

use crate::error::LlmError;

/// Inference parameters for coaching text generation.
const TEMPERATURE: f32 = 0.6;
const TOP_P: f32 = 0.9;
const MIN_P: f32 = 0.05;
const REPEAT_PENALTY_LAST_N: i32 = 64;
const REPEAT_PENALTY: f32 = 1.15;
const N_CTX: u32 = 512;

/// Manages a loaded LLM model. `LlamaModel` is `Send+Sync`, so this can live in an `Arc`.
/// `LlamaContext` is `!Send`, so each `generate()` call creates a fresh context.
/// Clone is cheap — both fields are `Arc`.
#[derive(Clone)]
pub struct ModelManager {
    backend: Arc<LlamaBackend>,
    model: Arc<LlamaModel>,
}

// LlamaBackend and LlamaModel are Send+Sync in llama-cpp-2
unsafe impl Send for ModelManager {}
unsafe impl Sync for ModelManager {}

impl ModelManager {
    /// Load a GGUF model from disk. Blocking — call from `spawn_blocking`.
    pub fn load(model_path: &Path) -> Result<Self, LlmError> {
        if !model_path.exists() {
            return Err(LlmError::ModelNotFound(model_path.display().to_string()));
        }

        info!("Loading LLM model from {}", model_path.display());

        let backend = LlamaBackend::init()
            .map_err(|e| LlmError::InferenceFailed(format!("backend init: {e}")))?;

        let model_params = LlamaModelParams::default();
        let model = LlamaModel::load_from_file(&backend, model_path, &model_params)
            .map_err(|e| LlmError::InferenceFailed(format!("model load: {e}")))?;

        info!(
            "LLM model loaded successfully ({} params)",
            model.n_params()
        );

        Ok(Self {
            backend: Arc::new(backend),
            model: Arc::new(model),
        })
    }

    /// Format a system+user message pair using the model's built-in chat template.
    pub fn apply_chat_template(&self, system: &str, user: &str) -> Result<String, LlmError> {
        let tmpl = self
            .model
            .chat_template(None)
            .map_err(|e| LlmError::InferenceFailed(format!("chat template: {e}")))?;

        let messages = &[
            LlamaChatMessage::new("system".to_string(), system.to_string())
                .map_err(|e| LlmError::InferenceFailed(format!("system message: {e}")))?,
            LlamaChatMessage::new("user".to_string(), user.to_string())
                .map_err(|e| LlmError::InferenceFailed(format!("user message: {e}")))?,
        ];

        self.model
            .apply_chat_template(&tmpl, messages, true)
            .map_err(|e| LlmError::InferenceFailed(format!("apply template: {e}")))
    }

    /// Generate text from a formatted prompt. Blocking — call from `spawn_blocking`.
    /// Creates a fresh `LlamaContext` for thread safety.
    pub fn generate(&self, prompt: &str, max_tokens: u32) -> Result<String, LlmError> {
        self.generate_streaming(prompt, max_tokens, |_| {})
    }

    /// Generate text with per-token streaming callback. Blocking.
    pub fn generate_streaming(
        &self,
        prompt: &str,
        max_tokens: u32,
        mut on_token: impl FnMut(&str),
    ) -> Result<String, LlmError> {
        let ctx_params =
            LlamaContextParams::default().with_n_ctx(Some(NonZeroU32::new(N_CTX).unwrap()));

        let mut ctx = self
            .model
            .new_context(&self.backend, ctx_params)
            .map_err(|e| LlmError::InferenceFailed(format!("context creation: {e}")))?;

        // Tokenize prompt
        let tokens = self
            .model
            .str_to_token(prompt, AddBos::Always)
            .map_err(|e| LlmError::InferenceFailed(format!("tokenization: {e}")))?;

        debug!("Prompt tokenized: {} tokens", tokens.len());

        if tokens.len() >= N_CTX as usize {
            return Err(LlmError::InferenceFailed(
                "prompt exceeds context window".to_string(),
            ));
        }

        // Process prompt tokens in a batch
        let mut batch = LlamaBatch::new(N_CTX as usize, 1);
        let last_idx = (tokens.len() - 1) as i32;
        for (i, token) in (0_i32..).zip(tokens.into_iter()) {
            batch
                .add(token, i, &[0], i == last_idx)
                .map_err(|e| LlmError::InferenceFailed(format!("batch add: {e}")))?;
        }

        ctx.decode(&mut batch)
            .map_err(|e| LlmError::InferenceFailed(format!("prompt decode: {e}")))?;

        // Set up sampler chain: temp → min_p → top_p → penalties → dist
        let mut sampler = LlamaSampler::chain_simple([
            LlamaSampler::temp(TEMPERATURE),
            LlamaSampler::min_p(MIN_P, 1),
            LlamaSampler::top_p(TOP_P, 1),
            LlamaSampler::penalties(REPEAT_PENALTY_LAST_N, REPEAT_PENALTY, 0.0, 0.0),
            LlamaSampler::dist(1234),
        ]);

        let mut output = String::new();
        let mut decoder = encoding_rs::UTF_8.new_decoder();
        let mut n_cur = batch.n_tokens();

        for _ in 0..max_tokens {
            let token = sampler.sample(&ctx, batch.n_tokens() - 1);
            sampler.accept(token);

            if self.model.is_eog_token(token) {
                debug!("End of generation token reached");
                break;
            }

            let piece = self
                .model
                .token_to_piece(token, &mut decoder, true, None)
                .map_err(|e| LlmError::InferenceFailed(format!("token decode: {e}")))?;

            on_token(&piece);
            output.push_str(&piece);

            batch.clear();
            batch
                .add(token, n_cur, &[0], true)
                .map_err(|e| LlmError::InferenceFailed(format!("batch add gen: {e}")))?;

            ctx.decode(&mut batch)
                .map_err(|e| LlmError::InferenceFailed(format!("decode gen: {e}")))?;

            n_cur += 1;
        }

        if output.is_empty() {
            warn!("LLM generated empty output");
        }

        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn load_missing_file_returns_error() {
        let result = ModelManager::load(&PathBuf::from("/nonexistent/model.gguf"));
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, LlmError::ModelNotFound(_)));
    }
}
