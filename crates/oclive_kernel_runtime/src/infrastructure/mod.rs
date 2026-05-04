//! Infrastructure：存储、LLM、目录插件、远程插件 JSON-RPC 等。

pub mod blocking_http;
pub mod cloud_llm;
pub mod db;
pub mod directory_plugins;
pub mod function_call_parser;
pub mod hotkey_bindings;
pub mod llm;
pub mod llm_params;
pub mod mcp_client;
#[cfg(feature = "default-llm-providers")]
pub mod ollama_client;
pub mod ollama_timeouts;
#[cfg(feature = "role-pack-zip")]
pub mod plugin_archive;
pub mod plugin_config_disk;
#[cfg(feature = "market-sync")]
pub mod plugin_index_sync;
pub mod plugin_install;
pub mod plugin_layout;
pub mod plugin_package_verify;
#[cfg(feature = "market-sync")]
pub mod plugin_reviews_index_sync;
pub mod plugin_state;
pub mod remote_plugin;
pub mod repositories_runtime;
#[cfg(feature = "market-sync")]
pub mod role_market_index_sync;
#[cfg(feature = "role-pack-zip")]
pub mod role_pack_archive;
pub mod storage;

#[cfg(not(feature = "default-llm-providers"))]
pub use llm::default_runtime_llm_arc;
#[cfg(feature = "default-llm-providers")]
pub use llm::{cloud_llm_from_env, ollama_llm};
pub use llm::{LlmClient, MockLlmClient, RemoteLlmPlaceholder};
pub use storage::RoleStorage;
