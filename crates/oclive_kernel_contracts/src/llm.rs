//! LLM generation port: orchestration and policy depend only on this trait; implementations are provided by the host `infrastructure`.

use async_trait::async_trait;
use oclive_kernel_types::Result;

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
