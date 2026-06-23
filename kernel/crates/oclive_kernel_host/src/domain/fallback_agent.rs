//! Agent backend wrapper: primary remote/directory with builtin fallback.

use crate::error::{AppError, Result};
use async_trait::async_trait;
use oclive_kernel_contracts::AgentProvider;
use oclive_kernel_types::{AgentInput, AgentOutput};
use std::sync::Arc;

/// Wraps a primary agent backend; on failure (except grant denial) delegates to builtin.
pub struct FallbackAgentProvider {
    primary: Arc<dyn AgentProvider>,
    fallback: Arc<dyn AgentProvider>,
    primary_label: &'static str,
}

impl FallbackAgentProvider {
    #[must_use]
    pub fn new(
        primary: Arc<dyn AgentProvider>,
        fallback: Arc<dyn AgentProvider>,
        primary_label: &'static str,
    ) -> Arc<Self> {
        Arc::new(Self {
            primary,
            fallback,
            primary_label,
        })
    }
}

#[async_trait]
impl AgentProvider for FallbackAgentProvider {
    async fn process(&self, input: AgentInput) -> Result<AgentOutput> {
        match self.primary.process(input.clone()).await {
            Ok(out) => Ok(out),
            Err(e) => {
                if matches!(e, AppError::HighRiskCapabilityNotGranted { .. }) {
                    return Err(e);
                }
                tracing::warn!(
                    target: "oclive_agent",
                    backend = self.primary_label,
                    error = %e,
                    "Agent remote/directory unavailable; fallback=builtin"
                );
                self.fallback.process(input).await
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;

    struct OkPrimary {
        reply: String,
    }

    #[async_trait]
    impl AgentProvider for OkPrimary {
        async fn process(&self, _input: AgentInput) -> Result<AgentOutput> {
            Ok(AgentOutput {
                handled: true,
                reply: self.reply.clone(),
            })
        }
    }

    struct ErrPrimary;

    #[async_trait]
    impl AgentProvider for ErrPrimary {
        async fn process(&self, _input: AgentInput) -> Result<AgentOutput> {
            Err(AppError::RemoteServiceUnavailable("down".into()))
        }
    }

    struct OkFallback;

    #[async_trait]
    impl AgentProvider for OkFallback {
        async fn process(&self, _input: AgentInput) -> Result<AgentOutput> {
            Ok(AgentOutput {
                handled: true,
                reply: "fallback".into(),
            })
        }
    }

    #[tokio::test]
    async fn primary_success_skips_fallback() {
        let fb = FallbackAgentProvider::new(
            Arc::new(OkPrimary {
                reply: "primary".into(),
            }),
            Arc::new(OkFallback),
            "remote",
        );
        let out = fb.process(AgentInput::default()).await.expect("ok");
        assert_eq!(out.reply, "primary");
    }

    #[tokio::test]
    async fn primary_failure_uses_fallback() {
        let fb = FallbackAgentProvider::new(Arc::new(ErrPrimary), Arc::new(OkFallback), "remote");
        let out = fb.process(AgentInput::default()).await.expect("ok");
        assert_eq!(out.reply, "fallback");
    }
}
