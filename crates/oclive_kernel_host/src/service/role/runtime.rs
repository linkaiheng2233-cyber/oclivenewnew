//! Runtime identity resolution: shared rules for `load_role`, `get_role_info`, and the chat engine.

use crate::domain::user_identity::resolve_effective_user_relation_key;
use crate::models::dto::UserRelationDto;
use crate::models::role::Role;
use crate::state::AppState;

use super::display::user_relations_to_dto;
use crate::command_error::CommandError;

/// Runtime fields shared by `load_role` / `get_role_info` to avoid drift between the two paths.
pub(crate) struct RoleRuntimeExtras {
    pub user_relations: Vec<UserRelationDto>,
    pub default_relation: String,
    pub current_user_relation: String,
    pub use_manifest_default: bool,
    pub event_impact_factor: f64,
}

async fn effective_event_impact(
    state: &AppState,
    role_id: &str,
    role: &Role,
) -> Result<f64, CommandError> {
    Ok(state
        .db_manager
        .get_event_impact_factor(role_id)
        .await?
        .unwrap_or(role.evolution_config.event_impact_factor))
}

async fn effective_user_relation(
    state: &AppState,
    role_id: &str,
    scene_id: Option<&str>,
    role: &Role,
) -> Result<String, CommandError> {
    Ok(resolve_effective_user_relation_key(state, role, role_id, scene_id).await?)
}

pub(crate) async fn role_runtime_extras(
    state: &AppState,
    role_id: &str,
    scene_id: Option<&str>,
    role: &Role,
) -> Result<RoleRuntimeExtras, CommandError> {
    let use_manifest_default = state.db_manager.get_use_manifest_default(role_id).await?;
    Ok(RoleRuntimeExtras {
        user_relations: user_relations_to_dto(role),
        default_relation: role.default_relation.clone(),
        current_user_relation: effective_user_relation(state, role_id, scene_id, role).await?,
        use_manifest_default,
        event_impact_factor: effective_event_impact(state, role_id, role).await?,
    })
}

/// When there is no dialogue memory yet and favorability is 0, write initial favorability for the current identity to DB (once only).
/// Caller must resolve `role_runtime_extras` first (same source as `current_favorability`) to avoid duplicate scene/identity lookups in one request.
pub(crate) async fn maybe_seed_initial_favorability_with_extras(
    state: &AppState,
    role_id: &str,
    role: &Role,
    rt: &RoleRuntimeExtras,
) -> Result<(), CommandError> {
    let memory_count = state.memory_repo.count_memories(role_id).await?;
    let eff = rt.current_user_relation.as_str();
    let seed = role.initial_favorability_for_relation(eff);
    state
        .db_manager
        .ensure_identity_stats_row(role_id, eff, seed)
        .await?;
    let fav = state
        .db_manager
        .get_favorability_for_identity(role_id, eff)
        .await?
        .unwrap_or(0.0);
    if memory_count > 0 || fav != 0.0 {
        return Ok(());
    }
    state
        .db_manager
        .set_identity_favorability_value(role_id, eff, seed)
        .await?;
    Ok(())
}

/// Same as chat engine: `role_identity_stats` keyed by effective identity; falls back to global `role_runtime.favorability` when missing.
pub(crate) async fn current_favorability_for_effective_identity(
    state: &AppState,
    role_id: &str,
    effective_relation_key: &str,
) -> Result<f64, CommandError> {
    Ok(state
        .db_manager
        .favorability_for_identity_with_runtime_fallback(role_id, effective_relation_key)
        .await?)
}

/// Prefer per-identity stats; fall back to legacy global `role_runtime` (UI relation display policy).
pub(crate) async fn resolve_relation_state_for_ui(
    state: &AppState,
    role_id: &str,
    effective_relation_key: &str,
) -> Result<String, CommandError> {
    let mut relation_state = state
        .db_manager
        .get_relation_state_for_identity(role_id, effective_relation_key)
        .await?;
    if relation_state.is_none() {
        relation_state = state.db_manager.get_relation_state(role_id).await?;
    }
    Ok(relation_state.unwrap_or_else(|| "Stranger".to_string()))
}
