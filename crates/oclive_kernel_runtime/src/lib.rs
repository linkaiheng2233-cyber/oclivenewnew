//! Oclive kernel runtime crate（无 Tauri 依赖）。
//!
//! 将原 `src-tauri` 中的编排与持久化逐步迁入本 crate，使以下宿主共用同一套实现：
//! - `crates/oclive_kernel_server`（无头 OOCP / HTTP）
//! - `src-tauri`（官方桌面发行版）
//! - 其它 AI 应用或设备侧服务（自建 HTTP / gRPC / 本地进程，持有 `KernelAppState` 即可）
//!
//! 入口形态：`state::KernelAppState`、`domain::chat_engine::process_message`、
//! `domain::role_info_snapshot`、`domain::role_lifecycle`、`domain::role_paths`、`domain::role_runtime_commands`、
//! `domain::expert_models_admin`、`domain::ollama_host_commands`、`domain::policy_host`、`domain::profile_preview`、
//! `domain::plugin_permission_commands`、`domain::session_plugin_override`、`http_api`（feature `kernel-http-api`）等。
//!
//! **对外可见性**：`domain` 下各模块为 **`pub mod`**，便于 **Tauri / `kernel_server` / 第三方嵌入** 与历史 `use` 路径兼容；稳定契约以 **`models::dto`**、**`error::AppError`** 与 OOCP 规范为准，而非「每个子模块均为稳定公有 API」。收窄导出需在主版本迭代中单独设计。

pub mod api;
pub mod domain;
pub mod env_flags;
pub mod error;
#[cfg(feature = "kernel-http-api")]
pub mod http_api;
pub mod infrastructure;
pub mod models;
pub mod state;
pub mod utils;
