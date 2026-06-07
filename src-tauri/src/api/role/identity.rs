//! User Identity Prompt Template API commands.

#![allow(clippy::missing_errors_doc)]

use crate::api::error::CommandError;
use oclive_kernel_types::models::dto::{
    GetUserIdentityStateRequest, SetSceneUserIdentityRequest, SetUserIdentityRequest,
    UserIdentityStateResponse,
};
use oclive_kernel_host::state::SharedAppState;
use oclive_kernel_host::service::role::{
    get_user_identity_state_impl, set_scene_user_identity_impl, set_user_identity_impl,
};
use tauri::State;

/// # Errors
///
/// Returns [`Err`] when the role pack or identity id is invalid.
#[tauri::command]
pub async fn set_user_identity(
    req: SetUserIdentityRequest,
    state: State<'_, SharedAppState>,
) -> Result<UserIdentityStateResponse, CommandError> {
    set_user_identity_impl(&state, &req).await
}

/// # Errors
///
/// Returns [`Err`] when the role pack, scene, or identity id is invalid.
#[tauri::command]
pub async fn set_scene_user_identity(
    req: SetSceneUserIdentityRequest,
    state: State<'_, SharedAppState>,
) -> Result<UserIdentityStateResponse, CommandError> {
    set_scene_user_identity_impl(&state, &req).await
}

/// # Errors
///
/// Returns [`Err`] when the role cannot be loaded.
#[tauri::command]
pub async fn get_user_identity_state(
    req: GetUserIdentityStateRequest,
    state: State<'_, SharedAppState>,
) -> Result<UserIdentityStateResponse, CommandError> {
    get_user_identity_state_impl(&state, &req).await
}
