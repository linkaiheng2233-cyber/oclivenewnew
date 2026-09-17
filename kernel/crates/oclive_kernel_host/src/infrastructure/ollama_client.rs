use crate::error::{AppError, Result};
use crate::infrastructure::ollama_timeouts;
use reqwest::Client;
use std::sync::Arc;
use std::sync::LazyLock;

static OLLAMA_HTTP_CLIENT: LazyLock<Client> = LazyLock::new(|| {
    Client::builder()
        .pool_max_idle_per_host(4)
        .build()
        .expect("ollama reqwest client")
});
static OLLAMA_LOOPBACK_HTTP_CLIENT: LazyLock<Client> = LazyLock::new(|| {
    Client::builder()
        .no_proxy()
        .pool_max_idle_per_host(4)
        .build()
        .expect("ollama loopback reqwest client")
});
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Ollama request body.
#[derive(Debug, Serialize)]
pub struct OllamaRequest {
    pub model: String,
    pub prompt: String,
    pub stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<OllamaRequestOptions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keep_alive: Option<String>,
}

/// Ollama `/api/generate` runtime parameters belong under `options`.
#[derive(Debug, Clone, Default, Serialize)]
pub struct OllamaRequestOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_predict: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_ctx: Option<u32>,
}

impl OllamaRequestOptions {
    fn from_generate(
        temperature: Option<f32>,
        top_p: Option<f32>,
        opts: Option<&OllamaGenerateOpts>,
    ) -> Option<Self> {
        let value = Self {
            temperature,
            top_p,
            num_predict: opts.and_then(|opts| opts.max_output_tokens),
            num_ctx: opts.and_then(|opts| opts.preferred_context_tokens),
        };
        (value.temperature.is_some()
            || value.top_p.is_some()
            || value.num_predict.is_some()
            || value.num_ctx.is_some())
        .then_some(value)
    }
}

/// Optional knobs for Deep prefix-cache sessions (Ollama-only; ignored by `LlmClient` trait).
#[derive(Debug, Clone, Default)]
pub struct OllamaGenerateOpts {
    pub keep_alive: Option<String>,
    pub want_metrics: bool,
    pub max_output_tokens: Option<u32>,
    pub preferred_context_tokens: Option<u32>,
}

impl OllamaGenerateOpts {
    #[must_use]
    pub fn deep_prefix_cache() -> Self {
        Self {
            keep_alive: std::env::var("OCLIVE_OLLAMA_KEEP_ALIVE")
                .ok()
                .filter(|s| !s.trim().is_empty())
                .or_else(|| Some("30m".to_string())),
            want_metrics: true,
            ..Self::default()
        }
    }
}

/// Timing fields from Ollama `/api/generate` (nanoseconds).
#[derive(Debug, Clone, Default)]
pub struct OllamaGenerateMetrics {
    pub total_duration_ns: Option<u64>,
    pub load_duration_ns: Option<u64>,
    pub prompt_eval_duration_ns: Option<u64>,
    pub prompt_eval_count: Option<u64>,
    pub eval_duration_ns: Option<u64>,
    pub eval_count: Option<u64>,
}

impl OllamaGenerateMetrics {
    #[must_use]
    pub fn prompt_eval_ms(&self) -> Option<u64> {
        self.prompt_eval_duration_ns.map(|ns| ns / 1_000_000)
    }

    #[must_use]
    pub fn load_ms(&self) -> Option<u64> {
        self.load_duration_ns.map(|ns| ns / 1_000_000)
    }
}

#[derive(Debug)]
pub struct OllamaGenerateResult {
    pub response: String,
    pub metrics: OllamaGenerateMetrics,
}

/// Ollama response body.
#[derive(Debug, Deserialize)]
pub struct OllamaResponse {
    pub response: String,
    pub model: String,
    pub created_at: String,
    pub done: bool,
    #[serde(default)]
    pub total_duration: Option<u64>,
    #[serde(default)]
    pub load_duration: Option<u64>,
    #[serde(default)]
    pub prompt_eval_duration: Option<u64>,
    #[serde(default)]
    pub prompt_eval_count: Option<u64>,
    #[serde(default)]
    pub eval_duration: Option<u64>,
    #[serde(default)]
    pub eval_count: Option<u64>,
}

/// Per-call knobs for one non-streaming request, as accepted by
/// [`OllamaClient::generate_with_settings`].
///
/// This is the parameter bundle the legacy entry and the Base adapter share; it carries the
/// caller's sampling values and optional Deep-session opts without inventing defaults of its own.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct OllamaGenerateSettings<'a> {
    /// Caller-supplied temperature; `None` leaves the value out of the request `options`.
    pub(crate) temperature: Option<f32>,
    /// Caller-supplied `top_p`; `None` leaves the value out of the request `options`.
    pub(crate) top_p: Option<f32>,
    /// Optional Deep-session knobs (keep-alive, metrics, token/context caps).
    pub(crate) opts: Option<&'a OllamaGenerateOpts>,
}

/// Result of one non-streaming call on the shared path.
///
/// It keeps the decoded body — including [`OllamaResponse::done`], which
/// [`OllamaGenerateResult`] does not carry — next to the optional bench timings derived for this
/// call. It is internal plumbing between the shared path and its two projections; it is not part of
/// any public wire.
#[derive(Debug)]
pub(crate) struct OllamaCallResponse {
    pub(crate) body: OllamaResponse,
    pub(crate) metrics: OllamaGenerateMetrics,
}

impl OllamaCallResponse {
    /// Builds the per-call value from an already decoded body; bench timings are derived only when
    /// the call asked for metrics, exactly as the legacy entry did.
    pub(crate) fn from_response(body: OllamaResponse, want_metrics: bool) -> Self {
        let metrics = if want_metrics {
            OllamaGenerateMetrics {
                total_duration_ns: body.total_duration,
                load_duration_ns: body.load_duration,
                prompt_eval_duration_ns: body.prompt_eval_duration,
                prompt_eval_count: body.prompt_eval_count,
                eval_duration_ns: body.eval_duration,
                eval_count: body.eval_count,
            }
        } else {
            OllamaGenerateMetrics::default()
        };
        Self { body, metrics }
    }

    /// The legacy projection: exactly what [`OllamaGenerateResult`] carries, and nothing else.
    ///
    /// It deliberately does **not** look at [`OllamaResponse::done`]: the legacy entry never did,
    /// and this repair does not change legacy behaviour. The real entry
    /// ([`OllamaClient::generate_with_opts`]) calls this method, so a test can assert the legacy
    /// projection of a given decoded body through the same function instead of a copy.
    pub(crate) fn into_legacy_result(self) -> OllamaGenerateResult {
        OllamaGenerateResult {
            response: self.body.response,
            metrics: self.metrics,
        }
    }

    /// The provider completion check for the Base path, returning text or a human-readable reason.
    ///
    /// This is deliberately **not** part of the legacy entry: this provider's `POST /api/generate`
    /// response carries `done`, so a call that came back with `done = false` did not finish
    /// normally. With `done = true` the decoded text is returned verbatim — including empty or
    /// whitespace-only text, which is a normal result and not an error here.
    ///
    /// `Ok` therefore means "this call returned text without violating this provider's completion
    /// flag"; it says nothing about content quality, task success, domain commit, tool effects or
    /// the end of any invocation.
    pub(crate) fn project_base(self) -> std::result::Result<String, String> {
        if !self.body.done {
            return Err(
                "incomplete Ollama response: done is false, generated text (if any) was not \
                 returned as a normal result"
                    .to_string(),
            );
        }
        Ok(self.body.response)
    }
}

/// One non-streaming call that did not complete normally, with the facts this layer could still read.
///
/// `known_timeout` is decided where the real `reqwest::Error` is still in hand, by asking that error
/// whether it is a timeout. `error` keeps the original message so both projections can stay
/// byte-compatible with the legacy text. Nothing downstream re-derives the reason from the message.
#[derive(Debug)]
pub(crate) struct OllamaCallFailure {
    pub(crate) known_timeout: bool,
    pub(crate) error: AppError,
}

impl OllamaCallFailure {
    /// Projection for the legacy entries: the original [`AppError`], unchanged.
    pub(crate) fn into_legacy_error(self) -> AppError {
        self.error
    }
}

/// Handles a `reqwest` error from one non-streaming step, keeping a timeout fact it already knows.
///
/// Both the `send()` and the body-read step go through this, so a timeout is recorded where the
/// typed error exists rather than being inferred later from text. The error is borrowed, so the same
/// value can still be formatted into the original message.
fn transport_failure(e: &reqwest::Error, message: String) -> OllamaCallFailure {
    OllamaCallFailure {
        known_timeout: e.is_timeout(),
        error: AppError::OllamaError(message),
    }
}

/// Decodes the already-read body of a non-streaming response and derives its metrics.
///
/// This is the one place the status check, the body decoding and their error text live. The shared
/// request path calls it; so do the unit tests, so an assertion about the 800 / 400 character
/// truncation or about a missing field exercises production code rather than a copy of it.
///
/// # Errors
///
/// Returns [`AppError::OllamaError`] with the original message when the status is not successful or
/// the body cannot be decoded as [`OllamaResponse`].
pub(crate) fn decode_non_streaming_body(
    status: reqwest::StatusCode,
    body: &str,
    base_url: &str,
    model: &str,
    want_metrics: bool,
) -> Result<OllamaCallResponse> {
    if !status.is_success() {
        return Err(AppError::OllamaError(format!(
            "HTTP {} — {} (请求: POST {}/api/generate, model={})",
            status,
            body.chars().take(800).collect::<String>(),
            base_url,
            model
        )));
    }

    let ollama_response: OllamaResponse = serde_json::from_str(body).map_err(|e| {
        AppError::OllamaError(format!(
            "Failed to parse response: {} — body: {}",
            e,
            body.chars().take(400).collect::<String>()
        ))
    })?;

    Ok(OllamaCallResponse::from_response(
        ollama_response,
        want_metrics,
    ))
}

/// Ollama HTTP client.
pub struct OllamaClient {
    base_url: String,
    client: Client,
    timeout: Duration,
}

/// Builds the `POST /api/generate` body for one non-streaming text-generation call.
///
/// This is the one place that body is assembled for the two text-generation entries. The shared
/// request path and any test that needs the exact body both go through it, so the two cannot drift
/// apart.
pub(crate) fn non_streaming_request(
    model: &str,
    prompt: &str,
    temperature: Option<f32>,
    top_p: Option<f32>,
    opts: Option<&OllamaGenerateOpts>,
) -> OllamaRequest {
    OllamaRequest {
        model: model.to_string(),
        prompt: prompt.to_string(),
        stream: false,
        options: OllamaRequestOptions::from_generate(temperature, top_p, opts),
        keep_alive: opts.and_then(|o| o.keep_alive.clone()),
    }
}

fn normalize_base_url(url: String) -> String {
    url.trim_end_matches('/').to_string()
}

impl OllamaClient {
    /// Creates a new Ollama client.
    pub fn new(base_url: impl Into<String>) -> Self {
        let base_url = normalize_base_url(base_url.into());
        let client = if crate::infrastructure::is_loopback_endpoint(&base_url) {
            OLLAMA_LOOPBACK_HTTP_CLIENT.clone()
        } else {
            OLLAMA_HTTP_CLIENT.clone()
        };
        Self {
            base_url,
            client,
            timeout: ollama_timeouts::http_client_timeout(),
        }
    }

    /// Load a model into Ollama memory without generating user-visible text.
    ///
    /// # Errors
    ///
    /// Returns an error when Ollama is unreachable or rejects the model.
    pub async fn preload(&self, model: &str, keep_alive: &str) -> Result<()> {
        let url = format!("{}/api/generate", self.base_url);
        let response = self
            .client
            .post(&url)
            .json(&OllamaRequest {
                model: model.to_string(),
                prompt: String::new(),
                stream: false,
                options: None,
                keep_alive: Some(keep_alive.to_string()),
            })
            .timeout(self.timeout)
            .send()
            .await
            .map_err(|e| AppError::OllamaError(format!("preload request failed: {e}")))?;
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(AppError::OllamaError(format!(
                "preload HTTP {status}: {}",
                body.chars().take(400).collect::<String>()
            )));
        }
        Ok(())
    }

    /// Ask Ollama to unload one model immediately.
    ///
    /// # Errors
    ///
    /// Returns an error when Ollama is unreachable or rejects the request.
    pub async fn unload(&self, model: &str) -> Result<()> {
        self.preload(model, "0").await
    }

    /// Sets the request timeout.
    #[must_use]
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
    /// # Errors
    ///
    /// Returns [`Err`] with a human-readable message when the operation fails.
    /// Checks whether the Ollama service is reachable.
    pub async fn health_check(&self) -> Result<bool> {
        let url = format!("{}/api/tags", self.base_url);

        match self.client.get(&url).timeout(self.timeout).send().await {
            Ok(response) => Ok(response.status().is_success()),
            Err(_) => Ok(false),
        }
    }
    /// # Errors
    ///
    /// Returns [`Err`] with a human-readable message when the operation fails.
    /// Lists available models.
    pub async fn list_models(&self) -> Result<Vec<String>> {
        let url = format!("{}/api/tags", self.base_url);

        #[derive(Deserialize)]
        struct TagsResponse {
            models: Option<Vec<ModelInfo>>,
        }

        #[derive(Deserialize)]
        struct ModelInfo {
            name: String,
        }

        let response = self
            .client
            .get(&url)
            .timeout(self.timeout)
            .send()
            .await
            .map_err(|e| AppError::OllamaError(format!("Failed to list models: {}", e)))?;

        let tags: TagsResponse = response
            .json()
            .await
            .map_err(|e| AppError::OllamaError(format!("Failed to parse models: {}", e)))?;

        let models = tags
            .models
            .unwrap_or_default()
            .into_iter()
            .map(|m| m.name)
            .collect();

        Ok(models)
    }
    /// # Errors
    ///
    /// Returns [`Err`] with a human-readable message when the operation fails.
    /// Calls Ollama to generate a reply.
    pub async fn generate(
        &self,
        model: &str,
        prompt: &str,
        temperature: Option<f32>,
        top_p: Option<f32>,
    ) -> Result<String> {
        let out = self
            .generate_with_opts(model, prompt, temperature, top_p, None)
            .await?;
        Ok(out.response)
    }

    /// Like [`generate`](Self::generate) but accepts Deep-session `keep_alive` and returns bench metrics.
    ///
    /// # Errors
    ///
    /// Returns [`anyhow::Error`] when the Ollama HTTP request fails or the response is invalid.
    pub async fn generate_with_opts(
        &self,
        model: &str,
        prompt: &str,
        temperature: Option<f32>,
        top_p: Option<f32>,
        opts: Option<&OllamaGenerateOpts>,
    ) -> Result<OllamaGenerateResult> {
        let response = self
            .generate_with_settings(
                model,
                prompt,
                OllamaGenerateSettings {
                    temperature,
                    top_p,
                    opts,
                },
            )
            .await
            .map_err(OllamaCallFailure::into_legacy_error)?;
        Ok(response.into_legacy_result())
    }

    /// The non-streaming text-generation path these two entries share: request preparation, send,
    /// body read, status check and decode.
    ///
    /// The legacy entry [`OllamaClient::generate_with_opts`] and the B2-C1 Base adapter both call
    /// it, so there is one such chain for text generation rather than two. (Other non-streaming
    /// requests in this module — `preload`, `list_models`, `create_model_from_path` — keep their own
    /// paths and are untouched.) Callers that need the decoded body itself, or facts the
    /// convenience type [`OllamaGenerateResult`] drops such as [`OllamaResponse::done`], can use
    /// this method; those needing the legacy behaviour project through
    /// [`OllamaCallResponse::into_legacy_result`].
    ///
    /// # Errors
    ///
    /// Returns [`OllamaCallFailure`] — carrying whether the step itself knew the call timed out,
    /// plus the original [`AppError::OllamaError`] message — when the request cannot be sent, the
    /// body cannot be read, the HTTP status is not successful, or the body cannot be decoded as
    /// [`OllamaResponse`].
    pub(crate) async fn generate_with_settings(
        &self,
        model: &str,
        prompt: &str,
        settings: OllamaGenerateSettings<'_>,
    ) -> std::result::Result<OllamaCallResponse, OllamaCallFailure> {
        let OllamaGenerateSettings {
            temperature,
            top_p,
            opts,
        } = settings;
        let url = format!("{}/api/generate", self.base_url);
        let request = non_streaming_request(model, prompt, temperature, top_p, opts);

        let response = self
            .client
            .post(&url)
            .json(&request)
            .timeout(self.timeout)
            .send()
            .await
            .map_err(|e| transport_failure(&e, format!("Request failed: {}", e)))?;

        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|e| transport_failure(&e, format!("Failed to read response body: {}", e)))?;

        decode_non_streaming_body(
            status,
            &body,
            &self.base_url,
            model,
            opts.is_some_and(|o| o.want_metrics),
        )
        .map_err(|error| OllamaCallFailure {
            known_timeout: false,
            error,
        })
    }
    /// # Errors
    ///
    /// Returns [`Err`] with a human-readable message when the operation fails.
    /// Calls Ollama to generate a reply with streaming, invoking `on_token` per chunk.
    pub async fn generate_stream_with_callback(
        &self,
        model: &str,
        prompt: &str,
        temperature: Option<f32>,
        top_p: Option<f32>,
        on_token: Arc<dyn Fn(&str) + Send + Sync>,
    ) -> Result<String> {
        let out = self
            .generate_stream_with_callback_and_opts(
                model,
                prompt,
                temperature,
                top_p,
                on_token,
                None,
            )
            .await?;
        Ok(out.response)
    }

    /// Streaming generate with optional `keep_alive` and final-frame metrics.
    ///
    /// # Errors
    ///
    /// Returns [`anyhow::Error`] when the Ollama stream request fails or the final frame is invalid.
    pub async fn generate_stream_with_callback_and_opts(
        &self,
        model: &str,
        prompt: &str,
        temperature: Option<f32>,
        top_p: Option<f32>,
        on_token: Arc<dyn Fn(&str) + Send + Sync>,
        opts: Option<&OllamaGenerateOpts>,
    ) -> Result<OllamaGenerateResult> {
        use futures_util::StreamExt;

        let url = format!("{}/api/generate", self.base_url);

        let request = OllamaRequest {
            model: model.to_string(),
            prompt: prompt.to_string(),
            stream: true,
            options: OllamaRequestOptions::from_generate(temperature, top_p, opts),
            keep_alive: opts.and_then(|o| o.keep_alive.clone()),
        };

        let response = self
            .client
            .post(&url)
            .json(&request)
            .timeout(self.timeout)
            .send()
            .await
            .map_err(|e| AppError::OllamaError(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(AppError::OllamaError(format!(
                "Ollama returned status: {} — {}",
                status,
                body.chars().take(400).collect::<String>()
            )));
        }

        let mut stream = response.bytes_stream();
        let mut line_buf = String::new();
        let mut full_response = String::new();
        let mut final_metrics = OllamaGenerateMetrics::default();

        while let Some(chunk) = stream.next().await {
            let chunk =
                chunk.map_err(|e| AppError::OllamaError(format!("Stream read failed: {}", e)))?;
            line_buf.push_str(&String::from_utf8_lossy(&chunk));
            while let Some(pos) = line_buf.find('\n') {
                let line = line_buf.drain(..=pos).collect::<String>();
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                if let Ok(json) = serde_json::from_str::<OllamaResponse>(line) {
                    if !json.response.is_empty() {
                        full_response.push_str(&json.response);
                        on_token(json.response.as_str());
                    }
                    if json.done {
                        final_metrics = OllamaGenerateMetrics {
                            total_duration_ns: json.total_duration,
                            load_duration_ns: json.load_duration,
                            prompt_eval_duration_ns: json.prompt_eval_duration,
                            prompt_eval_count: json.prompt_eval_count,
                            eval_duration_ns: json.eval_duration,
                            eval_count: json.eval_count,
                        };
                    }
                } else {
                    tracing::warn!(
                        target: "oclive_llm",
                        line = %line.chars().take(120).collect::<String>(),
                        "ollama stream NDJSON line parse skipped"
                    );
                }
            }
        }
        if !line_buf.trim().is_empty() {
            match serde_json::from_str::<OllamaResponse>(line_buf.trim()) {
                Ok(json) => {
                    if !json.response.is_empty() {
                        full_response.push_str(&json.response);
                        on_token(json.response.as_str());
                    }
                    if json.done {
                        final_metrics = OllamaGenerateMetrics {
                            total_duration_ns: json.total_duration,
                            load_duration_ns: json.load_duration,
                            prompt_eval_duration_ns: json.prompt_eval_duration,
                            prompt_eval_count: json.prompt_eval_count,
                            eval_duration_ns: json.eval_duration,
                            eval_count: json.eval_count,
                        };
                    }
                }
                Err(e) => {
                    tracing::warn!(
                        target: "oclive_llm",
                        error = %e,
                        "ollama stream trailing NDJSON parse skipped"
                    );
                }
            }
        }

        let metrics = if opts.is_some_and(|o| o.want_metrics) {
            final_metrics
        } else {
            OllamaGenerateMetrics::default()
        };

        Ok(OllamaGenerateResult {
            response: full_response,
            metrics,
        })
    }

    /// Buffered streaming (legacy): reads full body then merges lines.
    ///
    /// # Errors
    ///
    /// HTTP transport failures or malformed Ollama NDJSON stream.
    pub async fn generate_stream(
        &self,
        model: &str,
        prompt: &str,
        temperature: Option<f32>,
        top_p: Option<f32>,
    ) -> Result<String> {
        self.generate_stream_with_callback(model, prompt, temperature, top_p, Arc::new(|_| {}))
            .await
    }

    /// Register a local GGUF (or bin) as an Ollama model via `POST /api/create`.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::AppError`] when the HTTP request fails or Ollama rejects the create payload.
    pub async fn create_model_from_path(&self, name: &str, model_path: &str) -> Result<()> {
        let url = format!("{}/api/create", self.base_url);
        let path_escaped = model_path.replace('\\', "/");
        let modelfile = format!("FROM \"{path_escaped}\"\n");
        let body = serde_json::json!({
            "name": name.trim(),
            "modelfile": modelfile,
        });
        let response = self
            .client
            .post(&url)
            .json(&body)
            .timeout(self.timeout)
            .send()
            .await
            .map_err(|e| AppError::OllamaError(format!("create model request: {e}")))?;
        let status = response.status();
        let text = response
            .text()
            .await
            .map_err(|e| AppError::OllamaError(format!("create model body: {e}")))?;
        if !status.is_success() {
            return Err(AppError::OllamaError(format!(
                "create model HTTP {status}: {}",
                text.chars().take(500).collect::<String>()
            )));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ollama_client_new() {
        let client = OllamaClient::new("http://localhost:11434");
        assert_eq!(client.base_url, "http://localhost:11434");
    }

    #[test]
    fn test_ollama_client_with_timeout() {
        let client =
            OllamaClient::new("http://localhost:11434").with_timeout(Duration::from_secs(60));
        assert_eq!(client.timeout, Duration::from_secs(60));
    }

    #[test]
    fn test_ollama_request_serialization() {
        let request = OllamaRequest {
            model: "llama2".to_string(),
            prompt: "Hello".to_string(),
            stream: false,
            options: Some(OllamaRequestOptions {
                temperature: Some(0.7),
                top_p: None,
                num_predict: Some(512),
                num_ctx: Some(8192),
            }),
            keep_alive: Some("30m".to_string()),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"model\":\"llama2\""));
        assert!(json.contains("\"prompt\":\"Hello\""));
        assert!(json.contains("\"options\":{\"temperature\":0.7"));
        assert!(json.contains("\"num_predict\":512"));
        assert!(json.contains("\"num_ctx\":8192"));
        assert!(json.contains("\"keep_alive\":\"30m\""));
        assert!(!json.contains("\"top_p\"")); // should be omitted
    }

    #[test]
    fn test_ollama_response_deserialization() {
        let json = r#"{
            "response": "Hello there!",
            "model": "llama2",
            "created_at": "2024-01-01T00:00:00Z",
            "done": true,
            "load_duration": 12000000,
            "prompt_eval_duration": 34000000,
            "eval_count": 12
        }"#;

        let response: OllamaResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.response, "Hello there!");
        assert_eq!(response.model, "llama2");
        assert!(response.done);
        assert_eq!(response.load_duration, Some(12_000_000));
        assert_eq!(response.prompt_eval_duration, Some(34_000_000));
        assert_eq!(response.eval_count, Some(12));
    }

    #[tokio::test]
    async fn test_health_check_offline() {
        let client = OllamaClient::new("http://localhost:9999"); // unused port
        let result = client.health_check().await;
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    /// B2-C1(R1): facts the shared non-streaming path must keep, how the two projections read
    /// them, and what the production decode/error functions produce. No test here performs a
    /// network call: the only transport facts are the ones this module already recorded before it
    /// turned an error into text.
    mod b2_c1 {
        use super::super::{decode_non_streaming_body, OllamaCallResponse, OllamaResponse};

        fn sample(response: &str, done: bool) -> String {
            format!(
                r#"{{"response":{},"model":"llama2","created_at":"2024-01-01T00:00:00Z","done":{done}}}"#,
                serde_json::to_string(response).expect("sample text must encode")
            )
        }

        /// Decodes through the production decode function (status 200, no metrics).
        fn decode_ok(body: &str) -> OllamaCallResponse {
            decode_non_streaming_body(
                reqwest::StatusCode::OK,
                body,
                "http://localhost:11434",
                "llama2",
                false,
            )
            .expect("production decode must accept this sample")
        }

        /// The production HTTP-status error text for one non-success response.
        fn http_error(status: reqwest::StatusCode, body: &str) -> String {
            decode_non_streaming_body(status, body, "http://localhost:11434", "llama2", false)
                .expect_err("non-success status must fail")
                .to_string()
        }

        /// The production parse error text for one body.
        fn parse_error(body: &str) -> String {
            decode_non_streaming_body(
                reqwest::StatusCode::OK,
                body,
                "http://localhost:11434",
                "llama2",
                false,
            )
            .expect_err("undecodable body must fail")
            .to_string()
        }

        #[test]
        fn b2_c1_decoded_body_keeps_the_done_flag() {
            let incomplete = decode_ok(&sample("partial answer", false));
            assert!(!incomplete.body.done);
            assert_eq!(incomplete.body.response, "partial answer");

            let complete = decode_ok(&sample("finished", true));
            assert!(complete.body.done);
        }

        #[test]
        fn b2_c1_completion_check_returns_complete_text_verbatim() {
            for text in ["finished answer", "", "   ", "line one\nline two"] {
                let call = decode_ok(&sample(text, true));
                let projected = call.project_base().expect("done=true must return the text");
                assert_eq!(projected, text, "text must not be trimmed or rewritten");
            }
        }

        #[test]
        fn b2_c1_legacy_projection_keeps_an_incomplete_response_successful() {
            // Same decoded response, both real projections: the legacy entry keeps its old
            // behaviour, the Base projection does not hand partial text back as a normal result.
            for text in ["partial answer", ""] {
                let legacy = decode_ok(&sample(text, false)).into_legacy_result();
                assert_eq!(
                    legacy.response, text,
                    "legacy text must pass through unchanged"
                );

                let reason = decode_ok(&sample(text, false))
                    .project_base()
                    .expect_err("done=false must not return text");
                assert!(
                    reason.contains("done is false"),
                    "reason should name the actual fact, got: {reason}"
                );
                assert!(
                    text.is_empty() || !reason.contains(text),
                    "the partial text must not be presented as a normal result"
                );
            }
        }

        #[test]
        fn b2_c1_legacy_projection_keeps_complete_text_and_whitespace() {
            for text in ["finished answer", "", "   ", "line one\nline two"] {
                let legacy = decode_ok(&sample(text, true)).into_legacy_result();
                assert_eq!(legacy.response, text, "legacy text must not be trimmed");
            }
        }

        #[test]
        fn b2_c1_metrics_cover_all_six_fields_and_both_switches() {
            let body = r#"{
                "response": "Hello there!",
                "model": "llama2",
                "created_at": "2024-01-01T00:00:00Z",
                "done": true,
                "total_duration": 1000000,
                "load_duration": 12000000,
                "prompt_eval_duration": 34000000,
                "prompt_eval_count": 21,
                "eval_duration": 56000000,
                "eval_count": 12
            }"#;

            let wanted = decode_non_streaming_body(
                reqwest::StatusCode::OK,
                body,
                "http://localhost:11434",
                "llama2",
                true,
            )
            .expect("production decode must accept this sample");
            let m = &wanted.metrics;
            assert_eq!(m.total_duration_ns, Some(1_000_000));
            assert_eq!(m.load_duration_ns, Some(12_000_000));
            assert_eq!(m.prompt_eval_duration_ns, Some(34_000_000));
            assert_eq!(m.prompt_eval_count, Some(21));
            assert_eq!(m.eval_duration_ns, Some(56_000_000));
            assert_eq!(m.eval_count, Some(12));
            // the legacy millisecond conversions the old entry reports
            assert_eq!(m.prompt_eval_ms(), Some(34));
            assert_eq!(m.load_ms(), Some(12));

            let unwanted = decode_ok(body);
            let n = &unwanted.metrics;
            assert_eq!(n.total_duration_ns, None);
            assert_eq!(n.load_duration_ns, None);
            assert_eq!(n.prompt_eval_duration_ns, None);
            assert_eq!(n.prompt_eval_count, None);
            assert_eq!(n.eval_duration_ns, None);
            assert_eq!(n.eval_count, None);
            assert_eq!(n.prompt_eval_ms(), None);
            // the response text is unaffected by the metrics switch
            assert_eq!(unwanted.into_legacy_result().response, "Hello there!");
        }

        #[test]
        fn b2_c1_non_success_status_uses_the_production_error_text() {
            let cases = [
                (reqwest::StatusCode::INTERNAL_SERVER_ERROR, "boom"),
                (reqwest::StatusCode::NOT_FOUND, ""),
                (reqwest::StatusCode::BAD_GATEWAY, "upstream 502"),
            ];
            for (status, body) in cases {
                let text = http_error(status, body);
                let expected = format!(
                    "Ollama error: HTTP {} — {} (请求: POST http://localhost:11434/api/generate, model=llama2)",
                    status,
                    body.chars().take(800).collect::<String>()
                );
                assert_eq!(text, expected);
            }
        }

        #[test]
        fn b2_c1_http_error_truncates_at_800_chars_with_multibyte_text() {
            // 1200 CJK characters: a byte-based cut would split a character, and a naive limit
            // would keep far more than 800.
            let body: String = "响".repeat(1200);
            let text = http_error(reqwest::StatusCode::BAD_REQUEST, &body);
            let kept: String = body.chars().take(800).collect();
            assert!(text.contains(&kept), "800 characters must be kept intact");
            assert_eq!(
                text.matches('响').count(),
                800,
                "exactly 800 characters, not 800 bytes"
            );
            assert!(text.starts_with("Ollama error: HTTP 400 Bad Request — "));
        }

        #[test]
        fn b2_c1_parse_error_uses_the_production_error_text_and_400_char_truncation() {
            let truncated = "{\"response\":\"truncated\",\"model\":\"llama2\"";
            let text = parse_error(truncated);
            assert!(text.starts_with("Ollama error: Failed to parse response: "));
            assert!(text.contains(" — body: "));
            assert!(text.ends_with(truncated), "short bodies are kept in full");

            let long_body: String = "甲".repeat(900);
            let long_text = parse_error(&long_body);
            let kept: String = long_body.chars().take(400).collect();
            assert!(long_text.ends_with(&kept));
            assert_eq!(long_text.matches('甲').count(), 400);
        }

        #[test]
        fn b2_c1_missing_required_field_fails_through_the_production_decode() {
            // The wire type requires response/model/created_at/done: omitting one is a decode
            // failure, not a defaulted value.
            let body = r#"{"response":"hi","model":"llama2","created_at":"2024-01-01T00:00:00Z"}"#;
            let text = parse_error(body);
            assert!(text.starts_with("Ollama error: Failed to parse response: "));
            assert!(text.contains("done"));
            assert!(text.ends_with(body));
        }

        #[test]
        fn b2_c1_sample_must_actually_be_decodable() {
            // Guards the samples above: the wire type requires response/model/created_at/done.
            let decoded: OllamaResponse =
                serde_json::from_str(&sample("x", true)).expect("sample must decode");
            assert!(decoded.done);
            assert_eq!(decoded.model, "llama2");
        }
    }
}
