//! Resolve the model id used for LLM generate calls (session override → cloud app setting → pack → env → global).

use crate::models::role::Role;
use crate::state::AppState;

const KEY_LLM_PROVIDER: &str = "user_llm_provider";
const KEY_REMOTE_MODEL: &str = "user_remote_llm_model";

/// Session DB override → saved cloud model (`user_remote_llm_model`) when provider=cloud → [`Role::resolve_ollama_model`].
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
    let provider = state
        .db_manager
        .get_app_setting(KEY_LLM_PROVIDER)
        .await?
        .map(|s| s.trim().to_ascii_lowercase())
        .unwrap_or_default();
    if provider == "cloud" {
        if let Some(m) = state.db_manager.get_app_setting(KEY_REMOTE_MODEL).await? {
            let t = m.trim();
            if !t.is_empty() {
                return Ok(t.to_string());
            }
        }
    }
    Ok(role.resolve_ollama_model(state.ollama_model.as_str()))
}
