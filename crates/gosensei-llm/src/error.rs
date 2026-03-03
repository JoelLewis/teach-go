use std::fmt;

#[derive(Debug)]
pub enum LlmError {
    ModelNotLoaded,
    ModelNotFound(String),
    InferenceFailed(String),
    DownloadFailed(String),
    Timeout(u64),
    ParseError(String),
}

impl fmt::Display for LlmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ModelNotLoaded => write!(f, "LLM model not loaded"),
            Self::ModelNotFound(path) => write!(f, "model file not found: {path}"),
            Self::InferenceFailed(msg) => write!(f, "inference failed: {msg}"),
            Self::DownloadFailed(msg) => write!(f, "model download failed: {msg}"),
            Self::Timeout(secs) => write!(f, "LLM generation timed out after {secs}s"),
            Self::ParseError(msg) => write!(f, "failed to parse LLM output: {msg}"),
        }
    }
}

impl std::error::Error for LlmError {}
