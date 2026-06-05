//! Infrastructure layer.
//!
//! Provides foundational services and data access.

pub mod app_data_migration;
pub mod cache;
pub mod chat_storage;
pub mod db;
pub mod deep_link;
pub mod directory_plugins;
pub mod function_call_parser;
pub mod high_risk_grants;
pub mod hotkey_bindings;
pub mod llm;
pub mod llm_params;
pub mod mcp_client;
pub mod ollama_client;
pub mod ollama_timeouts;
pub mod openai_compatible_llm;
pub mod plugin_data;
pub mod plugin_installer;
pub mod plugin_protocol;
pub mod plugin_state;
pub mod policy_registry;
pub mod remote_fallback_policy;
pub mod remote_plugin;
pub mod repositories;
pub mod role_pack;
pub mod sql_migrate;
pub mod sqlite_pool;
pub mod storage;
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
