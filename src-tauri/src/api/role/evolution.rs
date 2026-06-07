//! Evolution / interaction-mode API commands.
#![allow(clippy::missing_errors_doc)]

use super::{ensure_manifest_role_ready, get_role_info_impl, EVENT_IMPACT_MAX, EVENT_IMPACT_MIN};
use crate::api::error::CommandError;
use crate::error::AppError;
use oclive_kernel_types::models::dto::{
    RoleInfo, SetEvolutionFactorRequest, SetRemoteLifeEnabledRequest, SetRoleInteractionModeRequest,
};
use oclive_kernel_host::state::{AppState, SharedAppState};
use tauri::State;
pub async fn set_evolution_factor_impl(
    state: &AppState,
    req: &SetEvolutionFactorRequest,
) -> Result<RoleInfo, CommandError> {
    let f = req.event_impact_factor;
    if !f.is_finite() || !(EVENT_IMPACT_MIN..=EVENT_IMPACT_MAX).contains(&f) {
        return Err(AppError::InvalidParameter(format!(
            "event_impact_factor must be between {} and {}",
            EVENT_IMPACT_MIN, EVENT_IMPACT_MAX
        ))
        .into());
    }
    state.load_role_cached_async(&req.role_id).await?;
    ensure_manifest_role_ready(state, &req.role_id).await?;
    state
        .db_manager
        .set_event_impact_factor(&req.role_id, f)
        .await?;
    get_role_info_impl(state, &req.role_id, None).await
}
#[tauri::command]
pub async fn set_evolution_factor(
    req: SetEvolutionFactorRequest,
    state: State<'_, SharedAppState>,
) -> Result<RoleInfo, CommandError> {
    set_evolution_factor_impl(&state, &req).await
}
pub async fn set_remote_life_enabled_impl(
    state: &AppState,
    req: &SetRemoteLifeEnabledRequest,
) -> Result<RoleInfo, CommandError> {
    state.load_role_cached_async(&req.role_id).await?;
    ensure_manifest_role_ready(state, &req.role_id).await?;
    state
        .db_manager
        .set_remote_life_enabled(&req.role_id, req.enabled)
        .await?;
    get_role_info_impl(state, &req.role_id, None).await
}
#[tauri::command]
pub async fn set_remote_life_enabled(
    req: SetRemoteLifeEnabledRequest,
    state: State<'_, SharedAppState>,
) -> Result<RoleInfo, CommandError> {
    set_remote_life_enabled_impl(&state, &req).await
}
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
/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
#[tauri::command]
pub async fn set_role_interaction_mode(
    req: SetRoleInteractionModeRequest,
    state: State<'_, SharedAppState>,
) -> Result<RoleInfo, CommandError> {
    set_role_interaction_mode_impl(&state, &req).await
}
