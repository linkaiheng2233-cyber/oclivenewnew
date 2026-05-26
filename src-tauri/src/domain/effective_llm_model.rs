//! Resolve the model id used for LLM generate calls (session override → pack → env → global).

use crate::models::role::Role;
use crate::state::AppState;

/// Session DB override → [`Role::resolve_ollama_model`].
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
    Ok(role.resolve_ollama_model(state.ollama_model.as_str()))
}
