//! LLM 调用抽象，便于测试与替换实现。
//!
//! 主对话与标签任务的温度、top_p 见 [`super::llm_params`]（环境变量 `OCLIVE_LLM_*`）。

use crate::error::{AppError, Result};
use crate::infrastructure::llm_params;
use crate::infrastructure::ollama_client::OllamaClient;
use crate::infrastructure::remote_fallback_policy::remote_fallback_load;
use async_trait::async_trait;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

#[async_trait]
pub trait LlmClient: Send + Sync {
    async fn generate(&self, model: &str, prompt: &str) -> Result<String>;
    /// 低温度短输出（立绘标签等分类任务）
    async fn generate_tag(&self, model: &str, prompt: &str) -> Result<String>;
    /// 启动期可选探活（不默认失败；Ollama 实现会 ping 服务，失败仅打日志）。
    async fn startup_probe(&self) -> Result<()> {
        Ok(())
    }
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

    async fn startup_probe(&self) -> Result<()> {
        match self.health_check().await {
            Ok(true) => Ok(()),
            Ok(false) => {
                tracing::warn!(
                    target: "oclive_startup",
                    "Ollama 服务不可达（/api/tags 非成功）；首条对话仍可能走 fallback"
                );
                Ok(())
            }
            Err(e) => {
                tracing::warn!(
                    target: "oclive_startup",
                    "Ollama health_check 异常: {}",
                    e
                );
                Ok(())
            }
        }
    }
}

/// 将 `OllamaClient` 包成 `Arc<dyn LlmClient>`
#[must_use]
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

/// `plugin_backends.llm = remote` 时占位：未配置 `OCLIVE_REMOTE_LLM_URL` 或侧车客户端构建失败时生效。
/// 允许降级时委托内置客户端并记一次警告；否则返回 [`AppError::RemoteServiceUnavailable`]。
pub struct RemoteLlmPlaceholder {
    inner: Arc<dyn LlmClient>,
    warned: AtomicBool,
    remote_fallback_allowed: Arc<AtomicBool>,
}

impl RemoteLlmPlaceholder {
    pub fn new(inner: Arc<dyn LlmClient>, remote_fallback_allowed: Arc<AtomicBool>) -> Self {
        Self {
            inner,
            warned: AtomicBool::new(false),
            remote_fallback_allowed,
        }
    }

    fn warn_once(&self) {
        if self
            .warned
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
        {
            tracing::warn!(
                target: "oclive_plugin",
                "llm backend Remote is not connected; using configured LlmClient"
            );
        }
    }
}

#[async_trait]
impl LlmClient for RemoteLlmPlaceholder {
    async fn generate(&self, model: &str, prompt: &str) -> Result<String> {
        if remote_fallback_load(&self.remote_fallback_allowed) {
            self.warn_once();
            return self.inner.generate(model, prompt).await;
        }
        Err(AppError::RemoteServiceUnavailable(
            "llm backend Remote is not connected (set OCLIVE_REMOTE_LLM_URL or enable remote fallback to builtin)"
                .to_string(),
        ))
    }

    async fn generate_tag(&self, model: &str, prompt: &str) -> Result<String> {
        if remote_fallback_load(&self.remote_fallback_allowed) {
            self.warn_once();
            return self.inner.generate_tag(model, prompt).await;
        }
        Err(AppError::RemoteServiceUnavailable(
            "llm backend Remote is not connected (set OCLIVE_REMOTE_LLM_URL or enable remote fallback to builtin)"
                .to_string(),
        ))
    }

    async fn startup_probe(&self) -> Result<()> {
        self.inner.startup_probe().await
    }
}
