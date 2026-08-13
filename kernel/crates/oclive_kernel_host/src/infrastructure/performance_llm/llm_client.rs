//! LlmClient trait implementation with Ollama fallback.

use super::PerformanceLlmClient;

use crate::domain::ports::LlmClient;
use crate::error::{AppError, Result};
use async_trait::async_trait;
use oclive_kernel_contracts::{LlmGenerateOpts, LlmGenerateOutcome, LlmTokenSink};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

impl PerformanceLlmClient {
    fn warn_fallback_once(&self, operation: &str, error: &AppError) {
        if self
            .fallback_warned
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
        {
            tracing::warn!(
                target: "oclive_llm",
                operation,
                error = %error,
                "performance LLM unavailable; falling back to Ollama"
            );
        }
    }

    fn record_fallback(&self, operation: &str, model: Option<&str>, primary_error: &AppError) {
        self.warn_fallback_once(operation, primary_error);
        if let Some(model) = model {
            self.record_fallback_model(model);
        }
    }

    fn fallback_error_with_primary(primary_error: &AppError, fallback_error: AppError) -> AppError {
        let fallback_detail = match fallback_error {
            AppError::OllamaError(detail) => detail,
            other => other.to_string(),
        };
        AppError::OllamaError(format!(
            "Performance LLM primary unavailable: {primary_error}; Ollama fallback unavailable: {fallback_detail}"
        ))
    }

    async fn primary_or_fallback_generate(&self, model: &str, prompt: &str) -> Result<String> {
        let _request_guard = self.request_gate.try_enter()?;
        if let Err(error) = self.ensure_primary_ready().await {
            self.record_fallback("generate", Some(model), &error);
            return self
                .fallback
                .generate(model, prompt)
                .await
                .map_err(|fallback_error| {
                    Self::fallback_error_with_primary(&error, fallback_error)
                });
        }
        match self
            .primary
            .generate(&self.profile.model_alias, prompt)
            .await
        {
            Ok(reply) => {
                self.set_status(true, "performance", "llama-server served the last request");
                Ok(reply)
            }
            Err(error) => {
                self.record_fallback("generate", Some(model), &error);
                self.degrade_to_ollama("llama-server request failed");
                self.fallback
                    .generate(model, prompt)
                    .await
                    .map_err(|fallback_error| {
                        Self::fallback_error_with_primary(&error, fallback_error)
                    })
            }
        }
    }
}

#[async_trait]
impl LlmClient for PerformanceLlmClient {
    fn supports_prefix_cache(&self) -> bool {
        true
    }

    async fn generate(&self, model: &str, prompt: &str) -> Result<String> {
        self.primary_or_fallback_generate(model, prompt).await
    }

    async fn generate_tag(&self, model: &str, prompt: &str) -> Result<String> {
        let _request_guard = self.request_gate.try_enter()?;
        if let Err(error) = self.ensure_primary_ready().await {
            self.record_fallback("generate_tag", Some(model), &error);
            return self
                .fallback
                .generate_tag(model, prompt)
                .await
                .map_err(|fallback_error| {
                    Self::fallback_error_with_primary(&error, fallback_error)
                });
        }
        match self
            .primary
            .generate_tag(&self.profile.model_alias, prompt)
            .await
        {
            Ok(reply) => Ok(reply),
            Err(error) => {
                self.record_fallback("generate_tag", Some(model), &error);
                self.degrade_to_ollama("llama-server tag request failed");
                self.fallback
                    .generate_tag(model, prompt)
                    .await
                    .map_err(|fallback_error| {
                        Self::fallback_error_with_primary(&error, fallback_error)
                    })
            }
        }
    }

    async fn generate_with_opts(
        &self,
        model: &str,
        prompt: &str,
        opts: Option<&LlmGenerateOpts>,
    ) -> Result<LlmGenerateOutcome> {
        let _request_guard = self.request_gate.try_enter()?;
        if let Err(error) = self.ensure_primary_ready().await {
            self.record_fallback("generate_with_opts", Some(model), &error);
            return self
                .fallback
                .generate_with_opts(model, prompt, opts)
                .await
                .map_err(|fallback_error| {
                    Self::fallback_error_with_primary(&error, fallback_error)
                });
        }
        match self
            .primary
            .generate_with_opts(&self.profile.model_alias, prompt, opts)
            .await
        {
            Ok(outcome) => Ok(outcome),
            Err(error) => {
                self.record_fallback("generate_with_opts", Some(model), &error);
                self.degrade_to_ollama("llama-server request failed");
                self.fallback
                    .generate_with_opts(model, prompt, opts)
                    .await
                    .map_err(|fallback_error| {
                        Self::fallback_error_with_primary(&error, fallback_error)
                    })
            }
        }
    }

    async fn generate_stream(
        &self,
        model: &str,
        prompt: &str,
        on_token: LlmTokenSink,
    ) -> Result<String> {
        self.generate_stream_with_opts(model, prompt, on_token, None)
            .await
            .map(|outcome| outcome.reply)
    }

    async fn generate_stream_with_opts(
        &self,
        model: &str,
        prompt: &str,
        on_token: LlmTokenSink,
        opts: Option<&LlmGenerateOpts>,
    ) -> Result<LlmGenerateOutcome> {
        let _request_guard = self.request_gate.try_enter()?;
        if let Err(error) = self.ensure_primary_ready().await {
            self.record_fallback("generate_stream", Some(model), &error);
            return self
                .fallback
                .generate_stream_with_opts(model, prompt, on_token, opts)
                .await
                .map_err(|fallback_error| {
                    Self::fallback_error_with_primary(&error, fallback_error)
                });
        }
        let emitted = Arc::new(AtomicBool::new(false));
        let emitted_for_sink = Arc::clone(&emitted);
        let downstream = Arc::clone(&on_token);
        let guarded_sink: LlmTokenSink = Arc::new(move |token| {
            if !token.is_empty() {
                emitted_for_sink.store(true, Ordering::Release);
            }
            downstream(token);
        });
        match self
            .primary
            .generate_stream_with_opts(&self.profile.model_alias, prompt, guarded_sink, opts)
            .await
        {
            Ok(outcome) => {
                self.set_status(true, "performance", "llama-server served the last stream");
                Ok(outcome)
            }
            Err(error) if !emitted.load(Ordering::Acquire) => {
                self.record_fallback("generate_stream", Some(model), &error);
                self.degrade_to_ollama("llama-server stream failed before first token");
                self.fallback
                    .generate_stream_with_opts(model, prompt, on_token, opts)
                    .await
                    .map_err(|fallback_error| {
                        Self::fallback_error_with_primary(&error, fallback_error)
                    })
            }
            Err(error) => {
                self.degrade_to_ollama("llama-server stream failed after emitting content");
                Err(error)
            }
        }
    }

    async fn startup_probe(&self) -> Result<()> {
        let _request_guard = self.request_gate.try_enter()?;
        match self.ensure_primary_ready().await {
            Ok(()) => Ok(()),
            Err(error) => {
                self.record_fallback("startup_probe", None, &error);
                self.fallback
                    .startup_probe()
                    .await
                    .map_err(|fallback_error| {
                        Self::fallback_error_with_primary(&error, fallback_error)
                    })
            }
        }
    }
}
