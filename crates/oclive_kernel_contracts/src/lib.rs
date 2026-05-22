//! # oclive_kernel_contracts — 内核端口（trait）层
//!
//! **角色**：定义编排层依赖的**全部抽象接口**（LLM、记忆、插件宿主、Agent 等）；**不含任何实现代码**。
//!
//! **上游**：仅依赖 [`oclive_kernel_types`](https://docs.rs/oclive_kernel_types)（DTO / 错误）。
//! **下游**：`oclivenewnew-tauri` 的 `domain` / `infrastructure` 提供实现；`oclive_kernel_runtime` 过渡期 re-export。
//!
//! **关键决策**：trait 与 Tauri 解耦，便于无头服务、嵌入式或测试注入 mock；插件作者实现本 crate 中的 trait，而非直接改编排代码。
//!
//! ## Trait 职责一览（单一职责）
//!
//! | Trait | 职责 | 典型实现方 |
//! |-------|------|------------|
//! | [`LlmClient`] | 文本生成 / 流式 | Ollama、Remote HTTP、目录插件 |
//! | [`MemoryRetrieval`] | 近期记忆排序与筛选 | Builtin、Remote |
//! | [`UserEmotionAnalyzer`] | 用户消息情绪 | Builtin、Remote |
//! | [`EventEstimator`] | 事件类型与 impact | Builtin + LLM、Remote |
//! | [`PromptAssembler`] | Prompt 片段组装 | Builtin、Remote |
//! | [`ComplexEmotionProvider`] | `narrative_hint` 解析 | Builtin 关键词、Remote |
//! | [`AgentProvider`] | Agent 回合短路 | Builtin ReAct、目录 |
//! | [`PluginHostPort`] | 解析 `plugin_backends` → `dyn` 句柄 | `PluginHost`（Tauri / 无头） |
//! | [`SlotRegistryResolver`] | `slot_registry` 多实例 → `ResolvedRoleSlots` | `SlotResolver` |
//! | [`MemoryRepository`] / [`FavorabilityRepository`] | 持久化端口 | SQL 实现 |
//! | [`EmotionPolicy`] / [`MemoryPolicy`] / [`EventPolicy`] | 回合后策略（持有/过滤） | Builtin 规则 |
//!
//! 各 trait 方法均应有编排调用方或文档中的预留说明；新增方法前请对照 `co_present` / `process_message` 热路径。

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
