//! 事件影响估计输出（纯数据结构）。

use crate::models::EventType;

/// LLM- or policy-derived estimate of how an event affects favorability/evolution.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct EventImpactEstimate {
    pub event_type: EventType,
    pub impact_factor: f64,
    pub confidence: f32,
}
