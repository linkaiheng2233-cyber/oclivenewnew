//! # oclive_kernel_runtime — domain engines, paths, and kernel discovery
//!
//! **Role**: hosts domain engine fragments and HTTP constants reusable in **headless / embedded** scenarios; the **main orchestration** lives in [`oclive_kernel_host`](https://docs.rs/oclive_kernel_host)'s `chat_engine::process_message` (re-exported by `oclivenewnew-tauri`).
//!
//! **Upstream**: [`oclive_kernel_contracts`](https://docs.rs/oclive_kernel_contracts), [`oclive_kernel_types`](https://docs.rs/oclive_kernel_types).
//! **Downstream**: `oclive_kernel_server`, `src-tauri` (paths / kernel discovery only).
//!
//! **Key decision**: DTOs and port traits live in `oclive_kernel_types` / `oclive_kernel_contracts`; import them directly — this crate does not re-export them.

pub mod app_data_migration;
pub mod distro_oclive_file;
pub mod domain;
pub mod http_error;
pub mod kernel_discovery;
pub mod kernel_distro_profile;
pub mod kernel_manifest;
pub mod kernel_policy_input;
pub mod kernel_port_ops;
pub mod kernel_runtime_health;
pub mod kernel_runtime_ops;
pub mod kernel_strategy;
pub mod paths;
pub(crate) mod utils;

pub use oclive_validation as validation;

pub use utils::json_loose::extract_json_object;

// Internal aliases for `domain/*` (not public API — downstream uses `oclive_kernel_types` directly).
pub(crate) use oclive_kernel_types::error;
pub(crate) use oclive_kernel_types::local_plugin;
pub(crate) use oclive_kernel_types::models;
pub(crate) use oclive_kernel_types::KernelErrorBody;

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
pub use http_error::{app_error_from_http_response, app_error_from_kernel_body};
pub use kernel_discovery::{
    discover_kernel_candidates, discover_spawn_kernel_candidates, find_monorepo_root,
    pick_best_kernel, promote_to_shared_runtime, shared_kernel_binary_path, shared_runtime_dir,
    should_promote, KernelCandidate, KernelTier, PROMOTE_SCORE_THRESHOLD, SCORE_BUNDLED,
    SCORE_DEV_FULL_DEBUG, SCORE_DEV_FULL_RELEASE, SCORE_DEV_HEADLESS_DEBUG,
    SCORE_DEV_HEADLESS_RELEASE, SCORE_ENV, SCORE_SETTINGS, SCORE_SHARED,
};
pub use distro_oclive_file::{
    parse_distro_oclive_file, parse_distro_oclive_toml, DistroOcliveFile, PluginBackendsToml,
};
pub use kernel_distro_profile::{
    active_summary_from_requirements, default_requirements_for_distro_id,
    evaluate_profile_compat, parse_distro_requirements_file, parse_distro_requirements_toml,
    profile_satisfies_caller, profiles_compatible_by_hash, resolve_caller_requirements,
};
pub use oclive_kernel_types::{
    ActiveProfileSummary, AttachReason, DistroProfileRequirements, KernelHealthJson,
    ProfileCompat, ReplaceReason,
};
pub use kernel_manifest::{KernelBinaryManifest, KernelBuildProfile};
pub use kernel_policy_input::{build_resolve_plan, PolicyContext, PolicyResolution};
pub use kernel_port_ops::{find_listener_pids, terminate_listeners_on_port};
pub use kernel_runtime_health::{
    distro_health_snapshot, profile_file_sha256_hex, DistroHealthSnapshot, ENV_DISTRO_ID,
    ENV_DISTRO_PROFILE,
};
pub use kernel_runtime_ops::{
    apply_promote_to_candidate, list_runtime_backups, promote_with_backup, rollback_shared_kernel,
    should_promote_binary, PromoteReport,
};
pub use kernel_strategy::{
    manifest_for_candidate, pick_best_by_capability, resolve_kernel_action, KernelActionCandidate,
    KernelActionKind, KernelActionPlan, ResolveKernelActionInput,
};
pub use paths::{
    canonical_brand_app_data_dir, ensure_app_data_dir, find_app_data_dir_for_api,
    find_app_data_dir_for_host, find_db_path, tauri_legacy_app_data_dir, temp_api_db_path,
    AppDataMode, ENV_APP_DATA, ENV_APP_DATA_LEGACY_TEMP, ENV_SKIP_APP_DATA_MIGRATION,
    ENV_USE_CANONICAL_APP_DATA, TAURI_APP_IDENTIFIER,
};

/// Resolve listen port: CLI `--port` wins, then `OCLIVE_API_PORT`, then default (host startup policy).
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
