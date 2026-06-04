//! Tauri commands and DTOs for user-facing LLM / model settings.

use super::canonical_llm_sync::{
    sync_session_ollama_model_to_canonical, sync_shell_llm_settings_to_canonical,
};
use super::llm_models::{
    canonical_models_dir, local_models_dir_for_state, model_name_from_gguf_path,
    persist_local_models_dir, scan_local_model_files_in, LocalModelFileDto,
};
use super::user_llm_env::probe_cloud_llm_inner;
use crate::domain::user_llm_env::{
    apply_user_llm_env, cloud_api_token_configured, ollama_base_from_db_or_env,
    resolve_remote_token, KEY_CLOUD_STYLE, KEY_CLOUD_VENDOR, KEY_LLM_PROVIDER, KEY_REMOTE_MODEL,
    KEY_REMOTE_TOKEN, KEY_REMOTE_URL,
};
use crate::api::error::CommandError;
use crate::api::role::{get_role_info_impl, session_namespace};
use crate::domain::effective_llm_model::resolve_effective_ollama_model;
use crate::error::AppError;
use crate::infrastructure::ollama_client::OllamaClient;
use crate::infrastructure::user_llm_secrets::{set_cached_remote_llm_token, write_token_file};
use crate::models::dto::RoleInfo;
use crate::models::plugin_backends::LlmBackend;
use crate::state::{is_managed_legacy_models_path, migrate_and_cleanup_models, SharedAppState};
use oclive_validation::NETWORK_GRANT_REMOTE_LLM;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager, State};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LlmUserSettingsDto {
    pub provider: String,
    pub cloud_vendor: String,
    pub cloud_api_style: String,
    pub ollama_base_url: String,
    pub ollama_reachable: bool,
    pub ollama_detail: String,
    pub local_models_dir: String,
    pub local_model_files: Vec<LocalModelFileDto>,
    pub pack_ollama_model: Option<String>,
    pub session_ollama_model: Option<String>,
    pub effective_model: String,
    pub remote_url: String,
    pub remote_token_configured: bool,
    pub remote_model: String,
    pub remote_url_env_active: bool,
    pub remote_token_env_active: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveLlmUserSettingsRequest {
    pub role_id: String,
    pub session_id: Option<String>,
    pub provider: String,
    pub cloud_vendor: Option<String>,
    pub cloud_api_style: Option<String>,
    pub ollama_base_url: Option<String>,
    pub local_models_dir: Option<String>,
    pub ollama_model: Option<String>,
    pub remote_url: Option<String>,
    pub remote_token: Option<String>,
    pub remote_model: Option<String>,
}

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
    state: State<'_, SharedAppState>,
    role_id: String,
    session_id: Option<String>,
) -> Result<LlmUserSettingsDto, CommandError> {
    let role = state.load_role_cached_async(&role_id).await?;
    let ns = session_namespace(&role_id, session_id.as_deref());
    state.db_manager.ensure_role_runtime(ns.as_str()).await?;

    let effective =
        resolve_effective_ollama_model(state.inner(), role.as_ref(), ns.as_str()).await?;
    let plugin_backends_effective =
        state.effective_plugin_backends_for_session(role.as_ref(), ns.as_str());
    let provider = state
        .db_manager
        .get_app_setting(KEY_LLM_PROVIDER)
        .await?
        .map(|s| s.trim().to_ascii_lowercase())
        .filter(|p| p == "local" || p == "cloud")
        .unwrap_or_else(|| {
            if matches!(plugin_backends_effective.llm, LlmBackend::Remote) {
                "cloud".to_string()
            } else {
                "local".to_string()
            }
        });

    let ollama_base_url = ollama_base_from_db_or_env(state.inner()).await;
    let client = OllamaClient::new(ollama_base_url.clone());
    let ollama_reachable = client.health_check().await.unwrap_or(false);
    let ollama_detail = if ollama_reachable {
        String::new()
    } else {
        "Ollama unreachable".to_string()
    };

    let session_ollama_model = state
        .db_manager
        .get_session_ollama_model_override(ns.as_str())
        .await?;
    let pack_ollama_model = role.ollama_model.clone();

    let remote_url = state
        .db_manager
        .get_app_setting(KEY_REMOTE_URL)
        .await?
        .unwrap_or_default();
    let remote_model = state
        .db_manager
        .get_app_setting(KEY_REMOTE_MODEL)
        .await?
        .unwrap_or_default();
    let remote_token_configured = state
        .db_manager
        .get_app_setting(KEY_REMOTE_TOKEN)
        .await?
        .map(|s| !s.trim().is_empty())
        .unwrap_or(false);
    let cloud_vendor = state
        .db_manager
        .get_app_setting(KEY_CLOUD_VENDOR)
        .await?
        .unwrap_or_else(|| "custom".to_string());
    let cloud_api_style = state
        .db_manager
        .get_app_setting(KEY_CLOUD_STYLE)
        .await?
        .unwrap_or_else(|| "openai".to_string());
    let local_models_dir = local_models_dir_for_state(state.inner()).await?;
    let local_model_files = scan_local_model_files_in(Path::new(local_models_dir.trim()));

    Ok(LlmUserSettingsDto {
        provider: provider.to_string(),
        cloud_vendor,
        cloud_api_style,
        ollama_base_url,
        ollama_reachable,
        ollama_detail,
        local_models_dir,
        local_model_files,
        pack_ollama_model,
        session_ollama_model,
        effective_model: effective,
        remote_url,
        remote_token_configured,
        remote_model,
        remote_url_env_active: std::env::var("OCLIVE_REMOTE_LLM_URL")
            .map(|s| !s.trim().is_empty())
            .unwrap_or(false),
        remote_token_env_active: std::env::var("OCLIVE_REMOTE_LLM_TOKEN")
            .map(|s| !s.trim().is_empty())
            .unwrap_or(false),
    })
}

/// # Errors
///
/// Returns [`Err`] when Ollama list fails.
pub async fn list_ollama_models(
    state: State<'_, SharedAppState>,
    ollama_base_url: Option<String>,
) -> Result<Vec<String>, CommandError> {
    let base = ollama_base_url
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_default();
    let base = if base.is_empty() {
        ollama_base_from_db_or_env(state.inner()).await
    } else {
        base
    };
    OllamaClient::new(base)
        .list_models()
        .await
        .map_err(CommandError::from)
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
pub async fn open_path_in_file_manager(
    path: String,
    app: AppHandle,
) -> Result<(), CommandError> {
    let p = path.trim();
    if p.is_empty() {
        return Err(AppError::InvalidParameter("empty path".into()).into());
    }
    tauri::api::shell::open(&app.shell_scope(), p, None).map_err(|e| {
        CommandError::from(AppError::InvalidParameter(format!("shell open: {e}")))
    })?;
    Ok(())
}

/// # Errors
///
/// Returns [`Err`] when Ollama create fails.
pub async fn import_gguf_to_ollama(
    state: State<'_, SharedAppState>,
    req: ImportGgufToOllamaRequest,
) -> Result<String, CommandError> {
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
    state: State<'_, SharedAppState>,
    role_id: String,
    session_id: Option<String>,
) -> Result<String, CommandError> {
    probe_cloud_llm_inner(state.inner(), role_id.as_str(), session_id.as_deref()).await?;
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
    let provider = req.provider.trim().to_ascii_lowercase();
    if provider != "local" && provider != "cloud" {
        return Err(AppError::InvalidParameter(
            "provider must be local or cloud".into(),
        )
        .into());
    }

    state
        .db_manager
        .upsert_app_setting(KEY_LLM_PROVIDER, &provider)
        .await?;

    if provider == "cloud" {
        let url_ok = req
            .remote_url
            .as_deref()
            .is_some_and(|s| !s.trim().is_empty())
            || state
                .db_manager
                .get_app_setting(KEY_REMOTE_URL)
                .await?
                .is_some_and(|s| !s.trim().is_empty());
        if !url_ok {
            return Err(AppError::InvalidParameter("云端 Base URL 不能为空".into()).into());
        }
        if !cloud_api_token_configured(&state.db_manager, req.remote_token.as_deref()).await? {
            return Err(AppError::InvalidParameter(
                "请填写云端 API Key 后再保存".into(),
            )
            .into());
        }
        state
            .high_risk_grants
            .require_network(NETWORK_GRANT_REMOTE_LLM)?;
    }

    if let Some(ref url) = req.ollama_base_url {
        state
            .db_manager
            .upsert_app_setting(crate::domain::user_llm_env::KEY_OLLAMA_BASE, url.trim())
            .await?;
    }
    if let Some(ref dir) = req.local_models_dir {
        let trimmed = dir.trim();
        let canonical = canonical_models_dir(state.inner());
        let canonical_str = canonical.to_string_lossy().into_owned();
        let app_data = state.directory_plugins.app_data_dir();
        if trimmed.is_empty() {
            persist_local_models_dir(state.inner(), &canonical_str).await?;
        } else {
            let stored = PathBuf::from(trimmed);
            if is_managed_legacy_models_path(&stored, &canonical, app_data) {
                migrate_and_cleanup_models(&stored, &canonical);
                persist_local_models_dir(state.inner(), &canonical_str).await?;
            } else {
                persist_local_models_dir(state.inner(), trimmed).await?;
            }
        }
    }
    if let Some(ref url) = req.remote_url {
        state
            .db_manager
            .upsert_app_setting(KEY_REMOTE_URL, url.trim())
            .await?;
    }
    if let Some(ref token) = req.remote_token {
        let t = token.trim();
        if !t.is_empty() {
            state
                .db_manager
                .upsert_app_setting(KEY_REMOTE_TOKEN, t)
                .await?;
            write_token_file(state.directory_plugins.app_data_dir(), t)
                .map_err(|e| AppError::InvalidParameter(format!("save API token: {e}")))?;
            set_cached_remote_llm_token(Some(t.to_string()));
            let read_back = state
                .db_manager
                .get_app_setting(KEY_REMOTE_TOKEN)
                .await?
                .filter(|s| !s.trim().is_empty());
            if read_back.as_deref() != Some(t) {
                return Err(AppError::InvalidParameter(
                    "API Key 未能写入数据库，请重试保存".into(),
                )
                .into());
            }
        }
    } else if provider == "cloud" {
        let app_data = state.directory_plugins.app_data_dir();
        let existing = resolve_remote_token(&state.db_manager, app_data).await?;
        set_cached_remote_llm_token(existing);
    }
    if let Some(ref model) = req.remote_model {
        state
            .db_manager
            .upsert_app_setting(KEY_REMOTE_MODEL, model.trim())
            .await?;
    }
    if let Some(ref vendor) = req.cloud_vendor {
        state
            .db_manager
            .upsert_app_setting(KEY_CLOUD_VENDOR, vendor.trim())
            .await?;
    }
    if let Some(ref style) = req.cloud_api_style {
        let s = style.trim().to_ascii_lowercase();
        let normalized = if s == "oclive_jsonrpc" {
            "oclive_jsonrpc"
        } else {
            "openai"
        };
        state
            .db_manager
            .upsert_app_setting(KEY_CLOUD_STYLE, normalized)
            .await?;
    } else if provider == "cloud" {
        state
            .db_manager
            .upsert_app_setting(KEY_CLOUD_STYLE, "openai")
            .await?;
    }

    state.mark_user_llm_env_dirty();
    apply_user_llm_env(state.inner()).await?;

    let ns = session_namespace(&req.role_id, req.session_id.as_deref());
    state.db_manager.ensure_role_runtime(ns.as_str()).await?;

    let model_for_session = if provider == "cloud" {
        req.remote_model
            .as_deref()
            .or(req.ollama_model.as_deref())
    } else {
        req.ollama_model.as_deref()
    };
    if let Some(model) = model_for_session {
        let t = model.trim();
        if t.is_empty() {
            state
                .db_manager
                .clear_session_ollama_model_override(ns.as_str())
                .await?;
        } else {
            state
                .db_manager
                .set_session_ollama_model_override(ns.as_str(), t)
                .await?;
        }
    }

    let backend = if provider == "cloud" {
        "remote"
    } else {
        "ollama"
    };
    let info = match crate::api::role::slot_session::set_session_plugin_backend_impl(
        state.inner(),
        &crate::models::dto::SetSessionPluginBackendRequest {
            role_id: req.role_id.clone(),
            module: "llm".to_string(),
            backend: Some(Some(backend.to_string())),
            local_memory_provider_id: None,
            session_id: req.session_id.clone(),
        },
    )
    .await
    {
        Ok(info) => info,
        Err(e) if e.to_string().contains("slot_registry") => {
            let ns = session_namespace(&req.role_id, req.session_id.as_deref());
            let llm_backend = if provider == "cloud" {
                LlmBackend::Remote
            } else {
                LlmBackend::Ollama
            };
            state.set_session_backend_override(
                ns.as_str(),
                crate::models::PluginBackendsOverride {
                    llm: Some(llm_backend),
                    ..Default::default()
                },
            );
            get_role_info_impl(state.inner(), &req.role_id, req.session_id.as_deref()).await?
        }
        Err(e) => return Err(e),
    };

    if provider == "cloud" {
        probe_cloud_llm_inner(
            state.inner(),
            req.role_id.as_str(),
            req.session_id.as_deref(),
        )
        .await?;
    }

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
