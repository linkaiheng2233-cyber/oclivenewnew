//! 高风险能力授权：`list` / `grant` / `revoke`（持久化 `{app_data}/high_risk_grants.json`）。

use crate::infrastructure::high_risk_grants::{normalize_grant_kind, GrantKind};
use crate::state::AppState;
use serde::Deserialize;
use serde_json::Value;
use tauri::State;

#[derive(Debug, Deserialize)]
pub struct MutateHighRiskGrantRequest {
    /// `mcp:http` | `mcp:stdio` | `process:spawn` | `network:*`（旧版 snake_case 别名仍接受）
    pub kind: String,
    pub id: String,
}

/// # Errors
///
/// JSON 序列化失败时返回 `String`。
pub fn list_high_risk_grants_impl(state: &AppState) -> Result<Value, String> {
    serde_json::to_value(state.high_risk_grants.snapshot()).map_err(|e| e.to_string())
}

/// # Errors
///
/// 未知 `kind` 或磁盘写入失败。
pub fn grant_high_risk_capability_impl(
    state: &AppState,
    req: &MutateHighRiskGrantRequest,
) -> Result<(), String> {
    let id = req.id.as_str();
    match normalize_grant_kind(&req.kind) {
        Some(GrantKind::McpHttp) => state.high_risk_grants.grant_mcp_http(id),
        Some(GrantKind::McpStdio) => state.high_risk_grants.grant_mcp_stdio(id),
        Some(GrantKind::ProcessSpawn) => state.high_risk_grants.grant_process_spawn(id),
        Some(GrantKind::Network) => state.high_risk_grants.grant_network(id),
        None => Err(format!("unknown high_risk grant kind: {}", req.kind.trim())),
    }
}

/// # Errors
///
/// 未知 `kind` 或磁盘写入失败。
pub fn revoke_high_risk_capability_impl(
    state: &AppState,
    req: &MutateHighRiskGrantRequest,
) -> Result<(), String> {
    let id = req.id.as_str();
    match normalize_grant_kind(&req.kind) {
        Some(GrantKind::McpHttp) => state.high_risk_grants.revoke_mcp_http(id),
        Some(GrantKind::McpStdio) => state.high_risk_grants.revoke_mcp_stdio(id),
        Some(GrantKind::ProcessSpawn) => state.high_risk_grants.revoke_process_spawn(id),
        Some(GrantKind::Network) => state.high_risk_grants.revoke_network(id),
        None => Err(format!("unknown high_risk grant kind: {}", req.kind.trim())),
    }
}

/// # Errors
///
/// JSON 序列化失败时返回 `String`。
#[tauri::command]
pub fn list_high_risk_grants(state: State<'_, AppState>) -> Result<Value, String> {
    list_high_risk_grants_impl(&state)
}

/// # Errors
///
/// 未知 `kind` 或磁盘写入失败。
#[tauri::command]
pub fn grant_high_risk_capability(
    req: MutateHighRiskGrantRequest,
    state: State<'_, AppState>,
) -> Result<(), String> {
    grant_high_risk_capability_impl(&state, &req)
}

/// # Errors
///
/// 未知 `kind` 或磁盘写入失败。
#[tauri::command]
pub fn revoke_high_risk_capability(
    req: MutateHighRiskGrantRequest,
    state: State<'_, AppState>,
) -> Result<(), String> {
    revoke_high_risk_capability_impl(&state, &req)
}
