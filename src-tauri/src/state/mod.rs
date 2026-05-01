//! 桌面发行版状态：与 `oclive_kernel_runtime::state::KernelAppState` 为同一实现。

pub use oclive_kernel_runtime::state::{resolve_roles_dir, KernelAppState, PolicySet};

/// 历史名称：Tauri `invoke`、HTTP API 与集成测试沿用 `AppState`。
pub type AppState = KernelAppState;
