use crate::infrastructure::{
    load_cached_plugin_reviews_index, sync_plugin_reviews_index_online, PluginReviewsIndexFile,
};
use crate::state::AppState;
use serde::Deserialize;
use tauri::State;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncPluginReviewsIndexRequest {
    #[serde(default)]
    pub source_url: Option<String>,
}

#[tauri::command]
pub async fn sync_plugin_reviews_index(
    req: SyncPluginReviewsIndexRequest,
    state: State<'_, AppState>,
) -> Result<PluginReviewsIndexFile, String> {
    let url = req.source_url.clone();
    let app_data_dir = state.directory_plugins.app_data_dir().to_path_buf();
    sync_plugin_reviews_index_online(&app_data_dir, url.as_deref())
        .await
        .map_err(|e| e.to_frontend_error())
}

#[tauri::command]
pub fn get_cached_plugin_reviews_index(
    state: State<'_, AppState>,
) -> Result<PluginReviewsIndexFile, String> {
    load_cached_plugin_reviews_index(state.directory_plugins.app_data_dir())
        .map_err(|e| e.to_frontend_error())
}
