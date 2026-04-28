//! Infrastructure 层
//!
//! 提供基础服务和数据访问功能

pub mod cache;
pub mod cloud_llm;
pub mod db;
pub mod deep_link;
pub mod directory_plugins;
pub mod function_call_parser;
pub mod hotkey_bindings;
pub mod llm;
pub mod llm_params;
pub mod mcp_client;
pub mod ollama_client;
pub mod ollama_timeouts;
pub mod plugin_data;
pub mod plugin_installer;
pub mod plugin_reviews;
pub mod plugin_state;
pub mod remote_plugin;
pub mod repositories;
pub mod role_market;
pub mod role_pack;
pub mod storage;

// 重新导出主要类型
pub use cache::Cache;
pub use db::DbManager;
pub use llm::{cloud_llm_from_env, ollama_llm, LlmClient, MockLlmClient};
pub use plugin_reviews::{
    load_cached_plugin_reviews_index, sync_plugin_reviews_index_online, PluginReviewEntry,
    PluginReviewsIndexFile, DEFAULT_PLUGIN_REVIEWS_INDEX_URL,
};
pub use repositories::{SqliteFavorabilityRepository, SqliteMemoryRepository};
pub use role_market::{
    install_role_pack_from_direct_url, sync_role_index_online, RoleIndexEntry, RoleIndexFile,
};
pub use role_pack::{export_role_pack, import_role_pack, peek_role_pack_manifest};
pub use storage::RoleStorage;
