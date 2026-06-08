//! One-row read model for frequently accessed `role_runtime` columns (hot path).

use crate::models::InteractionMode;

/// One-row read of frequently accessed `role_runtime` columns.
#[derive(Debug, Clone)]
pub struct RoleRuntimeSnapshot {
    pub favorability: Option<f64>,
    pub emotion: Option<String>,
    pub relation_state: Option<String>,
    pub scene: Option<String>,
    pub interaction_mode: Option<InteractionMode>,
    pub remote_life_enabled: Option<bool>,
    pub mutable_personality: Option<String>,
    pub event_impact_factor: Option<f64>,
}

impl Default for RoleRuntimeSnapshot {
    fn default() -> Self {
        Self {
            favorability: None,
            emotion: None,
            relation_state: None,
            scene: None,
            interaction_mode: None,
            remote_life_enabled: None,
            mutable_personality: None,
            event_impact_factor: None,
        }
    }
}
