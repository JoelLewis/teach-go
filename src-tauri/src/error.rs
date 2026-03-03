use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("game error: {0}")]
    Game(#[from] gosensei_core::rules::MoveError),

    #[error("KataGo error: {0}")]
    KataGo(String),

    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("LLM error: {0}")]
    #[cfg_attr(not(feature = "llm"), allow(dead_code))]
    Llm(String),

    #[error("{0}")]
    Other(String),
}

impl From<gosensei_katago::client::ClientError> for AppError {
    fn from(err: gosensei_katago::client::ClientError) -> Self {
        AppError::KataGo(err.to_string())
    }
}

// Tauri commands need errors to be serializable
impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
