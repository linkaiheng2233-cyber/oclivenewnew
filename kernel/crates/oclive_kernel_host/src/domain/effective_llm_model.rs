//! Resolve the model id used for LLM generate calls (session override → cloud app setting → pack → env → global).

use crate::models::role::Role;
use crate::state::AppState;

const KEY_LLM_PROVIDER: &str = "user_llm_provider";
const KEY_REMOTE_MODEL: &str = "user_remote_llm_model";
const LLM_MODEL_SETTING_KEYS: &[&str] = &[KEY_LLM_PROVIDER, KEY_REMOTE_MODEL];

/// Session DB override → saved cloud model → pack/env/global fallback chain (policy resolution, not path lookup).
///
/// # Errors
///
/// Returns [`crate::error::AppError`] when session or app settings cannot be read.
pub async fn resolve_effective_ollama_model(
    state: &AppState,
    role: &Role,
    session_namespace: &str,
) -> crate::error::Result<String> {
    if let Some(m) = state
        .db_manager
        .get_session_ollama_model_override(session_namespace)
        .await?
    {
        let t = m.trim();
        if !t.is_empty() {
            return Ok(t.to_string());
        }
    }
    let settings = state
        .db_manager
        .get_app_settings(LLM_MODEL_SETTING_KEYS)
        .await?;
    let provider = settings
        .get(KEY_LLM_PROVIDER)
        .map(|s| s.trim().to_ascii_lowercase())
        .unwrap_or_default();
    if provider == "cloud" {
        if let Some(m) = settings.get(KEY_REMOTE_MODEL) {
            let t = m.trim();
            if !t.is_empty() {
                return Ok(t.to_string());
            }
        }
    }
    Ok(role.resolve_ollama_model(state.global_ollama_model().as_str()))
}
