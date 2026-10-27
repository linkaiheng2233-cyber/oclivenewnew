//! Relation API adapter; persistence belongs to the connected kernel.
#![allow(clippy::missing_errors_doc)]
use crate::api::error::CommandError;
pub use oclive_kernel_host::service::role::relation::{
    clear_scene_user_relation_impl, set_scene_user_relation_impl, set_user_relation_impl,
};
use oclive_kernel_host::state::SharedAppState;
use oclive_kernel_types::models::dto::{
    ClearSceneUserRelationRequest, RoleInfo, SetSceneUserRelationRequest, SetUserRelationRequest,
};
use tauri::{AppHandle, Manager, State};
#[tauri::command]
pub async fn set_user_relation(
    req: SetUserRelationRequest,
    app: AppHandle,
    state: State<'_, SharedAppState>,
) -> Result<RoleInfo, CommandError> {
    if let Some(conn) = app.try_state::<crate::kernel_lifecycle::SharedKernelConnection>() {
        return crate::kernel_attach::KernelHttpClient::set_user_relation_via_http(&conn, &req)
            .await
            .map_err(Into::into);
    }
    set_user_relation_impl(&state, &req).await
}
#[tauri::command]
pub async fn set_scene_user_relation(
    req: SetSceneUserRelationRequest,
    app: AppHandle,
    state: State<'_, SharedAppState>,
) -> Result<RoleInfo, CommandError> {
    if let Some(conn) = app.try_state::<crate::kernel_lifecycle::SharedKernelConnection>() {
        return crate::kernel_attach::KernelHttpClient::set_scene_user_relation_via_http(
            &conn, &req,
        )
        .await
        .map_err(Into::into);
    }
    set_scene_user_relation_impl(&state, &req).await
}
/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
#[tauri::command]
pub async fn clear_scene_user_relation(
    req: ClearSceneUserRelationRequest,
    app: AppHandle,
    state: State<'_, SharedAppState>,
) -> Result<RoleInfo, CommandError> {
    if let Some(conn) = app.try_state::<crate::kernel_lifecycle::SharedKernelConnection>() {
        return crate::kernel_attach::KernelHttpClient::clear_scene_user_relation_via_http(
            &conn, &req,
        )
        .await
        .map_err(Into::into);
    }
    clear_scene_user_relation_impl(&state, &req).await
}
