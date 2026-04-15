//! Infrastructure 层
//!
//! 提供基础服务和数据访问功能

pub mod cache;
pub mod db;
pub mod directory_plugins;
pub mod llm;
pub mod llm_params;
pub mod ollama_client;
pub mod ollama_timeouts;
pub mod plugin_state;
pub mod remote_plugin;
pub mod repositories;
pub mod role_pack;
pub mod storage;

// 重新导出主要类型
pub use cache::Cache;
pub use db::DbManager;
pub use llm::{ollama_llm, LlmClient, MockLlmClient};
pub use repositories::{SqliteFavorabilityRepository, SqliteMemoryRepository};
pub use role_pack::{export_role_pack, import_role_pack, peek_role_pack_manifest};
pub use storage::RoleStorage;
