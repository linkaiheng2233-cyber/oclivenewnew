//! Ollama model discovery and global model selection.

use crate::command_error::CommandError;
use crate::domain::user_llm_env::{ollama_base_from_db_or_env, KEY_GLOBAL_OLLAMA_MODEL};
use crate::error::AppError;
use crate::infrastructure::ollama_client::OllamaClient;
use crate::service::role::session_namespace;
use crate::state::AppState;
use serde::{Deserialize, Serialize};

/// # Errors
///
/// Returns [`Err`] when Ollama list fails.
pub async fn list_ollama_models_impl(
    state: &AppState,
    ollama_base_url: Option<&str>,
) -> Result<Vec<String>, CommandError> {
    let base = ollama_base_url
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_default();
    let base = if base.is_empty() {
        ollama_base_from_db_or_env(state).await
    } else {
        base.to_string()
    };
    OllamaClient::new(base)
        .list_models()
        .await
        .map_err(CommandError::from)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GlobalOllamaModelDto {
    pub model: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SetGlobalOllamaModelRequest {
    pub model: String,
    /// When set, clears per-role session model override so global default applies.
    #[serde(default)]
    pub role_id: Option<String>,
}

/// # Errors
///
/// Returns [`Err`] when app settings cannot be read.
pub async fn get_global_ollama_model_impl(
    state: &AppState,
) -> Result<GlobalOllamaModelDto, CommandError> {
    Ok(GlobalOllamaModelDto {
        model: state.global_ollama_model(),
    })
}

/// # Errors
///
/// Returns [`Err`] when persistence fails or model name is empty.
pub async fn set_global_ollama_model_impl(
    state: &AppState,
    req: &SetGlobalOllamaModelRequest,
) -> Result<GlobalOllamaModelDto, CommandError> {
    let t = req.model.trim();
    if t.is_empty() {
        return Err(AppError::InvalidParameter("empty global ollama model".into()).into());
    }
    state
        .db_manager
        .upsert_app_setting(KEY_GLOBAL_OLLAMA_MODEL, t)
        .await?;
    state.set_global_ollama_model_in_memory(t.to_string());
    if let Some(performance) = state.performance_llm.as_ref() {
        performance.record_fallback_model(t);
    }
    state.schedule_ollama_preload(t.to_string());
    if let Some(role_id) = req
        .role_id
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        let ns = session_namespace(role_id, None);
        state
            .db_manager
            .clear_session_ollama_model_override(ns.as_str())
            .await?;
    }
    Ok(GlobalOllamaModelDto {
        model: t.to_string(),
    })
}
