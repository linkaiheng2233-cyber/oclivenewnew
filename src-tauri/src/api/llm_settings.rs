//! User-facing LLM / model settings (local Ollama + cloud OpenAI-compatible / JSON-RPC).

use crate::api::error::CommandError;
use crate::api::role::{get_role_info_impl, session_namespace};
use crate::domain::effective_llm_model::resolve_effective_ollama_model;
use crate::error::AppError;
use crate::infrastructure::ollama_client::OllamaClient;
use crate::infrastructure::user_llm_secrets::{
    self, read_token_file, set_cached_remote_llm_token, write_token_file,
};
use crate::models::dto::RoleInfo;
use crate::models::plugin_backends::LlmBackend;
use crate::state::AppState;
use oclive_validation::NETWORK_GRANT_REMOTE_LLM;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager, State};

const KEY_OLLAMA_BASE: &str = "user_ollama_base_url";
pub(crate) const KEY_REMOTE_URL: &str = "user_remote_llm_url";
const KEY_REMOTE_TOKEN: &str = "user_remote_llm_token";
pub(crate) const KEY_REMOTE_MODEL: &str = "user_remote_llm_model";
const KEY_CLOUD_STYLE: &str = "user_llm_cloud_api_style";
const KEY_CLOUD_VENDOR: &str = "user_llm_cloud_vendor";
pub(crate) const KEY_LLM_PROVIDER: &str = "user_llm_provider";
const KEY_LOCAL_MODELS_DIR: &str = "user_local_models_dir";

async fn ollama_base_from_db_or_env(state: &AppState) -> String {
    if let Ok(Some(v)) = state.db_manager.get_app_setting(KEY_OLLAMA_BASE).await {
        let t = v.trim();
        if !t.is_empty() {
            return t.to_string();
        }
    }
    std::env::var("OLLAMA_BASE_URL").unwrap_or_else(|_| "http://localhost:11434".to_string())
}

/// Re-apply saved user LLM env into the current process (desktop UI saves).
/// Returns resolved provider: `cloud` | `local` | empty.
///
/// # Errors
///
/// Returns [`crate::error::AppError`] when app settings cannot be read from the database.
pub async fn apply_user_llm_env_from_db(
    db: &crate::infrastructure::db::DbManager,
) -> crate::error::Result<String> {
    let pairs = [
        (KEY_OLLAMA_BASE, "OLLAMA_BASE_URL"),
        (KEY_REMOTE_URL, "OCLIVE_REMOTE_LLM_URL"),
        (KEY_REMOTE_TOKEN, "OCLIVE_REMOTE_LLM_TOKEN"),
        (KEY_CLOUD_STYLE, "OCLIVE_LLM_CLOUD_API_STYLE"),
    ];
    for (db_key, env_key) in pairs {
        if let Some(v) = db.get_app_setting(db_key).await? {
            let t = v.trim();
            if t.is_empty() {
                std::env::remove_var(env_key);
            } else {
                std::env::set_var(env_key, t);
            }
        }
    }
    let remote_url = db
        .get_app_setting(KEY_REMOTE_URL)
        .await?
        .map(|s| s.trim().to_string())
        .unwrap_or_default();
    let mut provider = db
        .get_app_setting(KEY_LLM_PROVIDER)
        .await?
        .map(|s| s.trim().to_ascii_lowercase())
        .unwrap_or_default();
    if provider.is_empty()
        && !remote_url.is_empty()
        && cloud_api_token_configured(db, None).await?
    {
        provider = "cloud".to_string();
        let _ = db
            .upsert_app_setting(KEY_LLM_PROVIDER, "cloud")
            .await;
    }
    match provider.as_str() {
        "cloud" if !remote_url.is_empty() => {
            std::env::set_var("OCLIVE_LLM_BACKEND", "remote");
        }
        "local" => {
            std::env::set_var("OCLIVE_LLM_BACKEND", "ollama");
        }
        _ => {
            std::env::remove_var("OCLIVE_LLM_BACKEND");
        }
    }
    Ok(provider)
}

async fn resolve_remote_token(
    db: &crate::infrastructure::db::DbManager,
    app_data: &std::path::Path,
) -> crate::error::Result<Option<String>> {
    let from_db = db.get_app_setting(KEY_REMOTE_TOKEN).await?;
    if from_db
        .as_ref()
        .is_some_and(|s| !s.trim().is_empty())
    {
        return Ok(from_db);
    }
    Ok(read_token_file(app_data))
}

/// Apply DB LLM settings and sync [`AppState::user_llm_provider`].
///
/// # Errors
///
/// Returns [`crate::error::AppError`] when settings or token resolution fails.
pub async fn apply_user_llm_env(state: &AppState) -> crate::error::Result<()> {
    let app_data = state.directory_plugins.app_data_dir();
    let token = resolve_remote_token(state.db_manager.as_ref(), app_data).await?;
    if let Some(ref t) = token {
        set_cached_remote_llm_token(Some(t.clone()));
        state
            .db_manager
            .upsert_app_setting(KEY_REMOTE_TOKEN, t.trim())
            .await?;
        let _ = write_token_file(app_data, t.trim());
    } else {
        set_cached_remote_llm_token(None);
    }
    let provider = apply_user_llm_env_from_db(state.db_manager.as_ref()).await?;
    tracing::info!(
        target: "oclive_llm",
        provider = %provider,
        remote_url_configured = std::env::var("OCLIVE_REMOTE_LLM_URL")
            .map(|s| !s.trim().is_empty())
            .unwrap_or(false),
        remote_token_configured = user_llm_secrets::cached_remote_llm_token().is_some(),
        "apply_user_llm_env"
    );
    *state.user_llm_provider.write() = provider;
    Ok(())
}

async fn cloud_api_token_configured(
    db: &crate::infrastructure::db::DbManager,
    req_token: Option<&str>,
) -> crate::error::Result<bool> {
    if req_token.is_some_and(|s| !s.trim().is_empty()) {
        return Ok(true);
    }
    Ok(db
        .get_app_setting(KEY_REMOTE_TOKEN)
        .await?
        .is_some_and(|s| !s.trim().is_empty()))
}

/// Ping cloud LLM with current DB/env settings (after [`apply_user_llm_env_from_db`]).
async fn probe_cloud_llm_inner(
    state: &AppState,
    role_id: &str,
    session_id: Option<&str>,
) -> crate::error::Result<()> {
    apply_user_llm_env(state).await?;
    if std::env::var("OCLIVE_REMOTE_LLM_URL")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .is_none()
    {
        return Err(AppError::InvalidParameter(
            "云端 Base URL 未配置".into(),
        ));
    }
    if std::env::var("OCLIVE_REMOTE_LLM_TOKEN")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .is_none()
    {
        return Err(AppError::InvalidParameter(
            "云端 API Key 未配置，请在模型管理中填写并保存".into(),
        ));
    }
    state
        .high_risk_grants
        .grant_network(NETWORK_GRANT_REMOTE_LLM)
        .map_err(AppError::InvalidParameter)?;

    let role = state.load_role_cached_async(role_id).await?;
    let ns = session_namespace(role_id, session_id);
    let model = resolve_effective_ollama_model(state, role.as_ref(), ns.as_str()).await?;
    if model.trim().is_empty() {
        return Err(AppError::InvalidParameter("云端模型名为空".into()));
    }
    let backends = state.effective_plugin_backends_for_session(role.as_ref(), ns.as_str());
    if !matches!(backends.llm, LlmBackend::Remote) {
        return Err(AppError::InvalidParameter(format!(
            "当前 LLM 后端未切到云端（{:?}），请重新保存模型管理中的云端配置",
            backends.llm
        )));
    }
    let llm = state.plugins.llm_for_plugin_backends(backends.as_ref());
    llm.generate(model.trim(), "请只回复一个字：好")
        .await
        .map(|_| ())
        .map_err(|e| {
            AppError::InvalidParameter(format!(
                "云端模型连通性测试失败：{}",
                e.to_frontend_error()
            ))
        })
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalModelFileDto {
    pub name: String,
    pub path: String,
    pub size_bytes: u64,
}

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

fn scan_local_model_files_in(dir: &Path) -> Vec<LocalModelFileDto> {
    let mut out = Vec::new();
    let Ok(rd) = std::fs::read_dir(dir) else {
        return out;
    };
    for entry in rd.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        if ext != "gguf" && ext != "bin" {
            continue;
        }
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("model")
            .to_string();
        let size_bytes = entry.metadata().map(|m| m.len()).unwrap_or(0);
        out.push(LocalModelFileDto {
            name,
            path: path.to_string_lossy().into_owned(),
            size_bytes,
        });
    }
    out.sort_by_key(|a| a.name.to_lowercase());
    out
}

fn model_name_from_gguf_path(path: &Path) -> String {
    path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("local-model")
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

/// # Errors
///
/// Returns [`Err`] when persistence or role reload fails.
#[tauri::command]
pub async fn get_llm_user_settings(
    state: State<'_, AppState>,
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
    let local_models_dir = state
        .db_manager
        .get_app_setting(KEY_LOCAL_MODELS_DIR)
        .await?
        .unwrap_or_default();
    let local_model_files = if local_models_dir.trim().is_empty() {
        Vec::new()
    } else {
        scan_local_model_files_in(Path::new(local_models_dir.trim()))
    };

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
#[tauri::command]
pub async fn list_ollama_models(
    state: State<'_, AppState>,
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
#[tauri::command]
pub async fn scan_local_model_files(
    state: State<'_, AppState>,
    directory: Option<String>,
) -> Result<Vec<LocalModelFileDto>, CommandError> {
    let dir = if let Some(d) = directory.filter(|s| !s.trim().is_empty()) {
        d
    } else {
        state
            .db_manager
            .get_app_setting(KEY_LOCAL_MODELS_DIR)
            .await?
            .unwrap_or_default()
    };
    if dir.trim().is_empty() {
        return Ok(Vec::new());
    }
    Ok(scan_local_model_files_in(Path::new(dir.trim())))
}

/// # Errors
///
/// Returns [`Err`] when the shell cannot open the path.
#[tauri::command]
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
#[tauri::command]
pub async fn import_gguf_to_ollama(
    state: State<'_, AppState>,
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
#[tauri::command]
pub async fn probe_cloud_llm(
    state: State<'_, AppState>,
    role_id: String,
    session_id: Option<String>,
) -> Result<String, CommandError> {
    probe_cloud_llm_inner(state.inner(), role_id.as_str(), session_id.as_deref()).await?;
    Ok("ok".to_string())
}

/// # Errors
///
/// Returns [`Err`] when persistence or role reload fails.
#[tauri::command]
pub async fn save_llm_user_settings(
    state: State<'_, AppState>,
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
            .grant_network(NETWORK_GRANT_REMOTE_LLM)
            .map_err(AppError::InvalidParameter)?;
    }

    if let Some(ref url) = req.ollama_base_url {
        state
            .db_manager
            .upsert_app_setting(KEY_OLLAMA_BASE, url.trim())
            .await?;
    }
    if let Some(ref dir) = req.local_models_dir {
        state
            .db_manager
            .upsert_app_setting(KEY_LOCAL_MODELS_DIR, dir.trim())
            .await?;
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

    Ok(info)
}
