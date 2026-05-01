//! 通用小工具：复用 `oclive_kernel_runtime::utils`（单一真相源）。
//!
//! - [`json_loose`]：从模型输出中截取 JSON 对象片段。
//! - [`emotion`] / [`ollama`]：可选直连本机 Ollama；主路径请用 [`crate::infrastructure::OllamaClient`] 与 [`crate::domain::prompt_builder`]。

pub use oclive_kernel_runtime::utils::{emotion, json_loose, ollama, other_helpers};
