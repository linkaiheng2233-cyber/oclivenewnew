//! Compatibility path for the independent six-slot Base call surface.
//!
//! The definitions and their Rust documentation are owned by `oclive_kernel_base`.
//! Reference-runtime ports remain alongside these re-exports, with their own semantics.

pub use oclive_kernel_base::{
    AgentBase, BaseCallFuture, EmotionBase, EventBase, LlmBase, MemoryBase, PromptBase,
};
