//! Generic small utilities (decoupled from `domain`).
//!
//! - [`json_loose`]: extract a JSON object fragment from model output.
//! - For main-path LLM / emotion analysis use [`crate::infrastructure::OllamaClient`] and [`crate::domain::slot_runner`].

pub mod block_on;
pub mod json_loose;
pub mod other_helpers;
