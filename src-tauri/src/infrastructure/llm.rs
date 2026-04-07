//! LLM 调用抽象，便于测试与替换实现。
//!
//! 主对话与标签任务的温度、top_p 见 [`super::llm_params`]（环境变量 `OCLIVE_LLM_*`）。

use crate::error::Result;
use crate::infrastructure::llm_params;
use crate::infrastructure::ollama_client::OllamaClient;
use async_trait::async_trait;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

#[async_trait]
pub trait LlmClient: Send + Sync {
    async fn generate(&self, model: &str, prompt: &str) -> Result<String>;
    /// 低温度短输出（立绘标签等分类任务）
    async fn generate_tag(&self, model: &str, prompt: &str) -> Result<String>;
}

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
pub fn ollama_llm(client: OllamaClient) -> Arc<dyn LlmClient> {
    Arc::new(client)
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
