pub mod adapters;
pub mod affect_policy;
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
pub mod life_schedule;
pub mod memory_engine;
pub mod memory_retrieval;
pub mod personality_engine;
pub mod portrait_emotion_engine;
pub mod policy;
pub mod prompt_builder;
pub mod relation_engine;
pub mod remote_life_prompt;
pub mod repository;
pub mod user_emotion_analyzer;
pub mod user_identity;

// Temporary shim: re-export the rest from `oclivenewnew-tauri`.
//
// Goal: progressively migrate modules here and remove this dependency.
pub use oclivenewnew_tauri::domain::knowledge_loader;
pub use oclivenewnew_tauri::domain::agent;
pub use oclivenewnew_tauri::domain::local_plugin_bridge;
pub use oclivenewnew_tauri::domain::mutable_profile_llm;
pub use oclivenewnew_tauri::domain::plugin_host;
pub use oclivenewnew_tauri::domain::profile_personality;
pub use oclivenewnew_tauri::domain::prompt_assembler;
pub use oclivenewnew_tauri::domain::role_manager;
