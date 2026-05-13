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
//! **稳定契约**以 **`models::dto`**、**`error::AppError`**、**OOCP 协议**及 **`domain::chat_engine::process_message`** 入口为准。`domain` 下 **无外部宿主引用的编排细节子模块** 已收窄为 **`pub(crate)`**；其余子模块仍为 **`pub`**，以便 **`src-tauri`、集成测试与示例** 继续通过 `oclive_kernel_runtime::domain::…` 访问——**上述均不作为对外稳定 API 承诺**，后续可继续将调用方迁移到显式 `pub use` 门面后再收紧模块可见性。

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
