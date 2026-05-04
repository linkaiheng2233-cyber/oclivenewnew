//! 内核共享 **纯数据** 模型（ facility crate 可依赖本 crate，而不依赖整个 `oclive_kernel_runtime`）。
//!
//! 与 [`oclive_kernel_core`] **互不依赖**；编排、`PromptInput` 完整定义、存储层仍留在 runtime。

pub mod event;
pub mod event_impact;
pub mod knowledge_augment;
pub mod personality;
pub mod role_config;

pub use event::{Event, EventType};
pub use event_impact::EventImpactEstimate;
pub use knowledge_augment::KnowledgeEventAugment;
pub use personality::PersonalityVector;
pub use role_config::{
    EvolutionBounds, EvolutionConfig, MemoryConfig, PersonalityDefaults, UserRelation,
};
