//! Cloud LLM API token: SQLite `app_settings` + process cache + app_data file backup.
//!
//! Restart must reload the token without re-pasting in the UI; env alone is insufficient if
//! `apply_user_llm_env` runs before DB is ready or the setting row was never written.

use parking_lot::RwLock;
use std::path::Path;
use std::sync::OnceLock;

const TOKEN_FILE: &str = "user_remote_llm_token";

static PROCESS_TOKEN: OnceLock<RwLock<Option<String>>> = OnceLock::new();

fn store() -> &'static RwLock<Option<String>> {
    PROCESS_TOKEN.get_or_init(|| RwLock::new(None))
}

/// Sync process env + in-memory cache (used by [`super::openai_compatible_llm::OpenAiCompatibleLlm::from_env`]).
pub fn set_cached_remote_llm_token(token: Option<String>) {
    let normalized = token
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    if let Some(ref t) = normalized {
        std::env::set_var("OCLIVE_REMOTE_LLM_TOKEN", t);
    } else {
        std::env::remove_var("OCLIVE_REMOTE_LLM_TOKEN");
    }
    *store().write() = normalized;
}

#[must_use]
pub fn cached_remote_llm_token() -> Option<String> {
    store().read().clone()
}

/// # Errors
///
/// Returns I/O error message when the file cannot be written.
pub fn write_token_file(app_data: &Path, token: &str) -> Result<(), String> {
    let t = token.trim();
    if t.is_empty() {
        return Ok(());
    }
    std::fs::create_dir_all(app_data).map_err(|e| e.to_string())?;
    let path = app_data.join(TOKEN_FILE);
    std::fs::write(&path, t.as_bytes()).map_err(|e| e.to_string())
}

#[must_use]
pub fn read_token_file(app_data: &Path) -> Option<String> {
    let raw = std::fs::read_to_string(app_data.join(TOKEN_FILE)).ok()?;
    let t = raw.trim();
    if t.is_empty() {
        None
    } else {
        Some(t.to_string())
    }
}

/// Default [`UserLlmSecretsPort`] for production wiring.
#[derive(Debug, Default, Clone, Copy)]
pub struct BuiltinUserLlmSecrets;

impl oclive_kernel_contracts::UserLlmSecretsPort for BuiltinUserLlmSecrets {
    fn set_cached_remote_llm_token(&self, token: Option<String>) {
        set_cached_remote_llm_token(token);
    }

    fn cached_remote_llm_token(&self) -> Option<String> {
        cached_remote_llm_token()
    }

    fn write_token_file(&self, app_data: &Path, token: &str) -> Result<(), String> {
        write_token_file(app_data, token)
    }

    fn read_token_file(&self, app_data: &Path) -> Option<String> {
        read_token_file(app_data)
    }
}
