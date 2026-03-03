pub mod error;
pub mod parse;
pub mod prompt;
pub mod types;

#[cfg(feature = "llm")]
pub mod download;
#[cfg(feature = "llm")]
pub mod model;

pub use error::LlmError;
pub use types::{CoachingPayload, LlmCoachingOutput, SessionContext};
