//! Oclive **kernel V2 核心层**（`oclive_kernel_core`）：协议与数据访问端口的共享基础类型。
//!
//! 当前阶段迁入：错误、记忆模型、Repository trait、LLM/记忆/情绪/复杂情感/Agent 等 **trait 与 DTO**。
//! 编排、SQLite、HTTP、插件运行时等仍在 `oclive_kernel_runtime` crate。

pub mod agent;
pub mod complex_emotion;
pub mod error;
pub mod function_call;
pub mod llm;
pub mod mcp;
pub mod memory_retrieval;
pub mod models;
pub mod repository;
pub mod user_emotion_analyzer;

pub use llm::LlmClient;
