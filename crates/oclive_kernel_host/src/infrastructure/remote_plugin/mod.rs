//! HTTP JSON-RPC sidecar: wired to `plugin_backends.* = remote` when enabled via environment variables.
//!
//! - `OCLIVE_REMOTE_PLUGIN_URL`: memory / emotion / event / prompt (shared endpoint, method names differ)
//! - `OCLIVE_REMOTE_LLM_URL`: main chat LLM (`llm.generate` / `llm.generate_tag`)
//!
//! See `docs/REMOTE_PLUGIN_PROTOCOL.md`.

mod adapter;
mod complex_emotion_directory_http;
mod complex_emotion_http;
mod config;
mod emotion_http;
mod event_http;
mod jsonrpc;
mod llm_http;
mod memory_http;
mod prompt_http;
mod remote_client;

pub use complex_emotion_directory_http::DirectoryComplexEmotionHttp;
pub use complex_emotion_http::RemoteComplexEmotionHttp;
pub use config::RemotePluginHttpConfig;
pub use emotion_http::RemoteUserEmotionAnalyzerHttp;
pub use event_http::RemoteEventEstimatorHttp;
pub use llm_http::RemoteLlmHttp;
pub use memory_http::RemoteMemoryRetrievalHttp;
pub use prompt_http::RemotePromptAssemblerHttp;

use crate::domain::event_estimator::{EventEstimator, RemoteEventEstimatorPlaceholder};
use crate::domain::memory_retrieval::{MemoryRetrieval, RemoteMemoryRetrievalPlaceholder};
use crate::domain::prompt_assembler::{PromptAssembler, RemotePromptAssemblerPlaceholder};
use crate::domain::user_emotion_analyzer::{
    RemoteUserEmotionAnalyzerPlaceholder, UserEmotionAnalyzer,
};
use crate::error::{AppError, Result};
use crate::infrastructure::high_risk_grants::HighRiskGrantStore;
use crate::infrastructure::llm::{LlmClient, RemoteLlmPlaceholder};
use serde_json::Value;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::time::Duration;

pub use jsonrpc::RemoteRpcChannel;
use oclive_validation::{NETWORK_GRANT_REMOTE_LLM, NETWORK_GRANT_REMOTE_PLUGIN};
pub use remote_client::{RemoteHttpClientAsync, RemoteHttpClientBlocking};

/// Shared Remote HTTP connection pool (no global request timeout; per-RPC timeout in [`jsonrpc`] layer).
pub(crate) fn build_shared_remote_http_client() -> Arc<reqwest::Client> {
    Arc::new(
        reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(15))
            .pool_max_idle_per_host(8)
            .build()
            .expect("shared remote reqwest client"),
    )
}

/// Shared config for four `plugin_backends.* = remote` slots; reads env once and logs once.
pub(crate) struct PluginRemoteGroup {
    pub memory: Arc<dyn MemoryRetrieval>,
    pub emotion: Arc<dyn UserEmotionAnalyzer>,
    pub event: Arc<dyn EventEstimator>,
    pub prompt: Arc<dyn PromptAssembler>,
}

fn plugin_remote_placeholder_group() -> PluginRemoteGroup {
    PluginRemoteGroup {
        memory: Arc::new(RemoteMemoryRetrievalPlaceholder::new()),
        emotion: Arc::new(RemoteUserEmotionAnalyzerPlaceholder::new()),
        event: Arc::new(RemoteEventEstimatorPlaceholder::new()),
        prompt: Arc::new(RemotePromptAssemblerPlaceholder::new()),
    }
}

pub(crate) fn plugin_remote_group(
    http_client: Arc<reqwest::Client>,
    remote_fallback_allowed: Arc<AtomicBool>,
    grants: Arc<HighRiskGrantStore>,
) -> PluginRemoteGroup {
    let Some(cfg) = RemotePluginHttpConfig::from_env_plugin() else {
        return plugin_remote_placeholder_group();
    };
    tracing::info!(
        target: "oclive_plugin",
        "remote plugin HTTP active (memory/emotion/event/prompt) -> {}",
        cfg.endpoint
    );
    let fb = remote_fallback_allowed.clone();
    let g = grants.clone();
    let ng = Some(NETWORK_GRANT_REMOTE_PLUGIN.to_string());
    let memory = RemoteMemoryRetrievalHttp::new(
        http_client.clone(),
        cfg.clone(),
        fb.clone(),
        g.clone(),
        ng.clone(),
    );
    let emotion = RemoteUserEmotionAnalyzerHttp::new(
        http_client.clone(),
        cfg.clone(),
        fb.clone(),
        g.clone(),
        ng.clone(),
    );
    let event = RemoteEventEstimatorHttp::new(
        http_client.clone(),
        cfg.clone(),
        fb.clone(),
        g.clone(),
        ng.clone(),
    );
    let prompt = RemotePromptAssemblerHttp::new(http_client, cfg, fb, g, ng);
    PluginRemoteGroup {
        memory: Arc::new(memory),
        emotion: Arc::new(emotion),
        event: Arc::new(event),
        prompt: Arc::new(prompt),
    }
}

fn cloud_api_style_is_openai() -> bool {
    !matches!(
        std::env::var("OCLIVE_LLM_CLOUD_API_STYLE")
            .ok()
            .as_deref()
            .map(str::trim)
            .map(str::to_ascii_lowercase)
            .as_deref(),
        Some("oclive_jsonrpc")
    )
}

pub fn llm_remote_backend(
    http_client: Arc<reqwest::Client>,
    default_llm: Arc<dyn LlmClient>,
    remote_fallback_allowed: Arc<AtomicBool>,
    grants: Arc<HighRiskGrantStore>,
) -> Arc<dyn LlmClient> {
    if cloud_api_style_is_openai() {
        if let Some(openai) =
            crate::infrastructure::openai_compatible_llm::OpenAiCompatibleLlm::from_env(
                (*http_client).clone(),
                grants.clone(),
            )
        {
            tracing::info!(
                target: "oclive_plugin",
                "remote LLM OpenAI-compatible active -> {}",
                openai.endpoint()
            );
            return Arc::new(openai);
        }
    }
    if let Some(cfg) = RemotePluginHttpConfig::from_env_llm() {
        tracing::info!(
            target: "oclive_plugin",
            "remote LLM JSON-RPC active -> {}",
            cfg.endpoint
        );
        return Arc::new(RemoteLlmHttp::new(
            http_client,
            cfg,
            grants,
            Some(NETWORK_GRANT_REMOTE_LLM.to_string()),
        ));
    }
    Arc::new(RemoteLlmPlaceholder::new(
        default_llm,
        remote_fallback_allowed,
    ))
}
/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
/// Issues a single blocking JSON-RPC `call` to a directory plugin (or any resolved RPC root URL); used by `directory_plugin_invoke` and similar.
pub fn invoke_directory_plugin_rpc_blocking(
    url: &str,
    method: &str,
    params: Value,
    channel: RemoteRpcChannel,
) -> Result<Value> {
    let cfg = RemotePluginHttpConfig::for_directory_plugin_rpc(
        url,
        matches!(channel, RemoteRpcChannel::Llm),
    );
    let http = RemoteHttpClientBlocking::new_standalone(
        cfg,
        HighRiskGrantStore::load(std::env::temp_dir(), false),
        None,
    )
    .map_err(|e| {
        AppError::OllamaError(format!(
            "directory plugin reqwest client build failed: {}",
            e
        ))
    })?;
    http.call(channel, method, params)
}
