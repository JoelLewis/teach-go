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

    #[error("{0}")]
    Other(String),
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
