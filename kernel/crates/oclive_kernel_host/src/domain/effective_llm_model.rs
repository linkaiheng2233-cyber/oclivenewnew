//! Resolve the model id used for LLM generate calls (session override → cloud app setting → pack → env → global).

use crate::models::role::Role;
use crate::state::AppState;

const KEY_LLM_PROVIDER: &str = "user_llm_provider";
const KEY_REMOTE_MODEL: &str = "user_remote_llm_model";
const LLM_MODEL_SETTING_KEYS: &[&str] = &[KEY_LLM_PROVIDER, KEY_REMOTE_MODEL];

/// Ollama model tag/id — not a GGUF path or UI `file:` placeholder.
#[must_use]
pub fn is_usable_ollama_model_id(model: &str) -> bool {
    let t = model.trim();
    if t.is_empty() || t.starts_with("file:") {
        return false;
    }
    // Windows absolute paths (e.g. `D:\models\foo.gguf`) must not be sent to Ollama as model names.
    if t.contains('\\') {
        return false;
    }
    if t.len() >= 2 {
        let bytes = t.as_bytes();
        if bytes[1] == b':' && bytes[0].is_ascii_alphabetic() {
            return false;
        }
    }
    if t.starts_with('/') || t.starts_with("\\\\") {
        return false;
    }
    true
}

#[cfg(test)]
mod tests {
    use super::is_usable_ollama_model_id;

    #[test]
    fn rejects_empty_file_prefix_and_paths() {
        assert!(!is_usable_ollama_model_id(""));
        assert!(!is_usable_ollama_model_id("file:D:\\models\\a.gguf"));
        assert!(!is_usable_ollama_model_id(r"D:\models\a.gguf"));
        assert!(!is_usable_ollama_model_id("/tmp/model.gguf"));
    }

    #[test]
    fn accepts_ollama_tags() {
        assert!(is_usable_ollama_model_id("qwen2.5:7b"));
        assert!(is_usable_ollama_model_id("mumu:latest"));
    }
}

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
        if is_usable_ollama_model_id(t) {
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
