//! 通用小工具（与 `domain` 解耦）。
//!
//! - [`json_loose`]：从模型输出中截取 JSON 对象片段。
//! - 主路径 LLM / 情绪分析请用 [`crate::infrastructure::OllamaClient`] 与 [`crate::domain::slot_runner`]。

pub mod block_on;
pub mod json_loose;
pub mod other_helpers;
