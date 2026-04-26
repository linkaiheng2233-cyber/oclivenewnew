//! 内核：平台无关的领域逻辑与调度。
//!
//! 本模块不依赖 Tauri / OS 窗口 / 快捷键 / 渲染。
//! 对外能力通过 trait 暴露给适配器（`super::adapters`）。
//!
//! 当前阶段：模块骨架；文件从 `domain/` 逐步迁入。

pub mod oocp_handler;

// TODO P0-A：逐步将以下子模块迁入
// pub mod chat_engine;
// pub mod chat_llm_fallback;
// pub mod chat_turn;
// pub mod chat_turn_rules;
// pub mod complex_emotion;
// pub mod emotion_analyzer;
// pub mod event_detector;
// pub mod event_estimator;
// pub mod event_impact_ai;
// pub mod knowledge_loader;
// pub mod life_schedule;
// pub mod memory_engine;
// pub mod memory_retrieval;
// pub mod personality_engine;
// pub mod plugin_host;
// pub mod policy;
// pub mod portrait_emotion_engine;
// pub mod profile_personality;
// pub mod prompt_assembler;
// pub mod prompt_builder;
// pub mod relation_engine;
// pub mod repository;
// pub mod role_manager;
// pub mod user_emotion_analyzer;
// pub mod agent;
// pub mod affect_policy;