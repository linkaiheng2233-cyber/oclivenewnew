pub mod adapters;
pub mod core;
pub mod knowledge_loader;
pub mod life_schedule;
pub use oclive_kernel_runtime::domain::local_plugin_bridge;
pub mod local_plugin_memory_pick;
pub mod role_manifest_validate;
pub mod user_identity;

pub use oclive_kernel_runtime::domain::affect_policy;
pub use oclive_kernel_runtime::domain::agent;
pub mod chat_engine;
pub mod chat_llm_fallback;
pub mod chat_turn;
pub mod chat_turn_rules;
pub use oclive_kernel_runtime::domain::complex_emotion;
pub use oclive_kernel_runtime::domain::emotion_analyzer;
pub use oclive_kernel_runtime::domain::event_detector;
pub use oclive_kernel_runtime::domain::event_estimator;
pub use oclive_kernel_runtime::domain::event_impact_ai;
pub mod expert_models;
pub use oclive_kernel_runtime::domain::memory_engine;
pub use oclive_kernel_runtime::domain::memory_retrieval;
pub mod mutable_profile_llm;
pub mod permission_tokens;
pub use oclive_kernel_runtime::domain::personality_engine;
pub mod plugin_host;
pub mod policy;
pub mod portrait_emotion_engine;
pub mod profile_personality;
pub use oclive_kernel_runtime::domain::prompt_assembler;
pub use oclive_kernel_runtime::domain::prompt_builder;
pub mod prompt_style_override;
pub mod relation_engine;
pub mod remote_life_prompt;
pub mod repository;
pub mod role_manager;
pub use oclive_kernel_runtime::domain::user_emotion_analyzer;

pub use agent::{AgentDebugTrace, AgentInput, AgentOutput, AgentProvider, BuiltinReActAgent};
pub use chat_engine::process_message;
pub use emotion_analyzer::EmotionAnalyzer;
pub use event_detector::EventDetector;
pub use event_estimator::{BuiltinEventEstimator, EventEstimator, RemoteEventEstimatorPlaceholder};
pub use local_plugin_bridge::{
    FileManifestLocalPluginBridge, LocalPluginBridge, LocalPluginCapability,
    LocalPluginProviderDescriptor, LocalPluginRegistry, LOCAL_PLUGIN_SCHEMA_VERSION,
};
pub use memory_engine::MemoryEngine;
pub use memory_retrieval::{
    BuiltinMemoryRetrieval, BuiltinMemoryRetrievalV2, LocalPluginMemoryRetrieval, MemoryRetrieval,
    MemoryRetrievalInput, RemoteMemoryRetrievalPlaceholder,
};
pub use personality_engine::PersonalityEngine;
pub use plugin_host::{PluginHost, ResolvedRolePlugins};
pub use policy::{
    DefaultEmotionPolicy, DefaultEventPolicy, DefaultMemoryPolicy, EmotionPolicy,
    EmotionPolicyConfig, EventPolicy, MemoryPolicy, MemoryPolicyConfig, PolicyConfig,
    PolicyContext,
};
pub use prompt_assembler::{
    BuiltinPromptAssembler, PromptAssembler, RemotePromptAssemblerPlaceholder,
};
pub use prompt_builder::{
    effective_reply_quality_anchor, PromptBuilder, PromptInput, DEFAULT_REPLY_QUALITY_ANCHOR,
};
pub use relation_engine::{RelationEngine, RelationState};
pub use repository::{FavorabilityRepository, MemoryRepository};
pub use role_manager::RoleManager;
pub use user_emotion_analyzer::{
    BuiltinUserEmotionAnalyzer, RemoteUserEmotionAnalyzerPlaceholder, UserEmotionAnalyzer,
};
