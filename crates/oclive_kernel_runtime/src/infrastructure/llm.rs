//! LLM 调用抽象与占位实现；**trait 定义**在 `oclive_kernel_core`。
//!
//! 主对话与标签任务的温度、top_p 见 [`super::llm_params`]（环境变量 `OCLIVE_LLM_*`）。

pub use oclive_kernel_core::llm::LlmClient;

use crate::error::{AppError, Result};
#[cfg(feature = "default-llm-providers")]
use crate::infrastructure::cloud_llm::{CloudLlmConfig, OpenAiCompatLlmClient};
#[cfg(feature = "default-llm-providers")]
use crate::infrastructure::llm_params;
#[cfg(feature = "default-llm-providers")]
use crate::infrastructure::ollama_client::OllamaClient;
use async_trait::async_trait;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

#[cfg(feature = "default-llm-providers")]
#[async_trait]
impl LlmClient for OllamaClient {
    async fn generate(&self, model: &str, prompt: &str) -> Result<String> {
        let (t, p) = llm_params::main_chat_options();
        OllamaClient::generate(self, model, prompt, t, p).await
    }

    async fn generate_tag(&self, model: &str, prompt: &str) -> Result<String> {
        let (t, p) = llm_params::tag_task_options();
        OllamaClient::generate(self, model, prompt, t, p).await
    }
}

/// 将 `OllamaClient` 包成 `Arc<dyn LlmClient>`
#[cfg(feature = "default-llm-providers")]
pub fn ollama_llm(client: OllamaClient) -> Arc<dyn LlmClient> {
    Arc::new(client)
}

/// 直连云端（OpenAI-compatible）优先；未配置则返回 `None`。
#[cfg(feature = "default-llm-providers")]
pub fn cloud_llm_from_env() -> Option<Arc<dyn LlmClient>> {
    let cfg = CloudLlmConfig::from_env_openai_compat()?;
    log::info!(
        target: "oclive_plugin",
        "cloud LLM HTTP active -> {}",
        cfg.base_url
    );
    Some(Arc::new(OpenAiCompatLlmClient::new(cfg)))
}

/// 内置 Ollama/云兼容与 `OCLIVE_REMOTE_LLM_URL` 侧车关闭时：占位实现（须用 **directory** 目录插件等提供 LLM）。
#[cfg(not(feature = "default-llm-providers"))]
#[derive(Debug, Default, Clone, Copy)]
pub struct BuiltinLlmDisabledClient;

#[cfg(not(feature = "default-llm-providers"))]
#[async_trait]
impl LlmClient for BuiltinLlmDisabledClient {
    async fn generate(&self, _model: &str, _prompt: &str) -> Result<String> {
        Err(AppError::InvalidParameter(
            "default-llm-providers feature disabled: no Ollama/cloud LLM or OCLIVE_REMOTE_LLM_URL sidecar; use plugin_backends.llm=directory (directory LLM plugin)"
                .into(),
        ))
    }

    async fn generate_tag(&self, _model: &str, _prompt: &str) -> Result<String> {
        Err(AppError::InvalidParameter(
            "default-llm-providers feature disabled: no built-in LLM for tag tasks".into(),
        ))
    }
}

/// 供 `KernelAppState::new` 在关闭内置 LLM 时使用。
#[cfg(not(feature = "default-llm-providers"))]
pub fn default_runtime_llm_arc() -> Arc<dyn LlmClient> {
    Arc::new(BuiltinLlmDisabledClient)
}

/// `LlmBackend::None`：拒绝主对话与标签生成（`MODULE_NONE_SEMANTICS.md` §5）。
#[derive(Debug, Default, Clone, Copy)]
pub struct NoneLlmClient;

#[async_trait]
impl LlmClient for NoneLlmClient {
    async fn generate(&self, _model: &str, _prompt: &str) -> Result<String> {
        Err(AppError::InvalidParameter(
            "当前对话引擎不可用（LLM 未启用，backend=none）。请在 Profile 或会话后端中选择可用 LLM。"
                .into(),
        ))
    }

    async fn generate_tag(&self, _model: &str, _prompt: &str) -> Result<String> {
        Err(AppError::InvalidParameter(
            "LLM 未启用（backend=none），无法执行标签/分类任务。".into(),
        ))
    }
}

#[must_use]
pub fn none_llm_client_arc() -> Arc<dyn LlmClient> {
    Arc::new(NoneLlmClient)
}

/// 测试或离线场景：固定返回，不访问网络
pub struct MockLlmClient {
    pub reply: String,
}

#[async_trait]
impl LlmClient for MockLlmClient {
    async fn generate(&self, _model: &str, _prompt: &str) -> Result<String> {
        Ok(self.reply.clone())
    }

    async fn generate_tag(&self, _model: &str, _prompt: &str) -> Result<String> {
        Ok("neutral".to_string())
    }
}

/// `plugin_backends.llm = remote` 时占位：委托内置客户端并记一次警告（与 memory Remote 策略一致）
pub struct RemoteLlmPlaceholder {
    inner: Arc<dyn LlmClient>,
    warned: AtomicBool,
}

impl RemoteLlmPlaceholder {
    pub fn new(inner: Arc<dyn LlmClient>) -> Self {
        Self {
            inner,
            warned: AtomicBool::new(false),
        }
    }

    fn warn_once(&self) {
        if self
            .warned
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
        {
            log::warn!(
                target: "oclive_plugin",
                "llm backend Remote is not connected; using configured LlmClient"
            );
        }
    }
}

#[async_trait]
impl LlmClient for RemoteLlmPlaceholder {
    async fn generate(&self, model: &str, prompt: &str) -> Result<String> {
        self.warn_once();
        self.inner.generate(model, prompt).await
    }

    async fn generate_tag(&self, model: &str, prompt: &str) -> Result<String> {
        self.warn_once();
        self.inner.generate_tag(model, prompt).await
    }
}
