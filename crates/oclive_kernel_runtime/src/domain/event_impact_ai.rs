//! 事件影响 LLM 流水线：实现位于 **`oclive_event_builtin`**（runtime 仅 re-export）。
pub use oclive_event_builtin::{
    estimate_event_impact, event_impact_ai_enabled, soften_impact_factor,
};
