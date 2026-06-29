use super::{ensure_role_info_ready, session_namespace};
use crate::command_error::CommandError;
use crate::domain::role_snapshot::build_display_metrics;
use crate::models::dto::DisplayMetricsDto;
use crate::state::AppState;

use super::runtime::{
    current_favorability_for_effective_identity, maybe_seed_initial_favorability_with_extras,
    resolve_relation_state_for_ui, role_runtime_extras,
};

/// GET-only affect snapshot for UI radar; sets `radar_deep_pending` for the session namespace.
///
/// # Errors
///
/// Returns [`Err`] when role runtime or personality reads fail.
pub async fn get_display_metrics_impl(
    state: &AppState,
    role_id: &str,
    session_id: Option<&str>,
) -> Result<DisplayMetricsDto, CommandError> {
    ensure_role_info_ready(state, role_id, session_id).await?;
    let session_ns = session_namespace(role_id, session_id);
    state
        .session_cache
        .set_radar_deep_pending(session_ns.as_str(), true);

    let role = state.load_role_cached_async(role_id).await?;
    let rt = role_runtime_extras(state, role_id, None, role.as_ref()).await?;
    maybe_seed_initial_favorability_with_extras(state, role_id, role.as_ref(), &rt).await?;
    let personality = state
        .get_current_personality(session_ns.as_str(), role.as_ref())
        .await?;
    let favor = current_favorability_for_effective_identity(
        state,
        role_id,
        rt.current_user_relation.as_str(),
    )
    .await?;
    let relation_state =
        resolve_relation_state_for_ui(state, role_id, rt.current_user_relation.as_str()).await?;

    Ok(build_display_metrics(
        favor,
        relation_state.as_str(),
        &personality,
    ))
}
