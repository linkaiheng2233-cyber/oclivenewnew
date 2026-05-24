//! 高风险能力授权：`list` / `grant` / `revoke`（持久化 `{app_data}/high_risk_grants.json`）。

use crate::infrastructure::high_risk_grants::{normalize_grant_kind, GrantKind};
use crate::state::AppState;
use serde::Deserialize;
use serde_json::Value;
use tauri::State;
use crate::api::error::CommandError;

#[derive(Debug, Deserialize)]
pub struct MutateHighRiskGrantRequest {
    /// `mcp:http` | `mcp:stdio` | `process:spawn` | `network:*`（旧版 snake_case 别名仍接受）
    pub kind: String,
    pub id: String,
}

/// # Errors
///
/// JSON 序列化失败时返回 `String`。
pub fn list_high_risk_grants_impl(state: &AppState) -> Result<Value, CommandError> {
    Ok(serde_json::to_value(state.high_risk_grants.snapshot())?)
}

/// # Errors
///
/// 未知 `kind` 或磁盘写入失败。
pub fn grant_high_risk_capability_impl(
    state: &AppState,
    req: &MutateHighRiskGrantRequest,
) -> Result<(), CommandError> {
    let id = req.id.as_str();
    match normalize_grant_kind(&req.kind) {
        Some(GrantKind::McpHttp) => state.high_risk_grants.grant_mcp_http(id).map_err(Into::into),
        Some(GrantKind::McpStdio) => state.high_risk_grants.grant_mcp_stdio(id).map_err(Into::into),
        Some(GrantKind::ProcessSpawn) => state.high_risk_grants.grant_process_spawn(id).map_err(Into::into),
        Some(GrantKind::Network) => state.high_risk_grants.grant_network(id).map_err(Into::into),
        None => Err(format!("unknown high_risk grant kind: {}", req.kind.trim()).into()),
    }
}

/// # Errors
///
/// 未知 `kind` 或磁盘写入失败。
pub fn revoke_high_risk_capability_impl(
    state: &AppState,
    req: &MutateHighRiskGrantRequest,
) -> Result<(), CommandError> {
    let id = req.id.as_str();
    match normalize_grant_kind(&req.kind) {
        Some(GrantKind::McpHttp) => state.high_risk_grants.revoke_mcp_http(id).map_err(Into::into),
        Some(GrantKind::McpStdio) => state.high_risk_grants.revoke_mcp_stdio(id).map_err(Into::into),
        Some(GrantKind::ProcessSpawn) => state.high_risk_grants.revoke_process_spawn(id).map_err(Into::into),
        Some(GrantKind::Network) => state.high_risk_grants.revoke_network(id).map_err(Into::into),
        None => Err(format!("unknown high_risk grant kind: {}", req.kind.trim()).into()),
    }
}

/// # Errors
///
/// JSON 序列化失败时返回 `String`。
#[tauri::command]
pub fn list_high_risk_grants(state: State<'_, AppState>) -> Result<Value, CommandError> {
    list_high_risk_grants_impl(&state)
}

/// # Errors
///
/// 未知 `kind` 或磁盘写入失败。
#[tauri::command]
pub fn grant_high_risk_capability(
    req: MutateHighRiskGrantRequest,
    state: State<'_, AppState>,
) -> Result<(), CommandError> {
    grant_high_risk_capability_impl(&state, &req)
}

/// # Errors
///
/// 未知 `kind` 或磁盘写入失败。
#[tauri::command]
pub fn revoke_high_risk_capability(
    req: MutateHighRiskGrantRequest,
    state: State<'_, AppState>,
) -> Result<(), CommandError> {
    revoke_high_risk_capability_impl(&state, &req)
}
