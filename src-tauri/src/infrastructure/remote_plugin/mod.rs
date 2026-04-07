//! HTTP JSON-RPC 侧车：环境变量启用后与 `plugin_backends.* = remote` 对接。
//!
//! - `OCLIVE_REMOTE_PLUGIN_URL`：记忆 / 情绪 / 事件 / Prompt（共用一端点，方法名区分）
//! - `OCLIVE_REMOTE_LLM_URL`：主对话 LLM（`llm.generate` / `llm.generate_tag`）
//!
//! 详见 `docs/REMOTE_PLUGIN_PROTOCOL.md`。

mod config;
mod emotion_http;
mod event_http;
mod jsonrpc;
mod llm_http;
mod memory_http;
mod prompt_http;

pub use config::RemotePluginHttpConfig;

use crate::domain::event_estimator::{EventEstimator, RemoteEventEstimatorPlaceholder};
use crate::domain::memory_retrieval::{MemoryRetrieval, RemoteMemoryRetrievalPlaceholder};
use crate::domain::prompt_assembler::{PromptAssembler, RemotePromptAssemblerPlaceholder};
use crate::domain::user_emotion_analyzer::{
    RemoteUserEmotionAnalyzerPlaceholder, UserEmotionAnalyzer,
};
use crate::infrastructure::llm::{LlmClient, RemoteLlmPlaceholder};
use emotion_http::RemoteUserEmotionAnalyzerHttp;
use event_http::RemoteEventEstimatorHttp;
use llm_http::RemoteLlmHttp;
use memory_http::RemoteMemoryRetrievalHttp;
use prompt_http::RemotePromptAssemblerHttp;
use std::sync::Arc;

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
