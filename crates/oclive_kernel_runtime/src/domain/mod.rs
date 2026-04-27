pub mod adapters;
pub mod chat_engine;
pub mod chat_llm_fallback;
pub mod chat_turn;
pub mod chat_turn_rules;
pub mod complex_emotion;
pub mod core;
pub mod emotion_analyzer;
pub mod event_detector;
pub mod relation_engine;
pub mod user_emotion_analyzer;

// Temporary shim: re-export the rest from `oclivenewnew-tauri`.
//
// Goal: progressively migrate modules here and remove this dependency.
pub use oclivenewnew_tauri::domain::event_estimator;
pub use oclivenewnew_tauri::domain::event_impact_ai;
pub use oclivenewnew_tauri::domain::knowledge_loader;
pub use oclivenewnew_tauri::domain::life_schedule;
pub use oclivenewnew_tauri::domain::local_plugin_bridge;
pub use oclivenewnew_tauri::domain::memory_engine;
pub use oclivenewnew_tauri::domain::memory_retrieval;
pub use oclivenewnew_tauri::domain::mutable_profile_llm;
pub use oclivenewnew_tauri::domain::personality_engine;
pub use oclivenewnew_tauri::domain::plugin_host;
pub use oclivenewnew_tauri::domain::policy;
pub use oclivenewnew_tauri::domain::portrait_emotion_engine;
pub use oclivenewnew_tauri::domain::profile_personality;
pub use oclivenewnew_tauri::domain::prompt_assembler;
pub use oclivenewnew_tauri::domain::prompt_builder;
pub use oclivenewnew_tauri::domain::repository;
pub use oclivenewnew_tauri::domain::role_manager;
pub use oclivenewnew_tauri::domain::user_identity;
