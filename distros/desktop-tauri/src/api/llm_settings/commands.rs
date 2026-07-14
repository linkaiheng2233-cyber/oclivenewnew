//! Tauri commands and DTOs for user-facing LLM / model settings.

use super::canonical_llm_sync::{
    sync_session_ollama_model_to_canonical, sync_shell_llm_settings_to_canonical,
};
use super::llm_models::{
    local_models_dir_for_state, model_name_from_gguf_path, scan_local_model_files_in,
};
use crate::api::error::CommandError;
use crate::error::AppError;
use crate::kernel_attach::KernelHttpClient;
use crate::kernel_lifecycle::SharedKernelConnection;
use oclive_kernel_host::infrastructure::ollama_client::OllamaClient;
use oclive_kernel_host::service::{
    get_global_ollama_model_impl, get_llm_user_settings_impl, list_cloud_models_impl,
    list_ollama_models_impl, probe_cloud_llm_impl, save_llm_user_settings_impl, session_namespace,
    set_global_ollama_model_impl, GlobalOllamaModelDto, SetGlobalOllamaModelRequest,
};
use oclive_kernel_host::state::SharedAppState;
use oclive_kernel_types::models::dto::RoleInfo;
use serde::Deserialize;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager, State};

pub use oclive_kernel_host::infrastructure::llm_models::LocalModelFileDto;
pub use oclive_kernel_host::service::{LlmUserSettingsDto, SaveLlmUserSettingsRequest};

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportGgufToOllamaRequest {
    pub file_path: String,
    pub model_name: Option<String>,
    pub ollama_base_url: Option<String>,
}

/// # Errors
///
/// Returns [`Err`] when persistence or role reload fails.
pub async fn get_llm_user_settings(
    app: AppHandle,
    state: State<'_, SharedAppState>,
    role_id: String,
    session_id: Option<String>,
) -> Result<LlmUserSettingsDto, CommandError> {
    if let Some(conn) = app.try_state::<SharedKernelConnection>() {
        return KernelHttpClient::get_llm_user_settings_via_http(
            &conn,
            role_id.as_str(),
            session_id.as_deref(),
        )
        .await
        .map_err(Into::into);
    }
    get_llm_user_settings_impl(state.inner(), role_id.as_str(), session_id.as_deref()).await
}

/// # Errors
///
/// Returns [`Err`] when Ollama list fails.
pub async fn list_ollama_models(
    state: State<'_, SharedAppState>,
    ollama_base_url: Option<String>,
) -> Result<Vec<String>, CommandError> {
    list_ollama_models_impl(state.inner(), ollama_base_url.as_deref()).await
}

/// # Errors
///
/// Returns [`Err`] when cloud credentials are missing or the provider list request fails.
pub async fn list_cloud_models(
    app: AppHandle,
    state: State<'_, SharedAppState>,
    remote_url: Option<String>,
    remote_token: Option<String>,
) -> Result<Vec<String>, CommandError> {
    if let Some(conn) = app.try_state::<SharedKernelConnection>() {
        return KernelHttpClient::list_cloud_models_via_http(
            &conn,
            remote_url.as_deref(),
            remote_token.as_deref(),
        )
        .await
        .map_err(Into::into);
    }
    list_cloud_models_impl(
        state.inner(),
        remote_url.as_deref(),
        remote_token.as_deref(),
    )
    .await
}

/// # Errors
///
/// Returns [`Err`] when the directory cannot be read.
pub async fn scan_local_model_files(
    state: State<'_, SharedAppState>,
    directory: Option<String>,
) -> Result<Vec<LocalModelFileDto>, CommandError> {
    let dir = if let Some(d) = directory.filter(|s| !s.trim().is_empty()) {
        d
    } else {
        local_models_dir_for_state(state.inner()).await?
    };
    Ok(scan_local_model_files_in(Path::new(dir.trim())))
}

/// # Errors
///
/// Returns [`Err`] when the shell cannot open the path.
pub async fn open_path_in_file_manager(path: String, app: AppHandle) -> Result<(), CommandError> {
    let p = path.trim();
    if p.is_empty() {
        return Err(AppError::InvalidParameter("empty path".into()).into());
    }
    use tauri_plugin_opener::OpenerExt;
    app.opener()
        .open_path(p, None::<&str>)
        .map_err(|e| CommandError::from(AppError::InvalidParameter(format!("shell open: {e}"))))?;
    Ok(())
}

/// # Errors
///
/// Returns [`Err`] when Ollama create fails.
pub async fn import_gguf_to_ollama(
    state: State<'_, SharedAppState>,
    req: ImportGgufToOllamaRequest,
) -> Result<String, CommandError> {
    use oclive_kernel_host::domain::user_llm_env::ollama_base_from_db_or_env;

    let path = PathBuf::from(req.file_path.trim());
    if !path.is_file() {
        return Err(AppError::InvalidParameter("model file not found".into()).into());
    }
    let base = req
        .ollama_base_url
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_default();
    let base = if base.is_empty() {
        ollama_base_from_db_or_env(state.inner()).await
    } else {
        base
    };
    let name = req
        .model_name
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| model_name_from_gguf_path(&path));
    let client = OllamaClient::new(base);
    client
        .create_model_from_path(name.trim(), &path.to_string_lossy())
        .await?;
    Ok(name.trim().to_string())
}

/// # Errors
///
/// Returns [`Err`] when cloud LLM is misconfigured or the probe request fails.
pub async fn probe_cloud_llm(
    app: AppHandle,
    state: State<'_, SharedAppState>,
    role_id: String,
    session_id: Option<String>,
) -> Result<String, CommandError> {
    if let Some(conn) = app.try_state::<SharedKernelConnection>() {
        KernelHttpClient::probe_cloud_llm_via_http(&conn, role_id.as_str(), session_id.as_deref())
            .await?;
        return Ok("ok".to_string());
    }
    probe_cloud_llm_impl(state.inner(), role_id.as_str(), session_id.as_deref()).await?;
    Ok("ok".to_string())
}

/// # Errors
///
/// Returns [`Err`] when persistence or role reload fails.
pub async fn save_llm_user_settings(
    app: AppHandle,
    state: State<'_, SharedAppState>,
    req: SaveLlmUserSettingsRequest,
) -> Result<RoleInfo, CommandError> {
    if let Some(conn) = app.try_state::<SharedKernelConnection>() {
        // Kernel `:8420` is the single writer; avoid post-save canonical mirror + env re-apply here
        // (deep async stack on Windows dev builds has caused main-thread stack overflow).
        return KernelHttpClient::save_llm_user_settings_via_http(&conn, &req)
            .await
            .map_err(Into::into);
    }

    let provider = req.provider.trim().to_ascii_lowercase();
    let model_for_session = if provider == "cloud" {
        req.remote_model.as_deref().or(req.ollama_model.as_deref())
    } else {
        req.ollama_model.as_deref()
    };

    let info = save_llm_user_settings_impl(state.inner(), &req).await?;

    let ns = session_namespace(&req.role_id, req.session_id.as_deref());
    let session_model = model_for_session
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(str::to_string);
    sync_shell_llm_settings_to_canonical(state.inner()).await;
    sync_session_ollama_model_to_canonical(ns.as_str(), session_model.as_deref()).await;
    if let Some(conn) = app.try_state::<crate::kernel_lifecycle::SharedKernelConnection>() {
        if let Err(e) = crate::kernel_attach::KernelHttpClient::reload_llm_via_http(&conn).await {
            tracing::warn!(
                target: "oclive_llm",
                error = %e,
                "kernel LLM reload after save failed"
            );
        }
    }

    Ok(info)
}

/// # Errors
///
/// Returns [`Err`] when app settings cannot be read.
pub async fn get_global_ollama_model(
    app: AppHandle,
    state: State<'_, SharedAppState>,
) -> Result<GlobalOllamaModelDto, CommandError> {
    if let Some(conn) = app.try_state::<SharedKernelConnection>() {
        return KernelHttpClient::get_global_ollama_model_via_http(&conn)
            .await
            .map_err(Into::into);
    }
    get_global_ollama_model_impl(state.inner()).await
}

/// # Errors
///
/// Returns [`Err`] when persistence fails or model name is empty.
pub async fn set_global_ollama_model(
    app: AppHandle,
    state: State<'_, SharedAppState>,
    req: SetGlobalOllamaModelRequest,
) -> Result<GlobalOllamaModelDto, CommandError> {
    if let Some(conn) = app.try_state::<SharedKernelConnection>() {
        return KernelHttpClient::set_global_ollama_model_via_http(&conn, &req)
            .await
            .map_err(Into::into);
    }
    set_global_ollama_model_impl(state.inner(), &req).await
}
