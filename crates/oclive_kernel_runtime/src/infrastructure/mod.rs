//! Infrastructure：存储、LLM、目录插件、远程插件 JSON-RPC 等。

pub mod cloud_llm;
pub mod db;
pub mod directory_plugins;
pub mod function_call_parser;
pub mod llm;
pub mod llm_params;
pub mod mcp_client;
pub mod ollama_client;
pub mod ollama_timeouts;
pub mod plugin_config_disk;
pub mod plugin_state;
pub mod remote_plugin;
pub mod repositories_runtime;
pub mod storage;

pub use llm::{cloud_llm_from_env, ollama_llm, LlmClient, MockLlmClient};
pub use storage::RoleStorage;
