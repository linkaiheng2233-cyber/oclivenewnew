//! 交互模式与「此刻日程」DTO 的编排（单入口，避免 load_role / get_role_info 重复）。

use crate::domain::life_schedule::resolve_life_state;
use crate::models::dto::LifeStateDto;
use crate::models::InteractionMode;
use crate::models::Role;
use crate::state::AppState;

/// 已 seed、已读 DB、含 pack 建议值与日程推断。
pub(super) struct InteractionUiSnapshot {
    pub mode_str: String,
    pub pack_default: Option<String>,
    pub current_life: Option<LifeStateDto>,
}

/// `ensure_interaction_mode_seeded` + 有效模式字符串 + pack 建议 + `current_life`。
pub(super) async fn resolve_interaction_ui_snapshot(
    state: &AppState,
    role_id: &str,
    role: &Role,
    virtual_time_ms: i64,
) -> Result<InteractionUiSnapshot, String> {
    state
        .db_manager
        .ensure_interaction_mode_seeded(role_id, role.interaction_mode.as_deref())
        .await
        .map_err(|e| e.to_frontend_error())?;
    let mode = state
        .db_manager
        .get_interaction_mode(role_id)
        .await
        .map_err(|e| e.to_frontend_error())?;
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
