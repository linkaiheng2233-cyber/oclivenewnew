use crate::error::AppError;
use crate::infrastructure::{
    install_role_pack_from_direct_url, sync_role_index_online, RoleIndexFile,
};
use crate::models::dto::ImportProgress;
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use tauri::Manager;
use tauri::State;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncRoleMarketIndexRequest {
    #[serde(default)]
    pub source_url: Option<String>,
}

#[tauri::command]
pub async fn sync_role_market_index(
    req: SyncRoleMarketIndexRequest,
    state: State<'_, AppState>,
) -> Result<RoleIndexFile, String> {
    let app_data_dir = state.directory_plugins.app_data_dir().to_path_buf();
    let url = req.source_url.clone();
    sync_role_index_online(&app_data_dir, url.as_deref())
        .await
        .map_err(|e: AppError| e.to_frontend_error())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallRolePackFromMarketRequest {
    pub role_id: String,
    pub download_url: String,
    pub sha256: String,
    #[serde(default)]
    pub overwrite: bool,
}

#[tauri::command]
pub async fn install_role_pack_from_market(
    app: tauri::AppHandle,
    req: InstallRolePackFromMarketRequest,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let app = app.clone();
    let app_data_dir = state.directory_plugins.app_data_dir().to_path_buf();
    let storage = state.storage.clone();
    install_role_pack_from_direct_url(
        &storage,
        &app_data_dir,
        &req.role_id,
        &req.download_url,
        &req.sha256,
        req.overwrite,
        move |prog: ImportProgress| {
            let _ = app.emit_all("import_progress", prog);
        },
    )
    .await
    .map_err(|e: AppError| e.to_frontend_error())
}
