//! 提示词构建：当前仍与 `PromptAssembler` 共享同一套 DTO（来自 `oclivenewnew-tauri`）。
//!
//! 在 `PromptAssembler` 迁移到 kernel runtime 之前，这里保持对 Tauri 侧 `PromptInput`/常量的兼容，
//! 避免 trait type mismatch。

pub use oclivenewnew_tauri::domain::prompt_builder::*;

