//! # oclive_kernel_contracts — kernel port (trait) layer
//!
//! **Role**: defines **all abstract interfaces** the orchestration layer depends on (LLM, memory, plugin host, Agent, etc.); contains **no implementation code**.
//!
//! **Upstream**: depends only on [`oclive_kernel_types`](https://docs.rs/oclive_kernel_types) (DTO / errors).
//! **Downstream**: implementations are provided by `oclivenewnew-tauri`'s `domain` / `infrastructure`; `oclive_kernel_runtime` re-exports during the transition period.
//!
//! **Key decision**: decoupling the traits from Tauri makes it easy to inject mocks for headless services, embedded use, or tests; plugin authors implement the traits in this crate rather than editing the orchestration code directly.
//!
//! ## Trait responsibilities at a glance (single responsibility)
//!
//! | Trait | Responsibility | Typical implementer |
//! |-------|------|------------|
//! | [`LlmClient`] | Text generation / streaming | Ollama, Remote HTTP, directory plugin |
//! | [`MemoryRetrieval`] | Recent memory ranking and filtering | Builtin, Remote |
//! | [`UserEmotionAnalyzer`] | User message emotion | Builtin, Remote |
//! | [`EventEstimator`] | Event type and impact | Builtin + LLM, Remote |
//! | [`PromptAssembler`] | Prompt fragment assembly | Builtin, Remote |
//! | [`ComplexEmotionProvider`] | `narrative_hint` resolution | Builtin keyword, Remote |
//! | [`AgentProvider`] | Agent turn short-circuit | Builtin ReAct, directory |
//! | [`PluginHostPort`] | Resolve `plugin_backends` → `dyn` handles | `PluginHost` (Tauri / headless) |
//! | [`SlotRegistryResolver`] | `slot_registry` multi-instance → `ResolvedRoleSlots` | `SlotResolver` |
//! | [`MemoryRepository`] / [`FavorabilityRepository`] | Persistence ports | SQL implementation |
//! | [`EmotionPolicy`] / [`MemoryPolicy`] / [`EventPolicy`] | Post-turn policies (retain/filter) | Builtin rules |
//!
//! Every trait method should have an orchestration caller or a documented reservation note; before adding a method, cross-check the `co_present` / `process_message` hot path.

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
