//! Shared kernel contracts for headless, embedded, and desktop hosts.
//!
//! Orchestration (`process_message`, `PluginHost`, repositories) still lives in
//! `oclivenewnew-tauri` while K2 extraction proceeds; this crate holds shared
//! error types, DTOs/models, and repository trait ports.

pub mod domain;
pub mod error;
pub mod models;
pub mod utils;

pub use oclive_validation as validation;

pub use domain::{FavorabilityRepository, MemoryRepository};
pub use error::{AppError, KernelErrorBody, Result};
pub use models::*;

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
