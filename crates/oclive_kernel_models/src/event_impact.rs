//! 事件影响估计输出（LLM / 规则回退 / Remote JSON-RPC 共用）。

use crate::EventType;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EventImpactEstimate {
    pub event_type: EventType,
    pub impact_factor: f64,
    pub confidence: f32,
}
