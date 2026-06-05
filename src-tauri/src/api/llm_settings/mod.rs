//! User-facing LLM / model settings (local Ollama + cloud OpenAI-compatible / JSON-RPC).

mod canonical_llm_sync;
mod commands;
mod llm_models;
mod user_llm_env;

pub use crate::domain::user_llm_env::{apply_user_llm_env, apply_user_llm_env_from_db};
pub(crate) use crate::domain::user_llm_env::{
    cloud_api_token_configured, ollama_base_from_db_or_env, resolve_remote_token, KEY_CLOUD_STYLE,
    KEY_CLOUD_VENDOR, KEY_LLM_PROVIDER, KEY_LOCAL_MODELS_DIR, KEY_OLLAMA_BASE, KEY_REMOTE_MODEL,
    KEY_REMOTE_URL, LLM_APP_SETTING_KEYS,
};
pub use canonical_llm_sync::{
    seed_shell_llm_from_canonical, sync_canonical_db_models_dir,
    sync_session_ollama_model_to_canonical, sync_shell_llm_settings_to_canonical,
};
pub use commands::{ImportGgufToOllamaRequest, LlmUserSettingsDto, SaveLlmUserSettingsRequest};
pub use llm_models::LocalModelFileDto;

#[tauri::command]
pub async fn get_llm_user_settings(
    state: tauri::State<'_, crate::state::SharedAppState>,
    role_id: String,
    session_id: Option<String>,
) -> Result<LlmUserSettingsDto, crate::api::error::CommandError> {
    commands::get_llm_user_settings(state, role_id, session_id).await
}

#[tauri::command]
pub async fn list_ollama_models(
    state: tauri::State<'_, crate::state::SharedAppState>,
    ollama_base_url: Option<String>,
) -> Result<Vec<String>, crate::api::error::CommandError> {
    commands::list_ollama_models(state, ollama_base_url).await
}

#[tauri::command]
pub async fn scan_local_model_files(
    state: tauri::State<'_, crate::state::SharedAppState>,
    directory: Option<String>,
) -> Result<Vec<LocalModelFileDto>, crate::api::error::CommandError> {
    commands::scan_local_model_files(state, directory).await
}

#[tauri::command]
pub async fn open_path_in_file_manager(
    path: String,
    app: tauri::AppHandle,
) -> Result<(), crate::api::error::CommandError> {
    commands::open_path_in_file_manager(path, app).await
}

#[tauri::command]
pub async fn import_gguf_to_ollama(
    state: tauri::State<'_, crate::state::SharedAppState>,
    req: ImportGgufToOllamaRequest,
) -> Result<String, crate::api::error::CommandError> {
    commands::import_gguf_to_ollama(state, req).await
}

#[tauri::command]
pub async fn probe_cloud_llm(
    state: tauri::State<'_, crate::state::SharedAppState>,
    role_id: String,
    session_id: Option<String>,
) -> Result<String, crate::api::error::CommandError> {
    commands::probe_cloud_llm(state, role_id, session_id).await
}

#[tauri::command]
pub async fn save_llm_user_settings(
    app: tauri::AppHandle,
    state: tauri::State<'_, crate::state::SharedAppState>,
    req: SaveLlmUserSettingsRequest,
) -> Result<crate::models::dto::RoleInfo, crate::api::error::CommandError> {
    commands::save_llm_user_settings(app, state, req).await
}
