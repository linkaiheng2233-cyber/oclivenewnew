//! 发行版适配：连接内核能力与各平台传输/UI 的薄层。
//!
//! OOCP WebSocket（`/oocp`）与无头 HTTP API 的完整路由在 **`oclive_kernel_runtime::http_api`**
//!（`kernel-http-api`）；桌面 `--api` 通过根 `crate::http_api` 对 runtime 的 **re-export** 暴露。
//! 壳层不再保留重复的 Axum OOCP 适配器，亦不再直接依赖 `axum`（集成测试见 `dev-dependencies`）。
