//! 事件影响估计输出（纯数据结构）。

use crate::models::EventType;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct EventImpactEstimate {
    pub event_type: EventType,
    pub impact_factor: f64,
    pub confidence: f32,
}
