//! CP-B3-C1: the shared checked non-streaming execution and the Host binding that uses it.
//!
//! This module is the **Host-private** production binding for the builtin non-streaming
//! **`generate_with_opts`** path. It exists so that path can honour the Base completion semantics
//! **without** going through an erased [`oclive_kernel_contracts::BaseCallFuture`]:
//!
//! * [`checked_generate_with_opts`] is the **one production shared path**: the caller reads this
//!   call's sampling defaults and passes them in, this helper merges them once with the caller's own
//!   options and lends the merged settings to the transport step, and it is the only thing that
//!   awaits the call and projects it. Its future is a concrete `async fn` future, and the production
//!   transport it is instantiated with boxes an explicitly `Send` future, which is what lets the Host
//!   await it across a `Send` boundary — unlike an erased `BaseCallFuture`. That `Send` fact belongs
//!   to this production instantiation and is asserted at the bottom of this file; it is not a
//!   property every `CheckedCall` implementation has (the test transport is deliberately not `Send`).
//! * [`LlmWithCheckedCompletion`]'s `generate_with_opts` **calls that production helper**, so model,
//!   prompt and the merged settings travel through the real path rather than a test-only copy.
//! * The Base view of one call is produced by
//!   [`crate::infrastructure::base_llm::project_ollama_call`], which projects the same
//!   `Result<OllamaCallResponse, OllamaCallFailure>` shape. The completion judgement both views rely
//!   on is [`OllamaCallResponse::project_base`]; the two views share that judgement, and the call
//!   itself happens once.
//!
//! # What is *not* upgraded here
//!
//! Only `generate_with_opts` uses the checked projection. `generate`, `generate_tag`, both streaming
//! entries, `startup_probe` and `supports_prefix_cache` **forward to the wrapped client unchanged**,
//! so those entry points keep exactly the behaviour they had behind `SharedOllamaClient` — including
//! `generate`, which other callers (for example a coordinating wrapper's fallback generate) already
//! use. No trait default is reached by accident.
//!
//! # The one approved completion difference
//!
//! The checked projection treats a provider reply with `done = false` as **not a normal result**
//! (whether or not it carried partial text), while a `done = true` reply returns its text verbatim,
//! including empty or whitespace-only text. Nothing else changes: request preparation, status
//! checking, decoding, error text and truncation rules stay in the existing shared request path, and
//! the legacy projections are untouched.
//!
//! Nothing here retries, and nothing here infers that an invocation ended, that no side effect
//! occurred, that anything was rolled back, or that a retry is safe.

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use async_trait::async_trait;
use oclive_kernel_contracts::{LlmClient, LlmGenerateOpts, LlmGenerateOutcome, LlmTokenSink};
use oclive_kernel_types::{AppError, Result};

use super::llm::{b2_c1_merge_options, log_ollama_metrics, B2C1OllamaSettings};
use super::llm_params;
use super::ollama_client::{OllamaCallFailure, OllamaCallResponse, OllamaClient};

/// The transport step one checked call performs: given the model, the prompt and the merged
/// settings, produce this call's result.
///
/// The shipped step is [`ProductionCheckedCall`]; the implementation seam exists so a test can
/// substitute the transport while every parameter still travels through the production path. The
/// future is an **associated type**, so each implementation chooses its own carrier: the production
/// step boxes an explicitly `Send` future (keeping the shared path awaitable across a `Send`
/// boundary), while the test step returns a plain non-`Send` future that borrows the call's
/// parameters.
trait CheckedCall<'p> {
    /// This step's own future type.
    type Call: Future<Output = std::result::Result<OllamaCallResponse, OllamaCallFailure>>;

    fn call(self, model: &'p str, prompt: &'p str, settings: &'p B2C1OllamaSettings) -> Self::Call;
}

/// The shipped transport step: the existing shared request path, unchanged.
struct ProductionCheckedCall {
    client: Arc<OllamaClient>,
}

impl<'p> CheckedCall<'p> for ProductionCheckedCall {
    // The boxed future is explicitly `Send`: the client is owned by the step and every borrow it
    // keeps lives for the call, so the whole shared path stays `Send` — the property this module
    // exists to keep, asserted at the bottom of this file.
    type Call = Pin<
        Box<
            dyn Future<Output = std::result::Result<OllamaCallResponse, OllamaCallFailure>>
                + Send
                + 'p,
        >,
    >;

    fn call(self, model: &'p str, prompt: &'p str, settings: &'p B2C1OllamaSettings) -> Self::Call {
        let client = self.client;
        let settings = settings.clone();
        Box::pin(async move {
            client
                .generate_with_settings(model, prompt, settings.request_settings())
                .await
        })
    }
}

/// **The one production shared path** for a checked non-streaming call.
///
/// Responsibilities, in order: take this call's sampling defaults, merge them once with the caller's
/// own `None`/`Some` options, hand the merged settings to the transport step, await it, and project
/// the result. The merged settings are owned by this call and live until it completes, so two
/// in-flight calls cannot overwrite each other's model or options.
async fn checked_generate_with_opts<C>(
    sampling_defaults: (Option<f32>, Option<f32>),
    opts: Option<&LlmGenerateOpts>,
    model: &str,
    prompt: &str,
    call: C,
) -> Result<LlmGenerateOutcome>
where
    C: for<'p> CheckedCall<'p>,
{
    let settings = b2_c1_merge_options(opts, sampling_defaults);
    let result = call.call(model, prompt, &settings).await;
    project_checked_call(result, model)
}

/// Projects one checked call result into the outcome the Host consumes.
///
/// * `Ok(response)` goes through the provider completion check
///   ([`OllamaCallResponse::project_base`]): a `done = true` reply returns its text verbatim (empty
///   and whitespace-only included) with this call's metrics; a `done = false` reply becomes an
///   [`AppError::OllamaError`] carrying the reason, so partial text never reaches a success branch.
/// * `Err(failure)` keeps the original [`AppError`] unchanged: it is neither re-wrapped nor parsed.
///
/// A `done = true` reply — including an empty one — writes the same metrics line the legacy entry
/// writes, once, from this call's own response.
fn project_checked_call(
    result: std::result::Result<OllamaCallResponse, OllamaCallFailure>,
    model: &str,
) -> Result<LlmGenerateOutcome> {
    match result {
        Ok(mut call) => {
            // `project_base` consumes the decoded body, so this call's metrics are moved out first.
            let metrics = std::mem::take(&mut call.metrics);
            match call.project_base() {
                Ok(reply) => {
                    log_ollama_metrics(model, &metrics);
                    Ok(LlmGenerateOutcome {
                        reply,
                        prompt_eval_ms: metrics.prompt_eval_ms(),
                    })
                }
                Err(reason) => Err(AppError::OllamaError(reason)),
            }
        }
        Err(failure) => Err(failure.into_legacy_error()),
    }
}

/// A builtin non-streaming [`LlmClient`] whose `generate_with_opts` honours the provider's `done`
/// flag.
///
/// That single method is re-implemented through [`checked_generate_with_opts`]; every other method
/// forwards to the wrapped client exactly as before, so `generate` and the streaming, tag and probe
/// entries keep their previous behaviour. The wrapper holds one shared client and no mutable state.
pub(crate) struct LlmWithCheckedCompletion {
    inner: Arc<OllamaClient>,
}

impl LlmWithCheckedCompletion {
    /// Wraps one already-constructed client; the constructor performs no call and reads nothing.
    pub(crate) fn new(inner: Arc<OllamaClient>) -> Self {
        Self { inner }
    }
}

#[async_trait]
impl LlmClient for LlmWithCheckedCompletion {
    fn supports_prefix_cache(&self) -> bool {
        self.inner.supports_prefix_cache()
    }

    /// Unchanged legacy behaviour: delegated to the same client, with no extra merge, completion
    /// check or metrics logging.
    async fn generate(&self, model: &str, prompt: &str) -> Result<String> {
        LlmClient::generate(self.inner.as_ref(), model, prompt).await
    }

    async fn generate_tag(&self, model: &str, prompt: &str) -> Result<String> {
        LlmClient::generate_tag(self.inner.as_ref(), model, prompt).await
    }

    async fn generate_with_opts(
        &self,
        model: &str,
        prompt: &str,
        opts: Option<&LlmGenerateOpts>,
    ) -> Result<LlmGenerateOutcome> {
        // The sampling defaults are read here, per call, from their existing home; the merge happens
        // once inside the shared path, so nothing moves into the constructor.
        let sampling_defaults = llm_params::main_chat_options();
        checked_generate_with_opts(
            sampling_defaults,
            opts,
            model,
            prompt,
            ProductionCheckedCall {
                client: Arc::clone(&self.inner),
            },
        )
        .await
    }

    async fn generate_stream(
        &self,
        model: &str,
        prompt: &str,
        on_token: LlmTokenSink,
    ) -> Result<String> {
        LlmClient::generate_stream(self.inner.as_ref(), model, prompt, on_token).await
    }

    async fn generate_stream_with_opts(
        &self,
        model: &str,
        prompt: &str,
        on_token: LlmTokenSink,
        opts: Option<&LlmGenerateOpts>,
    ) -> Result<LlmGenerateOutcome> {
        LlmClient::generate_stream_with_opts(self.inner.as_ref(), model, prompt, on_token, opts)
            .await
    }

    async fn startup_probe(&self) -> Result<()> {
        LlmClient::startup_probe(self.inner.as_ref()).await
    }
}

#[cfg(test)]
mod cp_b3_c1_tests {
    use super::*;
    use crate::infrastructure::base_llm::project_ollama_call;
    use crate::infrastructure::ollama_client::decode_non_streaming_body;
    use oclive_kernel_types::BaseCallErrorKind;
    use std::cell::RefCell;
    use std::rc::Rc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::task::{Context, Poll, Wake, Waker};

    /// Builds a decoded call response from the wire shape, through the production decoder.
    fn decoded(response_text: &str, done: bool, want_metrics: bool) -> OllamaCallResponse {
        let body = format!(
            r#"{{"response":{response_text},"model":"m","created_at":"2024-01-01T00:00:00Z","done":{done}}}"#
        );
        decode_non_streaming_body(
            reqwest::StatusCode::OK,
            &body,
            "http://x",
            "m",
            want_metrics,
        )
        .expect("a decodable body")
    }

    fn failure(message: &str, known_timeout: bool) -> OllamaCallFailure {
        OllamaCallFailure {
            known_timeout,
            error: AppError::OllamaError(message.to_string()),
        }
    }

    /// Counts wakes so a `Pending` poll can be shown to schedule one.
    struct WakeCounter(AtomicUsize);
    impl Wake for WakeCounter {
        fn wake(self: Arc<Self>) {
            self.0.fetch_add(1, Ordering::SeqCst);
        }
        fn wake_by_ref(self: &Arc<Self>) {
            self.0.fetch_add(1, Ordering::SeqCst);
        }
    }

    /// What one poll of a controlled call read from that call's **own borrowed** request values.
    #[derive(Clone, Debug)]
    struct PhaseReading {
        model: String,
        prompt: String,
        settings: B2C1OllamaSettings,
    }

    /// What one controlled transport call recorded, shared with the test.
    ///
    /// `calls` is the entry count of the transport step itself, counted separately from `polls` and
    /// from wakes. `first_poll` and `resumed_poll` are the two readings of the same borrowed
    /// parameters: the first poll's and the poll after the `Pending`.
    #[derive(Clone, Debug, Default)]
    struct Observation {
        calls: usize,
        first_poll: Option<PhaseReading>,
        resumed_poll: Option<PhaseReading>,
    }

    /// A controlled transport step whose future **holds this call's real borrowed parameters**
    /// (`&'p str` model, prompt and `&'p B2C1OllamaSettings`), stays `Pending` on its first poll and
    /// answers on the second.
    struct RecordingCall<'p> {
        model: &'p str,
        prompt: &'p str,
        settings: &'p B2C1OllamaSettings,
        observation: Rc<RefCell<Observation>>,
        reply: String,
        done: bool,
        polls: Arc<AtomicUsize>,
    }

    /// The test transport step: it counts its own entries, then hands its future the references the
    /// shared path passed, so every poll reads the borrowed values again.
    struct TestCheckedCall {
        observation: Rc<RefCell<Observation>>,
        reply: String,
        done: bool,
        polls: Arc<AtomicUsize>,
    }

    impl<'p> CheckedCall<'p> for TestCheckedCall {
        type Call = RecordingCall<'p>;

        fn call(
            self,
            model: &'p str,
            prompt: &'p str,
            settings: &'p B2C1OllamaSettings,
        ) -> Self::Call {
            // One entry into the transport step: not a poll and not a wake.
            self.observation.borrow_mut().calls += 1;
            RecordingCall {
                model,
                prompt,
                settings,
                observation: self.observation,
                reply: self.reply,
                done: self.done,
                polls: self.polls,
            }
        }
    }

    impl Future for RecordingCall<'_> {
        type Output = std::result::Result<OllamaCallResponse, OllamaCallFailure>;

        fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            let this = self.get_mut();
            let polls = this.polls.fetch_add(1, Ordering::SeqCst) + 1;
            // Read the borrowed request values *during this poll*. The slot borrow ends before the
            // poll returns, so no `Ref`/`RefMut` is held across `Pending`.
            let reading = PhaseReading {
                model: this.model.to_string(),
                prompt: this.prompt.to_string(),
                settings: this.settings.clone(),
            };
            {
                let mut slot = this.observation.borrow_mut();
                if polls == 1 {
                    slot.first_poll = Some(reading);
                } else {
                    slot.resumed_poll = Some(reading);
                }
            }
            if polls == 1 {
                cx.waker().wake_by_ref();
                return Poll::Pending;
            }
            Poll::Ready(Ok(decoded(&this.reply, this.done, true)))
        }
    }

    /// A controlled transport step that answers **immediately** with a preset result, so the
    /// production shared path can be driven over a synthesised success or failure. Nothing here
    /// sends a request: the result is assembled in memory (the decoder is the production one).
    struct PresetCheckedCall {
        result: Option<std::result::Result<OllamaCallResponse, OllamaCallFailure>>,
        calls: Arc<AtomicUsize>,
    }

    impl<'p> CheckedCall<'p> for PresetCheckedCall {
        type Call = std::future::Ready<std::result::Result<OllamaCallResponse, OllamaCallFailure>>;

        fn call(
            self,
            _model: &'p str,
            _prompt: &'p str,
            _settings: &'p B2C1OllamaSettings,
        ) -> Self::Call {
            self.calls.fetch_add(1, Ordering::SeqCst);
            std::future::ready(self.result.expect("the preset result is handed over once"))
        }
    }

    /// One controlled request: local model/prompt/options plus the shared observation slots.
    struct ControlledRequest {
        model: String,
        prompt: String,
        opts: Option<LlmGenerateOpts>,
        reply: String,
        done: bool,
        observation: Rc<RefCell<Observation>>,
        polls: Arc<AtomicUsize>,
    }

    impl ControlledRequest {
        fn new(
            model: &str,
            prompt: &str,
            opts: Option<LlmGenerateOpts>,
            reply: &str,
            done: bool,
        ) -> Self {
            Self {
                model: model.to_string(),
                prompt: prompt.to_string(),
                opts,
                reply: reply.to_string(),
                done,
                observation: Rc::new(RefCell::new(Observation::default())),
                polls: Arc::new(AtomicUsize::new(0)),
            }
        }
    }

    /// Everything one controlled request recorded, as the test sees it.
    struct Recorded {
        outcome: Result<LlmGenerateOutcome>,
        seen: Observation,
        polls: usize,
    }

    /// Runs one controlled request through the **production** shared path.
    async fn run_one(request: ControlledRequest) -> Recorded {
        let outcome = checked_generate_with_opts(
            (Some(0.11), Some(0.22)),
            request.opts.as_ref(),
            request.model.as_str(),
            request.prompt.as_str(),
            TestCheckedCall {
                observation: Rc::clone(&request.observation),
                reply: request.reply.clone(),
                done: request.done,
                polls: Arc::clone(&request.polls),
            },
        )
        .await;
        let seen = request.observation.borrow().clone();
        let polls = request.polls.load(Ordering::SeqCst);
        Recorded {
            outcome,
            seen,
            polls,
        }
    }

    /// Drives the **production** shared path once with a preset transport result (synthesised, no
    /// request is sent), and reports how many times the transport step was entered.
    async fn run_preset(
        result: std::result::Result<OllamaCallResponse, OllamaCallFailure>,
    ) -> (Result<LlmGenerateOutcome>, usize) {
        let calls = Arc::new(AtomicUsize::new(0));
        let outcome = checked_generate_with_opts(
            (Some(0.11), Some(0.22)),
            None,
            "preset-model",
            "preset-prompt",
            PresetCheckedCall {
                result: Some(result),
                calls: Arc::clone(&calls),
            },
        )
        .await;
        (outcome, calls.load(Ordering::SeqCst))
    }

    fn current_thread_runtime() -> tokio::runtime::Runtime {
        tokio::runtime::Builder::new_current_thread()
            .build()
            .expect("current-thread runtime")
    }

    /// T1: `done = true` returns its text verbatim — normal, empty and whitespace-only — through the
    /// production shared path.
    #[test]
    fn cp_b3_c1_shared_path_accepts_every_done_true_text() {
        let runtime = current_thread_runtime();
        let cases = [("\"正文\"", "正文"), ("\"\"", ""), ("\"   \"", "   ")];
        for (wire, expected) in cases {
            let request = ControlledRequest::new("m", "p", None, wire, true);
            let recorded = runtime.block_on(run_one(request));
            assert_eq!(recorded.polls, 2, "one Pending poll then the Ready poll");
            assert_eq!(recorded.seen.calls, 1, "the transport step is entered once");
            assert_eq!(
                recorded.outcome.expect("done=true must succeed").reply,
                expected
            );
        }
    }

    /// T1: `done = false` is not a normal result on the checked path, with or without partial text,
    /// and the Base projection of the same response reports the same failure.
    #[test]
    fn cp_b3_c1_checked_path_refuses_done_false_and_base_agrees() {
        let runtime = current_thread_runtime();
        for wire in ["\"部分文本\"", "\"\""] {
            let request = ControlledRequest::new("m", "p", None, wire, false);
            let recorded = runtime.block_on(run_one(request));
            let err = recorded
                .outcome
                .expect_err("done=false must not be a normal result");
            assert!(
                err.to_string().contains("done is false"),
                "the reason stays the provider's completion fact: {err}"
            );

            let base = project_ollama_call(Ok(decoded(wire, false, true)));
            let base_err = base.expect_err("the Base view must refuse done=false too");
            assert_eq!(base_err.kind, BaseCallErrorKind::Failed);
        }
    }

    /// T1: the legacy projection of the very same body still succeeds — the compatibility boundary
    /// this slice does not change.
    #[test]
    fn cp_b3_c1_legacy_projection_still_accepts_done_false() {
        let legacy = decoded("\"部分文本\"", false, true).into_legacy_result();
        assert_eq!(legacy.response, "部分文本");
    }

    /// T3: metrics come from the same response **through the production shared path**, and
    /// `want_metrics` still decides which of them exist.
    ///
    /// The six-field collection and both switch positions are covered by the existing regression
    /// `b2_c1_metrics_cover_all_six_fields_and_both_switches`, which this slice neither changes nor
    /// duplicates; here only the two outcome-level states are read, after the real helper awaited the
    /// transport step.
    #[test]
    fn cp_b3_c1_shared_path_carries_metrics_from_the_same_response() {
        let runtime = current_thread_runtime();
        let body = r#"{"response":"x","model":"m","created_at":"2024-01-01T00:00:00Z","done":true,
                       "prompt_eval_duration":2000000,"prompt_eval_count":5,"eval_count":7}"#;

        let call = decode_non_streaming_body(reqwest::StatusCode::OK, body, "http://x", "m", true)
            .expect("decodable");
        let (outcome, calls) = runtime.block_on(run_preset(Ok(call)));
        assert_eq!(calls, 1, "the transport step is entered once");
        let outcome = outcome.expect("done=true");
        assert_eq!(outcome.prompt_eval_ms, Some(2), "2_000_000 ns is 2 ms");
        assert_eq!(outcome.reply, "x");

        let call = decode_non_streaming_body(reqwest::StatusCode::OK, body, "http://x", "m", false)
            .expect("decodable");
        let (outcome, calls) = runtime.block_on(run_preset(Ok(call)));
        assert_eq!(calls, 1);
        assert_eq!(
            outcome.expect("done=true").prompt_eval_ms,
            None,
            "the metrics request is not a wire field: it decides what is collected locally"
        );
    }

    /// T3-host: a **synthesised** transport failure travels the production shared path and keeps the
    /// original `AppError` — the recorded timeout fact changes the Base view, not this one, and a
    /// misleading message never rewrites the variant or its text.
    #[test]
    fn cp_b3_c1_shared_path_keeps_the_original_failure() {
        let runtime = current_thread_runtime();

        let original = "Request failed: connection closed before message completed";
        let (outcome, calls) = runtime.block_on(run_preset(Err(failure(original, false))));
        assert_eq!(calls, 1);
        match outcome.expect_err("a failure must stay a failure") {
            AppError::OllamaError(message) => assert_eq!(message, original),
            other => panic!("the original variant must be kept: {other:?}"),
        }

        let misleading = "timeout wording without the recorded fact";
        let (outcome, _) = runtime.block_on(run_preset(Err(failure(misleading, false))));
        match outcome.expect_err("still a failure") {
            AppError::OllamaError(message) => assert_eq!(
                message, misleading,
                "the shared path does not reword the provider-side message"
            ),
            other => panic!("the original variant must be kept: {other:?}"),
        }

        // A transport that *did* know the call timed out changes the Base kind, not this variant: the
        // Host view carries the same original `AppError` either way.
        let (outcome, _) = runtime.block_on(run_preset(Err(failure(original, true))));
        match outcome.expect_err("a known timeout is still a failure here") {
            AppError::OllamaError(message) => assert_eq!(message, original),
            other => panic!("the original variant must be kept: {other:?}"),
        }
    }

    /// T3-base: the Base projection maps the transport's recorded fact to `TimedOut`/`Failed` and
    /// carries the original diagnostic text. This is the Base view only — the Host path above keeps
    /// `AppError` and never produces a `BaseCallError`.
    #[test]
    fn cp_b3_c1_base_projection_maps_the_recorded_timeout_fact() {
        let timeout_looking = "not a timeout, honestly unrelated wording";
        let base = project_ollama_call(Err(failure(timeout_looking, true)))
            .expect_err("a recorded timeout must stay a failure");
        assert_eq!(
            base.kind,
            BaseCallErrorKind::TimedOut,
            "the recorded fact wins over the wording"
        );
        assert_eq!(
            base.detail.as_deref(),
            Some(format!("Ollama error: {timeout_looking}").as_str())
        );

        let not_a_timeout = project_ollama_call(Err(failure(
            "connection closed before message completed (timeout wording, no fact)",
            false,
        )))
        .expect_err("still a failure");
        assert_eq!(not_a_timeout.kind, BaseCallErrorKind::Failed);
    }

    /// T1: a missing `done` member and an empty body stay decode failures through the shared path.
    #[test]
    fn cp_b3_c1_decode_failures_are_not_empty_successes() {
        for body in [
            r#"{"response":"x","model":"m","created_at":"2024-01-01T00:00:00Z"}"#,
            "",
        ] {
            let decoded =
                decode_non_streaming_body(reqwest::StatusCode::OK, body, "http://x", "m", true);
            assert!(
                decoded.is_err(),
                "body {body:?} must not decode into a normal result"
            );
        }
    }

    /// T2: the production shared path hands the **actual** model, prompt and merged settings to the
    /// transport, keeps `None` / `Some(default)` / a single override apart, and carries the other
    /// option fields verbatim.
    #[test]
    fn cp_b3_c1_shared_path_hands_model_prompt_and_settings_through() {
        let runtime = current_thread_runtime();

        let results: Vec<Recorded> = runtime.block_on(async {
            let mut out = Vec::new();
            out.push(
                run_one(ControlledRequest::new(
                    "model-none",
                    "prompt-none",
                    None,
                    "\"a\"",
                    true,
                ))
                .await,
            );
            out.push(
                run_one(ControlledRequest::new(
                    "model-default",
                    "prompt-default",
                    Some(LlmGenerateOpts::default()),
                    "\"b\"",
                    true,
                ))
                .await,
            );
            out.push(
                run_one(ControlledRequest::new(
                    "model-override",
                    "prompt-override",
                    Some(LlmGenerateOpts {
                        temperature: Some(0.2),
                        ..LlmGenerateOpts::default()
                    }),
                    "\"c\"",
                    true,
                ))
                .await,
            );
            out.push(
                run_one(ControlledRequest::new(
                    "model-capped",
                    "prompt-capped",
                    Some(LlmGenerateOpts {
                        keep_alive: Some("5m".to_string()),
                        want_metrics: true,
                        max_output_tokens: Some(64),
                        preferred_context_tokens: Some(2048),
                        ..LlmGenerateOpts::default()
                    }),
                    "\"d\"",
                    true,
                ))
                .await,
            );
            out
        });

        let settings = |index: usize| {
            results[index]
                .seen
                .resumed_poll
                .clone()
                .expect("the transport read its settings on the resumed poll")
                .settings
        };
        assert_eq!(results[0].seen.calls, 1);
        assert_eq!(
            results[0].seen.first_poll.as_ref().expect("read").model,
            "model-none"
        );
        assert_eq!(
            results[0].seen.first_poll.as_ref().expect("read").prompt,
            "prompt-none"
        );
        assert_eq!(
            results[3].seen.resumed_poll.as_ref().expect("read").model,
            "model-capped"
        );
        assert_eq!(
            results[3].seen.resumed_poll.as_ref().expect("read").prompt,
            "prompt-capped"
        );
        assert_ne!(
            results[0].seen.first_poll.as_ref().expect("read").model,
            results[3].seen.first_poll.as_ref().expect("read").model
        );

        // A bound `None` stays `None` while the sampling pair is still resolved for the request.
        assert!(settings(0).ollama_opts.is_none(), "a bound None stays None");
        assert_eq!(settings(0).temperature, Some(0.11));
        assert_eq!(settings(0).top_p, Some(0.22));
        // `Some(default)` is a distinct bound value with the same resolved sampling.
        assert!(settings(1).ollama_opts.is_some());
        assert_eq!(settings(1).temperature, Some(0.11));
        // A single explicit override wins for its own field only.
        assert_eq!(settings(2).temperature, Some(0.2));
        assert_eq!(settings(2).top_p, Some(0.22));
        // The other option fields travel verbatim.
        let capped = settings(3).ollama_opts.clone().expect("carried options");
        assert_eq!(capped.keep_alive.as_deref(), Some("5m"));
        assert!(capped.want_metrics);
        assert_eq!(capped.max_output_tokens, Some(64));
        assert_eq!(capped.preferred_context_tokens, Some(2048));

        for recorded in results {
            assert_eq!(recorded.polls, 2, "one Pending poll then the Ready poll");
            assert_eq!(recorded.seen.calls, 1, "entered once per request");
            assert!(recorded.outcome.is_ok(), "each request completed normally");
        }
    }

    /// T4: two requests with different local model, prompt and options stay isolated while their
    /// polls interleave. Each transport step is entered once and polled twice, and both polls of one
    /// request read that request's **own borrowed** parameters — the future holds the references, so
    /// the loopback-style "did the resumed poll still see its own values" question is answered by a
    /// reading taken during each poll rather than by a snapshot cloned before the wait.
    #[test]
    fn cp_b3_c1_two_pending_requests_keep_their_own_parameters() {
        let runtime = current_thread_runtime();
        runtime.block_on(async {
            let obs_a = Rc::new(RefCell::new(Observation::default()));
            let obs_b = Rc::new(RefCell::new(Observation::default()));
            let polls_a = Arc::new(AtomicUsize::new(0));
            let polls_b = Arc::new(AtomicUsize::new(0));

            let model_a = String::from("model-A");
            let prompt_a = String::from("prompt-A");
            let model_b = String::from("model-B");
            let prompt_b = String::from("prompt-B");
            let opts_a: Option<LlmGenerateOpts> = None;
            let opts_b = Some(LlmGenerateOpts {
                max_output_tokens: Some(128),
                ..LlmGenerateOpts::default()
            });

            let future_a = {
                checked_generate_with_opts(
                    (Some(0.11), Some(0.22)),
                    opts_a.as_ref(),
                    model_a.as_str(),
                    prompt_a.as_str(),
                    TestCheckedCall {
                        observation: Rc::clone(&obs_a),
                        reply: "\"A\"".to_string(),
                        done: true,
                        polls: Arc::clone(&polls_a),
                    },
                )
            };
            let future_b = {
                checked_generate_with_opts(
                    (Some(0.11), Some(0.22)),
                    opts_b.as_ref(),
                    model_b.as_str(),
                    prompt_b.as_str(),
                    TestCheckedCall {
                        observation: Rc::clone(&obs_b),
                        reply: "\"B\"".to_string(),
                        done: true,
                        polls: Arc::clone(&polls_b),
                    },
                )
            };

            let counter = Arc::new(WakeCounter(AtomicUsize::new(0)));
            let waker = Waker::from(Arc::clone(&counter));
            let mut cx = Context::from_waker(&waker);
            let mut a = std::pin::pin!(future_a);
            let mut b = std::pin::pin!(future_b);

            // A Pending -> B Pending -> B Ready -> A Ready.
            assert!(matches!(a.as_mut().poll(&mut cx), Poll::Pending));
            assert!(matches!(b.as_mut().poll(&mut cx), Poll::Pending));
            let b_out = match b.as_mut().poll(&mut cx) {
                Poll::Ready(out) => out,
                Poll::Pending => panic!("B must answer on its second poll"),
            };
            let a_out = match a.as_mut().poll(&mut cx) {
                Poll::Ready(out) => out,
                Poll::Pending => panic!("A must answer on its second poll"),
            };
            assert_eq!(b_out.expect("B ok").reply, "B");
            assert_eq!(a_out.expect("A ok").reply, "A");
            assert!(
                counter.0.load(Ordering::SeqCst) >= 2,
                "each Pending poll scheduled a wake"
            );
            assert_eq!(
                polls_a.load(Ordering::SeqCst),
                2,
                "A's transport step: Pending then Ready"
            );
            assert_eq!(polls_b.load(Ordering::SeqCst), 2);
            assert_eq!(obs_a.borrow().calls, 1, "A entered the transport step once");
            assert_eq!(obs_b.borrow().calls, 1, "B entered the transport step once");

            // The local inputs are still usable here: both futures borrowed them, and the values the
            // transport read during each poll are what this test compares — not the request variables.
            assert_eq!(
                (model_a.as_str(), prompt_a.as_str()),
                ("model-A", "prompt-A")
            );
            assert_eq!(
                (model_b.as_str(), prompt_b.as_str()),
                ("model-B", "prompt-B")
            );

            let seen_a = obs_a.borrow().clone();
            let seen_b = obs_b.borrow().clone();
            let first_a = seen_a.first_poll.clone().expect("A's first poll read");
            let resumed_a = seen_a.resumed_poll.clone().expect("A's resumed poll read");
            let first_b = seen_b.first_poll.clone().expect("B's first poll read");
            let resumed_b = seen_b.resumed_poll.clone().expect("B's resumed poll read");

            // Both readings of one request come from that request's own borrowed parameters, and the
            // resumed reading happens after the other request has already been polled to `Ready`.
            for reading in [&first_a, &resumed_a] {
                assert_eq!(reading.model, "model-A");
                assert_eq!(reading.prompt, "prompt-A");
                assert_eq!(reading.settings.temperature, Some(0.11));
                assert_eq!(reading.settings.top_p, Some(0.22));
                assert!(
                    reading.settings.ollama_opts.is_none(),
                    "A bound None stays None"
                );
            }
            for reading in [&first_b, &resumed_b] {
                assert_eq!(reading.model, "model-B");
                assert_eq!(reading.prompt, "prompt-B");
                assert_eq!(reading.settings.temperature, Some(0.11));
                assert_eq!(
                    reading
                        .settings
                        .ollama_opts
                        .as_ref()
                        .and_then(|opts| opts.max_output_tokens),
                    Some(128),
                    "B's own option cap reached the transport"
                );
            }
            assert_ne!(first_a.model, first_b.model);
            assert_ne!(resumed_a.model, resumed_b.model);
        });
    }

    /// T5: the binding type is `Send + Sync`, and both the production shared path's future and the
    /// `LlmClient` method's own future are `Send` — that is what makes the `async_trait` method
    /// usable from a `Send` boundary without `unsafe`, a new executor or a `block_on`. Neither future
    /// is polled here, so no request is sent.
    #[test]
    fn cp_b3_c1_host_binding_satisfies_the_send_boundary() {
        fn require_send_sync<T: Send + Sync>() {}
        require_send_sync::<LlmWithCheckedCompletion>();

        let client = Arc::new(OllamaClient::new("http://127.0.0.1:1"));
        let model = String::from("m");
        let prompt = String::from("p");
        let future = checked_generate_with_opts(
            (Some(0.7), Some(0.9)),
            None,
            model.as_str(),
            prompt.as_str(),
            ProductionCheckedCall {
                client: Arc::clone(&client),
            },
        );
        fn require_send<F: Future + Send>(_: &F) {}
        require_send(&future);

        // The trait method's future, as the Host would await it: constructed, never polled.
        let binding = LlmWithCheckedCompletion::new(client);
        let trait_future =
            LlmClient::generate_with_opts(&binding, model.as_str(), prompt.as_str(), None);
        require_send(&trait_future);
    }
}
