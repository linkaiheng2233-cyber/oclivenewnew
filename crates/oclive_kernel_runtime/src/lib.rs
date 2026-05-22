//! 无头 / 嵌入式 / 桌面宿主共用的内核运行时。
//!
//! - **纯数据**：[`oclive_kernel_types`]
//! - **trait 端口**：[`oclive_kernel_contracts`]
//! - **本 crate**：领域引擎实现与 HTTP 边界常量（主编排仍在 `oclivenewnew-tauri`）
//!
//! `domain` 模块为 **oclivenewnew-tauri 兼容** 而保持 `pub`；新下游请优先依赖 `kernel_types` / `kernel_contracts`。

pub mod domain;
pub(crate) mod utils;

pub use oclive_validation as validation;

// 过渡期兼容层：下游可继续 `use oclive_kernel_runtime::AppError` 等旧路径；
// 计划在后续次要版本收紧 surface，请新代码优先依赖 `oclive_kernel_types` / `oclive_kernel_contracts`。
pub use oclive_kernel_types::*;
pub use oclive_kernel_contracts::{
    self as kernel_contracts, ComplexEmotionProvider, EmotionPolicy, EventPolicy,
    FavorabilityRepository, LlmClient, LocalPluginBridge, MemoryPolicy, MemoryRepository,
    MemoryRetrieval, PluginHostPort, PromptAssembler, SlotRegistryResolver, UserEmotionAnalyzer,
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
