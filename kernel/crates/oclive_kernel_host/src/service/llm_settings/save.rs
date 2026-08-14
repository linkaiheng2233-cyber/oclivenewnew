//! Save / session-override path for LLM user settings.

use super::{SaveLlmUserSettingsRequest, SetSessionLlmModelRequest, LOCAL_LORA_MUTATION};

use crate::command_error::CommandError;
use crate::domain::effective_llm_model::is_usable_ollama_model_id;
use crate::domain::user_llm_env::{
    apply_user_llm_env, cloud_api_token_configured, load_remote_token, KEY_CLOUD_STYLE,
    KEY_CLOUD_VENDOR, KEY_LLM_PROVIDER, KEY_LOCAL_LORA_ADAPTER_ID, KEY_LOCAL_LORA_ADAPTER_PATH,
    KEY_LOCAL_MODEL_PATH, KEY_OLLAMA_BASE, KEY_REMOTE_MODEL, KEY_REMOTE_TOKEN, KEY_REMOTE_URL,
};
use crate::error::AppError;
use crate::infrastructure::llm_models::{
    canonical_models_dir, describe_local_model_file, persist_local_models_dir,
    verify_local_model_file,
};
use crate::infrastructure::lora_adapters::{
    gguf_base_model_architecture, resolve_local_lora_adapter,
};
use crate::infrastructure::user_llm_secrets::{set_cached_remote_llm_token, write_token_file};
use crate::models::dto::{RoleInfo, SetSessionPluginBackendRequest};
use crate::service::role::{
    get_role_info_impl, session_namespace, set_session_plugin_backend_impl,
};
use crate::state::{
    is_managed_legacy_models_path, migrate_and_cleanup_models, paths_equal, AppState,
};
use oclive_kernel_types::models::plugin_backends::LlmBackend;
use oclive_kernel_types::models::{ContentRating, PluginBackendsOverride};
use oclive_validation::NETWORK_GRANT_REMOTE_LLM;
use std::path::PathBuf;

fn require_adult_base_model_acknowledgement(
    base_changed: bool,
    content_rating: &ContentRating,
    acknowledged: bool,
) -> Result<(), CommandError> {
    if base_changed && *content_rating == ContentRating::Adult && !acknowledged {
        return Err(AppError::InvalidParameter(
            "selecting an adult-only local base model requires explicit acknowledgement".into(),
        )
        .into());
    }
    Ok(())
}

async fn apply_session_model_override(
    state: &AppState,
    ns: &str,
    model: Option<&str>,
) -> Result<(), CommandError> {
    if let Some(model) = model {
        let t = model.trim();
        if t.is_empty() || !is_usable_ollama_model_id(t) {
            state
                .db_manager
                .clear_session_ollama_model_override(ns)
                .await?;
        } else {
            state
                .db_manager
                .set_session_ollama_model_override(ns, t)
                .await?;
        }
    }
    Ok(())
}

/// # Errors
///
/// Returns [`Err`] when persistence or role reload fails.
pub async fn save_llm_user_settings_impl(
    state: &AppState,
    req: &SaveLlmUserSettingsRequest,
) -> Result<RoleInfo, CommandError> {
    let _lora_guard = LOCAL_LORA_MUTATION.lock().await;
    let provider = req.provider.trim().to_ascii_lowercase();
    if provider != "local" && provider != "cloud" {
        return Err(AppError::InvalidParameter("provider must be local or cloud".into()).into());
    }

    let active_lora_id = state
        .db_manager
        .get_app_setting(KEY_LOCAL_LORA_ADAPTER_ID)
        .await?
        .unwrap_or_default();
    let current_model_path = state
        .db_manager
        .get_app_setting(KEY_LOCAL_MODEL_PATH)
        .await?
        .unwrap_or_default();
    let mut deactivate_lora_for_base_change = false;

    if let Some(model_path) = req.local_model_path.as_deref() {
        let model_path = model_path.trim();
        let base_changed = if current_model_path.trim().is_empty() || model_path.is_empty() {
            current_model_path.trim() != model_path
        } else {
            !paths_equal(
                PathBuf::from(current_model_path.trim()).as_path(),
                PathBuf::from(model_path).as_path(),
            )
        };
        deactivate_lora_for_base_change = base_changed && !active_lora_id.trim().is_empty();

        if !model_path.is_empty() {
            let base_path = PathBuf::from(model_path);
            let descriptor = if base_changed {
                tokio::task::spawn_blocking({
                    let base_path = base_path.clone();
                    move || verify_local_model_file(&base_path)
                })
                .await
                .map_err(|error| {
                    CommandError::from(AppError::InvalidParameter(format!(
                        "base-model verification worker failed: {error}"
                    )))
                })??
            } else {
                describe_local_model_file(&base_path)?
            };
            require_adult_base_model_acknowledgement(
                base_changed,
                &descriptor.content_rating,
                req.adult_content_acknowledged,
            )?;

            // A base switch never carries the old adapter implicitly. When the
            // path is unchanged, keep validating the active pair so corrupt or
            // replaced files cannot bypass the architecture check.
            if !active_lora_id.trim().is_empty() && !base_changed {
                if !base_path
                    .extension()
                    .and_then(|extension| extension.to_str())
                    .is_some_and(|extension| extension.eq_ignore_ascii_case("gguf"))
                {
                    return Err(AppError::InvalidParameter(
                        "an active llama.cpp LoRA requires an existing GGUF base model".into(),
                    )
                    .into());
                }
                let models_dir = canonical_models_dir(state);
                let active_id = active_lora_id.trim().to_string();
                let adapter = tokio::task::spawn_blocking(move || {
                    resolve_local_lora_adapter(&models_dir, &active_id, Some(&active_id))
                        .map_err(CommandError::from)
                })
                .await
                .map_err(|error| {
                    CommandError::from(AppError::InvalidParameter(format!(
                        "LoRA verification worker failed: {error}"
                    )))
                })??;
                let base_architecture =
                    tokio::task::spawn_blocking(move || gguf_base_model_architecture(&base_path))
                        .await
                        .map_err(|error| {
                            CommandError::from(AppError::InvalidParameter(format!(
                                "base-model GGUF verification worker failed: {error}"
                            )))
                        })??;
                if let (Some(adapter_architecture), Some(base_architecture)) = (
                    adapter.dto.architecture.as_deref(),
                    base_architecture.as_deref(),
                ) {
                    if !adapter_architecture.eq_ignore_ascii_case(base_architecture) {
                        return Err(AppError::InvalidParameter(format!(
                            "the active LoRA is incompatible with base architecture '{base_architecture}'"
                        ))
                        .into());
                    }
                }
            }
        }
    }

    state
        .db_manager
        .upsert_app_setting(KEY_LLM_PROVIDER, &provider)
        .await?;

    if provider == "cloud" {
        let url_ok = req
            .remote_url
            .as_deref()
            .is_some_and(|s| !s.trim().is_empty())
            || state
                .db_manager
                .get_app_setting(KEY_REMOTE_URL)
                .await?
                .is_some_and(|s| !s.trim().is_empty());
        if !url_ok {
            return Err(AppError::InvalidParameter("云端 Base URL 不能为空".into()).into());
        }
        let settings = crate::infrastructure::db_ports::DbSettingsPort(state.db_manager.as_ref());
        if !cloud_api_token_configured(&settings, req.remote_token.as_deref()).await? {
            return Err(AppError::InvalidParameter("请填写云端 API Key 后再保存".into()).into());
        }
        // BYOK save is explicit user consent for outbound LLM API calls.
        if let Err(e) = state
            .high_risk_grants
            .grant_network(NETWORK_GRANT_REMOTE_LLM)
        {
            tracing::warn!(
                target: "oclive_llm",
                error = %e,
                "auto-grant remote LLM on cloud save failed"
            );
        }
    }

    if let Some(ref url) = req.ollama_base_url {
        state
            .db_manager
            .upsert_app_setting(KEY_OLLAMA_BASE, url.trim())
            .await?;
    }
    if let Some(ref dir) = req.local_models_dir {
        let trimmed = dir.trim();
        let canonical = canonical_models_dir(state);
        let canonical_str = canonical.to_string_lossy().into_owned();
        let app_data = state.directory_plugins.app_data_dir();
        if trimmed.is_empty() {
            persist_local_models_dir(state, &canonical_str).await?;
        } else {
            let stored = PathBuf::from(trimmed);
            if is_managed_legacy_models_path(&stored, &canonical, app_data) {
                migrate_and_cleanup_models(&stored, &canonical);
                persist_local_models_dir(state, &canonical_str).await?;
            } else {
                persist_local_models_dir(state, trimmed).await?;
            }
        }
    }
    if let Some(ref model_path) = req.local_model_path {
        let trimmed = model_path.trim();
        state
            .db_manager
            .upsert_app_setting(KEY_LOCAL_MODEL_PATH, trimmed)
            .await?;
    }
    if deactivate_lora_for_base_change {
        state
            .db_manager
            .upsert_app_setting(KEY_LOCAL_LORA_ADAPTER_ID, "")
            .await?;
        state
            .db_manager
            .upsert_app_setting(KEY_LOCAL_LORA_ADAPTER_PATH, "")
            .await?;
        tracing::info!(
            target: "oclive_lora",
            previous_adapter_id = active_lora_id.trim(),
            previous_base = current_model_path.trim(),
            next_base = req.local_model_path.as_deref().unwrap_or_default().trim(),
            "deactivated local LoRA because the base model changed"
        );
    }
    if let Some(ref url) = req.remote_url {
        state
            .db_manager
            .upsert_app_setting(KEY_REMOTE_URL, url.trim())
            .await?;
    }
    if let Some(ref token) = req.remote_token {
        let t = token.trim();
        if !t.is_empty() {
            state
                .db_manager
                .upsert_app_setting(KEY_REMOTE_TOKEN, t)
                .await?;
            write_token_file(state.directory_plugins.app_data_dir(), t)
                .map_err(|e| AppError::InvalidParameter(format!("save API token: {e}")))?;
            set_cached_remote_llm_token(Some(t.to_string()));
            let read_back = state
                .db_manager
                .get_app_setting(KEY_REMOTE_TOKEN)
                .await?
                .filter(|s| !s.trim().is_empty());
            if read_back.as_deref() != Some(t) {
                return Err(AppError::InvalidParameter(
                    "API Key 未能写入数据库，请重试保存".into(),
                )
                .into());
            }
        }
    } else if provider == "cloud" {
        let app_data = state.directory_plugins.app_data_dir();
        let secrets = state.user_llm_secrets.as_ref();
        let settings = crate::infrastructure::db_ports::DbSettingsPort(state.db_manager.as_ref());
        let existing = load_remote_token(&settings, secrets, app_data).await?;
        secrets.set_cached_remote_llm_token(existing);
    }
    if let Some(ref model) = req.remote_model {
        state
            .db_manager
            .upsert_app_setting(KEY_REMOTE_MODEL, model.trim())
            .await?;
    }
    if let Some(ref vendor) = req.cloud_vendor {
        state
            .db_manager
            .upsert_app_setting(KEY_CLOUD_VENDOR, vendor.trim())
            .await?;
    }
    if let Some(ref style) = req.cloud_api_style {
        let s = style.trim().to_ascii_lowercase();
        let normalized = if s == "oclive_jsonrpc" {
            "oclive_jsonrpc"
        } else {
            "openai"
        };
        state
            .db_manager
            .upsert_app_setting(KEY_CLOUD_STYLE, normalized)
            .await?;
    } else if provider == "cloud" {
        state
            .db_manager
            .upsert_app_setting(KEY_CLOUD_STYLE, "openai")
            .await?;
    }

    state.mark_user_llm_env_dirty();
    apply_user_llm_env(state).await?;
    if provider == "local" {
        if let Some(performance) = state.performance_llm.as_ref() {
            performance.schedule_warmup();
        }
    } else if let Some(performance) = state.performance_llm.as_ref() {
        performance.suspend_managed_runtime("cloud provider is active");
    }

    let ns = session_namespace(&req.role_id, req.session_id.as_deref());
    state.db_manager.ensure_role_runtime(ns.as_str()).await?;

    let model_for_session = if provider == "cloud" {
        req.remote_model.as_deref().or(req.ollama_model.as_deref())
    } else {
        req.ollama_model.as_deref()
    };
    apply_session_model_override(state, ns.as_str(), model_for_session).await?;

    let backend = if provider == "cloud" {
        "remote"
    } else {
        "ollama"
    };
    let info = match set_session_plugin_backend_impl(
        state,
        &SetSessionPluginBackendRequest {
            role_id: req.role_id.clone(),
            module: "llm".to_string(),
            backend: Some(Some(backend.to_string())),
            local_memory_provider_id: None,
            session_id: req.session_id.clone(),
        },
    )
    .await
    {
        Ok(info) => info,
        Err(e) if e.to_string().contains("slot_registry") => {
            let llm_backend = if provider == "cloud" {
                LlmBackend::Remote
            } else {
                LlmBackend::Ollama
            };
            state.set_session_backend_override(
                ns.as_str(),
                PluginBackendsOverride {
                    llm: Some(llm_backend),
                    ..Default::default()
                },
            );
            get_role_info_impl(state, &req.role_id, req.session_id.as_deref()).await?
        }
        Err(e) => return Err(e),
    };

    Ok(info)
}

/// # Errors
///
/// Returns [`Err`] when persistence or role reload fails.
pub async fn set_session_llm_model_impl(
    state: &AppState,
    req: &SetSessionLlmModelRequest,
) -> Result<RoleInfo, CommandError> {
    let ns = session_namespace(&req.role_id, req.session_id.as_deref());
    state.db_manager.ensure_role_runtime(ns.as_str()).await?;
    apply_session_model_override(state, ns.as_str(), req.model.as_deref()).await?;
    get_role_info_impl(state, &req.role_id, req.session_id.as_deref()).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adult_base_model_change_is_rejected_without_its_own_acknowledgement() {
        let error = require_adult_base_model_acknowledgement(true, &ContentRating::Adult, false)
            .expect_err("adult base acknowledgement");

        assert!(error.to_string().contains("explicit acknowledgement"));
        assert!(
            require_adult_base_model_acknowledgement(true, &ContentRating::Adult, true,).is_ok()
        );
    }
}
