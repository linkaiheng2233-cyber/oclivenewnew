//! HTTP JSON-RPC 侧车：环境变量启用后与 `plugin_backends.* = remote` 对接。
//!
//! - `OCLIVE_REMOTE_PLUGIN_URL`：记忆 / 情绪 / 事件 / Prompt（共用一端点，方法名区分）
//! - `OCLIVE_REMOTE_LLM_URL`：主对话 LLM（`llm.generate` / `llm.generate_tag`）
//!
//! 详见 `docs/REMOTE_PLUGIN_PROTOCOL.md`。

mod complex_emotion_directory_http;
mod complex_emotion_http;
mod config;
mod emotion_http;
mod event_http;
mod jsonrpc;
mod llm_http;
mod memory_http;
mod prompt_http;

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
use crate::infrastructure::llm::{LlmClient, RemoteLlmPlaceholder};
use serde_json::Value;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use crate::error::{AppError, Result};
use crate::infrastructure::high_risk_grants::HighRiskGrantStore;
use jsonrpc::call_blocking;
pub use jsonrpc::RemoteRpcChannel;
use oclive_validation::{NETWORK_GRANT_REMOTE_LLM, NETWORK_GRANT_REMOTE_PLUGIN};

/// 四类 `plugin_backends.* = remote` 共用一套配置，只读一次环境变量并打一条日志。
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
    let memory = RemoteMemoryRetrievalHttp::new(cfg.clone(), fb.clone(), g.clone(), ng.clone());
    let emotion =
        RemoteUserEmotionAnalyzerHttp::new(cfg.clone(), fb.clone(), g.clone(), ng.clone());
    let event = RemoteEventEstimatorHttp::new(cfg.clone(), fb.clone(), g.clone(), ng.clone());
    let prompt = RemotePromptAssemblerHttp::new(cfg, fb, g, ng);
    match (memory, emotion, event, prompt) {
        (Ok(memory), Ok(emotion), Ok(event), Ok(prompt)) => PluginRemoteGroup {
            memory: Arc::new(memory),
            emotion: Arc::new(emotion),
            event: Arc::new(event),
            prompt: Arc::new(prompt),
        },
        (m, e, ev, p) => {
            let mut parts = Vec::new();
            if let Err(err) = m {
                parts.push(format!("memory: {}", err));
            }
            if let Err(err) = e {
                parts.push(format!("emotion: {}", err));
            }
            if let Err(err) = ev {
                parts.push(format!("event: {}", err));
            }
            if let Err(err) = p {
                parts.push(format!("prompt: {}", err));
            }
            tracing::error!(
                target: "oclive_plugin",
                "remote plugin HTTP reqwest client build failed ({}); disabling remote plugin group",
                parts.join("; ")
            );
            plugin_remote_placeholder_group()
        }
    }
}

pub fn llm_remote_backend(
    default_llm: Arc<dyn LlmClient>,
    remote_fallback_allowed: Arc<AtomicBool>,
    grants: Arc<HighRiskGrantStore>,
) -> Arc<dyn LlmClient> {
    if let Some(cfg) = RemotePluginHttpConfig::from_env_llm() {
        tracing::info!(
            target: "oclive_plugin",
            "remote LLM HTTP active -> {}",
            cfg.endpoint
        );
        match RemoteLlmHttp::new(cfg, grants, Some(NETWORK_GRANT_REMOTE_LLM.to_string())) {
            Ok(remote) => Arc::new(remote),
            Err(e) => {
                tracing::error!(
                    target: "oclive_plugin",
                    "remote LLM HTTP reqwest client build failed: {}; using default LLM",
                    e
                );
                Arc::new(RemoteLlmPlaceholder::new(
                    default_llm,
                    remote_fallback_allowed.clone(),
                ))
            }
        }
    } else {
        Arc::new(RemoteLlmPlaceholder::new(
            default_llm,
            remote_fallback_allowed,
        ))
    }
}
/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
/// 对目录插件（或任意已解析 RPC 根 URL）发起单次 JSON-RPC `call`（阻塞）；供 `directory_plugin_invoke` 等使用。
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
    let client = reqwest::blocking::Client::builder()
        .connect_timeout(cfg.connect_timeout())
        .timeout(cfg.timeout)
        .build()
        .map_err(|e| {
            AppError::OllamaError(format!(
                "directory plugin reqwest client build failed: {}",
                e
            ))
        })?;
    call_blocking(
        channel,
        &client,
        &cfg.endpoint,
        method,
        params,
        cfg.bearer_token.as_deref(),
        None,
    )
}
