//! 领域端口（traits）：`domain` 依赖本模块，**不**依赖 `infrastructure` / `api` 实现。
//!
//! 具体适配器（Ollama、Remote HTTP、目录插件等）在 `infrastructure/` 实现这些 trait。

pub mod llm;
pub mod plugin_host;

pub use llm::LlmClient;
pub use plugin_host::PluginHostPort;
