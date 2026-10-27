//! User Identity Prompt Template API commands.

#![allow(clippy::missing_errors_doc)]

use crate::api::error::CommandError;
use oclive_kernel_host::service::role::{
    get_user_identity_state_impl, set_scene_user_identity_impl, set_user_identity_impl,
};
use oclive_kernel_host::state::SharedAppState;
use oclive_kernel_types::models::dto::{
    GetUserIdentityStateRequest, SetSceneUserIdentityRequest, SetUserIdentityRequest,
    UserIdentityStateResponse,
};
use tauri::{AppHandle, Manager, State};

/// # Errors
///
/// Returns [`Err`] when the role pack or identity id is invalid.
#[tauri::command]
pub async fn set_user_identity(
    req: SetUserIdentityRequest,
    app: AppHandle,
    state: State<'_, SharedAppState>,
) -> Result<UserIdentityStateResponse, CommandError> {
    if let Some(conn) = app.try_state::<crate::kernel_lifecycle::SharedKernelConnection>() {
        return crate::kernel_attach::KernelHttpClient::set_user_identity_via_http(&conn, &req)
            .await
            .map_err(Into::into);
    }
    set_user_identity_impl(&state, &req).await
}

/// # Errors
///
/// Returns [`Err`] when the role pack, scene, or identity id is invalid.
#[tauri::command]
pub async fn set_scene_user_identity(
    req: SetSceneUserIdentityRequest,
    app: AppHandle,
    state: State<'_, SharedAppState>,
) -> Result<UserIdentityStateResponse, CommandError> {
    if let Some(conn) = app.try_state::<crate::kernel_lifecycle::SharedKernelConnection>() {
        return crate::kernel_attach::KernelHttpClient::set_scene_user_identity_via_http(
            &conn, &req,
        )
        .await
        .map_err(Into::into);
    }
    set_scene_user_identity_impl(&state, &req).await
}

/// # Errors
///
/// Returns [`Err`] when the role cannot be loaded.
#[tauri::command]
pub async fn get_user_identity_state(
    req: GetUserIdentityStateRequest,
    app: AppHandle,
    state: State<'_, SharedAppState>,
) -> Result<UserIdentityStateResponse, CommandError> {
    if let Some(conn) = app.try_state::<crate::kernel_lifecycle::SharedKernelConnection>() {
        return crate::kernel_attach::KernelHttpClient::get_user_identity_state_via_http(
            &conn, &req,
        )
        .await
        .map_err(Into::into);
    }
    get_user_identity_state_impl(&state, &req).await
}
