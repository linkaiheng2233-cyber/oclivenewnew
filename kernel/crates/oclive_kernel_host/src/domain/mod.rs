pub mod ports;

pub mod error_helpers;

/// Kernel domain modules re-exported from `oclive_kernel_runtime` for legacy `crate::domain::*` paths.
/// New code should import from `oclive_kernel_runtime::domain` directly (see RFC_PROFILE_AND_DOMAIN_REEXPORT).
#[deprecated(
    since = "0.3.1",
    note = "Import from oclive_kernel_runtime::domain instead of oclive_kernel_host::domain"
)]
pub use oclive_kernel_runtime::domain::{
    affect_policy, builtin_reply_post_processor, chat_llm_fallback, chat_turn, chat_turn_rules,
    complex_emotion, emotion_analyzer, event_detector, knowledge_loader, life_schedule,
    local_plugin_bridge, local_plugin_memory_pick, memory_engine, memory_retrieval,
    personality_engine, policy, profile_personality, prompt_assembler, prompt_builder,
    relation_engine, remote_life_prompt, repository, user_emotion_analyzer,
};

pub mod builtin_theater_director;
pub mod complex_emotion_store;
pub mod effective_llm_model;
pub mod relation_estrangement;
pub mod relation_transition;
pub mod reply_post_processor;
pub mod role_manifest_validate;
pub mod role_runtime_snapshot;
pub mod role_snapshot;
pub mod startup_health;
pub mod theater_director;
pub mod time_driven_evolution;
pub mod user_identity;
pub mod user_identity_loader;
pub mod user_llm_env;
pub mod virtual_time_sync;

pub mod agent;
pub mod agent_context;
pub mod chat_engine;
pub mod debug_trace;
#[cfg(feature = "dual_core")]
pub mod dual_pipeline;
#[cfg(feature = "dual_core")]
pub mod dual_pipeline_registry;
#[cfg(feature = "dual_core")]
pub mod dual_pipeline_steps;
pub mod event_estimator;
pub mod event_impact_ai;
#[cfg(feature = "dual_core")]
pub mod expert_routing;
pub mod fallback_agent;
pub mod host_profile;
pub mod mutable_profile_llm;
pub mod noop_slot_backends;
pub mod plugin_host;
pub mod portrait_emotion_engine;
pub mod portrait_facility;
#[cfg(test)]
pub mod role_manager;
pub mod slot_resolver;
pub mod slot_runner;
pub mod theater;
pub mod turn_thinking;
pub mod visual_presentation;

pub use agent::{AgentDebugTrace, AgentInput, AgentOutput, AgentProvider, BuiltinReActAgent};
pub use chat_engine::process_message;
pub use event_detector::EventDetector;
pub use event_estimator::{BuiltinEventEstimator, EventEstimator, RemoteEventEstimatorPlaceholder};
pub use local_plugin_bridge::{
    FileManifestLocalPluginBridge, LocalPluginBridge, LocalPluginCapability,
    LocalPluginProviderDescriptor, LocalPluginRegistry, LOCAL_PLUGIN_SCHEMA_VERSION,
};
pub use memory_engine::MemoryEngine;
pub use memory_retrieval::{
    BuiltinMemoryRetrieval, MemoryRetrieval, RemoteMemoryRetrievalPlaceholder,
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
pub use prompt_builder::{effective_reply_quality_anchor, PromptBuilder, PromptInput};
#[cfg(test)]
pub use role_manager::RoleManager;
pub use user_emotion_analyzer::{
    BuiltinUserEmotionAnalyzer, RemoteUserEmotionAnalyzerPlaceholder, UserEmotionAnalyzer,
};
