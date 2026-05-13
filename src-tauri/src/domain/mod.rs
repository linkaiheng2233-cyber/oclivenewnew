//! 领域层：实现以 `oclive_kernel_runtime::domain` 为准；此处仅保留 Tauri 专用 `adapters/`。

pub mod adapters;

pub use oclive_kernel_runtime::domain::{
    agent, app_settings_commands, chat_engine, complex_emotion, conversation_query,
    directory_plugin_commands, emotion_analyzer, event_commands, event_detector, event_estimator,
    expert_models, expert_models_admin, export_chat_logs, knowledge_loader, local_imports,
    local_plugin_bridge, memory_engine, memory_query, memory_retrieval, ollama_host_commands,
    permission_tokens, personality_engine, plugin_host, plugin_install_consent,
    plugin_permission_commands, plugin_resolution_debug, policy, policy_host, profile_preview,
    prompt_assembler, prompt_builder, repository, role_feedback_commands, role_info_snapshot,
    role_lifecycle, role_paths, role_runtime_commands, scene_commands, session_plugin_override,
    user_emotion_analyzer, virtual_time,
};

pub use agent::{AgentDebugTrace, AgentInput, AgentOutput, AgentProvider};
pub use chat_engine::process_message;
pub use emotion_analyzer::EmotionAnalyzer;
pub use event_detector::EventDetector;
pub use event_estimator::{EventEstimator, RemoteEventEstimatorPlaceholder};
pub use local_plugin_bridge::{
    FileManifestLocalPluginBridge, LocalPluginBridge, LocalPluginCapability,
    LocalPluginProviderDescriptor, LocalPluginRegistry, LOCAL_PLUGIN_SCHEMA_VERSION,
};
pub use memory_engine::MemoryEngine;
pub use memory_retrieval::{
    LocalPluginMemoryRetrieval, MemoryRetrieval, MemoryRetrievalInput,
    RemoteMemoryRetrievalPlaceholder,
};
pub use personality_engine::PersonalityEngine;
pub use plugin_host::{PluginHost, ResolvedRolePlugins};
pub use policy::{
    DefaultEmotionPolicy, DefaultEventPolicy, DefaultMemoryPolicy, EmotionPolicy,
    EmotionPolicyConfig, EventPolicy, MemoryPolicy, MemoryPolicyConfig, PolicyConfig,
    PolicyContext,
};
pub use prompt_assembler::{PromptAssembler, RemotePromptAssemblerPlaceholder};
pub use prompt_builder::{
    effective_reply_quality_anchor, PromptBuilder, PromptInput, DEFAULT_REPLY_QUALITY_ANCHOR,
};
pub use repository::{ExpertModelsRepository, FavorabilityRepository, MemoryRepository};
pub use user_emotion_analyzer::{RemoteUserEmotionAnalyzerPlaceholder, UserEmotionAnalyzer};
