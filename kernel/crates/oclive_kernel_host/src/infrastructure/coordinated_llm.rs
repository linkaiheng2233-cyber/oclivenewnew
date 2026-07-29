//! Resource-control wrapper for observe-only external LLM runtimes.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use async_trait::async_trait;
use oclive_kernel_contracts::{LlmClient, LlmGenerateOpts, LlmGenerateOutcome, LlmTokenSink};
use oclive_kernel_types::{ResourcePriority, Result};

use crate::domain::resource_coordinator::ResourceCoordinator;

pub struct CoordinatedExternalLlm {
    inner: Arc<dyn LlmClient>,
    coordinator: Arc<ResourceCoordinator>,
    adapter_id: String,
    next_request_id: AtomicU64,
}

struct ObservedActivityGuard {
    coordinator: Arc<ResourceCoordinator>,
    lease_id: String,
}

impl Drop for ObservedActivityGuard {
    fn drop(&mut self) {
        self.coordinator.release(&self.lease_id);
    }
}

impl CoordinatedExternalLlm {
    #[must_use]
    pub fn new(
        inner: Arc<dyn LlmClient>,
        coordinator: Arc<ResourceCoordinator>,
        adapter_id: impl Into<String>,
    ) -> Self {
        Self {
            inner,
            coordinator,
            adapter_id: adapter_id.into(),
            next_request_id: AtomicU64::new(1),
        }
    }

    fn begin_request(&self, model: &str, operation: &str) -> ObservedActivityGuard {
        let sequence = self.next_request_id.fetch_add(1, Ordering::Relaxed);
        let workload_id = format!("{operation}:{model}:{sequence}");
        let lease_id = self.coordinator.begin_observed_activity(
            self.adapter_id.clone(),
            workload_id,
            ResourcePriority::ForegroundInteractive,
        );
        ObservedActivityGuard {
            coordinator: Arc::clone(&self.coordinator),
            lease_id,
        }
    }
}

#[async_trait]
impl LlmClient for CoordinatedExternalLlm {
    fn supports_prefix_cache(&self) -> bool {
        self.inner.supports_prefix_cache()
    }

    async fn generate(&self, model: &str, prompt: &str) -> Result<String> {
        let _activity = self.begin_request(model, "generate");
        self.inner.generate(model, prompt).await
    }

    async fn generate_tag(&self, model: &str, prompt: &str) -> Result<String> {
        let _activity = self.begin_request(model, "generate_tag");
        self.inner.generate_tag(model, prompt).await
    }

    async fn generate_with_opts(
        &self,
        model: &str,
        prompt: &str,
        opts: Option<&LlmGenerateOpts>,
    ) -> Result<LlmGenerateOutcome> {
        let _activity = self.begin_request(model, "generate_with_opts");
        self.inner.generate_with_opts(model, prompt, opts).await
    }

    async fn generate_stream(
        &self,
        model: &str,
        prompt: &str,
        on_token: LlmTokenSink,
    ) -> Result<String> {
        let _activity = self.begin_request(model, "generate_stream");
        self.inner.generate_stream(model, prompt, on_token).await
    }

    async fn generate_stream_with_opts(
        &self,
        model: &str,
        prompt: &str,
        on_token: LlmTokenSink,
        opts: Option<&LlmGenerateOpts>,
    ) -> Result<LlmGenerateOutcome> {
        let _activity = self.begin_request(model, "generate_stream_with_opts");
        self.inner
            .generate_stream_with_opts(model, prompt, on_token, opts)
            .await
    }

    async fn startup_probe(&self) -> Result<()> {
        self.inner.startup_probe().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::MockLlmClient;
    use async_trait::async_trait;
    use oclive_kernel_contracts::ResourceSnapshotSource;
    use oclive_kernel_types::{ResourceCoordinatorPolicy, ResourceSnapshot};

    struct UnexpectedSnapshot;

    #[async_trait]
    impl ResourceSnapshotSource for UnexpectedSnapshot {
        async fn snapshot(&self) -> ResourceSnapshot {
            panic!("foreground observe-only requests must not launch a device probe")
        }
    }

    #[tokio::test]
    async fn observe_only_llm_is_visible_during_call_and_released_afterward() {
        let coordinator = Arc::new(ResourceCoordinator::new(
            ResourceCoordinatorPolicy::default(),
            Arc::new(UnexpectedSnapshot),
        ));
        let client = CoordinatedExternalLlm::new(
            Arc::new(MockLlmClient { reply: "ok".into() }),
            coordinator.clone(),
            "builtin.llm.ollama",
        );
        assert_eq!(client.generate("qwen", "hello").await.unwrap(), "ok");
        assert!(coordinator.diagnostics_snapshot().leases.is_empty());
    }
}
