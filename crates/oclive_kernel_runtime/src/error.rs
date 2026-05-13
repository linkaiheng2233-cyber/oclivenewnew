//! 与 `oclive_kernel_core::error` 共用同一 `AppError` / `Result`（再导出以保持既有 `crate::error` 路径）。
//!
//! 桌面侧将错误交给 Tauri 时，请使用 `AppError::to_frontend_error()` 或 `map_err`，勿依赖已移除的 `tauri_invoke` 特性。

pub use oclive_kernel_core::error::{AppError, Result};
