//! Turn-scoped policy limits (decouples domain from `PolicySet`).

/// Memory / event limits for one turn (resolved from scene policy registry).
#[derive(Debug, Clone)]
pub struct TurnPolicies {
    pub memory_fifo_limit: i32,
}

/// Resolves turn policy limits for a scene.
pub trait TurnPoliciesPort: Send + Sync {
    #[must_use]
    fn policies_for_scene(&self, scene_id: Option<&str>) -> TurnPolicies;
}
