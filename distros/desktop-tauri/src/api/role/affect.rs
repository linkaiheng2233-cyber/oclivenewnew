//! GET-only affect display metrics (radar refresh + deep-update pending flag).

use crate::api::error::CommandError;
use crate::error::AppError;
use oclive_kernel_host::service::get_display_metrics_impl;
use oclive_kernel_host::state::SharedAppState;
use oclive_kernel_types::models::dto::{DisplayMetricsDto, GetDisplayMetricsRequest};
use tauri::{AppHandle, Manager, State};

/// Read-only affect snapshot; sets `radar_deep_pending` on the kernel session namespace.
#[tauri::command]
pub async fn get_display_metrics(
    req: GetDisplayMetricsRequest,
    app: AppHandle,
    state: State<'_, SharedAppState>,
) -> Result<DisplayMetricsDto, CommandError> {
    if let Some(conn) = app.try_state::<crate::kernel_lifecycle::SharedKernelConnection>() {
        match crate::kernel_attach::KernelHttpClient::get_display_metrics_via_http(&conn, &req)
            .await
        {
            Ok(metrics) => return Ok(metrics),
            Err(AppError::RoleRuntimeNotReady) => {
                crate::kernel_attach::KernelHttpClient::load_role_via_http(
                    &conn,
                    req.role_id.trim(),
                )
                .await?;
                return crate::kernel_attach::KernelHttpClient::get_display_metrics_via_http(
                    &conn, &req,
                )
                .await
                .map_err(Into::into);
            }
            Err(e) => return Err(e.into()),
        }
    }
    get_display_metrics_impl(&state, &req.role_id, req.session_id.as_deref()).await
}
