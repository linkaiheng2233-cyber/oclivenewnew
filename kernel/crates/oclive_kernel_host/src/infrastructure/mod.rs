//! Infrastructure layer.
//!
//! Provides foundational services and data access.

/// Return whether an HTTP endpoint resolves to the local machine.
///
/// Loopback traffic must bypass inherited proxy environment variables so local
/// sidecars and test servers cannot be redirected to an external proxy.
#[must_use]
pub(crate) fn is_loopback_endpoint(endpoint: &str) -> bool {
    let Ok(url) = reqwest::Url::parse(endpoint) else {
        return false;
    };
    let Some(host) = url.host_str() else {
        return false;
    };
    host.eq_ignore_ascii_case("localhost")
        || host
            .trim_start_matches('[')
            .trim_end_matches(']')
            .parse::<std::net::IpAddr>()
            .is_ok_and(|address| address.is_loopback())
}

pub mod agent_mcp_bridge;
pub mod app_data_migration;
pub mod backend_registry;
pub mod cache;
pub mod chat_storage;
pub mod db;
pub mod db_ports;
pub mod deep_link;
pub mod directory_plugins;
mod directory_slots_impl;
pub mod function_call_parser;
pub mod high_risk_grants;
pub mod hotkey_bindings;
pub mod llm;
pub mod llm_models;
pub mod llm_params;
pub mod mcp_client;
pub mod ollama_client;
pub mod ollama_timeouts;
pub mod openai_compatible_llm;
pub mod plugin_data;
pub mod plugin_installer;
pub mod plugin_protocol;
pub mod plugin_state;
pub mod plugin_wiring;
pub mod policy_registry;
pub mod remote_fallback_policy;
pub mod remote_plugin;
pub mod reply_post_processor_wiring;
pub mod repositories;
pub mod role_pack;
pub mod slot_resolver_port;
pub mod sql_migrate;
pub mod sqlite_pool;
pub mod storage;
pub mod theater_director_plugin_seed;
pub mod theater_director_wiring;
pub mod turn_ports;
pub mod user_llm_secrets;

#[cfg(test)]
pub mod test_db;

// Re-export primary types.
pub use cache::Cache;
pub use db::DbManager;
pub use llm::{ollama_llm, LlmClient, MockLlmClient};
pub use repositories::{SqliteFavorabilityRepository, SqliteMemoryRepository};
pub use role_pack::{export_role_pack, import_role_pack, peek_role_pack_manifest};
pub use storage::RoleStorage;
