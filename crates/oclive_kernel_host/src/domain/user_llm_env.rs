//! Process-wide LLM env vars and token resolution (DB → `std::env`).

use crate::domain::ports::AppSettingsPort;
use crate::state::AppState;
use oclive_kernel_contracts::UserLlmSecretsPort;
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use std::sync::atomic::Ordering;

pub const KEY_OLLAMA_BASE: &str = "user_ollama_base_url";
pub const KEY_REMOTE_URL: &str = "user_remote_llm_url";
pub const KEY_REMOTE_TOKEN: &str = "user_remote_llm_token";
pub const KEY_REMOTE_MODEL: &str = "user_remote_llm_model";
pub const KEY_CLOUD_STYLE: &str = "user_llm_cloud_api_style";
pub const KEY_CLOUD_VENDOR: &str = "user_llm_cloud_vendor";
pub const KEY_LLM_PROVIDER: &str = "user_llm_provider";
pub const KEY_LOCAL_MODELS_DIR: &str = "user_local_models_dir";

pub const LLM_APP_SETTING_KEYS: &[&str] = &[
    KEY_LLM_PROVIDER,
    KEY_OLLAMA_BASE,
    KEY_REMOTE_URL,
    KEY_REMOTE_TOKEN,
    KEY_REMOTE_MODEL,
    KEY_CLOUD_STYLE,
    KEY_CLOUD_VENDOR,
    KEY_LOCAL_MODELS_DIR,
];

static USER_LLM_ENV: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

pub async fn ollama_base_from_db_or_env(state: &AppState) -> String {
    if let Ok(Some(v)) = state.db_manager.get_app_setting(KEY_OLLAMA_BASE).await {
        let t = v.trim();
        if !t.is_empty() {
            return t.to_string();
        }
    }
    std::env::var("OLLAMA_BASE_URL").unwrap_or_else(|_| "http://localhost:11434".to_string())
}

/// # Errors
///
/// Returns [`crate::error::AppError`] when app settings cannot be read from the database.
pub async fn apply_user_llm_env_from_db(
    db: &impl AppSettingsPort,
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
    if provider.is_empty() && !remote_url.is_empty() && cloud_api_token_configured(db, None).await?
    {
        provider = "cloud".to_string();
        let _ = db.upsert_app_setting(KEY_LLM_PROVIDER, "cloud").await;
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
        match settings
            .get(db_key)
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
        {
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

/// # Errors
///
/// Database or settings read failures propagate as [`crate::error::AppError`].
pub async fn load_remote_token(
    db: &impl AppSettingsPort,
    secrets: &dyn UserLlmSecretsPort,
    app_data: &std::path::Path,
) -> crate::error::Result<Option<String>> {
    let from_db = db.get_app_setting(KEY_REMOTE_TOKEN).await?;
    if from_db.as_ref().is_some_and(|s| !s.trim().is_empty()) {
        return Ok(from_db);
    }
    Ok(secrets.read_token_file(app_data))
}

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
    let secrets = state.user_llm_secrets.as_ref();
    let settings = crate::infrastructure::db_ports::DbSettingsPort(state.db_manager.as_ref());
    let token = load_remote_token(&settings, secrets, app_data).await?;
    if let Some(ref t) = token {
        secrets.set_cached_remote_llm_token(Some(t.clone()));
        state
            .db_manager
            .upsert_app_setting(KEY_REMOTE_TOKEN, t.trim())
            .await?;
        let _ = secrets.write_token_file(app_data, t.trim());
    } else {
        secrets.set_cached_remote_llm_token(None);
    }
    let provider = apply_user_llm_env_from_db(&settings).await?;
    tracing::info!(
        target: "oclive_llm",
        provider = %provider,
        remote_url_configured = std::env::var("OCLIVE_REMOTE_LLM_URL")
            .map(|s| !s.trim().is_empty())
            .unwrap_or(false),
        remote_token_configured = secrets.cached_remote_llm_token().is_some(),
        "apply_user_llm_env"
    );
    *state.user_llm_provider.write() = provider;
    let end_version = state.user_llm_env_version.load(Ordering::Acquire);
    state
        .user_llm_env_applied_version
        .store(end_version, Ordering::Release);
    state.user_llm_env_dirty.store(false, Ordering::Release);
    Ok(())
}

/// # Errors
///
/// Database read failures propagate as [`crate::error::AppError`].
pub async fn cloud_api_token_configured(
    db: &impl AppSettingsPort,
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
