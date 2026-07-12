//! GoSensei's LLM coaching domain: prompt content, the GBNF grammar for the
//! tagged coaching format, and output parsing.
//!
//! Model download, loading, and generation live in the shared `sensei-llm`
//! crate; this crate is pure domain logic with no llama.cpp dependency.

pub mod grammar;
pub mod parse;
pub mod prompt;
pub mod types;

pub use types::{CoachingPayload, LlmCoachingOutput, SessionContext};
