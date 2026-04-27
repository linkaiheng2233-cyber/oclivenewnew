pub mod adapters;
pub mod affect_policy;
pub mod agent;
pub mod chat_engine;
pub mod chat_llm_fallback;
pub mod chat_turn;
pub mod chat_turn_rules;
pub mod complex_emotion;
pub mod core;
pub mod emotion_analyzer;
pub mod event_detector;
pub mod event_estimator;
pub mod event_impact_ai;
pub mod knowledge_loader;
pub mod life_schedule;
pub mod local_plugin_bridge;
pub mod local_plugin_memory_pick;
pub mod memory_engine;
pub mod memory_retrieval;
pub mod personality_engine;
pub mod portrait_emotion_engine;
pub mod plugin_host;
pub mod policy;
pub mod profile_personality;
pub mod prompt_assembler;
pub mod prompt_builder;
pub mod relation_engine;
pub mod remote_life_prompt;
pub mod repository;
pub mod role_manager;
pub mod user_emotion_analyzer;
pub mod user_identity;
pub mod mutable_profile_llm;

// Temporary shim: re-export the rest from `oclivenewnew-tauri`.
//
// Goal: progressively migrate modules here and remove this dependency.
pub use local_plugin_bridge::LOCAL_PLUGIN_SCHEMA_VERSION;
