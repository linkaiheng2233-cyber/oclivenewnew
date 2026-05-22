//! 内核核心 trait 端口：编排层与宿主通过本 crate 依赖抽象，不耦合具体实现。

pub mod complex_emotion;
pub mod local_plugin_bridge;
pub mod memory_retrieval;
pub mod policy;
pub mod prompt_assembler;
pub mod repository;
pub mod user_emotion_analyzer;

pub use complex_emotion::*;
pub use local_plugin_bridge::*;
pub use memory_retrieval::*;
pub use policy::*;
pub use prompt_assembler::*;
pub use repository::*;
pub use user_emotion_analyzer::*;
