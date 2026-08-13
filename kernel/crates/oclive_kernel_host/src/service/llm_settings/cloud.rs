//! Cloud (OpenAI-compatible) model discovery and connectivity probe.

use crate::command_error::CommandError;
use crate::domain::effective_llm_model::resolve_effective_ollama_model;
use crate::domain::user_llm_env::{apply_user_llm_env, load_remote_token, KEY_REMOTE_URL};
use crate::error::AppError;
use crate::infrastructure::openai_compatible_llm::list_openai_compatible_models;
use crate::service::role::session_namespace;
use crate::state::AppState;
use oclive_kernel_types::models::plugin_backends::LlmBackend;
use oclive_validation::NETWORK_GRANT_REMOTE_LLM;
use reqwest::Client;
use serde::{Deserialize, Serialize};

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
