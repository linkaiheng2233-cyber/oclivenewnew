use crate::state::AppState;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginAuditLogRowDto {
    pub created_at: String,
    pub action: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permission: Option<String>,
    pub allowed: bool,
    pub meta_json: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetPluginAuditLogsRequest {
    pub plugin_id: String,
    #[serde(default)]
    pub limit: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetPluginAuditLogsResponse {
    pub logs: Vec<PluginAuditLogRowDto>,
}

#[tauri::command]
pub fn get_plugin_audit_logs(
    req: GetPluginAuditLogsRequest,
    state: State<'_, AppState>,
) -> Result<GetPluginAuditLogsResponse, String> {
    let pid = req.plugin_id.trim();
    if pid.is_empty() {
        return Err("plugin_id required".to_string());
    }
    let lim = req.limit.unwrap_or(50);
    let rows = tauri::async_runtime::block_on(async {
        state.db_manager.list_plugin_audit_logs(pid, lim).await
    })
    .map_err(|e| e.to_frontend_error())?;
    let logs = rows
        .into_iter()
        .map(
            |(created_at, action, permission, allowed, meta_json)| PluginAuditLogRowDto {
                created_at,
                action,
                permission,
                allowed,
                meta_json,
            },
        )
        .collect::<Vec<_>>();
    Ok(GetPluginAuditLogsResponse { logs })
}
