//! 插件权限授予与 token 列表：实现于 `oclive_kernel_runtime::domain::plugin_permission_commands`。

pub use oclive_kernel_runtime::domain::plugin_permission_commands::{
    GetPluginPermissionGrantsResponse, ListPermissionTokensResponse, PluginPermissionGrantDto,
    SetPluginPermissionGrantRequest,
};

use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub async fn get_plugin_permission_grants(
    plugin_id: String,
    state: State<'_, AppState>,
) -> Result<GetPluginPermissionGrantsResponse, String> {
    oclive_kernel_runtime::domain::plugin_permission_commands::get_plugin_permission_grants(
        &state,
        &plugin_id,
    )
    .await
    .map_err(|e| e.to_frontend_error())
}

#[tauri::command]
pub async fn set_plugin_permission_grant(
    req: SetPluginPermissionGrantRequest,
    state: State<'_, AppState>,
) -> Result<(), String> {
    oclive_kernel_runtime::domain::plugin_permission_commands::set_plugin_permission_grant(
        &state, &req,
    )
    .await
    .map_err(|e| e.to_frontend_error())
}

#[tauri::command]
pub async fn list_permission_tokens() -> Result<ListPermissionTokensResponse, String> {
    Ok(
        oclive_kernel_runtime::domain::plugin_permission_commands::list_permission_tokens(),
    )
}
