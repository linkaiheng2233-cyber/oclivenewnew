//! 领域端口：**纯 re-export 与宿主 `impl` 层**。
//!
//! 所有 trait 定义在 [`oclive_kernel_contracts`]；本目录不含任何 `trait` 定义。
//! 新代码请优先 `use oclive_kernel_contracts::…` 或 `oclive_kernel_runtime::…`。

pub mod llm;
pub mod plugin_host;
pub mod slot_resolver;

pub use llm::LlmClient;
pub use oclive_kernel_contracts::{
    AgentProvider, EventEstimator, SlotRegistryResolver,
};
pub use plugin_host::PluginHostPort;
