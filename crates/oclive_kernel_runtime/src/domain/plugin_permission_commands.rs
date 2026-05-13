//! 目录插件权限授予与静态 token 列表（无 Tauri）。

use crate::domain::permission_tokens::{PermissionTokenInfo, PERMISSION_TOKENS_V1};
use crate::error::{AppError, Result};
use crate::state::KernelAppState;
use serde::{Deserialize, Serialize};

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

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetPluginPermissionGrantRequest {
    pub plugin_id: String,
    pub permission: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListPermissionTokensResponse {
    pub tokens: Vec<PermissionTokenInfo>,
}

pub async fn get_plugin_permission_grants(
    state: &KernelAppState,
    plugin_id: &str,
) -> Result<GetPluginPermissionGrantsResponse> {
    let pid = plugin_id.trim();
    if pid.is_empty() {
        return Err(AppError::InvalidParameter("plugin_id required".into()));
    }
    let rows = state.db_manager.list_plugin_permission_grants(pid).await?;
    Ok(GetPluginPermissionGrantsResponse {
        plugin_id: pid.to_string(),
        grants: rows
            .into_iter()
            .map(|(permission, enabled)| PluginPermissionGrantDto {
                permission,
                enabled,
            })
            .collect(),
    })
}

pub async fn set_plugin_permission_grant(
    state: &KernelAppState,
    req: &SetPluginPermissionGrantRequest,
) -> Result<()> {
    let pid = req.plugin_id.trim();
    let perm = req.permission.trim();
    if pid.is_empty() || perm.is_empty() {
        return Err(AppError::InvalidParameter(
            "plugin_id and permission required".into(),
        ));
    }
    state
        .db_manager
        .upsert_plugin_permission_grant(pid, perm, req.enabled)
        .await
}

#[must_use]
pub fn list_permission_tokens() -> ListPermissionTokensResponse {
    ListPermissionTokensResponse {
        tokens: PERMISSION_TOKENS_V1.to_vec(),
    }
}
