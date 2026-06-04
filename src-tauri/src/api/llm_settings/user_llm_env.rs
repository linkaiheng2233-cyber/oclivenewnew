//! Process-wide LLM env vars and token resolution (DB → `std::env`).

use crate::domain::effective_llm_model::resolve_effective_ollama_model;
use crate::error::AppError;
use crate::infrastructure::user_llm_secrets::{
    self, read_token_file, set_cached_remote_llm_token, write_token_file,
};
use crate::models::plugin_backends::LlmBackend;
use crate::state::AppState;
use oclive_validation::NETWORK_GRANT_REMOTE_LLM;
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use std::sync::atomic::Ordering;

pub(crate) const KEY_OLLAMA_BASE: &str = "user_ollama_base_url";
pub(crate) const KEY_REMOTE_URL: &str = "user_remote_llm_url";
pub(crate) const KEY_REMOTE_TOKEN: &str = "user_remote_llm_token";
pub(crate) const KEY_REMOTE_MODEL: &str = "user_remote_llm_model";
pub(crate) const KEY_CLOUD_STYLE: &str = "user_llm_cloud_api_style";
pub(crate) const KEY_CLOUD_VENDOR: &str = "user_llm_cloud_vendor";
pub(crate) const KEY_LLM_PROVIDER: &str = "user_llm_provider";
pub(crate) const KEY_LOCAL_MODELS_DIR: &str = "user_local_models_dir";

pub(crate) const LLM_APP_SETTING_KEYS: &[&str] = &[
    KEY_LLM_PROVIDER,
    KEY_OLLAMA_BASE,
    KEY_REMOTE_URL,
    KEY_REMOTE_TOKEN,
    KEY_REMOTE_MODEL,
    KEY_CLOUD_STYLE,
    KEY_CLOUD_VENDOR,
    KEY_LOCAL_MODELS_DIR,
];

/// Serializes `std::env` mutations from [`apply_user_llm_env`]. LLM settings are
/// single-writer: concurrent apply would race on process environment variables.
static USER_LLM_ENV: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

pub(crate) async fn ollama_base_from_db_or_env(state: &AppState) -> String {
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
    const LLM_ENV_KEYS: &[&str] = &[
        KEY_OLLAMA_BASE,
        KEY_REMOTE_URL,
        KEY_REMOTE_TOKEN,
        KEY_CLOUD_STYLE,
        KEY_LLM_PROVIDER,
    ];
    let settings = db.get_app_settings(LLM_ENV_KEYS).await?;
    let remote_url = settings
        .get(KEY_REMOTE_URL)
        .map(|s| s.trim().to_string())
        .unwrap_or_default();
    let mut provider = settings
        .get(KEY_LLM_PROVIDER)
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
    let backend_env = match provider.as_str() {
        "cloud" if !remote_url.is_empty() => Some("remote"),
        "local" => Some("ollama"),
        _ => None,
    };
    let _guard = USER_LLM_ENV.lock();
    let env_pairs = [
        (KEY_OLLAMA_BASE, "OLLAMA_BASE_URL"),
        (KEY_REMOTE_URL, "OCLIVE_REMOTE_LLM_URL"),
        (KEY_REMOTE_TOKEN, "OCLIVE_REMOTE_LLM_TOKEN"),
        (KEY_CLOUD_STYLE, "OCLIVE_LLM_CLOUD_API_STYLE"),
    ];
    for (db_key, env_key) in env_pairs {
        match settings.get(db_key).map(|s| s.trim()).filter(|s| !s.is_empty()) {
            Some(t) => std::env::set_var(env_key, t),
            None => std::env::remove_var(env_key),
        }
    }
    match backend_env {
        Some("remote") => std::env::set_var("OCLIVE_LLM_BACKEND", "remote"),
        Some("ollama") => std::env::set_var("OCLIVE_LLM_BACKEND", "ollama"),
        _ => std::env::remove_var("OCLIVE_LLM_BACKEND"),
    }
    Ok(provider)
}

pub(crate) async fn resolve_remote_token(
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
    let start_version = state.user_llm_env_version.load(Ordering::Acquire);
    if !state.user_llm_env_dirty.load(Ordering::Acquire)
        && state.user_llm_env_applied_version.load(Ordering::Acquire) == start_version
    {
        return Ok(());
    }

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
    let end_version = state.user_llm_env_version.load(Ordering::Acquire);
    state
        .user_llm_env_applied_version
        .store(end_version, Ordering::Release);
    state
        .user_llm_env_dirty
        .store(end_version != start_version, Ordering::Release);
    Ok(())
}

pub(crate) async fn cloud_api_token_configured(
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
            AppError::InvalidParameter(format!(
                "云端模型连通性测试失败：{}",
                e.to_frontend_error()
            ))
        })
}
