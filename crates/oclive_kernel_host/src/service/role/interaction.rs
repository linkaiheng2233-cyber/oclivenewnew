//! Interaction mode and "current schedule" DTO orchestration (single entry to avoid duplication in load_role / get_role_info).

use crate::domain::life_schedule::resolve_life_state;
use crate::models::dto::LifeStateDto;
use crate::models::InteractionMode;
use crate::models::Role;
use crate::state::AppState;
use crate::command_error::CommandError;

/// Seeded, read from DB, includes pack suggested values and schedule inference.
pub(crate) struct InteractionUiSnapshot {
    pub mode_str: String,
    pub pack_default: Option<String>,
    pub current_life: Option<LifeStateDto>,
}

/// `ensure_interaction_mode_seeded` + effective mode string + pack suggestion + `current_life`.
pub(crate) async fn resolve_interaction_ui_snapshot(
    state: &AppState,
    role_id: &str,
    role: &Role,
    virtual_time_ms: i64,
) -> Result<InteractionUiSnapshot, CommandError> {
    state
        .db_manager
        .ensure_interaction_mode_seeded(role_id, role.interaction_mode.as_deref())
        .await
        ?;
    let mode = state
        .db_manager
        .get_interaction_mode(role_id)
        .await
        ?;
    let mode_str = mode.as_str().to_string();
    let pack_default = InteractionMode::pack_default_for_api(role.interaction_mode.as_deref());
    let current_life = if mode.is_immersive() {
        role.life_schedule
            .as_ref()
            .and_then(|s| resolve_life_state(virtual_time_ms, s))
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
