//! # oclive_kernel_contracts — 内核端口（trait）层
//!
//! **角色**：定义编排层依赖的**全部抽象接口**（LLM、记忆、插件宿主、Agent 等）；**不含任何实现代码**。
//!
//! **上游**：仅依赖 [`oclive_kernel_types`](https://docs.rs/oclive_kernel_types)（DTO / 错误）。
//! **下游**：`oclivenewnew-tauri` 的 `domain` / `infrastructure` 提供实现；`oclive_kernel_runtime` 过渡期 re-export。
//!
//! **关键决策**：trait 与 Tauri 解耦，便于无头服务、嵌入式或测试注入 mock；插件作者实现本 crate 中的 trait，而非直接改编排代码。

pub(crate) mod agent_provider;
pub(crate) mod complex_emotion;
pub(crate) mod event_estimator;
pub(crate) mod llm;
pub(crate) mod local_plugin_bridge;
pub(crate) mod memory_retrieval;
pub(crate) mod plugin_host;
pub(crate) mod policy;
pub(crate) mod prompt_assembler;
pub(crate) mod repository;
pub(crate) mod slot_resolver;
pub(crate) mod user_emotion_analyzer;

pub use agent_provider::AgentProvider;
pub use complex_emotion::ComplexEmotionProvider;
pub use event_estimator::EventEstimator;
pub use llm::LlmClient;
pub use local_plugin_bridge::LocalPluginBridge;
pub use memory_retrieval::MemoryRetrieval;
pub use plugin_host::PluginHostPort;
pub use policy::{EmotionPolicy, EventPolicy, MemoryPolicy};
pub use prompt_assembler::PromptAssembler;
pub use repository::{FavorabilityRepository, MemoryRepository};
pub use slot_resolver::SlotRegistryResolver;
pub use user_emotion_analyzer::UserEmotionAnalyzer;
