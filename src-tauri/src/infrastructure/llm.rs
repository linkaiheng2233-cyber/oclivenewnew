//! LLM invocation abstraction for testing and swappable implementations.
//!
//! Main chat and tag-task temperature / top_p: see [`super::llm_params`] (env vars `OCLIVE_LLM_*`).

use crate::error::{AppError, Result};
use crate::infrastructure::llm_params;
use crate::infrastructure::ollama_client::OllamaClient;
use crate::infrastructure::remote_fallback_policy::remote_fallback_load;
use async_trait::async_trait;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub use crate::domain::ports::LlmClient;

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

/// Wraps `OllamaClient` as `Arc<dyn LlmClient>`.
#[must_use]
pub fn ollama_llm(client: OllamaClient) -> Arc<dyn LlmClient> {
    Arc::new(client)
}

/// Fixed reply for tests or offline scenarios; no network access.
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

/// Placeholder when `plugin_backends.llm = remote`: active when `OCLIVE_REMOTE_LLM_URL` is unset or sidecar client construction fails.
/// When graceful degradation is allowed, delegates to the builtin client and logs one warning; otherwise returns [`AppError::RemoteServiceUnavailable`].
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

    fn user_explicitly_chose_cloud_remote() -> bool {
        std::env::var("OCLIVE_LLM_BACKEND")
            .ok()
            .is_some_and(|v| v.trim().eq_ignore_ascii_case("remote"))
    }

    fn remote_url_configured() -> bool {
        std::env::var("OCLIVE_REMOTE_LLM_URL")
            .ok()
            .is_some_and(|s| !s.trim().is_empty())
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
        if Self::user_explicitly_chose_cloud_remote() || Self::remote_url_configured() {
            let msg = if Self::remote_url_configured() {
                "云端 LLM 未能连接（请重新在「模型管理」保存 URL/API Key，或检查网络授权）"
            } else {
                "云端 LLM 已启用但未配置 API 地址；请在「模型管理」保存 DeepSeek URL 与 API Key"
            };
            return Err(AppError::RemoteServiceUnavailable(msg.to_string()));
        }
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
        if Self::user_explicitly_chose_cloud_remote() {
            return Err(AppError::RemoteServiceUnavailable(
                "云端 LLM 未连接，无法执行标签任务".to_string(),
            ));
        }
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
