//! LLM / model settings impls shared by HTTP routes and Tauri invoke.

use crate::command_error::CommandError;
use crate::domain::effective_llm_model::{is_usable_ollama_model_id, resolve_effective_ollama_model};
use crate::domain::user_llm_env::{
    apply_user_llm_env, cloud_api_token_configured, load_remote_token, ollama_base_from_db_or_env,
    KEY_CLOUD_STYLE, KEY_CLOUD_VENDOR, KEY_GLOBAL_OLLAMA_MODEL, KEY_LLM_PROVIDER, KEY_OLLAMA_BASE,
    KEY_REMOTE_MODEL, KEY_REMOTE_TOKEN, KEY_REMOTE_URL,
};
use crate::error::AppError;
use crate::infrastructure::llm_models::{
    canonical_models_dir, local_models_dir_for_state, persist_local_models_dir,
    scan_local_model_files_in, LocalModelFileDto,
};
use crate::infrastructure::ollama_client::OllamaClient;
use crate::infrastructure::openai_compatible_llm::list_openai_compatible_models;
use crate::infrastructure::user_llm_secrets::{set_cached_remote_llm_token, write_token_file};
use crate::models::dto::{RoleInfo, SetSessionPluginBackendRequest};
use crate::service::role::{
    get_role_info_impl, session_namespace, set_session_plugin_backend_impl,
};
use crate::state::{is_managed_legacy_models_path, migrate_and_cleanup_models, AppState};
use oclive_kernel_types::models::plugin_backends::LlmBackend;
use oclive_kernel_types::models::PluginBackendsOverride;
use oclive_validation::NETWORK_GRANT_REMOTE_LLM;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Deserialize, Serialize)]
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
pub struct SetSessionLlmModelRequest {
    pub role_id: String,
    #[serde(default)]
    pub session_id: Option<String>,
    /// Model id; null/empty clears override.
    #[serde(default)]
    pub model: Option<String>,
}

/// # Errors
///
/// Returns [`Err`] when persistence or role reload fails.
pub async fn get_llm_user_settings_impl(
    state: &AppState,
    role_id: &str,
    session_id: Option<&str>,
) -> Result<LlmUserSettingsDto, CommandError> {
    let role = state.load_role_cached_async(role_id).await?;
    let ns = session_namespace(role_id, session_id);
    state.db_manager.ensure_role_runtime(ns.as_str()).await?;

    let mut session_ollama_model = state
        .db_manager
        .get_session_ollama_model_override(ns.as_str())
        .await?;
    if session_ollama_model
        .as_deref()
        .is_some_and(|m| !is_usable_ollama_model_id(m))
    {
        state
            .db_manager
            .clear_session_ollama_model_override(ns.as_str())
            .await?;
        session_ollama_model = None;
    }

    let effective = resolve_effective_ollama_model(state, role.as_ref(), ns.as_str()).await?;
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

    let ollama_base_url = ollama_base_from_db_or_env(state).await;
    let client = OllamaClient::new(ollama_base_url.clone());
    let ollama_reachable = client.health_check().await.unwrap_or(false);
    let ollama_detail = if ollama_reachable {
        String::new()
    } else {
        "Ollama unreachable".to_string()
    };

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
    let local_models_dir = local_models_dir_for_state(state).await?;
    let local_model_files =
        scan_local_model_files_in(std::path::Path::new(local_models_dir.trim()));

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
pub async fn list_ollama_models_impl(
    state: &AppState,
    ollama_base_url: Option<&str>,
) -> Result<Vec<String>, CommandError> {
    let base = ollama_base_url
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_default();
    let base = if base.is_empty() {
        ollama_base_from_db_or_env(state).await
    } else {
        base.to_string()
    };
    OllamaClient::new(base)
        .list_models()
        .await
        .map_err(CommandError::from)
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListCloudModelsRequest {
    pub remote_url: Option<String>,
    pub remote_token: Option<String>,
}

/// List model ids from the configured OpenAI-compatible cloud endpoint.
///
/// # Errors
///
/// Returns [`Err`] when URL/token are missing or the provider request fails.
pub async fn list_cloud_models_impl(
    state: &AppState,
    remote_url: Option<&str>,
    remote_token: Option<&str>,
) -> Result<Vec<String>, CommandError> {
    let url = if let Some(u) = remote_url.filter(|s| !s.trim().is_empty()) {
        u.trim().to_string()
    } else {
        state
            .db_manager
            .get_app_setting(KEY_REMOTE_URL)
            .await?
            .unwrap_or_default()
    };
    if url.trim().is_empty() {
        return Err(AppError::InvalidParameter("云端 Base URL 未配置".into()).into());
    }

    let token = if let Some(t) = remote_token.filter(|s| !s.trim().is_empty()) {
        Some(t.trim().to_string())
    } else {
        let app_data = state.directory_plugins.app_data_dir();
        let settings = crate::infrastructure::db_ports::DbSettingsPort(state.db_manager.as_ref());
        load_remote_token(&settings, state.user_llm_secrets.as_ref(), app_data).await?
    };
    if token.as_deref().unwrap_or("").trim().is_empty() {
        return Err(
            AppError::InvalidParameter("请先填写或保存 API Key 后再拉取模型列表".into()).into(),
        );
    }

    if let Err(e) = state
        .high_risk_grants
        .grant_network(NETWORK_GRANT_REMOTE_LLM)
    {
        tracing::warn!(
            target: "oclive_llm",
            error = %e,
            "auto-grant remote LLM on cloud model list failed"
        );
    }

    let timeout_ms = std::env::var("OCLIVE_REMOTE_LLM_TIMEOUT_MS")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(120_000);
    let client = Client::builder()
        .build()
        .map_err(|e| AppError::InvalidParameter(format!("HTTP client: {e}")))?;
    list_openai_compatible_models(
        &client,
        url.trim(),
        token.as_deref(),
        std::time::Duration::from_millis(timeout_ms.clamp(1_000, 600_000)),
        state.high_risk_grants.as_ref(),
    )
    .await
    .map_err(CommandError::from)
}

/// Map provider/HTTP failures to a short user-facing probe message (no nested JSON).
fn humanize_cloud_probe_error(detail: &str) -> String {
    let lower = detail.to_ascii_lowercase();
    if lower.contains("401")
        || lower.contains("authentication")
        || lower.contains("invalid api key")
        || lower.contains("api key") && (lower.contains("invalid") || lower.contains("incorrect"))
    {
        return "API Key 无效或未授权，请检查密钥是否正确".to_string();
    }
    if lower.contains("404") {
        return "Base URL 或模型 ID 可能有误（HTTP 404）".to_string();
    }
    if lower.contains("timeout") || lower.contains("timed out") {
        return "连接超时，请检查网络或 Base URL".to_string();
    }
    if lower.contains("high_risk") || lower.contains("not granted") {
        return "尚未授予云端 LLM 网络权限，请重新保存配置".to_string();
    }
    let n = detail.chars().count();
    if n > 160 {
        format!("{}…", detail.chars().take(160).collect::<String>())
    } else {
        detail.to_string()
    }
}

/// Ping cloud LLM with current DB/env settings (after [`apply_user_llm_env`]).
///
/// # Errors
///
/// Returns configuration, network, or provider errors from the probe request.
pub async fn probe_cloud_llm_impl(
    state: &AppState,
    role_id: &str,
    session_id: Option<&str>,
) -> Result<(), CommandError> {
    apply_user_llm_env(state).await?;
    if std::env::var("OCLIVE_REMOTE_LLM_URL")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .is_none()
    {
        return Err(AppError::InvalidParameter("云端 Base URL 未配置".into()).into());
    }
    if std::env::var("OCLIVE_REMOTE_LLM_TOKEN")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .is_none()
    {
        return Err(AppError::InvalidParameter(
            "云端 API Key 未配置，请在模型管理中填写并保存".into(),
        )
        .into());
    }
    let _ = state
        .high_risk_grants
        .grant_network(NETWORK_GRANT_REMOTE_LLM);

    let role = state.load_role_cached_async(role_id).await?;
    let ns = session_namespace(role_id, session_id);
    let model = resolve_effective_ollama_model(state, role.as_ref(), ns.as_str()).await?;
    if model.trim().is_empty() {
        return Err(AppError::InvalidParameter("云端模型名为空".into()).into());
    }
    let backends = state.effective_plugin_backends_for_session(role.as_ref(), ns.as_str());
    if !matches!(backends.llm, LlmBackend::Remote) {
        return Err(AppError::InvalidParameter(format!(
            "当前 LLM 后端未切到云端（{:?}），请重新保存模型管理中的云端配置",
            backends.llm
        ))
        .into());
    }
    let llm = state.plugins.llm_for_plugin_backends(backends.as_ref());
    llm.generate(model.trim(), "请只回复一个字：好")
        .await
        .map(|_| ())
        .map_err(|e| {
            let detail = humanize_cloud_probe_error(e.to_frontend_error().as_str());
            AppError::InvalidParameter(format!("云端连通性测试失败：{detail}")).into()
        })
}

async fn apply_session_model_override(
    state: &AppState,
    ns: &str,
    model: Option<&str>,
) -> Result<(), CommandError> {
    if let Some(model) = model {
        let t = model.trim();
        if t.is_empty() || !is_usable_ollama_model_id(t) {
            state
                .db_manager
                .clear_session_ollama_model_override(ns)
                .await?;
        } else {
            state
                .db_manager
                .set_session_ollama_model_override(ns, t)
                .await?;
        }
    }
    Ok(())
}

/// # Errors
///
/// Returns [`Err`] when persistence or role reload fails.
pub async fn save_llm_user_settings_impl(
    state: &AppState,
    req: &SaveLlmUserSettingsRequest,
) -> Result<RoleInfo, CommandError> {
    let provider = req.provider.trim().to_ascii_lowercase();
    if provider != "local" && provider != "cloud" {
        return Err(AppError::InvalidParameter("provider must be local or cloud".into()).into());
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
        let settings = crate::infrastructure::db_ports::DbSettingsPort(state.db_manager.as_ref());
        if !cloud_api_token_configured(&settings, req.remote_token.as_deref()).await? {
            return Err(AppError::InvalidParameter("请填写云端 API Key 后再保存".into()).into());
        }
        // BYOK save is explicit user consent for outbound LLM API calls.
        if let Err(e) = state
            .high_risk_grants
            .grant_network(NETWORK_GRANT_REMOTE_LLM)
        {
            tracing::warn!(
                target: "oclive_llm",
                error = %e,
                "auto-grant remote LLM on cloud save failed"
            );
        }
    }

    if let Some(ref url) = req.ollama_base_url {
        state
            .db_manager
            .upsert_app_setting(KEY_OLLAMA_BASE, url.trim())
            .await?;
    }
    if let Some(ref dir) = req.local_models_dir {
        let trimmed = dir.trim();
        let canonical = canonical_models_dir(state);
        let canonical_str = canonical.to_string_lossy().into_owned();
        let app_data = state.directory_plugins.app_data_dir();
        if trimmed.is_empty() {
            persist_local_models_dir(state, &canonical_str).await?;
        } else {
            let stored = PathBuf::from(trimmed);
            if is_managed_legacy_models_path(&stored, &canonical, app_data) {
                migrate_and_cleanup_models(&stored, &canonical);
                persist_local_models_dir(state, &canonical_str).await?;
            } else {
                persist_local_models_dir(state, trimmed).await?;
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
        let secrets = state.user_llm_secrets.as_ref();
        let settings = crate::infrastructure::db_ports::DbSettingsPort(state.db_manager.as_ref());
        let existing = load_remote_token(&settings, secrets, app_data).await?;
        secrets.set_cached_remote_llm_token(existing);
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
    apply_user_llm_env(state).await?;

    let ns = session_namespace(&req.role_id, req.session_id.as_deref());
    state.db_manager.ensure_role_runtime(ns.as_str()).await?;

    let model_for_session = if provider == "cloud" {
        req.remote_model.as_deref().or(req.ollama_model.as_deref())
    } else {
        req.ollama_model.as_deref()
    };
    apply_session_model_override(state, ns.as_str(), model_for_session).await?;

    let backend = if provider == "cloud" {
        "remote"
    } else {
        "ollama"
    };
    let info = match set_session_plugin_backend_impl(
        state,
        &SetSessionPluginBackendRequest {
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
            let llm_backend = if provider == "cloud" {
                LlmBackend::Remote
            } else {
                LlmBackend::Ollama
            };
            state.set_session_backend_override(
                ns.as_str(),
                PluginBackendsOverride {
                    llm: Some(llm_backend),
                    ..Default::default()
                },
            );
            get_role_info_impl(state, &req.role_id, req.session_id.as_deref()).await?
        }
        Err(e) => return Err(e),
    };

    Ok(info)
}

/// # Errors
///
/// Returns [`Err`] when persistence or role reload fails.
pub async fn set_session_llm_model_impl(
    state: &AppState,
    req: &SetSessionLlmModelRequest,
) -> Result<RoleInfo, CommandError> {
    let ns = session_namespace(&req.role_id, req.session_id.as_deref());
    state.db_manager.ensure_role_runtime(ns.as_str()).await?;
    apply_session_model_override(state, ns.as_str(), req.model.as_deref()).await?;
    get_role_info_impl(state, &req.role_id, req.session_id.as_deref()).await
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GlobalOllamaModelDto {
    pub model: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SetGlobalOllamaModelRequest {
    pub model: String,
    /// When set, clears per-role session model override so global default applies.
    #[serde(default)]
    pub role_id: Option<String>,
}

/// # Errors
///
/// Returns [`Err`] when app settings cannot be read.
pub async fn get_global_ollama_model_impl(
    state: &AppState,
) -> Result<GlobalOllamaModelDto, CommandError> {
    Ok(GlobalOllamaModelDto {
        model: state.global_ollama_model(),
    })
}

/// # Errors
///
/// Returns [`Err`] when persistence fails or model name is empty.
pub async fn set_global_ollama_model_impl(
    state: &AppState,
    req: &SetGlobalOllamaModelRequest,
) -> Result<GlobalOllamaModelDto, CommandError> {
    let t = req.model.trim();
    if t.is_empty() {
        return Err(AppError::InvalidParameter("empty global ollama model".into()).into());
    }
    state
        .db_manager
        .upsert_app_setting(KEY_GLOBAL_OLLAMA_MODEL, t)
        .await?;
    state.set_global_ollama_model_in_memory(t.to_string());
    if let Some(role_id) = req
        .role_id
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        let ns = session_namespace(role_id, None);
        state
            .db_manager
            .clear_session_ollama_model_override(ns.as_str())
            .await?;
    }
    Ok(GlobalOllamaModelDto {
        model: t.to_string(),
    })
}
