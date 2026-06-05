//! API-layer cloud probe; core env apply lives in [`crate::domain::user_llm_env`].

use crate::domain::effective_llm_model::resolve_effective_ollama_model;
use crate::domain::user_llm_env::apply_user_llm_env;
use crate::error::AppError;
use crate::models::plugin_backends::LlmBackend;
use crate::state::AppState;
use oclive_validation::NETWORK_GRANT_REMOTE_LLM;

/// Ping cloud LLM with current DB/env settings (after [`apply_user_llm_env`]).
pub(crate) async fn probe_cloud_llm_inner(
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
        return Err(AppError::InvalidParameter("云端 Base URL 未配置".into()));
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
        .require_network(NETWORK_GRANT_REMOTE_LLM)?;

    let role = state.load_role_cached_async(role_id).await?;
    let ns = crate::api::role::session_namespace(role_id, session_id);
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
            AppError::InvalidParameter(format!("云端模型连通性测试失败：{}", e.to_frontend_error()))
        })
}
