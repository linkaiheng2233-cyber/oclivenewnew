//! 通用小工具（与 `domain` 解耦）。
//!
//! 目标：逐步迁移到 `oclive_kernel_runtime`，并移除对 `src-tauri` 的依赖。

pub mod json_loose;

// Temporary shim: re-export the rest from `oclivenewnew-tauri`.
pub use oclivenewnew_tauri::utils::emotion;
pub use oclivenewnew_tauri::utils::ollama;
pub use oclivenewnew_tauri::utils::other_helpers;
