use crate::api::error::CommandError;
use crate::error::AppError;
use crate::kernel_attach::KernelHttpClient;
use crate::kernel_lifecycle::SharedKernelConnection;
use oclive_kernel_types::models::dto::{
    GetRoleInfoRequest, JumpTimeRequest, JumpTimeResponse, TimeStateResponse,
};
use oclive_kernel_host::state::SharedAppState;
use oclive_kernel_host::service::jump_time_impl;
use tauri::{AppHandle, Manager, State};

pub use oclive_kernel_host::service::get_time_state_impl;
async fn get_time_state_via_kernel(
    conn: &SharedKernelConnection,
    role_id: &str,
) -> Result<TimeStateResponse, AppError> {
    match KernelHttpClient::get_time_state_via_http(conn, role_id).await {
        Ok(ts) => Ok(ts),
        Err(e) if time_state_route_unavailable(&e) => {
            get_time_state_via_role_info(conn, role_id).await
        }
        Err(AppError::RoleRuntimeNotReady) => {
            KernelHttpClient::load_role_via_http(conn, role_id.trim()).await?;
            match KernelHttpClient::get_time_state_via_http(conn, role_id).await {
                Ok(ts) => Ok(ts),
                Err(e) if time_state_route_unavailable(&e) => {
                    get_time_state_via_role_info(conn, role_id).await
                }
                Err(e) => Err(e),
            }
        }
        Err(e) => Err(e),
    }
}

fn time_state_route_unavailable(err: &AppError) -> bool {
    match err {
        AppError::OllamaError(msg) => {
            msg.contains("404") || msg.contains("Not Found") || msg.contains("not found")
        }
        _ => false,
    }
}

async fn get_time_state_via_role_info(
    conn: &SharedKernelConnection,
    role_id: &str,
) -> Result<TimeStateResponse, AppError> {
    let req = GetRoleInfoRequest {
        role_id: role_id.to_string(),
        session_id: None,
    };
    let info = KernelHttpClient::get_role_info_via_http(conn, &req).await?;
    let ms = info.virtual_time_ms;
    let dt = chrono::DateTime::from_timestamp_millis(ms).unwrap_or_else(chrono::Utc::now);
    Ok(TimeStateResponse {
        virtual_time_ms: ms,
        iso_datetime: dt.to_rfc3339(),
    })
}

/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
#[tauri::command]
pub async fn get_time_state(
    role_id: String,
    app: AppHandle,
    state: State<'_, SharedAppState>,
) -> Result<TimeStateResponse, CommandError> {
    if let Some(conn) = app.try_state::<SharedKernelConnection>() {
        return get_time_state_via_kernel(&conn, role_id.trim())
            .await
            .map_err(Into::into);
    }
    get_time_state_impl(&state, &role_id).await
}

/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
#[tauri::command]
pub async fn jump_time(
    req: JumpTimeRequest,
    app: AppHandle,
    state: State<'_, SharedAppState>,
) -> Result<JumpTimeResponse, CommandError> {
    if let Some(conn) = app.try_state::<SharedKernelConnection>() {
        match KernelHttpClient::jump_time_via_http(&conn, &req).await {
            Ok(res) => return Ok(res),
            Err(AppError::RoleRuntimeNotReady) => {
                KernelHttpClient::load_role_via_http(&conn, req.role_id.trim()).await?;
                return KernelHttpClient::jump_time_via_http(&conn, &req)
                    .await
                    .map_err(Into::into);
            }
            Err(e) => return Err(e.into()),
        }
    }
    jump_time_impl(&state, &req).await
}
