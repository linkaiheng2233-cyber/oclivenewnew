//! 应用内云端 OpenAI 兼容 LLM 与全局对话模型 id（`app_settings`）。

use crate::models::{HostCloudLlmPublicDto, HostCloudLlmSaveDto};
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub async fn get_host_cloud_llm_public(
    state: State<'_, AppState>,
) -> Result<HostCloudLlmPublicDto, String> {
    state
        .get_host_cloud_llm_public()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_host_cloud_llm(state: State<'_, AppState>, dto: HostCloudLlmSaveDto) -> Result<(), String> {
    state
        .set_host_cloud_llm(&dto)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_host_chat_model(state: State<'_, AppState>) -> Result<String, String> {
    Ok(state.global_chat_model())
}

#[tauri::command]
pub async fn set_host_chat_model(state: State<'_, AppState>, model: String) -> Result<(), String> {
    state
        .set_global_chat_model(model)
        .await
        .map_err(|e| e.to_string())
}
