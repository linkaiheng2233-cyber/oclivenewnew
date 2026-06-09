//! Evolution / interaction-mode API commands.
#![allow(clippy::missing_errors_doc)]

use super::{ensure_manifest_role_ready, get_role_info_impl, EVENT_IMPACT_MAX, EVENT_IMPACT_MIN};
use crate::api::error::CommandError;
use crate::error::AppError;
use oclive_kernel_types::models::dto::{
    RoleInfo, SetEvolutionFactorRequest, SetRemoteLifeEnabledRequest, SetRoleInteractionModeRequest,
};
use oclive_kernel_host::service::set_role_interaction_mode_impl as set_role_interaction_mode_kernel_impl;
use oclive_kernel_host::state::{AppState, SharedAppState};
use tauri::{AppHandle, Manager, State};

fn interaction_mode_route_unavailable(err: &AppError) -> bool {
    match err {
        AppError::OllamaError(msg) => {
            msg.contains("404") || msg.contains("Not Found") || msg.contains("not found")
        }
        _ => false,
    }
}
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
    set_role_interaction_mode_kernel_impl(state, req).await
}
/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
#[tauri::command]
pub async fn set_role_interaction_mode(
    req: SetRoleInteractionModeRequest,
    app: AppHandle,
    state: State<'_, SharedAppState>,
) -> Result<RoleInfo, CommandError> {
    if let Some(conn) = app.try_state::<crate::kernel_lifecycle::SharedKernelConnection>() {
        match crate::kernel_attach::KernelHttpClient::set_role_interaction_mode_via_http(
            &conn, &req,
        )
        .await
        {
            Ok(info) => return Ok(info),
            Err(e) if interaction_mode_route_unavailable(&e) => {
                tracing::warn!(
                    target: "oclive_desktop",
                    "kernel missing POST /role/interaction_mode; falling back to shell impl"
                );
            }
            Err(AppError::RoleRuntimeNotReady) => {
                crate::kernel_attach::KernelHttpClient::load_role_via_http(
                    &conn,
                    req.role_id.trim(),
                )
                .await?;
                return crate::kernel_attach::KernelHttpClient::set_role_interaction_mode_via_http(
                    &conn, &req,
                )
                .await
                .map_err(Into::into);
            }
            Err(e) => return Err(e.into()),
        }
    }
    set_role_interaction_mode_impl(&state, &req).await
}
