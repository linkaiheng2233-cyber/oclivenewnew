//! Interaction mode and "current schedule" DTO orchestration (single entry to avoid duplication in load_role / get_role_info).

use super::{ensure_manifest_role_ready, get_role_info_impl};
use crate::command_error::CommandError;
use crate::domain::life_schedule::pick_life_state;
use crate::models::dto::{LifeStateDto, RoleInfo, SetRoleInteractionModeRequest};
use crate::models::InteractionMode;
use crate::models::Role;
use crate::state::AppState;

/// Seeded, read from DB, includes pack suggested values and schedule inference.
pub(crate) struct InteractionUiSnapshot {
    pub mode_str: String,
    pub pack_default: Option<String>,
    pub current_life: Option<LifeStateDto>,
}

/// `ensure_interaction_mode_seeded` + effective mode + pack suggestion + schedule inference (UI policy snapshot).
pub(crate) async fn resolve_interaction_ui_snapshot(
    state: &AppState,
    role_id: &str,
    role: &Role,
    virtual_time_ms: i64,
) -> Result<InteractionUiSnapshot, CommandError> {
    state
        .db_manager
        .ensure_interaction_mode_seeded(
            role_id,
            role.interaction_mode.as_deref(),
            Some(state.host_profile.interaction.default_mode.as_str()),
        )
        .await?;
    let mode = state.db_manager.get_interaction_mode(role_id).await?;
    let mode_str = mode.as_str().to_string();
    let pack_default = InteractionMode::pack_default_for_api(role.interaction_mode.as_deref());
    let current_life = if mode.is_immersive() {
        role.life_schedule
            .as_ref()
            .and_then(|s| pick_life_state(virtual_time_ms, s))
            .map(|st| LifeStateDto::from(&st))
    } else {
        None
    };
    Ok(InteractionUiSnapshot {
        mode_str,
        pack_default,
        current_life,
    })
}

/// Persist per-role interaction mode and return refreshed [`RoleInfo`].
///
/// # Errors
///
/// Returns [`Err`] when role loading, DB write, or snapshot assembly fails.
pub async fn set_role_interaction_mode_impl(
    state: &AppState,
    req: &SetRoleInteractionModeRequest,
) -> Result<RoleInfo, CommandError> {
    state.load_role_cached_async(&req.role_id).await?;
    ensure_manifest_role_ready(state, &req.role_id).await?;
    state
        .db_manager
        .set_interaction_mode_for_role(&req.role_id, req.mode.trim())
        .await?;
    get_role_info_impl(state, &req.role_id, None).await
}
