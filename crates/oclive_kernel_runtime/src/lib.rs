//! # oclive_kernel_runtime — 编排实现与兼容 re-export
//!
//! **角色**：承载可在**无头 / 嵌入式**复用的领域引擎片段与 HTTP 常量；**桌面主编排**仍在 `oclivenewnew-tauri` 的 `chat_engine::process_message`。
//!
//! **上游**：[`oclive_kernel_contracts`](https://docs.rs/oclive_kernel_contracts)、[`oclive_kernel_types`](https://docs.rs/oclive_kernel_types)。
//! **下游**：`oclive_kernel_server`、`src-tauri`（经 re-export 保持旧路径可用）。
//!
//! **关键决策**：新代码应直接依赖 `kernel_types` / `kernel_contracts`；本 crate `pub use *` 为**过渡期**，后续将收紧 surface。

pub mod domain;
pub(crate) mod utils;

pub use oclive_validation as validation;

// 过渡期兼容层：下游可继续 `use oclive_kernel_runtime::AppError` 等旧路径；
// 计划在后续次要版本收紧 surface，请新代码优先依赖 `oclive_kernel_types` / `oclive_kernel_contracts`。
pub use oclive_kernel_types::*;
pub use oclive_kernel_contracts::{
    self as kernel_contracts, AgentProvider, ComplexEmotionProvider, EmotionPolicy,
    EventEstimator, EventPolicy, FavorabilityRepository, LlmClient, LocalPluginBridge, MemoryPolicy,
    MemoryRepository, MemoryRetrieval, PluginHostPort, PromptAssembler, SlotRegistryResolver,
    UserEmotionAnalyzer,
};
pub use oclive_kernel_types::{self as kernel_types};

pub use utils::json_loose::extract_json_object;

/// Runtime API / contract revision (bump when HTTP or DTO breaking changes ship).
pub const RUNTIME_API_VERSION: &str = "0.2.0";

/// Default TCP port for `oclivenewnew-tauri --api` and `oclive-kernel-server`.
pub const DEFAULT_API_PORT: u16 = 8420;

/// Environment variable for HTTP API listen port.
pub const ENV_API_PORT: &str = "OCLIVE_API_PORT";

/// When set to `1` or `true`, HTTP API uses in-memory DB + mock LLM (CI / bring-up).
pub const ENV_HTTP_API_MOCK_LLM: &str = "OCLIVE_HTTP_API_MOCK_LLM";

/// Roles directory override (same semantics as desktop host).
pub const ENV_ROLES_DIR: &str = "OCLIVE_ROLES_DIR";

/// Resolve listen port: CLI `--port` wins, then `OCLIVE_API_PORT`, then [`DEFAULT_API_PORT`].
#[must_use]
pub fn resolve_api_port(cli_port: Option<u16>) -> u16 {
    if let Some(p) = cli_port.filter(|p| *p > 0) {
        return p;
    }
    std::env::var(ENV_API_PORT)
        .ok()
        .and_then(|s| s.parse().ok())
        .filter(|p| *p > 0)
        .unwrap_or(DEFAULT_API_PORT)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_api_port_prefers_cli() {
        assert_eq!(resolve_api_port(Some(9000)), 9000);
    }

    #[test]
    fn resolve_api_port_default() {
        assert_eq!(resolve_api_port(None), DEFAULT_API_PORT);
    }
}
