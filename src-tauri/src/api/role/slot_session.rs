//! Session slot override API commands (Tauri wrappers).

#![allow(clippy::missing_errors_doc)]

use crate::api::error::CommandError;
use crate::models::dto::*;
use crate::state::SharedAppState;
use tauri::State;

pub use oclive_kernel_host::service::role::{
    apply_author_suggested_plugin_backends_impl, build_plugin_resolution_debug_info,
    clear_all_session_slot_overrides_impl, clear_session_slot_override_impl,
    get_plugin_resolution_debug_impl, save_role_slot_registry_impl,
    set_session_plugin_backend_impl, set_session_slot_override_impl,
};

#[derive(Debug, serde::Deserialize)]
pub struct ApplyAuthorSuggestedBackendsRequest {
    pub role_id: String,
    #[serde(default)]
    pub session_id: Option<String>,
}

/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
#[tauri::command]
pub async fn save_role_slot_registry(
    req: SaveRoleSlotRegistryRequest,
    state: State<'_, SharedAppState>,
) -> Result<RoleInfo, CommandError> {
    save_role_slot_registry_impl(&state, &req).await
}

/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
#[tauri::command]
pub async fn set_session_plugin_backend(
    req: SetSessionPluginBackendRequest,
    state: State<'_, SharedAppState>,
) -> Result<RoleInfo, CommandError> {
    set_session_plugin_backend_impl(&state, &req).await
}

/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
#[tauri::command]
pub async fn set_session_slot_override(
    req: SetSessionSlotOverrideRequest,
    state: State<'_, SharedAppState>,
) -> Result<RoleInfo, CommandError> {
    set_session_slot_override_impl(&state, &req).await
}

/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
#[tauri::command]
pub async fn clear_session_slot_override(
    req: ClearSessionSlotOverrideRequest,
    state: State<'_, SharedAppState>,
) -> Result<RoleInfo, CommandError> {
    clear_session_slot_override_impl(&state, &req).await
}

/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
#[tauri::command]
pub async fn clear_all_session_slot_overrides(
    req: ClearAllSessionSlotOverridesRequest,
    state: State<'_, SharedAppState>,
) -> Result<RoleInfo, CommandError> {
    clear_all_session_slot_overrides_impl(&state, &req).await
}

/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
#[tauri::command]
pub async fn apply_author_suggested_plugin_backends(
    req: ApplyAuthorSuggestedBackendsRequest,
    state: State<'_, SharedAppState>,
) -> Result<RoleInfo, CommandError> {
    apply_author_suggested_plugin_backends_impl(&state, &req.role_id, req.session_id.as_deref())
        .await
}

/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
#[tauri::command]
pub async fn get_plugin_resolution_debug(
    req: GetPluginResolutionDebugRequest,
    state: State<'_, SharedAppState>,
) -> Result<PluginResolutionDebugInfo, CommandError> {
    get_plugin_resolution_debug_impl(&state, &req).await
}
