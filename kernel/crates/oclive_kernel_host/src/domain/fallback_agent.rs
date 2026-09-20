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

    /// A primary that refuses with the typed authorisation error.
    struct RefusingPrimary;

    #[async_trait]
    impl AgentProvider for RefusingPrimary {
        async fn process(&self, _input: AgentInput) -> Result<AgentOutput> {
            Err(AppError::HighRiskCapabilityNotGranted {
                capability: "process:spawn".into(),
                id: "mcp-server-1".into(),
            })
        }
    }

    /// A fallback that records every call, so "the fallback was not used" is counted rather than
    /// assumed from the returned error alone.
    #[derive(Default)]
    struct CountingFallback {
        calls: std::sync::atomic::AtomicUsize,
    }

    #[async_trait]
    impl AgentProvider for CountingFallback {
        async fn process(&self, _input: AgentInput) -> Result<AgentOutput> {
            self.calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
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

    /// CP-B3-ALL unit A (tests only): a typed authorisation refusal is re-raised instead of being
    /// delegated to the builtin fallback, so the grant decision is not bypassed — and the typed
    /// refusal reaches the caller unchanged.
    #[tokio::test]
    async fn cp_b3_all_agent_authorization_denial_does_not_fall_back() {
        let fallback = Arc::new(CountingFallback::default());
        let fb = FallbackAgentProvider::new(
            Arc::new(RefusingPrimary),
            Arc::clone(&fallback) as Arc<dyn AgentProvider>,
            "remote",
        );

        let error = fb
            .process(AgentInput::default())
            .await
            .expect_err("a refusal is not a fallback trigger");
        match error {
            AppError::HighRiskCapabilityNotGranted { capability, id } => {
                assert_eq!(capability, "process:spawn");
                assert_eq!(id, "mcp-server-1");
            }
            other => panic!("the typed refusal must be re-raised unchanged: {other:?}"),
        }
        assert_eq!(
            fallback.calls.load(std::sync::atomic::Ordering::SeqCst),
            0,
            "the builtin fallback must not be called for a refusal"
        );
    }
}
