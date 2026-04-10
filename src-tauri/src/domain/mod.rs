pub mod knowledge_loader;
pub mod life_schedule;
pub mod role_manifest_validate;
pub mod user_identity;

pub mod affect_policy;
pub mod chat_engine;
pub mod chat_llm_fallback;
pub mod chat_turn;
pub mod chat_turn_rules;
pub mod emotion_analyzer;
pub mod event_detector;
pub mod event_estimator;
pub mod event_impact_ai;
pub mod memory_engine;
pub mod memory_retrieval;
pub mod mutable_profile_llm;
pub mod personality_engine;
pub mod plugin_host;
pub mod policy;
pub mod portrait_emotion_engine;
pub mod profile_personality;
pub mod prompt_assembler;
pub mod prompt_builder;
pub mod relation_engine;
pub mod remote_life_prompt;
pub mod repository;
pub mod role_manager;
pub mod user_emotion_analyzer;

pub use chat_engine::process_message;
pub use emotion_analyzer::EmotionAnalyzer;
pub use event_detector::EventDetector;
pub use event_estimator::{BuiltinEventEstimator, EventEstimator, RemoteEventEstimatorPlaceholder};
pub use memory_engine::MemoryEngine;
pub use memory_retrieval::{
    BuiltinMemoryRetrieval, BuiltinMemoryRetrievalV2, MemoryRetrieval, MemoryRetrievalInput,
    RemoteMemoryRetrievalPlaceholder,
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
pub use prompt_builder::{PromptBuilder, PromptInput};
pub use relation_engine::{RelationEngine, RelationState};
pub use repository::{FavorabilityRepository, MemoryRepository};
pub use role_manager::RoleManager;
pub use user_emotion_analyzer::{
    BuiltinUserEmotionAnalyzer, RemoteUserEmotionAnalyzerPlaceholder, UserEmotionAnalyzer,
};
