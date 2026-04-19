//! HTTP JSON-RPC 侧车：环境变量启用后与 `plugin_backends.* = remote` 对接。
//!
//! - `OCLIVE_REMOTE_PLUGIN_URL`：记忆 / 情绪 / 事件 / Prompt（共用一端点，方法名区分）
//! - `OCLIVE_REMOTE_LLM_URL`：主对话 LLM（`llm.generate` / `llm.generate_tag`）
//!
//! 详见 `docs/REMOTE_PLUGIN_PROTOCOL.md`。

mod complex_emotion_http;
mod config;
mod emotion_http;
mod event_http;
mod jsonrpc;
mod llm_http;
mod memory_http;
mod prompt_http;

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
use std::sync::Arc;

use crate::error::{AppError, Result};
use jsonrpc::call_blocking;
pub use jsonrpc::RemoteRpcChannel;

/// 四类 `plugin_backends.* = remote` 共用一套配置，只读一次环境变量并打一条日志。
pub(crate) struct PluginRemoteGroup {
    pub memory: Arc<dyn MemoryRetrieval>,
    pub emotion: Arc<dyn UserEmotionAnalyzer>,
    pub event: Arc<dyn EventEstimator>,
    pub prompt: Arc<dyn PromptAssembler>,
}

pub(crate) fn plugin_remote_group() -> PluginRemoteGroup {
    match RemotePluginHttpConfig::from_env_plugin() {
        Some(cfg) => {
            log::info!(
                target: "oclive_plugin",
                "remote plugin HTTP active (memory/emotion/event/prompt) -> {}",
                cfg.endpoint
            );
            PluginRemoteGroup {
                memory: Arc::new(RemoteMemoryRetrievalHttp::new(cfg.clone())),
                emotion: Arc::new(RemoteUserEmotionAnalyzerHttp::new(cfg.clone())),
                event: Arc::new(RemoteEventEstimatorHttp::new(cfg.clone())),
                prompt: Arc::new(RemotePromptAssemblerHttp::new(cfg)),
            }
        }
        None => PluginRemoteGroup {
            memory: Arc::new(RemoteMemoryRetrievalPlaceholder::new()),
            emotion: Arc::new(RemoteUserEmotionAnalyzerPlaceholder::new()),
            event: Arc::new(RemoteEventEstimatorPlaceholder::new()),
            prompt: Arc::new(RemotePromptAssemblerPlaceholder::new()),
        },
    }
}

pub fn llm_remote_backend(default_llm: Arc<dyn LlmClient>) -> Arc<dyn LlmClient> {
    if let Some(cfg) = RemotePluginHttpConfig::from_env_llm() {
        log::info!(
            target: "oclive_plugin",
            "remote LLM HTTP active -> {}",
            cfg.endpoint
        );
        Arc::new(RemoteLlmHttp::new(cfg))
    } else {
        Arc::new(RemoteLlmPlaceholder::new(default_llm))
    }
}

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
    )
}
