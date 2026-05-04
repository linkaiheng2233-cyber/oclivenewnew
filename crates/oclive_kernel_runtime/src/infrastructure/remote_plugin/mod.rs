//! HTTP JSON-RPC 侧车：环境变量启用后与 `plugin_backends.* = remote` 对接。
//!
//! - `OCLIVE_REMOTE_PLUGIN_URL`：记忆 / 情绪 / 事件 / Prompt（共用一端点，方法名区分）
//! - `OCLIVE_REMOTE_LLM_URL`：主对话 LLM（`llm.generate` / `llm.generate_tag`）
//!
//! 详见 `docs/REMOTE_PLUGIN_PROTOCOL.md`。

#[cfg(feature = "kernel-agent")]
mod agent_http;
mod complex_emotion_http;
mod config;
mod emotion_http;
mod event_http;
mod jsonrpc;
mod llm_http;
mod memory_http;
mod prompt_http;

#[cfg(feature = "kernel-agent")]
pub use agent_http::RemoteAgentHttp;
pub use complex_emotion_http::RemoteComplexEmotionHttp;
pub use config::RemotePluginHttpConfig;
pub use emotion_http::RemoteUserEmotionAnalyzerHttp;
pub use event_http::RemoteEventEstimatorHttp;
pub use llm_http::RemoteLlmHttp;
pub use memory_http::RemoteMemoryRetrievalHttp;
pub use prompt_http::RemotePromptAssemblerHttp;

use crate::domain::complex_emotion::{
    ComplexEmotionProvider, DegradedToBuiltinComplexEmotionProvider,
};
use crate::domain::event_estimator::{EventEstimator, RemoteEventEstimatorPlaceholder};
use crate::domain::memory_retrieval::{MemoryRetrieval, RemoteMemoryRetrievalPlaceholder};
use crate::domain::prompt_assembler::{PromptAssembler, RemotePromptAssemblerPlaceholder};
use crate::domain::user_emotion_analyzer::{
    RemoteUserEmotionAnalyzerPlaceholder, UserEmotionAnalyzer,
};
use crate::infrastructure::cloud_llm::{
    effective_cloud_llm_config, CloudLlmConfig, OpenAiCompatLlmClient,
};
use crate::infrastructure::llm::{LlmClient, RemoteLlmPlaceholder};
use async_trait::async_trait;
use parking_lot::RwLock;
use serde_json::Value;
use std::sync::Arc;

use crate::error::{AppError, Result};
/// 对 `OCLIVE_REMOTE_PLUGIN_URL` 同源端点发起异步 JSON-RPC（供宿主/集成测试使用）。
pub use jsonrpc::call_async as remote_plugin_call_async;
pub use jsonrpc::RemoteRpcChannel;

/// 四类 `plugin_backends.* = remote` 共用一套配置，只读一次环境变量并打一条日志。
pub struct PluginRemoteGroup {
    pub memory: Arc<dyn MemoryRetrieval>,
    pub emotion: Arc<dyn UserEmotionAnalyzer>,
    pub event: Arc<dyn EventEstimator>,
    pub prompt: Arc<dyn PromptAssembler>,
}

pub fn plugin_remote_group() -> PluginRemoteGroup {
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

pub fn llm_remote_backend(
    default_llm: Arc<dyn LlmClient>,
    cloud_store: Arc<RwLock<Option<CloudLlmConfig>>>,
) -> Arc<dyn LlmClient> {
    Arc::new(LlmRemoteCloudAware::new(default_llm, cloud_store))
}

/// 远程 LLM 槽：应用内/环境变量 OpenAI 兼容云端优先，其次 JSON-RPC 侧车，否则占位回退。
struct LlmRemoteCloudAware {
    cloud_store: Arc<RwLock<Option<CloudLlmConfig>>>,
    chain: Arc<dyn LlmClient>,
}

impl LlmRemoteCloudAware {
    fn new(
        default_llm: Arc<dyn LlmClient>,
        cloud_store: Arc<RwLock<Option<CloudLlmConfig>>>,
    ) -> Self {
        let chain: Arc<dyn LlmClient> = if let Some(cfg) = RemotePluginHttpConfig::from_env_llm() {
            log::info!(
                target: "oclive_plugin",
                "remote LLM HTTP active -> {}",
                cfg.endpoint
            );
            Arc::new(RemoteLlmHttp::new(cfg))
        } else {
            Arc::new(RemoteLlmPlaceholder::new(default_llm))
        };
        Self { cloud_store, chain }
    }
}

#[async_trait]
impl LlmClient for LlmRemoteCloudAware {
    async fn generate(&self, model: &str, prompt: &str) -> Result<String> {
        if let Some(cfg) = effective_cloud_llm_config(&self.cloud_store) {
            return OpenAiCompatLlmClient::new(cfg)
                .generate(model, prompt)
                .await;
        }
        self.chain.generate(model, prompt).await
    }

    async fn generate_tag(&self, model: &str, prompt: &str) -> Result<String> {
        if let Some(cfg) = effective_cloud_llm_config(&self.cloud_store) {
            return OpenAiCompatLlmClient::new(cfg)
                .generate_tag(model, prompt)
                .await;
        }
        self.chain.generate_tag(model, prompt).await
    }
}

pub fn agent_remote_backend(
    default_agent: Arc<dyn crate::domain::agent::AgentProvider>,
) -> Arc<dyn crate::domain::agent::AgentProvider> {
    #[cfg(feature = "kernel-agent")]
    {
        if let Some(cfg) = RemotePluginHttpConfig::from_env_agent() {
            log::info!(
                target: "oclive_plugin",
                "remote Agent HTTP active -> {}",
                cfg.endpoint
            );
            return Arc::new(RemoteAgentHttp::new(cfg));
        }
    }
    #[cfg(not(feature = "kernel-agent"))]
    if RemotePluginHttpConfig::from_env_agent().is_some() {
        log::warn!(
            target: "oclive_plugin",
            "OCLIVE_REMOTE_AGENT_URL is set but kernel-agent feature is disabled; ignoring"
        );
    }
    default_agent
}

pub fn complex_emotion_remote_backend() -> Arc<dyn ComplexEmotionProvider> {
    if let Some(cfg) = RemotePluginHttpConfig::from_env_complex_emotion() {
        log::info!(
            target: "oclive_plugin",
            "remote complex_emotion HTTP active -> {}",
            cfg.endpoint
        );
        Arc::new(RemoteComplexEmotionHttp::new(cfg))
    } else {
        Arc::new(DegradedToBuiltinComplexEmotionProvider::new(
            "complex_emotion backend Remote is not connected; using builtin complex emotion",
        ))
    }
}

/// 对目录插件（或任意已解析 RPC 根 URL）发起单次 JSON-RPC `call`（原生 async）。
pub async fn invoke_directory_plugin_rpc(
    url: &str,
    method: &str,
    params: Value,
    channel: RemoteRpcChannel,
) -> Result<Value> {
    let cfg = RemotePluginHttpConfig::for_directory_plugin_rpc(
        url,
        matches!(channel, RemoteRpcChannel::Llm),
    );
    let client = reqwest::Client::builder()
        .connect_timeout(cfg.connect_timeout())
        .timeout(cfg.timeout)
        .build()
        .map_err(|e| {
            AppError::OllamaError(format!(
                "directory plugin reqwest client build failed: {}",
                e
            ))
        })?;
    jsonrpc::call_async(
        channel,
        &client,
        &cfg.endpoint,
        method,
        params,
        cfg.bearer_token.as_deref(),
    )
    .await
}
