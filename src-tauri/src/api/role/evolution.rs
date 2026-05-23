//! Evolution / interaction-mode API commands.
#![allow(clippy::missing_errors_doc)]

use super::{get_role_info_impl, EVENT_IMPACT_MAX, EVENT_IMPACT_MIN};
use crate::error::AppError;
use crate::models::dto::{RoleInfo, SetEvolutionFactorRequest, SetRemoteLifeEnabledRequest, SetRoleInteractionModeRequest};
use crate::state::AppState;
use tauri::State;
pub async fn set_evolution_factor_impl(
    state: &AppState,
    req: &SetEvolutionFactorRequest,
) -> Result<RoleInfo, String> {
    let f = req.event_impact_factor;
    if !f.is_finite() || !(EVENT_IMPACT_MIN..=EVENT_IMPACT_MAX).contains(&f) {
        return Err(AppError::InvalidParameter(format!(
            "event_impact_factor must be between {} and {}",
            EVENT_IMPACT_MIN, EVENT_IMPACT_MAX
        ))
        .to_frontend_error());
    }
    state
        .load_role_cached(&req.role_id)
        .map_err(|e| e.to_frontend_error())?;
    if !state
        .db_manager
        .role_runtime_exists(&req.role_id)
        .await
        .map_err(|e| e.to_frontend_error())?
    {
        return Err(AppError::RoleRuntimeNotReady.to_frontend_error());
    }
    state
        .db_manager
        .set_event_impact_factor(&req.role_id, f)
        .await
        .map_err(|e| e.to_frontend_error())?;
    get_role_info_impl(state, &req.role_id, None).await
}
#[tauri::command]
pub async fn set_evolution_factor(
    req: SetEvolutionFactorRequest,
    state: State<'_, AppState>,
) -> Result<RoleInfo, String> {
    set_evolution_factor_impl(&state, &req).await
}
pub async fn set_remote_life_enabled_impl(
    state: &AppState,
    req: &SetRemoteLifeEnabledRequest,
) -> Result<RoleInfo, String> {
    state
        .load_role_cached(&req.role_id)
        .map_err(|e| e.to_frontend_error())?;
    if !state
        .db_manager
        .role_runtime_exists(&req.role_id)
        .await
        .map_err(|e| e.to_frontend_error())?
    {
        return Err(AppError::RoleRuntimeNotReady.to_frontend_error());
    }
    state
        .db_manager
        .set_remote_life_enabled(&req.role_id, req.enabled)
        .await
        .map_err(|e| e.to_frontend_error())?;
    get_role_info_impl(state, &req.role_id, None).await
}
#[tauri::command]
pub async fn set_remote_life_enabled(
    req: SetRemoteLifeEnabledRequest,
    state: State<'_, AppState>,
) -> Result<RoleInfo, String> {
    set_remote_life_enabled_impl(&state, &req).await
}
pub async fn set_role_interaction_mode_impl(
    state: &AppState,
    req: &SetRoleInteractionModeRequest,
) -> Result<RoleInfo, String> {
    state
        .load_role_cached(&req.role_id)
        .map_err(|e| e.to_frontend_error())?;
    if !state
        .db_manager
        .role_runtime_exists(&req.role_id)
        .await
        .map_err(|e| e.to_frontend_error())?
    {
        return Err(AppError::RoleRuntimeNotReady.to_frontend_error());
    }
    state
        .db_manager
        .set_interaction_mode_for_role(&req.role_id, req.mode.trim())
        .await
        .map_err(|e| e.to_frontend_error())?;
    get_role_info_impl(state, &req.role_id, None).await
}
/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
#[tauri::command]
pub async fn set_role_interaction_mode(
    req: SetRoleInteractionModeRequest,
    state: State<'_, AppState>,
) -> Result<RoleInfo, String> {
    set_role_interaction_mode_impl(&state, &req).await
}
