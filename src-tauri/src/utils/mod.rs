//! 通用小工具（与 `domain` 解耦）。
//!
//! - [`json_loose`]：从模型输出中截取 JSON 对象片段。
//! - [`emotion`] / [`ollama`]：可选直连本机 Ollama 的辅助；主路径请用 [`crate::infrastructure::OllamaClient`] 与 [`crate::domain::prompt_builder`]。

pub mod emotion;
pub mod json_loose;
pub mod ollama;
pub mod other_helpers;
