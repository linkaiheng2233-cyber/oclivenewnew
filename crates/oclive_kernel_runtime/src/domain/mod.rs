//! Pure domain logic (no `AppState`, Tauri, or SQL). Orchestration glue stays in `oclive_kernel_host` (`chat_engine::process_message`).

pub mod affect_policy;
pub mod chat_engine;
pub mod chat_llm_fallback;
pub mod chat_turn;
pub mod chat_turn_rules;
pub mod complex_emotion;
pub mod emotion_analyzer;
pub mod event_detector;
pub mod knowledge_loader;
pub mod life_schedule;
pub mod local_plugin_bridge;
pub mod local_plugin_memory_pick;
pub mod memory_engine;
pub mod memory_retrieval;
pub mod personality_engine;
pub mod policy;
pub mod profile_personality;
pub mod prompt_assembler;
pub mod prompt_builder;
pub mod relation_engine;
pub mod remote_life_prompt;
pub mod repository;
pub mod time_decay;
pub mod virtual_time;
pub mod user_emotion_analyzer;

pub use emotion_analyzer::EmotionAnalyzer;
pub use event_detector::EventDetector;
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
    KERNEL_DIALOGUE_GUARDRAILS,
};
pub use relation_engine::{RelationEngine, RelationState};
pub use repository::{FavorabilityRepository, MemoryRepository};
pub use user_emotion_analyzer::{
    BuiltinUserEmotionAnalyzer, RemoteUserEmotionAnalyzerPlaceholder, UserEmotionAnalyzer,
};
