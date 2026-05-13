//! 插件后端解析诊断（导出聊天记录等场景复用）。

use crate::domain::chat_engine::conversation_state_role_id;
use crate::error::Result;
use crate::infrastructure::storage::resolve_llm_backend_env_override;
use crate::models::dto::{PluginResolutionDebugInfo, API_VERSION, SCHEMA_VERSION};
use crate::models::LlmBackend;
use crate::state::KernelAppState;

/// `app_version` 由嵌入方传入（桌面为 Tauri crate 的 `CARGO_PKG_VERSION`，无头服务为各自版本号）。
pub async fn build_plugin_resolution_debug_info(
    state: &KernelAppState,
    role_id: &str,
    session_id: Option<&str>,
    app_version: impl Into<String>,
) -> Result<PluginResolutionDebugInfo> {
    let role = state.load_role_cached(role_id)?;
    let session_ns = conversation_state_role_id(role_id, session_id);
    state
        .db_manager
        .ensure_role_runtime(session_ns.as_str())
        .await?;
    let session_override = state.session_backend_override(session_ns.as_str());
    let effective = state.effective_plugin_backends_for_session(role.as_ref(), session_ns.as_str());
    let effective_sources = state.effective_plugin_backend_sources_for_session(session_ns.as_str());
    let llm_env_override = resolve_llm_backend_env_override().map(|b| match b {
        LlmBackend::Ollama => "ollama".to_string(),
        LlmBackend::Remote => "remote".to_string(),
        LlmBackend::Directory => "directory".to_string(),
        LlmBackend::None => "none".to_string(),
    });
    let remote_plugin_url_configured = std::env::var("OCLIVE_REMOTE_PLUGIN_URL")
        .ok()
        .map(|v| !v.trim().is_empty())
        .unwrap_or(false);
    let remote_llm_url_configured = std::env::var("OCLIVE_REMOTE_LLM_URL")
        .ok()
        .map(|v| !v.trim().is_empty())
        .unwrap_or(false);
    let mut local_provider_ids: Vec<String> = state
        .local_plugin_all_providers()
        .iter()
        .map(|d| d.provider_id.clone())
        .collect();
    local_provider_ids.sort();
    local_provider_ids.dedup();

    Ok(PluginResolutionDebugInfo {
        app_version: app_version.into(),
        api_version: API_VERSION,
        schema_version: SCHEMA_VERSION,
        role_id: role_id.to_string(),
        session_namespace: session_ns,
        plugin_backends_pack_default: role.plugin_backends.clone(),
        plugin_backends_session_override: session_override,
        plugin_backends_effective: effective,
        plugin_backends_effective_sources: effective_sources,
        llm_env_override,
        remote_plugin_url_configured,
        remote_llm_url_configured,
        local_provider_count: local_provider_ids.len(),
        local_provider_ids,
    })
}
