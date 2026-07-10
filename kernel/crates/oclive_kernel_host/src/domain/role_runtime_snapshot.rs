//! One-row read model for frequently accessed `role_runtime` columns (hot path).

use crate::models::InteractionMode;

/// One-row read of frequently accessed `role_runtime` columns.
#[derive(Debug, Clone, Default)]
pub struct RoleRuntimeSnapshot {
    pub favorability: Option<f64>,
    pub emotion: Option<String>,
    pub relation_state: Option<String>,
    pub scene: Option<String>,
    pub interaction_mode: Option<InteractionMode>,
    pub remote_life_enabled: Option<bool>,
    pub mutable_personality: Option<String>,
    pub event_impact_factor: Option<f64>,
    /// Ephemeral situation summary (Wave F); empty when `ephemeral_ttl_turns == 0`.
    pub ephemeral_personality: Option<String>,
    pub ephemeral_ttl_turns: Option<u32>,
    pub deep_latch_active: Option<bool>,
}
