//! High-risk capability grants: `list` / `grant` / `revoke` (kernel writer when attached).

use crate::api::error::CommandError;
use crate::error::AppError;
use crate::kernel_attach::KernelHttpClient;
use crate::kernel_lifecycle::SharedKernelConnection;
use oclive_kernel_host::service::{
    grant_high_risk_capability_impl, list_high_risk_grants_impl, revoke_high_risk_capability_impl,
    MutateHighRiskGrantRequest,
};
use oclive_kernel_host::state::SharedAppState;
use serde_json::Value;
use tauri::{AppHandle, Manager, State};

async fn list_high_risk_via_kernel(conn: &SharedKernelConnection) -> Result<Value, CommandError> {
    if !KernelHttpClient::probe_health(&conn.base_url).await {
        return Err(AppError::KernelOffline.into());
    }
    let res = conn
        .http_client()
        .get(format!("{}/high_risk/grants", conn.base_url))
        .send()
        .await
        .map_err(|e| CommandError::from(AppError::OllamaError(e.to_string())))?;
    let status = res.status();
    let text = res
        .text()
        .await
        .map_err(|e| CommandError::from(AppError::OllamaError(e.to_string())))?;
    if !status.is_success() {
        return Err(
            oclive_kernel_runtime::app_error_from_http_response(status.as_u16(), &text).into(),
        );
    }
    serde_json::from_str(&text)
        .map_err(|e| CommandError::from(AppError::OllamaError(e.to_string())))
}

async fn mutate_high_risk_via_kernel(
    conn: &SharedKernelConnection,
    path: &str,
    req: &MutateHighRiskGrantRequest,
) -> Result<(), CommandError> {
    if !KernelHttpClient::probe_health(&conn.base_url).await {
        return Err(AppError::KernelOffline.into());
    }
    let res = conn
        .http_client()
        .post(format!("{}/high_risk/{path}", conn.base_url))
        .json(req)
        .send()
        .await
        .map_err(|e| CommandError::from(AppError::OllamaError(e.to_string())))?;
    let status = res.status();
    if status.is_success() {
        Ok(())
    } else {
        let text = res.text().await.unwrap_or_default();
        Err(oclive_kernel_runtime::app_error_from_http_response(status.as_u16(), &text).into())
    }
}

/// # Errors
///
/// Returns `String` when JSON serialization fails.
#[tauri::command]
pub async fn list_high_risk_grants(
    app: AppHandle,
    state: State<'_, SharedAppState>,
) -> Result<Value, CommandError> {
    if let Some(conn) = app.try_state::<SharedKernelConnection>() {
        return list_high_risk_via_kernel(&conn).await;
    }
    list_high_risk_grants_impl(&state)
}

/// # Errors
///
/// Unknown `kind` or disk write failure.
#[tauri::command]
pub async fn grant_high_risk_capability(
    req: MutateHighRiskGrantRequest,
    app: AppHandle,
    state: State<'_, SharedAppState>,
) -> Result<(), CommandError> {
    if let Some(conn) = app.try_state::<SharedKernelConnection>() {
        return mutate_high_risk_via_kernel(&conn, "grant", &req).await;
    }
    grant_high_risk_capability_impl(&state, &req)
}

/// # Errors
///
/// Unknown `kind` or disk write failure.
#[tauri::command]
pub async fn revoke_high_risk_capability(
    req: MutateHighRiskGrantRequest,
    app: AppHandle,
    state: State<'_, SharedAppState>,
) -> Result<(), CommandError> {
    if let Some(conn) = app.try_state::<SharedKernelConnection>() {
        return mutate_high_risk_via_kernel(&conn, "revoke", &req).await;
    }
    revoke_high_risk_capability_impl(&state, &req)
}
