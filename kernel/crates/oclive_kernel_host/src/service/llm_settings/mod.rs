//! LLM / model settings impls shared by HTTP routes and Tauri invoke.

pub mod cloud;
pub mod lora;
pub mod ollama;
pub mod save;

pub use cloud::{list_cloud_models_impl, probe_cloud_llm_impl, ListCloudModelsRequest};
pub use lora::{
    activate_local_lora_adapter_impl, delete_local_lora_adapter_impl,
    import_local_lora_adapter_impl,
};
pub use ollama::{
    get_global_ollama_model_impl, list_ollama_models_impl, set_global_ollama_model_impl,
    GlobalOllamaModelDto, SetGlobalOllamaModelRequest,
};
pub use save::{save_llm_user_settings_impl, set_session_llm_model_impl};

use crate::command_error::CommandError;
use crate::domain::effective_llm_model::{
    is_usable_ollama_model_id, resolve_effective_ollama_model,
};
use crate::domain::user_llm_env::{
    apply_user_llm_env, ollama_base_from_db_or_env, KEY_CLOUD_STYLE, KEY_CLOUD_VENDOR,
    KEY_LLM_PROVIDER, KEY_LOCAL_LORA_ADAPTER_ID, KEY_LOCAL_MODEL_PATH, KEY_REMOTE_MODEL,
    KEY_REMOTE_TOKEN, KEY_REMOTE_URL,
};
use crate::infrastructure::llm_models::{
    canonical_models_dir, describe_local_model_file, local_models_dir_for_state,
    scan_local_model_files_in, LocalModelFileDto,
};
use crate::infrastructure::lora_adapters::list_local_lora_adapters;
use crate::infrastructure::ollama_client::OllamaClient;
use crate::service::role::session_namespace;
use crate::state::{paths_equal, AppState};
use oclive_kernel_types::models::plugin_backends::LlmBackend;
use oclive_kernel_types::models::LocalLoraAdapterDto;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

pub(super) static LOCAL_LORA_MUTATION: Lazy<tokio::sync::Mutex<()>> =
    Lazy::new(|| tokio::sync::Mutex::new(()));

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LlmUserSettingsDto {
    pub provider: String,
    pub cloud_vendor: String,
    pub cloud_api_style: String,
    pub ollama_base_url: String,
    pub ollama_reachable: bool,
    pub ollama_detail: String,
    pub local_models_dir: String,
    pub local_model_files: Vec<LocalModelFileDto>,
    pub local_model_path: String,
    pub local_lora_adapters: Vec<LocalLoraAdapterDto>,
    pub active_local_lora_adapter_id: Option<String>,
    pub local_runtime_mode: String,
    pub performance_endpoint: String,
    pub performance_runtime_available: bool,
    pub performance_model_configured: bool,
    pub performance_ready: bool,
    pub performance_active_backend: String,
    pub performance_detail: String,
    pub pack_ollama_model: Option<String>,
    pub session_ollama_model: Option<String>,
    pub effective_model: String,
    pub remote_url: String,
    pub remote_token_configured: bool,
    pub remote_model: String,
    pub remote_url_env_active: bool,
    pub remote_token_env_active: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveLlmUserSettingsRequest {
    pub role_id: String,
    pub session_id: Option<String>,
    pub provider: String,
    pub cloud_vendor: Option<String>,
    pub cloud_api_style: Option<String>,
    pub ollama_base_url: Option<String>,
    pub local_models_dir: Option<String>,
    pub local_model_path: Option<String>,
    #[serde(default)]
    pub adult_content_acknowledged: bool,
    pub ollama_model: Option<String>,
    pub remote_url: Option<String>,
    pub remote_token: Option<String>,
    pub remote_model: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetSessionLlmModelRequest {
    pub role_id: String,
    #[serde(default)]
    pub session_id: Option<String>,
    /// Model id; null/empty clears override.
    #[serde(default)]
    pub model: Option<String>,
}

/// Re-apply persisted LLM environment settings without racing model/LoRA mutations.
///
/// # Errors
///
/// Returns database or environment-application errors.
pub async fn reload_llm_user_env_impl(state: &AppState) -> Result<String, CommandError> {
    let _guard = LOCAL_LORA_MUTATION.lock().await;
    state.mark_user_llm_env_dirty();
    apply_user_llm_env(state).await?;
    Ok(state.user_llm_provider.read().clone())
}

/// # Errors
///
/// Returns [`Err`] when persistence or role reload fails.
pub async fn get_llm_user_settings_impl(
    state: &AppState,
    role_id: &str,
    session_id: Option<&str>,
) -> Result<LlmUserSettingsDto, CommandError> {
    let role = state.load_role_cached_async(role_id).await?;
    let ns = session_namespace(role_id, session_id);
    state.db_manager.ensure_role_runtime(ns.as_str()).await?;

    let mut session_ollama_model = state
        .db_manager
        .get_session_ollama_model_override(ns.as_str())
        .await?;
    if session_ollama_model
        .as_deref()
        .is_some_and(|m| !is_usable_ollama_model_id(m))
    {
        state
            .db_manager
            .clear_session_ollama_model_override(ns.as_str())
            .await?;
        session_ollama_model = None;
    }

    let effective = resolve_effective_ollama_model(state, role.as_ref(), ns.as_str()).await?;
    let plugin_backends_effective =
        state.effective_plugin_backends_for_session(role.as_ref(), ns.as_str());
    let provider = state
        .db_manager
        .get_app_setting(KEY_LLM_PROVIDER)
        .await?
        .map(|s| s.trim().to_ascii_lowercase())
        .filter(|p| p == "local" || p == "cloud")
        .unwrap_or_else(|| {
            if matches!(plugin_backends_effective.llm, LlmBackend::Remote) {
                "cloud".to_string()
            } else {
                "local".to_string()
            }
        });

    let ollama_base_url = ollama_base_from_db_or_env(state).await;
    let client = OllamaClient::new(ollama_base_url.clone());
    let ollama_reachable = client.health_check().await.unwrap_or(false);
    let ollama_detail = if ollama_reachable {
        String::new()
    } else {
        "Ollama unreachable".to_string()
    };

    let pack_ollama_model = role.ollama_model.clone();

    let remote_url = state
        .db_manager
        .get_app_setting(KEY_REMOTE_URL)
        .await?
        .unwrap_or_default();
    let remote_model = state
        .db_manager
        .get_app_setting(KEY_REMOTE_MODEL)
        .await?
        .unwrap_or_default();
    let remote_token_configured = state
        .db_manager
        .get_app_setting(KEY_REMOTE_TOKEN)
        .await?
        .map(|s| !s.trim().is_empty())
        .unwrap_or(false);
    let cloud_vendor = state
        .db_manager
        .get_app_setting(KEY_CLOUD_VENDOR)
        .await?
        .unwrap_or_else(|| "custom".to_string());
    let cloud_api_style = state
        .db_manager
        .get_app_setting(KEY_CLOUD_STYLE)
        .await?
        .unwrap_or_else(|| "openai".to_string());
    let local_models_dir = local_models_dir_for_state(state).await?;
    let mut local_model_files =
        scan_local_model_files_in(std::path::Path::new(local_models_dir.trim()));
    let local_model_path = state
        .db_manager
        .get_app_setting(KEY_LOCAL_MODEL_PATH)
        .await?
        .unwrap_or_default();
    if !local_model_path.trim().is_empty()
        && !local_model_files.iter().any(|model| {
            paths_equal(
                std::path::Path::new(&model.path),
                std::path::Path::new(local_model_path.trim()),
            )
        })
    {
        match describe_local_model_file(std::path::Path::new(local_model_path.trim())) {
            Ok(model) => {
                local_model_files.push(model);
                local_model_files.sort_by_key(|model| model.name.to_lowercase());
            }
            Err(error) => {
                tracing::warn!(
                    target: "oclive_models",
                    path = local_model_path.trim(),
                    %error,
                    "configured local base model metadata could not be loaded"
                );
            }
        }
    }
    let configured_lora_id = state
        .db_manager
        .get_app_setting(KEY_LOCAL_LORA_ADAPTER_ID)
        .await?
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());
    let managed_models_dir = canonical_models_dir(state);
    let local_lora_adapters =
        list_local_lora_adapters(&managed_models_dir, configured_lora_id.as_deref());
    let active_local_lora_adapter_id = configured_lora_id.filter(|configured| {
        local_lora_adapters
            .iter()
            .any(|adapter| adapter.active && adapter.id == *configured)
    });
    let performance = if let Some(runtime) = state.performance_llm.as_ref() {
        runtime.inspect().await
    } else {
        crate::infrastructure::performance_llm::PerformanceLlmStatus {
            mode: state.host_profile.llm_runtime.mode.as_str().into(),
            endpoint: state.host_profile.llm_runtime.endpoint.clone(),
            ready: false,
            runtime_installed: false,
            model_configured: !local_model_path.trim().is_empty(),
            active_backend: "ollama".into(),
            detail: "distro profile uses Ollama-only local runtime".into(),
        }
    };

    Ok(LlmUserSettingsDto {
        provider: provider.to_string(),
        cloud_vendor,
        cloud_api_style,
        ollama_base_url,
        ollama_reachable,
        ollama_detail,
        local_models_dir,
        local_model_files,
        local_model_path,
        local_lora_adapters,
        active_local_lora_adapter_id,
        local_runtime_mode: performance.mode,
        performance_endpoint: performance.endpoint,
        performance_runtime_available: performance.runtime_installed,
        performance_model_configured: performance.model_configured,
        performance_ready: performance.ready,
        performance_active_backend: performance.active_backend,
        performance_detail: performance.detail,
        pack_ollama_model,
        session_ollama_model,
        effective_model: effective,
        remote_url,
        remote_token_configured,
        remote_model,
        remote_url_env_active: std::env::var("OCLIVE_REMOTE_LLM_URL")
            .map(|s| !s.trim().is_empty())
            .unwrap_or(false),
        remote_token_env_active: std::env::var("OCLIVE_REMOTE_LLM_TOKEN")
            .map(|s| !s.trim().is_empty())
            .unwrap_or(false),
    })
}
