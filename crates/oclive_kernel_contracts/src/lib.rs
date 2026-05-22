//! # oclive_kernel_contracts
//!
//! Core trait definitions for the oclive kernel.
//! All abstractions that the orchestration layer depends on.

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
