//! B2-C1: the concrete Ollama non-streaming adapter that implements the B1 LLM Base.
//!
//! [`OllamaBaseAdapter`] wraps a concrete [`OllamaClient`] and offers it as the B1 LLM Base
//! ([`LlmBase`]). It is one specific implementation, not a general-purpose wrapper for arbitrary
//! clients: it does not take `Arc<dyn LlmClient>`, does not probe capabilities and does not
//! translate between providers.
//!
//! # What it reuses
//!
//! The non-streaming text-generation path shared by this crate's two generation entries
//! ([`OllamaClient::generate_with_opts`] and this adapter): the same request preparation, the same
//! `reqwest` client and timeout, the same body read, the same status check and the same `serde`
//! decode. Parameter merging is the shared helper in [`super::llm`]. No second HTTP path, parser or
//! retry loop exists here. Other non-streaming requests in that module (`preload`, `list_models`,
//! `create_model_from_path`) keep their own paths and are not involved.
//!
//! # What it promises
//!
//! Only text. The caller binds the client, the already chosen model and the optional generation
//! options; the request `input` is the prepared generation input. There is no model discovery, no
//! credential loading, no service probe, no preload and no extra request or retry.
//!
//! A response whose provider completion flag says generation did not finish is reported as a
//! failed call even when it already carries partial text; partial text is never handed back as a
//! normal result. A response that did finish may carry empty text — that is a normal result here,
//! not an error. `Ok` therefore means "this call returned text without violating the provider's
//! completion flag" and nothing more: it does not establish content quality, task success, domain
//! commit, tool effects or the end of any invocation.
//!
//! Known timeouts are reported as timed out. Request errors, body read errors, non-success HTTP
//! status and response parse failures are reported as failures without guessing a more specific
//! reason from text. The error `detail` is human-readable only.
//!
//! # Scope and limits
//!
//! This adapter is **not registered anywhere**: the reference Host does not call it, and building
//! one sends no request — the actual call only happens when a caller polls
//! [`LlmBase::generate`]. The production composition (`AppState`, `SharedOllamaClient`,
//! `CoordinatedExternalLlm`, `PerformanceLlmClient` and the resource / authorization wrappers) is
//! untouched by this adapter and not replaced by it; this adapter does not provide those
//! observation, gating or fallback promises, and a deployment that wires it still owes whatever
//! authorization, resource and scheduling rules apply to it.
//!
//! Independent of that, the reference Host does have a **private** non-streaming binding that uses
//! the same completion judgement (the `pub(crate)` module `infrastructure::base_llm_binding`,
//! deliberately not linked here because it is not part of this public page); it is wired into the
//! builtin assembly for `generate_with_opts`. That is a separate, Host-private piece of composition:
//! it does not register this public adapter, and it does not make the Base trait itself usable from
//! a `Send` boundary for arbitrary implementations.
//!
//! There is no streaming, tag, metrics or structured-output promise here. Cancellation has no
//! protocol: dropping the returned future means the caller stopped waiting, nothing more.
//!
//! [`OllamaClient`]'s base URL is configurable, so this type does not treat itself as proof that
//! traffic stays on the loopback interface.
//!
//! # Example
//!
//! Constructing the adapter only binds values; it sends nothing.
//!
//! ```no_run
//! use std::sync::Arc;
//!
//! use oclive_kernel_host::infrastructure::base_llm::OllamaBaseAdapter;
//! use oclive_kernel_host::infrastructure::ollama_client::OllamaClient;
//!
//! let client = Arc::new(OllamaClient::new("http://localhost:11434"));
//! let adapter = OllamaBaseAdapter::new(client, "qwen2.5:7b", None);
//! let _: &dyn oclive_kernel_contracts::LlmBase = &adapter;
//! ```

use std::sync::Arc;

use oclive_kernel_contracts::{BaseCallFuture, LlmBase, LlmGenerateOpts};
use oclive_kernel_types::{BaseCallError, BaseCallErrorKind, LlmBaseRequest};

use super::llm::b2_c1_merge_options;
use super::llm_params;
use super::ollama_client::OllamaClient;

/// A concrete Ollama non-streaming [`LlmBase`] implementation.
///
/// The type binds three things and nothing else: the client that will carry the request, the model
/// the caller has already chosen, and the caller's optional generation options. The model is never
/// guessed from the prompt, and no default sampling is invented here — the sampling defaults stay
/// where they already live (read per call from [`super::llm_params`]) and the shared merge applies
/// them only to a bound `temperature` / `top_p` that the caller left unset. Every other option keeps
/// its existing per-field conversion and serialization rule.
pub struct OllamaBaseAdapter {
    client: Arc<OllamaClient>,
    model: String,
    options: Option<LlmGenerateOpts>,
}

impl OllamaBaseAdapter {
    /// Binds one concrete client, one already chosen model and optional generation options.
    ///
    /// The bound value keeps the caller's own distinction between "no options supplied" and "options
    /// supplied but left at their defaults". That distinction is a fact about this binding and does
    /// not by itself decide whether the request carries an `options` object: after the shared merge,
    /// `temperature` and `top_p` are always resolved (the caller's value when set, otherwise the
    /// sampling pair supplied for that call), so the request typically carries both regardless. The
    /// remaining options — keep-alive, output-token and context-token caps — are carried when the
    /// caller set them and omitted when it did not; the metrics request selects local metrics
    /// handling and is not a wire field. Nothing here reads the environment, resolves a model, loads
    /// a credential, probes the service or sends a request.
    #[must_use]
    pub fn new(
        client: Arc<OllamaClient>,
        model: impl Into<String>,
        options: Option<LlmGenerateOpts>,
    ) -> Self {
        Self {
            client,
            model: model.into(),
            options,
        }
    }
}

/// Projects one shared non-streaming call result into the Base view.
///
/// This is the Base view's single entry point: the public C1 adapter
/// ([`OllamaBaseAdapter`]) calls it. The Host's checked non-streaming binding does **not** call it —
/// it projects the same result itself in [`super::base_llm_binding::project_checked_call`], keeping
/// the caller-side metrics and the original `AppError` instead of round-tripping through a
/// `BaseCallError`. What both views genuinely share is the underlying completion judgement
/// ([`super::ollama_client::OllamaCallResponse::project_base`]) and the transport
/// (`generate_with_settings`); neither re-implements them.
///
/// A `done = true` reply returns its text verbatim — including empty and whitespace-only text, which
/// is a normal result — while `done = false` becomes `Failed` with the provider's reason. A transport
/// failure keeps the kind the transport recorded (`TimedOut` only when it knew that) and its original
/// diagnostic text; nothing here parses that text.
pub(crate) fn project_ollama_call(
    result: std::result::Result<
        super::ollama_client::OllamaCallResponse,
        super::ollama_client::OllamaCallFailure,
    >,
) -> std::result::Result<String, BaseCallError> {
    match result {
        Ok(call) => match call.project_base() {
            Ok(text) => Ok(text),
            Err(reason) => Err(BaseCallError {
                kind: BaseCallErrorKind::Failed,
                detail: Some(reason),
            }),
        },
        Err(failure) => Err(BaseCallError {
            kind: failure.base_error_kind(),
            detail: Some(failure.base_detail()),
        }),
    }
}

impl LlmBase for OllamaBaseAdapter {
    /// Generates text for `request.input` with this adapter's bound client, model and options.
    ///
    /// # Errors
    ///
    /// Returns a failed Base call when the shared request path fails, when the response cannot be
    /// decoded, or when the decoded response says generation did not finish. A timeout the shared
    /// path could still observe while holding the typed error is reported as timed out.
    fn generate<'a>(&'a self, request: LlmBaseRequest<'a>) -> BaseCallFuture<'a, String> {
        Box::pin(async move {
            // Same per-call defaults and same merge as the legacy entry; the bound options decide
            // the rest, and a bound `None` stays `None`.
            let (temperature, top_p) = llm_params::main_chat_options();
            let settings = b2_c1_merge_options(self.options.as_ref(), (temperature, top_p));
            // One shared execution, then the one shared Base projection.
            let call = self
                .client
                .generate_with_settings(&self.model, request.input, settings.request_settings())
                .await;
            project_ollama_call(call)
        })
    }
}

#[cfg(test)]
mod b2_c1_tests {
    use super::super::ollama_client::OllamaCallFailure;
    use super::*;

    fn failure(message: &str, known_timeout: bool) -> OllamaCallFailure {
        OllamaCallFailure {
            known_timeout,
            error: crate::error::AppError::OllamaError(message.to_string()),
        }
    }

    #[test]
    fn b2_c1_adapter_binds_model_and_options_verbatim() {
        let bound = OllamaBaseAdapter::new(
            Arc::new(OllamaClient::new("http://127.0.0.1:1")),
            "bound-model",
            None,
        );
        assert_eq!(bound.model, "bound-model");
        assert!(bound.options.is_none(), "None must stay None");

        let with_defaults = OllamaBaseAdapter::new(
            Arc::new(OllamaClient::new("http://127.0.0.1:1")),
            "bound-model",
            Some(LlmGenerateOpts::default()),
        );
        assert!(
            with_defaults.options.is_some(),
            "Some(Default::default()) must stay a distinct bound value"
        );
    }

    #[test]
    fn b2_c1_known_timeout_projects_to_timed_out() {
        // The fact comes from the shared path, not from this text: the message deliberately does
        // NOT contain any timeout wording, and the projection must still report a timeout.
        let projected = project_ollama_call(Err(failure(
            "Request failed: error sending request for url (http://127.0.0.1:1/api/generate)",
            true,
        )))
        .expect_err("a failure stays a failure");
        assert_eq!(projected.kind, BaseCallErrorKind::TimedOut);
        assert!(projected
            .detail
            .as_deref()
            .unwrap_or_default()
            .starts_with("Ollama error: Request failed:"));
    }

    #[test]
    fn b2_c1_body_read_timeout_also_projects_to_timed_out() {
        let projected = project_ollama_call(Err(failure(
            "Failed to read response body: connection closed before message completed",
            true,
        )))
        .expect_err("a failure stays a failure");
        assert_eq!(projected.kind, BaseCallErrorKind::TimedOut);
    }

    #[test]
    fn b2_c1_failure_kind_follows_the_recorded_fact_not_the_text() {
        // Exact misleading phrase: no timeout was recorded, so it must stay Failed.
        let misleading = project_ollama_call(Err(failure(
            "HTTP 500 Internal Server Error — Request failed: operation timed out \
             (请求: POST http://localhost:11434/api/generate, model=m)",
            false,
        )))
        .expect_err("a failure stays a failure");
        assert_eq!(misleading.kind, BaseCallErrorKind::Failed);
        assert!(misleading
            .detail
            .as_deref()
            .unwrap_or_default()
            .contains("operation timed out"));

        // Plain failure without any keyword. The harness wraps the message once, exactly as the
        // shared path builds it.
        let plain_message = "Failed to read response body: connection reset by peer";
        let plain = project_ollama_call(Err(failure(plain_message, false)))
            .expect_err("a failure stays a failure");
        assert_eq!(plain.kind, BaseCallErrorKind::Failed);
        let expected = format!("Ollama error: {plain_message}");
        assert_eq!(
            plain.detail.as_deref(),
            Some(expected.as_str()),
            "detail must carry the error text unchanged"
        );
    }
}
