//! 高风险能力授权：`list` / `grant` / `revoke`（持久化 `{app_data}/high_risk_grants.json`）。

use crate::state::AppState;
use serde::Deserialize;
use serde_json::Value;
use tauri::State;

#[derive(Debug, Deserialize)]
pub struct MutateHighRiskGrantRequest {
    /// `mcp_http` | `mcp_stdio` | `directory_plugin_process_spawn`
    pub kind: String,
    pub id: String,
}

/// # Errors
///
/// JSON 序列化失败时返回 `String`。
#[tauri::command]
pub fn list_high_risk_grants(state: State<'_, AppState>) -> Result<Value, String> {
    serde_json::to_value(state.high_risk_grants.snapshot()).map_err(|e| e.to_string())
}

/// # Errors
///
/// 未知 `kind` 或磁盘写入失败。
#[tauri::command]
pub fn grant_high_risk_capability(
    req: MutateHighRiskGrantRequest,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let id = req.id.as_str();
    match req.kind.trim() {
        "mcp_http" => state.high_risk_grants.grant_mcp_http(id),
        "mcp_stdio" => state.high_risk_grants.grant_mcp_stdio(id),
        "directory_plugin_process_spawn" => state
            .high_risk_grants
            .grant_directory_plugin_spawn(id),
        other => Err(format!("unknown high_risk grant kind: {}", other)),
    }
}

/// # Errors
///
/// 未知 `kind` 或磁盘写入失败。
#[tauri::command]
pub fn revoke_high_risk_capability(
    req: MutateHighRiskGrantRequest,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let id = req.id.as_str();
    match req.kind.trim() {
        "mcp_http" => state.high_risk_grants.revoke_mcp_http(id),
        "mcp_stdio" => state.high_risk_grants.revoke_mcp_stdio(id),
        "directory_plugin_process_spawn" => state
            .high_risk_grants
            .revoke_directory_plugin_spawn(id),
        other => Err(format!("unknown high_risk grant kind: {}", other)),
    }
}
