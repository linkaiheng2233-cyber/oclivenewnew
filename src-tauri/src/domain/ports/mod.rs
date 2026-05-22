//! 领域端口（traits）：**兼容 re-export 层**。
//!
//! 核心 trait 定义在 [`oclive_kernel_contracts`]；本目录仅保留宿主侧 `impl` 与过渡期路径。
//! 新代码请优先 `use oclive_kernel_contracts::…` 或 `oclive_kernel_runtime::…`。

pub mod llm;
pub mod plugin_host;
pub mod slot_resolver;

pub use llm::LlmClient;
pub use oclive_kernel_contracts::SlotRegistryResolver;
pub use plugin_host::PluginHostPort;
