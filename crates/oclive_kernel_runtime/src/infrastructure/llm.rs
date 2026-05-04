//! LLM 调用抽象与占位实现；**trait 定义**在 `oclive_kernel_core`。
//!
//! 主对话与标签任务的温度、top_p 见 [`super::llm_params`]（环境变量 `OCLIVE_LLM_*`）。

pub use oclive_kernel_core::llm::LlmClient;

use crate::error::Result;
#[cfg(not(feature = "default-llm-providers"))]
use crate::error::AppError;
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

/// 内置 Ollama/云兼容客户端关闭时：占位实现（须改用 remote/directory LLM）。
#[cfg(not(feature = "default-llm-providers"))]
#[derive(Debug, Default, Clone, Copy)]
pub struct BuiltinLlmDisabledClient;

#[cfg(not(feature = "default-llm-providers"))]
#[async_trait]
impl LlmClient for BuiltinLlmDisabledClient {
    async fn generate(&self, _model: &str, _prompt: &str) -> Result<String> {
        Err(AppError::InvalidParameter(
            "default-llm-providers feature disabled: no built-in Ollama/cloud LLM; set plugin_backends.llm to remote or directory"
                .into(),
        ))
    }

    async fn generate_tag(&self, _model: &str, _prompt: &str) -> Result<String> {
        Err(AppError::InvalidParameter(
            "default-llm-providers feature disabled: no built-in LLM for tag tasks"
                .into(),
        ))
    }
}

/// 供 `KernelAppState::new` 在关闭内置 LLM 时使用。
#[cfg(not(feature = "default-llm-providers"))]
pub fn default_runtime_llm_arc() -> Arc<dyn LlmClient> {
    Arc::new(BuiltinLlmDisabledClient)
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
