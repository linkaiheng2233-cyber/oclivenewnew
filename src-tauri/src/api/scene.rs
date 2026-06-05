use crate::api::error::CommandError;
use crate::error::AppError;
use crate::kernel_attach::KernelHttpClient;
use crate::kernel_lifecycle::SharedKernelConnection;
use crate::models::dto::{
    RoleInfo, SetUserPresenceSceneRequest, SwitchSceneRequest, SwitchSceneResponse,
};
use crate::state::SharedAppState;
use oclive_kernel_host::service::{set_user_presence_scene_impl, switch_scene_impl};
use tauri::{AppHandle, Manager, State};

/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
#[tauri::command]
pub async fn switch_scene(
    req: SwitchSceneRequest,
    app: AppHandle,
    state: State<'_, SharedAppState>,
) -> Result<SwitchSceneResponse, CommandError> {
    if let Some(conn) = app.try_state::<SharedKernelConnection>() {
        match KernelHttpClient::switch_scene_via_http(&conn, &req).await {
            Ok(res) => return Ok(res),
            Err(AppError::RoleRuntimeNotReady) => {
                KernelHttpClient::load_role_via_http(&conn, req.role_id.trim()).await?;
                return KernelHttpClient::switch_scene_via_http(&conn, &req)
                    .await
                    .map_err(Into::into);
            }
            Err(e) => return Err(e.into()),
        }
    }
    switch_scene_impl(&state, &req).await
}

/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
#[tauri::command]
pub async fn set_user_presence_scene(
    req: SetUserPresenceSceneRequest,
    app: AppHandle,
    state: State<'_, SharedAppState>,
) -> Result<RoleInfo, CommandError> {
    if let Some(conn) = app.try_state::<SharedKernelConnection>() {
        match KernelHttpClient::set_user_presence_scene_via_http(&conn, &req).await {
            Ok(res) => return Ok(res),
            Err(AppError::RoleRuntimeNotReady) => {
                KernelHttpClient::load_role_via_http(&conn, req.role_id.trim()).await?;
                return KernelHttpClient::set_user_presence_scene_via_http(&conn, &req)
                    .await
                    .map_err(Into::into);
            }
            Err(e) => return Err(e.into()),
        }
    }
    set_user_presence_scene_impl(&state, &req).await
}
