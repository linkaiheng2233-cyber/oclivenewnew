//! LLM generation port: orchestration and policy depend only on this trait; implementations are provided by the host `infrastructure`.

use async_trait::async_trait;
use oclive_kernel_types::Result;
use std::sync::Arc;

/// Incremental token callback for [`LlmClient::generate_stream`].
pub type LlmTokenSink = Arc<dyn Fn(&str) + Send + Sync>;

/// Optional knobs for Deep prefix-cache sessions (Ollama-only; ignored by default trait impls).
#[derive(Debug, Clone, Default)]
pub struct LlmGenerateOpts {
    pub keep_alive: Option<String>,
    pub want_metrics: bool,
}

impl LlmGenerateOpts {
    /// Interactive local session: keep the model resident and collect backend timings.
    #[must_use]
    pub fn interactive() -> Self {
        Self {
            keep_alive: std::env::var("OCLIVE_OLLAMA_KEEP_ALIVE")
                .ok()
                .filter(|s| !s.trim().is_empty())
                .or_else(|| Some("30m".to_string())),
            want_metrics: true,
        }
    }

    /// Deep prefix-cache session: keep model loaded and request Ollama bench metrics.
    #[must_use]
    pub fn deep_prefix_cache() -> Self {
        Self::interactive()
    }
}

/// Main LLM call result; `prompt_eval_ms` is set when the backend reports prompt eval timing.
#[derive(Debug, Clone)]
pub struct LlmGenerateOutcome {
    pub reply: String,
    pub prompt_eval_ms: Option<u64>,
}

/// Text generation port used by orchestration and policy (Ollama, remote, mock, etc.).
///
/// ## When to implement
///
/// - **Who**: LLM backend integrators (Ollama client, Remote HTTP LLM, test `MockLlmClient`).
/// - **When**: when orchestration needs to call a language model to generate a reply or `generate_tag` classification output.
///
/// ## When not to implement
///
/// - When only the host's builtin Ollama / a configured Remote is used and the inference path is unchanged, a custom implementation is **not** required.
/// - Toolchains that make no LLM calls (pure rule-based replies) can skip this.
///
/// # Examples
///
/// ```no_run
/// use oclive_kernel_contracts::LlmClient;
/// use std::sync::Arc;
///
/// async fn ask(llm: Arc<dyn LlmClient>) -> oclive_kernel_types::Result<()> {
///     let reply = llm.generate("qwen2.5:7b", "你好").await?;
///     assert!(!reply.is_empty());
///     Ok(())
/// }
/// ```
#[async_trait]
pub trait LlmClient: Send + Sync {
    /// Main dialogue generation (temperature defaulted by the implementation).
    ///
    /// # Errors
    ///
    /// Returns `Err` on network failure, upstream 4xx/5xx, timeout, or when the response body cannot be parsed.
    ///
    /// # Panics
    ///
    /// Does not panic.
    async fn generate(&self, model: &str, prompt: &str) -> Result<String>;

    /// Low-temperature short output (classification tasks such as portrait tags).
    ///
    /// # Errors
    ///
    /// Same as [`generate`](Self::generate); additional constraints (lower temperature / shorter output) are guaranteed by the implementation.
    ///
    /// # Panics
    ///
    /// Does not panic.
    async fn generate_tag(&self, model: &str, prompt: &str) -> Result<String>;

    /// Whether this backend supports Ollama-style prefix-cache opts ([`LlmGenerateOpts::deep_prefix_cache`]).
    fn supports_prefix_cache(&self) -> bool {
        false
    }

    /// Dialogue generation with optional backend-specific opts (default: [`generate`](Self::generate)).
    ///
    /// # Errors
    ///
    /// Same as [`generate`](Self::generate).
    async fn generate_with_opts(
        &self,
        model: &str,
        prompt: &str,
        opts: Option<&LlmGenerateOpts>,
    ) -> Result<LlmGenerateOutcome> {
        let _ = opts;
        let reply = self.generate(model, prompt).await?;
        Ok(LlmGenerateOutcome {
            reply,
            prompt_eval_ms: None,
        })
    }

    /// Optional streaming dialogue generation (default: [`generate`](Self::generate) then one callback).
    ///
    /// # Errors
    ///
    /// Same as [`generate`](Self::generate).
    async fn generate_stream(
        &self,
        model: &str,
        prompt: &str,
        on_token: LlmTokenSink,
    ) -> Result<String> {
        let full = self.generate(model, prompt).await?;
        on_token(full.as_str());
        Ok(full)
    }

    /// Streaming variant of [`generate_with_opts`](Self::generate_with_opts) (default: [`generate_stream`](Self::generate_stream)).
    ///
    /// # Errors
    ///
    /// Same as [`generate_stream`](Self::generate_stream).
    async fn generate_stream_with_opts(
        &self,
        model: &str,
        prompt: &str,
        on_token: LlmTokenSink,
        opts: Option<&LlmGenerateOpts>,
    ) -> Result<LlmGenerateOutcome> {
        let _ = opts;
        let reply = self.generate_stream(model, prompt, on_token).await?;
        Ok(LlmGenerateOutcome {
            reply,
            prompt_eval_ms: None,
        })
    }

    /// Optional startup probe (default succeeds; hosts may ping remote LLM).
    ///
    /// # Errors
    ///
    /// Returns `Err` when the probe request fails and the host is configured to require availability at startup; the default implementation always returns `Ok(())`.
    ///
    /// # Panics
    ///
    /// Does not panic.
    async fn startup_probe(&self) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::LlmGenerateOpts;

    #[test]
    fn interactive_and_deep_sessions_keep_the_local_model_resident() {
        let interactive = LlmGenerateOpts::interactive();
        let deep = LlmGenerateOpts::deep_prefix_cache();

        assert!(interactive.want_metrics);
        assert!(interactive
            .keep_alive
            .as_deref()
            .is_some_and(|v| !v.is_empty()));
        assert_eq!(deep.keep_alive, interactive.keep_alive);
        assert_eq!(deep.want_metrics, interactive.want_metrics);
    }
}
