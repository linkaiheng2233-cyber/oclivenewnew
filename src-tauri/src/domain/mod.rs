//! 领域层：实现以 `oclive_kernel_runtime::domain` 为准；此处仅保留 Tauri 专用 `adapters/`。

pub mod adapters;

pub use oclive_kernel_runtime::domain::{
    affect_policy, agent, app_settings_commands, chat_engine, chat_llm_fallback, chat_turn,
    chat_turn_rules, complex_emotion, conversation_query, core, directory_plugin_commands,
    emotion_analyzer, event_commands,
    event_detector, event_estimator, event_impact_ai,
    expert_models, expert_models_admin, export_chat_logs, knowledge_loader, life_schedule,
    local_plugin_bridge,
    local_plugin_memory_pick, memory_engine, memory_query, memory_retrieval, mutable_profile_llm,
    ollama_host_commands,
    permission_tokens, personality_engine, plugin_host, plugin_permission_commands,
    plugin_resolution_debug, policy, policy_host, portrait_emotion_engine, profile_personality,
    profile_preview,
    prompt_assembler, prompt_builder,
    prompt_style_override, relation_engine, remote_life_prompt, repository, role_feedback_commands,
    role_info_snapshot, role_lifecycle, role_manager, role_manifest_validate, role_paths,
    role_runtime_commands, scene_commands, session_plugin_override,
    user_emotion_analyzer,
    user_identity, virtual_time,
};

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
pub use repository::{ExpertModelsRepository, FavorabilityRepository, MemoryRepository};
pub use role_manager::RoleManager;
pub use user_emotion_analyzer::{
    BuiltinUserEmotionAnalyzer, RemoteUserEmotionAnalyzerPlaceholder, UserEmotionAnalyzer,
};
