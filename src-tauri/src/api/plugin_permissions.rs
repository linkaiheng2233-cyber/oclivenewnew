use crate::api::error::ApiError;
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginPermissionGrantDto {
    pub permission: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetPluginPermissionGrantsResponse {
    pub plugin_id: String,
    pub grants: Vec<PluginPermissionGrantDto>,
}

#[tauri::command]
pub async fn get_plugin_permission_grants(
    plugin_id: String,
    state: State<'_, AppState>,
) -> Result<GetPluginPermissionGrantsResponse, String> {
    let pid = plugin_id.trim();
    if pid.is_empty() {
        return Err(ApiError::InvalidParameter {
            message: "plugin_id required".into(),
        }
        .to_string());
    }
    let rows = state
        .db_manager
        .list_plugin_permission_grants(pid)
        .await
        .map_err(|e| e.to_frontend_error())?;
    Ok(GetPluginPermissionGrantsResponse {
        plugin_id: pid.to_string(),
        grants: rows
            .into_iter()
            .map(|(permission, enabled)| PluginPermissionGrantDto { permission, enabled })
            .collect(),
    })
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetPluginPermissionGrantRequest {
    pub plugin_id: String,
    pub permission: String,
    pub enabled: bool,
}

#[tauri::command]
pub async fn set_plugin_permission_grant(
    req: SetPluginPermissionGrantRequest,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let pid = req.plugin_id.trim();
    let perm = req.permission.trim();
    if pid.is_empty() || perm.is_empty() {
        return Err(ApiError::InvalidParameter {
            message: "plugin_id and permission required".into(),
        }
        .to_string());
    }
    state
        .db_manager
        .upsert_plugin_permission_grant(pid, perm, req.enabled)
        .await
        .map_err(|e| e.to_frontend_error())?;
    Ok(())
}

