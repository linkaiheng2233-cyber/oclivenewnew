//! 远程 LLM：OpenAI 兼容云端（env / UI）→ JSON-RPC 侧车 → 占位回退。

use crate::error::Result;
use crate::infrastructure::cloud_llm::{
    resolve_cloud_llm_config, CloudLlmRuntime, OpenAiCompatLlmClient,
};
use crate::infrastructure::llm::{LlmClient, RemoteLlmPlaceholder};
use crate::infrastructure::remote_plugin::config::RemotePluginHttpConfig;
use crate::infrastructure::remote_plugin::llm_http::RemoteLlmHttp;
use async_trait::async_trait;
use std::sync::Arc;

pub struct LlmRemoteStack {
    cloud_runtime: Arc<CloudLlmRuntime>,
    sidecar: Option<Arc<RemoteLlmHttp>>,
    fallback: Arc<RemoteLlmPlaceholder>,
}

impl LlmRemoteStack {
    pub fn new(cloud_runtime: Arc<CloudLlmRuntime>, default_llm: Arc<dyn LlmClient>) -> Self {
        let sidecar = RemotePluginHttpConfig::from_env_llm().map(|cfg| {
            if resolve_cloud_llm_config(cloud_runtime.as_ref()).is_none() {
                log::info!(
                    target: "oclive_plugin",
                    "remote LLM HTTP active -> {}",
                    cfg.endpoint
                );
            }
            Arc::new(RemoteLlmHttp::new(cfg))
        });
        if let Some(cfg) = resolve_cloud_llm_config(cloud_runtime.as_ref()) {
            log::info!(
                target: "oclive_plugin",
                "cloud LLM HTTP active (env or app settings) -> {}",
                cfg.base_url
            );
        } else if sidecar.is_none() {
            log::debug!(
                target: "oclive_plugin",
                "remote LLM stack: no cloud config and no OCLIVE_REMOTE_LLM_URL; placeholder until remote is configured"
            );
        }
        let fallback = Arc::new(RemoteLlmPlaceholder::new(default_llm));
        Self {
            cloud_runtime,
            sidecar,
            fallback,
        }
    }
}

#[async_trait]
impl LlmClient for LlmRemoteStack {
    async fn generate(&self, model: &str, prompt: &str) -> Result<String> {
        if let Some(cfg) = resolve_cloud_llm_config(self.cloud_runtime.as_ref()) {
            let c = OpenAiCompatLlmClient::new(cfg);
            return c.generate(model, prompt).await;
        }
        if let Some(ref http) = self.sidecar {
            return http.generate(model, prompt).await;
        }
        self.fallback.generate(model, prompt).await
    }

    async fn generate_tag(&self, model: &str, prompt: &str) -> Result<String> {
        if let Some(cfg) = resolve_cloud_llm_config(self.cloud_runtime.as_ref()) {
            let c = OpenAiCompatLlmClient::new(cfg);
            return c.generate_tag(model, prompt).await;
        }
        if let Some(ref http) = self.sidecar {
            return http.generate_tag(model, prompt).await;
        }
        self.fallback.generate_tag(model, prompt).await
    }
}
