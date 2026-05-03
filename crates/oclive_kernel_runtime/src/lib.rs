//! Oclive kernel runtime crate（无 Tauri 依赖）。
//!
//! 将原 `src-tauri` 中的编排与持久化逐步迁入本 crate，使以下宿主共用同一套实现：
//! - `crates/oclive_kernel_server`（无头 OOCP / HTTP）
//! - `src-tauri`（官方桌面发行版）
//! - 其它 AI 应用或设备侧服务（自建 HTTP / gRPC / 本地进程，持有 `KernelAppState` 即可）
//!
//! 入口形态：`state::KernelAppState`、`domain::chat_engine::process_message`、
//! `domain::role_info_snapshot`、`domain::role_lifecycle`、`domain::role_runtime_commands`、`domain::expert_models_admin`、
//! `domain::session_plugin_override`、`http_api` 等。

pub mod api;
pub mod domain;
pub mod env_flags;
pub mod error;
pub mod http_api;
pub mod infrastructure;
pub mod models;
pub mod state;
pub mod utils;
