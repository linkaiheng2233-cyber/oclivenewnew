pub mod ports;

pub mod error_helpers;

/// Kernel domain shims (consolidated from one-line `*.rs` re-export files).
macro_rules! kernel_domain_reexport {
    ($name:ident) => {
        pub mod $name {
            pub use oclive_kernel_runtime::domain::$name::*;
        }
    };
}

kernel_domain_reexport!(affect_policy);
kernel_domain_reexport!(chat_llm_fallback);
kernel_domain_reexport!(chat_turn);
kernel_domain_reexport!(chat_turn_rules);
kernel_domain_reexport!(complex_emotion);
kernel_domain_reexport!(emotion_analyzer);
kernel_domain_reexport!(event_detector);
kernel_domain_reexport!(knowledge_loader);
kernel_domain_reexport!(life_schedule);
kernel_domain_reexport!(local_plugin_bridge);
kernel_domain_reexport!(local_plugin_memory_pick);
kernel_domain_reexport!(memory_engine);
kernel_domain_reexport!(memory_retrieval);
kernel_domain_reexport!(personality_engine);
kernel_domain_reexport!(policy);
kernel_domain_reexport!(profile_personality);
kernel_domain_reexport!(prompt_assembler);
kernel_domain_reexport!(prompt_builder);
kernel_domain_reexport!(relation_engine);
kernel_domain_reexport!(remote_life_prompt);
kernel_domain_reexport!(repository);
kernel_domain_reexport!(user_emotion_analyzer);

pub mod role_manifest_validate;
pub mod startup_health;
pub mod user_identity;
pub mod effective_llm_model;
pub mod role_snapshot;
pub mod relation_estrangement;
pub mod complex_emotion_store;
pub mod time_driven_evolution;
pub mod virtual_time_sync;

pub mod agent;
pub mod host_profile;
pub mod chat_engine;
#[cfg(feature = "dual_core")]
pub mod dual_pipeline;
#[cfg(feature = "dual_core")]
pub mod dual_pipeline_registry;
#[cfg(feature = "dual_core")]
pub mod dual_pipeline_steps;
#[cfg(feature = "dual_core")]
pub mod expert_routing;
pub mod debug_trace;
pub mod event_estimator;
pub mod event_impact_ai;
pub mod mutable_profile_llm;
pub mod plugin_host;
pub mod portrait_emotion_engine;
#[cfg(test)]
pub mod role_manager;
pub mod slot_resolver;
pub mod slot_runner;

pub use agent::{AgentDebugTrace, AgentInput, AgentOutput, AgentProvider, BuiltinReActAgent};
pub use chat_engine::process_message;
pub use event_detector::EventDetector;
pub use event_estimator::{BuiltinEventEstimator, EventEstimator, RemoteEventEstimatorPlaceholder};
pub use memory_engine::MemoryEngine;
pub use memory_retrieval::{
    BuiltinMemoryRetrieval, MemoryRetrieval, RemoteMemoryRetrievalPlaceholder,
};
pub use local_plugin_bridge::{
    FileManifestLocalPluginBridge, LocalPluginBridge, LocalPluginCapability,
    LocalPluginProviderDescriptor, LocalPluginRegistry, LOCAL_PLUGIN_SCHEMA_VERSION,
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
