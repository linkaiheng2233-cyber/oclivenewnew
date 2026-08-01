//! Process-wide LLM env vars and token resolution (DB → `std::env`).

use crate::domain::ports::AppSettingsPort;
use crate::state::AppState;
use oclive_kernel_contracts::UserLlmSecretsPort;
use once_cell::sync::Lazy;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use tokio::sync::Mutex;

pub const KEY_OLLAMA_BASE: &str = "user_ollama_base_url";
pub const KEY_REMOTE_URL: &str = "user_remote_llm_url";
pub const KEY_REMOTE_TOKEN: &str = "user_remote_llm_token";
pub const KEY_REMOTE_MODEL: &str = "user_remote_llm_model";
pub const KEY_CLOUD_STYLE: &str = "user_llm_cloud_api_style";
pub const KEY_CLOUD_VENDOR: &str = "user_llm_cloud_vendor";
pub const KEY_LLM_PROVIDER: &str = "user_llm_provider";
pub const KEY_LOCAL_MODELS_DIR: &str = "user_local_models_dir";
pub const KEY_LOCAL_MODEL_PATH: &str = "user_local_llm_model_path";
pub const KEY_LOCAL_LORA_ADAPTER_ID: &str = "user_local_lora_adapter_id";
pub const KEY_LOCAL_LORA_ADAPTER_PATH: &str = "user_local_lora_adapter_path";
pub const KEY_GLOBAL_OLLAMA_MODEL: &str = "global_ollama_model";

pub const LLM_APP_SETTING_KEYS: &[&str] = &[
    KEY_LLM_PROVIDER,
    KEY_OLLAMA_BASE,
    KEY_REMOTE_URL,
    KEY_REMOTE_TOKEN,
    KEY_REMOTE_MODEL,
    KEY_CLOUD_STYLE,
    KEY_CLOUD_VENDOR,
    KEY_LOCAL_MODELS_DIR,
    KEY_LOCAL_MODEL_PATH,
    KEY_LOCAL_LORA_ADAPTER_ID,
    KEY_LOCAL_LORA_ADAPTER_PATH,
    KEY_GLOBAL_OLLAMA_MODEL,
];

const DEFAULT_GLOBAL_OLLAMA_MODEL: &str = "qwen2.5:7b";

/// Resolve global default Ollama model: DB app_setting → `OLLAMA_MODEL` env → built-in default.
pub async fn global_ollama_model_from_db_or_env(db: &impl AppSettingsPort) -> String {
    if let Ok(Some(v)) = db.get_app_setting(KEY_GLOBAL_OLLAMA_MODEL).await {
        let t = v.trim();
        if !t.is_empty() {
            return t.to_string();
        }
    }
    std::env::var("OLLAMA_MODEL").unwrap_or_else(|_| DEFAULT_GLOBAL_OLLAMA_MODEL.to_string())
}

// Process environment is global. Serialize the complete DB -> cache -> env
// transaction, including its async reads, so an older caller cannot overwrite
// a newer settings snapshot after merely waiting for the final env-write lock.
static USER_LLM_ENV_APPLY: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

pub async fn ollama_base_from_db_or_env(state: &AppState) -> String {
    if let Ok(Some(v)) = state.db_manager.get_app_setting(KEY_OLLAMA_BASE).await {
        let t = v.trim();
        if !t.is_empty() {
            return t.to_string();
        }
    }
    std::env::var("OLLAMA_BASE_URL").unwrap_or_else(|_| "http://localhost:11434".to_string())
}

async fn apply_user_llm_env_from_db_unlocked(
    db: &impl AppSettingsPort,
) -> crate::error::Result<String> {
    const LLM_ENV_KEYS: &[&str] = &[
        KEY_OLLAMA_BASE,
        KEY_REMOTE_URL,
        KEY_REMOTE_TOKEN,
        KEY_CLOUD_STYLE,
        KEY_LLM_PROVIDER,
        KEY_LOCAL_MODEL_PATH,
        KEY_LOCAL_LORA_ADAPTER_PATH,
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
        db.upsert_app_setting(KEY_LLM_PROVIDER, "cloud").await?;
    }
    let backend_env = match provider.as_str() {
        "cloud" if !remote_url.is_empty() => Some("remote"),
        "local" => Some("ollama"),
        _ => None,
    };
    let provider_for_env = settings
        .get(KEY_LLM_PROVIDER)
        .map(|s| s.trim().to_ascii_lowercase())
        .unwrap_or_default();
    let env_pairs = [
        (KEY_OLLAMA_BASE, "OLLAMA_BASE_URL"),
        (KEY_REMOTE_URL, "OCLIVE_REMOTE_LLM_URL"),
        (KEY_REMOTE_TOKEN, "OCLIVE_REMOTE_LLM_TOKEN"),
        (KEY_CLOUD_STYLE, "OCLIVE_LLM_CLOUD_API_STYLE"),
        (KEY_LOCAL_MODEL_PATH, "OCLIVE_LOCAL_LLM_MODEL_PATH"),
        (KEY_LOCAL_LORA_ADAPTER_PATH, "OCLIVE_LOCAL_LLM_LORA_PATH"),
    ];
    for (db_key, env_key) in env_pairs {
        if db_key == KEY_REMOTE_URL && provider_for_env == "local" {
            std::env::remove_var(env_key);
            continue;
        }
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

/// Applies DB-backed LLM settings to the process environment without updating
/// the full [`AppState`] cache.
///
/// # Errors
///
/// Returns [`crate::error::AppError`] when app settings cannot be read from the database.
pub async fn apply_user_llm_env_from_db(db: &impl AppSettingsPort) -> crate::error::Result<String> {
    let _guard = USER_LLM_ENV_APPLY.lock().await;
    apply_user_llm_env_from_db_unlocked(db).await
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
    let _guard = USER_LLM_ENV_APPLY.lock().await;
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
        if let Err(error) = secrets.write_token_file(app_data, t.trim()) {
            tracing::warn!(
                target: "oclive_llm",
                error_code = "LLM_TOKEN_BACKUP_WRITE_FAILED",
                app_data = %app_data.display(),
                %error,
                "cloud LLM token file backup could not be written"
            );
        }
    } else {
        secrets.set_cached_remote_llm_token(None);
    }
    let provider = apply_user_llm_env_from_db_unlocked(&settings).await?;
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
    complete_user_llm_env_apply(
        &state.user_llm_env_version,
        &state.user_llm_env_applied_version,
        &state.user_llm_env_dirty,
        start_version,
    );
    Ok(())
}

fn complete_user_llm_env_apply(
    version: &AtomicU64,
    applied_version: &AtomicU64,
    dirty: &AtomicBool,
    start_version: u64,
) {
    // Only the snapshot actually read by this transaction is considered
    // applied. Clearing dirty first and re-checking the version prevents a
    // concurrent marker from being lost between the version check and store.
    applied_version.store(start_version, Ordering::Release);
    dirty.store(false, Ordering::Release);
    if version.load(Ordering::Acquire) != start_version {
        dirty.store(true, Ordering::Release);
    }
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

#[cfg(test)]
mod tests {
    use super::complete_user_llm_env_apply;
    use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

    #[test]
    fn stable_snapshot_is_marked_clean() {
        let version = AtomicU64::new(7);
        let applied = AtomicU64::new(0);
        let dirty = AtomicBool::new(true);

        complete_user_llm_env_apply(&version, &applied, &dirty, 7);

        assert_eq!(applied.load(Ordering::Acquire), 7);
        assert!(!dirty.load(Ordering::Acquire));
    }

    #[test]
    fn newer_snapshot_remains_dirty_for_the_waiting_caller() {
        let version = AtomicU64::new(8);
        let applied = AtomicU64::new(0);
        let dirty = AtomicBool::new(true);

        complete_user_llm_env_apply(&version, &applied, &dirty, 7);

        assert_eq!(applied.load(Ordering::Acquire), 7);
        assert!(dirty.load(Ordering::Acquire));
    }
}
