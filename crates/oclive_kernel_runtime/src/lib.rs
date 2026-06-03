//! # oclive_kernel_runtime — orchestration implementation and compatibility re-exports
//!
//! **Role**: hosts domain engine fragments and HTTP constants reusable in **headless / embedded** scenarios; the **desktop main orchestration** still lives in `oclivenewnew-tauri`'s `chat_engine::process_message`.
//!
//! **Upstream**: [`oclive_kernel_contracts`](https://docs.rs/oclive_kernel_contracts), [`oclive_kernel_types`](https://docs.rs/oclive_kernel_types).
//! **Downstream**: `oclive_kernel_server`, `src-tauri` (kept working on old paths via re-export).
//!
//! **Key decision**: new code should depend directly on `kernel_types` / `kernel_contracts`; this crate's `pub use *` is a **transitional** measure, and the surface will be tightened later.

pub mod app_data_migration;
pub mod domain;
pub mod paths;
pub(crate) mod utils;

pub use oclive_validation as validation;

// Transitional compatibility layer: downstream can keep using old paths like `use oclive_kernel_runtime::AppError`;
// the surface is planned to be tightened in a later minor version, so new code should prefer depending on `oclive_kernel_types` / `oclive_kernel_contracts`.
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

pub use app_data_migration::ensure_canonical_app_data_ready;
pub use paths::{
    canonical_brand_app_data_dir, ensure_app_data_dir, resolve_app_data_dir_for_api,
    resolve_app_data_dir_for_host, resolve_db_path, tauri_legacy_app_data_dir, temp_api_db_path,
    AppDataMode,
    ENV_APP_DATA, ENV_APP_DATA_LEGACY_TEMP, ENV_SKIP_APP_DATA_MIGRATION,
    ENV_USE_CANONICAL_APP_DATA, TAURI_APP_IDENTIFIER,
};

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
