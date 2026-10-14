//! Compatibility path for six-slot requests and the Base call-failure carrier.
//!
//! Re-exporting preserves type identity with the independent public library.
//! Legacy DTOs, AppError and the transport error-code registry remain unchanged.

pub use oclive_kernel_base::{
    AgentBaseRequest, BaseCallError, BaseCallErrorKind, EmotionBaseRequest, EventBaseRequest,
    LlmBaseRequest, MemoryBaseRequest, PromptBaseRequest,
};
