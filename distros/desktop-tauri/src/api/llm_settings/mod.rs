//! User-facing LLM / model settings (local Ollama + cloud OpenAI-compatible / JSON-RPC).

mod canonical_llm_sync;
mod commands;
mod llm_models;

pub use canonical_llm_sync::{
    seed_shell_llm_from_canonical, sync_canonical_db_models_dir,
    sync_session_ollama_model_to_canonical, sync_shell_llm_settings_to_canonical,
};
pub use commands::ImportGgufToOllamaRequest;
pub use oclive_kernel_host::domain::user_llm_env::{
    apply_user_llm_env, apply_user_llm_env_from_db,
};
pub use oclive_kernel_host::infrastructure::llm_models::LocalModelFileDto;
pub use oclive_kernel_host::service::{
    GlobalOllamaModelDto, LlmUserSettingsDto, SaveLlmUserSettingsRequest,
    SetGlobalOllamaModelRequest,
};

#[tauri::command]
pub async fn get_llm_user_settings(
    app: tauri::AppHandle,
    state: tauri::State<'_, oclive_kernel_host::state::SharedAppState>,
    role_id: String,
    session_id: Option<String>,
) -> Result<LlmUserSettingsDto, crate::api::error::CommandError> {
    commands::get_llm_user_settings(app, state, role_id, session_id).await
}

#[tauri::command]
pub async fn list_ollama_models(
    state: tauri::State<'_, oclive_kernel_host::state::SharedAppState>,
    ollama_base_url: Option<String>,
) -> Result<Vec<String>, crate::api::error::CommandError> {
    commands::list_ollama_models(state, ollama_base_url).await
}

#[tauri::command]
pub async fn list_cloud_models(
    app: tauri::AppHandle,
    state: tauri::State<'_, oclive_kernel_host::state::SharedAppState>,
    remote_url: Option<String>,
    remote_token: Option<String>,
) -> Result<Vec<String>, crate::api::error::CommandError> {
    commands::list_cloud_models(app, state, remote_url, remote_token).await
}

#[tauri::command]
pub async fn scan_local_model_files(
    state: tauri::State<'_, oclive_kernel_host::state::SharedAppState>,
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
    state: tauri::State<'_, oclive_kernel_host::state::SharedAppState>,
    req: ImportGgufToOllamaRequest,
) -> Result<String, crate::api::error::CommandError> {
    commands::import_gguf_to_ollama(state, req).await
}

#[tauri::command]
pub async fn probe_cloud_llm(
    app: tauri::AppHandle,
    state: tauri::State<'_, oclive_kernel_host::state::SharedAppState>,
    role_id: String,
    session_id: Option<String>,
) -> Result<String, crate::api::error::CommandError> {
    commands::probe_cloud_llm(app, state, role_id, session_id).await
}

#[tauri::command]
pub async fn save_llm_user_settings(
    app: tauri::AppHandle,
    state: tauri::State<'_, oclive_kernel_host::state::SharedAppState>,
    req: SaveLlmUserSettingsRequest,
) -> Result<oclive_kernel_types::models::dto::RoleInfo, crate::api::error::CommandError> {
    commands::save_llm_user_settings(app, state, req).await
}

#[tauri::command]
pub async fn get_global_ollama_model(
    app: tauri::AppHandle,
    state: tauri::State<'_, oclive_kernel_host::state::SharedAppState>,
) -> Result<GlobalOllamaModelDto, crate::api::error::CommandError> {
    commands::get_global_ollama_model(app, state).await
}

#[tauri::command]
pub async fn set_global_ollama_model(
    app: tauri::AppHandle,
    state: tauri::State<'_, oclive_kernel_host::state::SharedAppState>,
    req: SetGlobalOllamaModelRequest,
) -> Result<GlobalOllamaModelDto, crate::api::error::CommandError> {
    commands::set_global_ollama_model(app, state, req).await
}
