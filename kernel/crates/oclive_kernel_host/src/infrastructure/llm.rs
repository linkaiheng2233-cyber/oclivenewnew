//! LLM invocation abstraction for testing and swappable implementations.
//!
//! Main chat and tag-task temperature / top_p: see [`super::llm_params`] (env vars `OCLIVE_LLM_*`).

use crate::error::{AppError, Result};
use crate::infrastructure::llm_params;
use crate::infrastructure::ollama_client::{
    OllamaCallFailure, OllamaClient, OllamaGenerateOpts, OllamaGenerateSettings,
};
use crate::infrastructure::remote_fallback_policy::remote_fallback_load;
use async_trait::async_trait;
use oclive_kernel_contracts::{LlmGenerateOpts, LlmGenerateOutcome, LlmTokenSink};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub use crate::domain::ports::LlmClient;

/// One non-streaming text-generation request's resolved parameters, independent of any caller-side
/// borrow.
///
/// This is what the legacy entry and the B2-C1 Base adapter both hand to the shared request path,
/// so the merge exists once and owns everything it returns. `temperature` and `top_p` are the
/// resolved sampling values; the keep-alive, the token / context caps and the metrics request stay
/// inside `ollama_opts`, so no field is stored twice.
#[derive(Debug, Clone, Default)]
pub(crate) struct B2C1OllamaSettings {
    pub(crate) temperature: Option<f32>,
    pub(crate) top_p: Option<f32>,
    /// Present only when the caller supplied options; owns the forwardable Ollama knobs.
    pub(crate) ollama_opts: Option<OllamaGenerateOpts>,
}

impl B2C1OllamaSettings {
    /// Borrows these settings as the request-path parameter bundle.
    ///
    /// `opts` is `Some` exactly when the caller supplied options. That distinction is about the
    /// binding, not about the wire: whether an `options` object appears is decided by
    /// [`OllamaRequestOptions::from_generate`](super::ollama_client::OllamaRequestOptions), which
    /// looks at the individual fields below.
    pub(crate) fn request_settings(&self) -> OllamaGenerateSettings<'_> {
        OllamaGenerateSettings {
            temperature: self.temperature,
            top_p: self.top_p,
            opts: self.ollama_opts.as_ref(),
        }
    }
}

/// The one place where a caller's [`LlmGenerateOpts`] becomes Ollama request settings.
///
/// Both non-streaming text-generation entries share it: the legacy [`LlmClient`] implementation
/// below and the B2-C1 Base adapter in [`super::base_llm`].
///
/// What the merge does, field by field:
///
/// - **temperature / top_p** are the only fields that fall back to the sampling pair the caller
///   passes in, and each falls back independently: the caller's value wins when it sets one, and an
///   unset one uses that pair.
/// - **`None` vs `Some`** here preserves whether the caller supplied options at all. It does not by
///   itself decide whether the request ends up carrying an `options` object: the request is built
///   from the merged fields, so a caller-supplied value whose sampling fields are unset still
///   reaches the wire with the pair above filled in.
/// - **the other options** — keep-alive, output-token and context-token caps, and the metrics
///   request — keep the existing per-field conversion and serialization rules. They are not
///   "defaulted to the sampling pair"; each is simply carried when set and omitted when unset.
/// - **`want_metrics` is not a wire field.** It only selects whether the decoded body's timings are
///   turned into this call's local metrics; the wire body is unaffected by it.
///
/// Nothing here invents defaults, and callers read the sampling pair per call (the legacy entry
/// calls [`llm_params::main_chat_options`] on every invocation), so it is not fixed at construction
/// time.
pub(crate) fn b2_c1_merge_options(
    opts: Option<&LlmGenerateOpts>,
    sampling_defaults: (Option<f32>, Option<f32>),
) -> B2C1OllamaSettings {
    let (default_temperature, default_top_p) = sampling_defaults;
    let ollama_opts = opts.map(|opts| OllamaGenerateOpts {
        keep_alive: opts.keep_alive.clone(),
        want_metrics: opts.want_metrics,
        max_output_tokens: opts.max_output_tokens,
        preferred_context_tokens: opts.preferred_context_tokens,
    });
    B2C1OllamaSettings {
        temperature: opts
            .and_then(|opts| opts.temperature)
            .or(default_temperature),
        top_p: opts.and_then(|opts| opts.top_p).or(default_top_p),
        ollama_opts,
    }
}

fn log_ollama_metrics(
    model: &str,
    metrics: &crate::infrastructure::ollama_client::OllamaGenerateMetrics,
) {
    tracing::debug!(
        target: "oclive_llm",
        model,
        load_ms = ?metrics.load_ms(),
        prompt_eval_ms = ?metrics.prompt_eval_ms(),
        prompt_tokens = ?metrics.prompt_eval_count,
        eval_tokens = ?metrics.eval_count,
        "ollama generation metrics"
    );
}

#[async_trait]
impl LlmClient for OllamaClient {
    fn supports_prefix_cache(&self) -> bool {
        true
    }

    async fn generate(&self, model: &str, prompt: &str) -> Result<String> {
        let (t, p) = llm_params::main_chat_options();
        OllamaClient::generate(self, model, prompt, t, p).await
    }

    async fn generate_tag(&self, model: &str, prompt: &str) -> Result<String> {
        let (t, p) = llm_params::tag_task_options();
        OllamaClient::generate(self, model, prompt, t, p).await
    }

    async fn generate_with_opts(
        &self,
        model: &str,
        prompt: &str,
        opts: Option<&LlmGenerateOpts>,
    ) -> Result<LlmGenerateOutcome> {
        let (t, p) = llm_params::main_chat_options();
        let settings = b2_c1_merge_options(opts, (t, p));
        // The shared path reports a failure together with whether it knew the call timed out; the
        // legacy entry projects that straight back to its original AppError.
        let out =
            OllamaClient::generate_with_settings(self, model, prompt, settings.request_settings())
                .await
                .map_err(OllamaCallFailure::into_legacy_error)?;
        log_ollama_metrics(model, &out.metrics);
        Ok(LlmGenerateOutcome {
            reply: out.body.response,
            prompt_eval_ms: out.metrics.prompt_eval_ms(),
        })
    }

    async fn generate_stream(
        &self,
        model: &str,
        prompt: &str,
        on_token: LlmTokenSink,
    ) -> Result<String> {
        let (t, p) = llm_params::main_chat_options();
        OllamaClient::generate_stream_with_callback(self, model, prompt, t, p, on_token).await
    }

    async fn generate_stream_with_opts(
        &self,
        model: &str,
        prompt: &str,
        on_token: LlmTokenSink,
        opts: Option<&LlmGenerateOpts>,
    ) -> Result<LlmGenerateOutcome> {
        let (t, p) = llm_params::main_chat_options();
        let ollama_opts = opts.map(
            |o| crate::infrastructure::ollama_client::OllamaGenerateOpts {
                keep_alive: o.keep_alive.clone(),
                want_metrics: o.want_metrics,
                max_output_tokens: o.max_output_tokens,
                preferred_context_tokens: o.preferred_context_tokens,
            },
        );
        let temperature = opts.and_then(|opts| opts.temperature).or(t);
        let top_p = opts.and_then(|opts| opts.top_p).or(p);
        let out = OllamaClient::generate_stream_with_callback_and_opts(
            self,
            model,
            prompt,
            temperature,
            top_p,
            on_token,
            ollama_opts.as_ref(),
        )
        .await?;
        log_ollama_metrics(model, &out.metrics);
        Ok(LlmGenerateOutcome {
            reply: out.response,
            prompt_eval_ms: out.metrics.prompt_eval_ms(),
        })
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

/// Share one [`OllamaClient`] as [`LlmClient`] (production `AppState` wiring).
pub struct SharedOllamaClient(pub Arc<OllamaClient>);

#[async_trait]
impl LlmClient for SharedOllamaClient {
    fn supports_prefix_cache(&self) -> bool {
        true
    }

    async fn generate(&self, model: &str, prompt: &str) -> Result<String> {
        let (t, p) = llm_params::main_chat_options();
        OllamaClient::generate(self.0.as_ref(), model, prompt, t, p).await
    }

    async fn generate_tag(&self, model: &str, prompt: &str) -> Result<String> {
        let (t, p) = llm_params::tag_task_options();
        OllamaClient::generate(self.0.as_ref(), model, prompt, t, p).await
    }

    async fn generate_with_opts(
        &self,
        model: &str,
        prompt: &str,
        opts: Option<&LlmGenerateOpts>,
    ) -> Result<LlmGenerateOutcome> {
        LlmClient::generate_with_opts(self.0.as_ref(), model, prompt, opts).await
    }

    async fn generate_stream(
        &self,
        model: &str,
        prompt: &str,
        on_token: LlmTokenSink,
    ) -> Result<String> {
        let (t, p) = llm_params::main_chat_options();
        OllamaClient::generate_stream_with_callback(self.0.as_ref(), model, prompt, t, p, on_token)
            .await
    }

    async fn generate_stream_with_opts(
        &self,
        model: &str,
        prompt: &str,
        on_token: LlmTokenSink,
        opts: Option<&LlmGenerateOpts>,
    ) -> Result<LlmGenerateOutcome> {
        LlmClient::generate_stream_with_opts(self.0.as_ref(), model, prompt, on_token, opts).await
    }

    async fn startup_probe(&self) -> Result<()> {
        LlmClient::startup_probe(self.0.as_ref()).await
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
        Ok(self.reply.clone())
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

    fn ollama_backend_active() -> bool {
        std::env::var("OCLIVE_LLM_BACKEND")
            .ok()
            .is_none_or(|v| !v.trim().eq_ignore_ascii_case("remote"))
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
        if Self::user_explicitly_chose_cloud_remote()
            || (Self::remote_url_configured() && !Self::ollama_backend_active())
        {
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

#[cfg(test)]
mod tests {
    use super::super::ollama_client::{non_streaming_request, OllamaGenerateOpts};
    use super::b2_c1_merge_options;
    use oclive_kernel_contracts::LlmGenerateOpts;

    const DEFAULT_SAMPLING: (Option<f32>, Option<f32>) = (Some(0.8), Some(0.9));

    fn full_options() -> LlmGenerateOpts {
        LlmGenerateOpts {
            keep_alive: Some("30m".to_string()),
            want_metrics: true,
            temperature: Some(0.5),
            top_p: Some(0.7),
            max_output_tokens: Some(512),
            preferred_context_tokens: Some(8192),
        }
    }

    /// Serializes the request the shared production path would send for these settings, by calling
    /// the production factory with the same arguments that path uses. Nothing here re-implements
    /// the wire shape.
    fn serialized(
        model: &str,
        prompt: &str,
        settings: &super::B2C1OllamaSettings,
    ) -> serde_json::Value {
        let path_settings = settings.request_settings();
        let request = non_streaming_request(
            model,
            prompt,
            path_settings.temperature,
            path_settings.top_p,
            path_settings.opts,
        );
        serde_json::to_value(&request).expect("request must serialize")
    }

    #[test]
    fn b2_c1_merge_options_keeps_none_options_absent() {
        let settings = b2_c1_merge_options(None, DEFAULT_SAMPLING);
        assert_eq!(settings.temperature, Some(0.8));
        assert_eq!(settings.top_p, Some(0.9));
        assert!(
            settings.ollama_opts.is_none(),
            "a caller that passed no options has no forwardable Ollama knobs"
        );

        let body = serialized("m", "p", &settings);
        assert_eq!(body["model"], "m");
        assert_eq!(body["prompt"], "p");
        assert_eq!(body["stream"], false);
        // The sampling pair is always supplied by the caller, so the options block carries it; what
        // a `None` caller must *not* get is a keep_alive or a token/context cap of our invention.
        assert_eq!(body["options"]["temperature"], 0.8_f32 as f64);
        assert_eq!(body["options"]["top_p"], 0.9_f32 as f64);
        assert!(body["options"].get("num_predict").is_none());
        assert!(body["options"].get("num_ctx").is_none());
        assert!(body.get("keep_alive").is_none());
    }

    #[test]
    fn b2_c1_merge_options_distinguishes_none_from_some_default() {
        let none = b2_c1_merge_options(None, DEFAULT_SAMPLING);
        let default_opts = LlmGenerateOpts::default();
        let some_default = b2_c1_merge_options(Some(&default_opts), DEFAULT_SAMPLING);

        // Same sampling outcome — nothing in the caller's value overrides the defaults, so the two
        // produce the same request here ...
        assert_eq!(some_default.temperature, none.temperature);
        assert_eq!(some_default.top_p, none.top_p);
        let none_body = serialized("m", "p", &none);
        let some_body = serialized("m", "p", &some_default);
        assert_eq!(none_body, some_body, "unset fields keep the same defaults");
        assert_eq!(some_body["options"]["temperature"], 0.8_f32 as f64);
        assert!(some_body.get("keep_alive").is_none());

        // ... while the two bound values stay distinct: only `Some` carries forwardable knobs, so a
        // caller that sets only `want_metrics` still gets the old metrics behaviour.
        assert!(none.ollama_opts.is_none());
        assert!(some_default.ollama_opts.is_some());
        let metrics_opts = LlmGenerateOpts {
            want_metrics: true,
            ..LlmGenerateOpts::default()
        };
        let with_metrics = b2_c1_merge_options(Some(&metrics_opts), DEFAULT_SAMPLING);
        assert!(with_metrics
            .request_settings()
            .opts
            .is_some_and(|o| o.want_metrics));
        assert_eq!(serialized("m", "p", &none), none_body);
    }

    #[test]
    fn b2_c1_merge_options_keeps_caller_fields_verbatim() {
        let opts = full_options();
        let settings = b2_c1_merge_options(Some(&opts), DEFAULT_SAMPLING);
        assert_eq!(settings.temperature, Some(0.5));
        assert_eq!(settings.top_p, Some(0.7));
        let forwarded = settings.ollama_opts.as_ref().expect("Some forwards knobs");
        assert_eq!(forwarded.keep_alive.as_deref(), Some("30m"));
        assert!(forwarded.want_metrics);
        assert_eq!(forwarded.max_output_tokens, Some(512));
        assert_eq!(forwarded.preferred_context_tokens, Some(8192));

        let body = serialized("bound-model", "exact\nprompt text", &settings);
        assert_eq!(body["model"], "bound-model");
        assert_eq!(body["prompt"], "exact\nprompt text");
        assert_eq!(body["options"]["temperature"], 0.5_f32 as f64);
        assert_eq!(body["options"]["top_p"], 0.7_f32 as f64);
        assert_eq!(body["options"]["num_predict"], 512);
        assert_eq!(body["options"]["num_ctx"], 8192);
        assert_eq!(body["keep_alive"], "30m");
    }

    #[test]
    fn b2_c1_merge_options_prefers_the_single_explicit_override() {
        let opts = LlmGenerateOpts {
            temperature: Some(0.25),
            ..LlmGenerateOpts::default()
        };
        let settings = b2_c1_merge_options(Some(&opts), DEFAULT_SAMPLING);
        assert_eq!(settings.temperature, Some(0.25));
        assert_eq!(settings.top_p, Some(0.9), "unset field keeps the default");

        let body = serialized("m", "p", &settings);
        assert_eq!(body["options"]["temperature"], 0.25_f32 as f64);
        assert_eq!(body["options"]["top_p"], 0.9_f32 as f64);
        assert!(body["options"].get("num_predict").is_none());
        assert!(body["options"].get("num_ctx").is_none());
    }

    #[test]
    fn b2_c1_merge_options_uses_the_sampling_defaults_it_is_given() {
        // The pair is an input, so the merge never hard-codes 0.8 / 0.9 and never reads the
        // environment itself: callers keep reading `main_chat_options()` per call.
        let settings = b2_c1_merge_options(None, (Some(0.31), Some(0.42)));
        assert_eq!(settings.temperature, Some(0.31));
        assert_eq!(settings.top_p, Some(0.42));
    }

    #[test]
    fn b2_c1_request_settings_round_trip_into_the_factory_arguments() {
        let opts = full_options();
        let settings = b2_c1_merge_options(Some(&opts), DEFAULT_SAMPLING);
        let borrowed = settings.request_settings();
        assert_eq!(borrowed.temperature, Some(0.5));
        assert_eq!(borrowed.top_p, Some(0.7));
        let forwarded: &OllamaGenerateOpts = borrowed.opts.expect("Some(opts) forwards options");
        assert_eq!(forwarded.keep_alive.as_deref(), Some("30m"));
        assert!(forwarded.want_metrics);
        assert_eq!(forwarded.max_output_tokens, Some(512));
        assert_eq!(forwarded.preferred_context_tokens, Some(8192));
    }
}
