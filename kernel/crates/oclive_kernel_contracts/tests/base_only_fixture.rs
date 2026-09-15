//! Base-only fixture: pure in-memory implementations and the B1 acceptance matrix.
//!
//! No Host, no `Role` model, no Extension install, no real model, no executor, no
//! network, no database and no real external effect. Futures are driven by a
//! fixture-local loop that observes the future's own wakeups, and every returned memory
//! item is drawn from the material of that same call. Nothing here is a production
//! driver.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::cell::{Cell, RefCell};
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::task::{Context, Poll, Waker};

use oclive_kernel_contracts::{
    AgentBase, BaseCallFuture, EmotionBase, EventBase, LlmBase, MemoryBase, PromptBase,
};
use oclive_kernel_types::{
    AgentBaseRequest, BaseCallError, BaseCallErrorKind, EmotionBaseRequest, EventBaseRequest,
    LlmBaseRequest, MemoryBaseRequest, PromptBaseRequest,
};

// ---------------------------------------------------------------- driving helpers

/// Test-local, observable waker: counts how often the future wakes its task.
#[derive(Default)]
struct WakeCounter {
    wakes: AtomicUsize,
}

impl std::task::Wake for WakeCounter {
    fn wake(self: Arc<Self>) {
        self.wakes.fetch_add(1, Ordering::SeqCst);
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.wakes.fetch_add(1, Ordering::SeqCst);
    }
}

/// Polls `future` to completion with an observable waker and reports
/// `(polls, wake_count, output)`.
///
/// After every `Pending` this loop asserts that the future actually scheduled a wake
/// through the `Context` it was given. A fixture that ignored its `Context` would fail
/// here rather than pass through blind re-polling. No executor, timer or sleep is used.
fn drive<F: Future>(future: F) -> (usize, usize, F::Output) {
    let mut future = Box::pin(future);
    let wake_counter = Arc::new(WakeCounter::default());
    let waker = Waker::from(Arc::clone(&wake_counter));
    let mut cx = Context::from_waker(&waker);
    let mut polls = 0usize;
    loop {
        polls += 1;
        assert!(
            polls < 16,
            "fixture future must not require unbounded polling"
        );
        match future.as_mut().poll(&mut cx) {
            Poll::Ready(out) => {
                return (polls, wake_counter.wakes.load(Ordering::SeqCst), out);
            }
            Poll::Pending => {
                assert!(
                    wake_counter.wakes.load(Ordering::SeqCst) >= polls,
                    "future returned Pending without waking the task"
                );
            }
        }
    }
}

/// A future that reports `Pending` once — scheduling its continuation through the
/// caller's waker — and then yields a clone of its payload. Its state lives in
/// `Rc<Cell<..>>`, which is deliberately not `Send`/`Sync`.
struct PendingOnce<T> {
    polls: Rc<Cell<u32>>,
    payload: T,
}

impl<T: Clone> Future for PendingOnce<T> {
    type Output = Result<T, BaseCallError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let seen = self.polls.get();
        self.polls.set(seen + 1);
        if seen == 0 {
            // The continuation is available on the next poll: request it through this
            // task's waker instead of depending on the driver to poll again blindly.
            cx.waker().wake_by_ref();
            Poll::Pending
        } else {
            Poll::Ready(Ok(self.payload.clone()))
        }
    }
}

// ------------------------------------------------------------------ Memory fixtures

/// The one selection rule these fixtures use: pick this call's explicit material that
/// contains the (trimmed, non-empty) query keyword. No ranking, summary or model.
fn select_material(materials: &[&str], query: &str) -> Vec<String> {
    let keyword = query.trim();
    if keyword.is_empty() {
        return Vec::new();
    }
    materials
        .iter()
        .filter(|item| item.contains(keyword))
        .map(|item| (*item).to_string())
        .collect()
}

/// Selects material containing the query keyword; an empty query selects nothing.
struct KeywordMemory;

impl MemoryBase for KeywordMemory {
    fn retrieve<'a>(&'a self, request: MemoryBaseRequest<'a>) -> BaseCallFuture<'a, Vec<String>> {
        Box::pin(async move { Ok(select_material(request.materials, request.query)) })
    }
}

/// State lives in `Rc<RefCell<..>>`; the trait declares no `Send`/`Sync` bound, so such an
/// implementation is usable through `&dyn MemoryBase` on a single thread. Its result is
/// still drawn from this call's material — it merely applies the same selection rule in
/// the opposite order, which keeps the two implementations distinguishable.
struct LocalStateMemory {
    seen_queries: Rc<RefCell<Vec<String>>>,
}

impl MemoryBase for LocalStateMemory {
    fn retrieve<'a>(&'a self, request: MemoryBaseRequest<'a>) -> BaseCallFuture<'a, Vec<String>> {
        let seen_queries = Rc::clone(&self.seen_queries);
        Box::pin(async move {
            seen_queries.borrow_mut().push(request.query.to_string());
            let mut hits = select_material(request.materials, request.query);
            hits.reverse();
            Ok(hits)
        })
    }
}

/// Thread-safe implementation with atomically shared state. The call counter is internal
/// state for a separate assertion; the returned content comes from this call's material.
struct AtomicMemory {
    calls: Arc<AtomicUsize>,
}

impl MemoryBase for AtomicMemory {
    fn retrieve<'a>(&'a self, request: MemoryBaseRequest<'a>) -> BaseCallFuture<'a, Vec<String>> {
        let calls = Arc::clone(&self.calls);
        Box::pin(async move {
            calls.fetch_add(1, Ordering::SeqCst);
            Ok(select_material(request.materials, request.query))
        })
    }
}

/// Completes on the second poll, keeping its material-derived payload across `Pending`.
struct YieldingMemory {
    polls: Rc<Cell<u32>>,
}

impl MemoryBase for YieldingMemory {
    fn retrieve<'a>(&'a self, request: MemoryBaseRequest<'a>) -> BaseCallFuture<'a, Vec<String>> {
        Box::pin(PendingOnce {
            polls: Rc::clone(&self.polls),
            payload: select_material(request.materials, request.query),
        })
    }
}

/// A future that stays `Pending` once and, both before and after that `Pending`, uses
/// state and material that it only borrows — no owned copy and no `'static` requirement.
struct BorrowingPending<'a, 'b> {
    polls: &'a Cell<u32>,
    materials: &'b [&'b str],
    query: &'b str,
    first: bool,
}

impl Future for BorrowingPending<'_, '_> {
    type Output = Result<Vec<String>, BaseCallError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        this.polls.set(this.polls.get() + 1);
        if this.first {
            this.first = false;
            cx.waker().wake_by_ref();
            return Poll::Pending;
        }
        // Uses the borrowed material and the borrowed counter after the Pending.
        Poll::Ready(Ok(select_material(this.materials, this.query)))
    }
}

/// Implementation whose own state is a borrow of local data (`&Cell<u32>`), not an owned
/// handle: it can only exist while that local lives, which is what proves the trait does
/// not require `'static`.
struct BorrowingMemory<'s> {
    calls: &'s Cell<u32>,
}

impl<'s> MemoryBase for BorrowingMemory<'s> {
    fn retrieve<'a>(&'a self, request: MemoryBaseRequest<'a>) -> BaseCallFuture<'a, Vec<String>> {
        Box::pin(BorrowingPending {
            polls: self.calls,
            materials: request.materials,
            query: request.query,
            first: true,
        })
    }
}

// ----------------------------------------------------------------- Emotion fixtures

/// Reports a readable analysis for marked material, `None` otherwise.
struct MarkerEmotion;

impl EmotionBase for MarkerEmotion {
    fn analyze<'a>(
        &'a self,
        request: EmotionBaseRequest<'a>,
    ) -> BaseCallFuture<'a, Option<String>> {
        Box::pin(async move {
            if request.material.contains("shaking") {
                Ok(Some("tense, worried".to_string()))
            } else {
                Ok(None)
            }
        })
    }
}

// ------------------------------------------------------------------- Event fixtures

/// Analyses described-event material; conditions stay conditions.
struct ConditionalEvent;

impl EventBase for ConditionalEvent {
    fn analyze<'a>(&'a self, request: EventBaseRequest<'a>) -> BaseCallFuture<'a, Option<String>> {
        Box::pin(async move {
            if request.material.contains("if ") {
                Ok(Some(
                    "material states a condition or plan, not an accomplished event".to_string(),
                ))
            } else if request.material.trim().is_empty() {
                Ok(None)
            } else {
                Ok(Some("described change with no explicit impact".to_string()))
            }
        })
    }
}

// ----------------------------------------------------------------- Prompt fixtures

/// Organises material into its own layout and appends the stated requirements.
struct OrganizingPrompt;

impl PromptBase for OrganizingPrompt {
    fn assemble<'a>(&'a self, request: PromptBaseRequest<'a>) -> BaseCallFuture<'a, String> {
        Box::pin(async move {
            let mut out = String::new();
            for item in request.materials {
                out.push_str(item);
                out.push('\n');
            }
            out.push_str(request.requirements);
            Ok(out)
        })
    }
}

/// Verbatim assembly: when this call's agreement requires the already-prepared input to
/// be preserved as-is, the output is that input, byte for byte — no added newline,
/// heading, requirement text or other content.
struct VerbatimPrompt;

impl PromptBase for VerbatimPrompt {
    fn assemble<'a>(&'a self, request: PromptBaseRequest<'a>) -> BaseCallFuture<'a, String> {
        Box::pin(async move { Ok(request.materials.concat()) })
    }
}

// --------------------------------------------------------------------- LLM fixture

/// Echoes the prepared input; empty input is a normal empty result.
struct EchoLlm;

impl LlmBase for EchoLlm {
    fn generate<'a>(&'a self, request: LlmBaseRequest<'a>) -> BaseCallFuture<'a, String> {
        Box::pin(async move { Ok(request.input.to_string()) })
    }
}

// ------------------------------------------------------------------- Agent fixtures

/// Declines the task; it reports that it did not take it up, without claiming success.
struct DecliningAgent;

impl AgentBase for DecliningAgent {
    fn execute<'a>(&'a self, _request: AgentBaseRequest<'a>) -> BaseCallFuture<'a, String> {
        Box::pin(async move { Ok("not taken up: no tool is bound for this task".to_string()) })
    }
}

/// Takes the task and reports that no action was needed because the goal already holds.
struct AlreadySatisfiedAgent;

impl AgentBase for AlreadySatisfiedAgent {
    fn execute<'a>(&'a self, _request: AgentBaseRequest<'a>) -> BaseCallFuture<'a, String> {
        Box::pin(async move {
            Ok("no action needed: the requested condition already holds".to_string())
        })
    }
}

// ------------------------------------------------------------------ error fixtures

/// Fails every call with one machine-readable reason.
struct FailingMemory {
    kind: BaseCallErrorKind,
}

impl MemoryBase for FailingMemory {
    fn retrieve<'a>(&'a self, _request: MemoryBaseRequest<'a>) -> BaseCallFuture<'a, Vec<String>> {
        let kind = self.kind;
        Box::pin(async move {
            Err(BaseCallError {
                kind,
                detail: Some("fixture-reported reason".to_string()),
            })
        })
    }
}

fn all_kinds() -> [BaseCallErrorKind; 5] {
    [
        BaseCallErrorKind::Failed,
        BaseCallErrorKind::Unavailable,
        BaseCallErrorKind::Unsupported,
        BaseCallErrorKind::Cancelled,
        BaseCallErrorKind::TimedOut,
    ]
}

/// Conservative handling of reasons added after this fixture was written.
fn classify_known(kind: BaseCallErrorKind) -> &'static str {
    match kind {
        BaseCallErrorKind::Failed => "failed",
        BaseCallErrorKind::Unavailable => "unavailable",
        BaseCallErrorKind::Unsupported => "unsupported",
        BaseCallErrorKind::Cancelled => "cancelled",
        BaseCallErrorKind::TimedOut => "timed out",
        _ => "did not complete normally",
    }
}

// ------------------------------------------------------------------ extension demo

/// Test-local extension: never exported, never required by a Base-only implementation.
trait MemoryScoredExtension {
    fn scored(&self) -> u32;
}

struct ScoredMemory;

impl MemoryBase for ScoredMemory {
    fn retrieve<'a>(&'a self, request: MemoryBaseRequest<'a>) -> BaseCallFuture<'a, Vec<String>> {
        Box::pin(async move { Ok(select_material(request.materials, request.query)) })
    }
}

impl MemoryScoredExtension for ScoredMemory {
    fn scored(&self) -> u32 {
        7
    }
}

// ------------------------------------------------------------------ acceptance tests

#[test]
fn six_slots_are_independently_callable_without_host_or_extension() {
    let materials = [
        "she said she may come back later",
        "she asked about the weather",
    ];

    let (memory_polls, memory_wakes, memory) = drive(KeywordMemory.retrieve(MemoryBaseRequest {
        materials: &materials,
        query: "weather",
    }));
    assert_eq!(memory_polls, 1);
    assert_eq!(memory_wakes, 0, "a ready future needs no wake");
    let memory = memory.expect("memory call");
    assert_eq!(memory, vec!["she asked about the weather".to_string()]);

    let (_, _, emotion) = drive(MarkerEmotion.analyze(EmotionBaseRequest {
        material: "her hands are shaking",
        context: None,
    }));
    assert_eq!(
        emotion.expect("emotion call").as_deref(),
        Some("tense, worried")
    );

    let (_, _, event) = drive(ConditionalEvent.analyze(EventBaseRequest {
        material: "if the disk fills, writes will fail",
        context: Some("current conversation"),
    }));
    assert!(event
        .expect("event call")
        .expect("analysis")
        .contains("condition"));

    let (_, _, prompt) = drive(OrganizingPrompt.assemble(PromptBaseRequest {
        materials: &materials,
        requirements: "reply as her",
    }));
    let prompt = prompt.expect("prompt call");
    assert!(prompt.contains("weather"));
    assert!(prompt.ends_with("reply as her"));

    let (_, _, llm) = drive(EchoLlm.generate(LlmBaseRequest { input: "hello" }));
    assert_eq!(llm.expect("llm call"), "hello");

    let (_, _, agent) = drive(DecliningAgent.execute(AgentBaseRequest {
        task: "book a table",
        context: None,
    }));
    let agent = agent.expect("agent call");
    assert!(!agent.is_empty());

    // Independent results are meaningful, not a uniform empty value.
    assert!(!memory.is_empty() && !prompt.is_empty() && !agent.is_empty());
}

#[test]
fn slots_are_selectable_by_the_caller_and_usable_through_dyn() {
    let fast = KeywordMemory;
    let local = LocalStateMemory {
        seen_queries: Rc::new(RefCell::new(Vec::new())),
    };
    let slots: [&dyn MemoryBase; 2] = [&fast, &local];
    let materials = ["alpha first material", "alpha second material"];

    let (_, _, first) = drive(slots[0].retrieve(MemoryBaseRequest {
        materials: &materials,
        query: "alpha",
    }));
    let (_, _, second) = drive(slots[1].retrieve(MemoryBaseRequest {
        materials: &materials,
        query: "alpha",
    }));
    // Both results are drawn from this call's material; the second implementation applies
    // the same simple rule in the opposite order, so the caller can tell them apart.
    assert_eq!(
        first.expect("first slot"),
        vec![
            "alpha first material".to_string(),
            "alpha second material".to_string()
        ]
    );
    assert_eq!(
        second.expect("second slot"),
        vec![
            "alpha second material".to_string(),
            "alpha first material".to_string()
        ]
    );
    assert_eq!(local.seen_queries.borrow().len(), 1);
}

#[test]
fn memory_selects_relevant_material_and_preserves_negation_and_subject() {
    let materials = [
        "she said: I did not promise to come back",
        "he said: I promised to come back",
        "unrelated note about the weather",
    ];
    let (_, _, hits) = drive(KeywordMemory.retrieve(MemoryBaseRequest {
        materials: &materials,
        query: "promise",
    }));
    let hits = hits.expect("memory call");
    assert_eq!(hits.len(), 2);
    // Negation and speaker attribution are preserved verbatim, not resolved.
    assert!(hits.iter().any(|item| item.contains("did not promise")));
    assert!(hits.iter().any(|item| item.starts_with("she said")));
}

#[test]
fn memory_reports_no_applicable_content_without_claiming_absence_of_memories() {
    let materials = ["alpha", "beta"];
    let (_, _, none) = drive(KeywordMemory.retrieve(MemoryBaseRequest {
        materials: &materials,
        query: "   ",
    }));
    assert!(none.expect("memory call").is_empty());

    let (_, _, missing) = drive(KeywordMemory.retrieve(MemoryBaseRequest {
        materials: &materials,
        query: "gamma",
    }));
    // An empty selection is a normal result, not an error and not proof about any store.
    assert!(missing.expect("memory call").is_empty());
}

#[test]
fn emotion_and_event_separate_no_analysis_from_analysis() {
    let (_, _, analysed) = drive(MarkerEmotion.analyze(EmotionBaseRequest {
        material: "shaking hands before the meeting",
        context: None,
    }));
    assert!(analysed.expect("emotion call").is_some());

    let (_, _, not_analysed) = drive(MarkerEmotion.analyze(EmotionBaseRequest {
        material: "the meeting starts at nine",
        context: None,
    }));
    assert!(not_analysed.expect("emotion call").is_none());

    // Event works with no Emotion result at all; no slot dependency is implied.
    let (_, _, event) = drive(ConditionalEvent.analyze(EventBaseRequest {
        material: "a quiet afternoon",
        context: None,
    }));
    assert!(event.expect("event call").is_some());

    let (_, _, empty_event) = drive(ConditionalEvent.analyze(EventBaseRequest {
        material: "   ",
        context: None,
    }));
    assert!(empty_event.expect("event call").is_none());
}

#[test]
fn prompt_supports_verbatim_passthrough_and_organising() {
    // Verbatim: this call's agreement requires the already-prepared input to be preserved
    // exactly, so the output is that input byte for byte.
    let prepared = "system: stay in character\nuser: where did we stop?";
    let (_, _, verbatim) = drive(VerbatimPrompt.assemble(PromptBaseRequest {
        materials: &[prepared],
        requirements: "preserve the prepared input verbatim and add nothing",
    }));
    let verbatim = verbatim.expect("verbatim assembly");
    assert_eq!(
        verbatim, prepared,
        "passthrough must not add a newline, heading or requirement text"
    );

    // Organising: material plus the stated requirements, in the implementation's layout.
    let materials = ["reference material only"];
    let (_, _, assembled) = drive(OrganizingPrompt.assemble(PromptBaseRequest {
        materials: &materials,
        requirements: "keep it short",
    }));
    let assembled = assembled.expect("prompt call");
    assert!(assembled.starts_with("reference material only"));
    assert!(assembled.ends_with("keep it short"));
}

#[test]
fn llm_consumes_caller_prepared_text_directly() {
    // LLM consumes text the caller prepared; a Prompt slot is not required.
    let (_, _, generated) = drive(EchoLlm.generate(LlmBaseRequest {
        input: "caller-prepared input",
    }));
    assert_eq!(generated.expect("llm call"), "caller-prepared input");

    // Empty text from a normally completed generation is a valid result, not a failure.
    let (_, _, empty) = drive(EchoLlm.generate(LlmBaseRequest { input: "" }));
    assert_eq!(empty.expect("llm call"), "");
}

#[test]
fn agent_reports_decline_and_no_action_as_different_facts() {
    let (_, _, declined) = drive(DecliningAgent.execute(AgentBaseRequest {
        task: "run the deployment",
        context: None,
    }));
    let (_, _, satisfied) = drive(AlreadySatisfiedAgent.execute(AgentBaseRequest {
        task: "make sure the file is absent",
        context: None,
    }));

    let declined = declined.expect("agent call");
    let satisfied = satisfied.expect("agent call");
    assert_ne!(declined, satisfied);
    assert!(declined.contains("not taken up"));
    assert!(satisfied.contains("no action needed"));
    // Neither report claims that no external effect occurred, and neither is authoritative
    // task state: the caller still decides whether the task is done.
    assert!(!declined.contains("completed"));
    assert!(!satisfied.contains("completed"));
}

#[test]
fn every_known_reason_is_machine_distinguishable() {
    for expected in all_kinds() {
        let slot = FailingMemory { kind: expected };
        let (_, _, out) = drive(slot.retrieve(MemoryBaseRequest {
            materials: &[],
            query: "anything",
        }));
        let error = out.expect_err("fixture always fails");
        assert_eq!(error.kind, expected);
        assert_eq!(error.detail.as_deref(), Some("fixture-reported reason"));
        assert_eq!(classify_known(error.kind), classify_known(expected));
    }
}

#[test]
fn known_cancellation_and_timeout_are_not_downgraded_to_plain_failure() {
    let cancelled = FailingMemory {
        kind: BaseCallErrorKind::Cancelled,
    };
    let timed_out = FailingMemory {
        kind: BaseCallErrorKind::TimedOut,
    };
    let (_, _, cancelled_out) = drive(cancelled.retrieve(MemoryBaseRequest {
        materials: &[],
        query: "q",
    }));
    let (_, _, timed_out_out) = drive(timed_out.retrieve(MemoryBaseRequest {
        materials: &[],
        query: "q",
    }));

    let cancelled_out = cancelled_out.expect_err("cancelled");
    let timed_out_out = timed_out_out.expect_err("timed out");
    assert_ne!(cancelled_out.kind, BaseCallErrorKind::Failed);
    assert_ne!(timed_out_out.kind, BaseCallErrorKind::Failed);
    assert_ne!(cancelled_out.kind, timed_out_out.kind);
    // A reason is a reason, not a terminal state or an effect verdict.
    assert!(cancelled_out.to_string().contains("cancelled"));
    assert!(timed_out_out.to_string().contains("timed out"));
}

#[test]
fn failure_reasons_do_not_replace_normal_empty_results() {
    let (_, _, empty_ok) = drive(KeywordMemory.retrieve(MemoryBaseRequest {
        materials: &["material"],
        query: "absent-keyword",
    }));
    assert!(empty_ok.expect("normal empty result").is_empty());

    let (_, _, failure) = drive(
        FailingMemory {
            kind: BaseCallErrorKind::Unavailable,
        }
        .retrieve(MemoryBaseRequest {
            materials: &["material"],
            query: "absent-keyword",
        }),
    );
    assert!(failure.is_err());
}

#[test]
fn pending_fixture_schedules_a_wake_instead_of_relying_on_busy_polling() {
    let polls = Rc::new(Cell::new(0));
    let slot = YieldingMemory {
        polls: Rc::clone(&polls),
    };
    let materials = ["material kept across pending"];
    let mut future = slot.retrieve(MemoryBaseRequest {
        materials: &materials,
        query: "kept",
    });

    let wake_counter = Arc::new(WakeCounter::default());
    let waker = Waker::from(Arc::clone(&wake_counter));
    let mut cx = Context::from_waker(&waker);

    // First poll: not ready, and the fixture must request the continuation itself.
    assert!(matches!(future.as_mut().poll(&mut cx), Poll::Pending));
    assert_eq!(
        wake_counter.wakes.load(Ordering::SeqCst),
        1,
        "a Pending result must schedule a wake through the given Context"
    );

    // Second poll: the material-derived payload arrives, still from this call's material.
    match future.as_mut().poll(&mut cx) {
        Poll::Ready(Ok(items)) => {
            assert_eq!(items, vec!["material kept across pending".to_string()])
        }
        other => panic!("expected a ready result, got {other:?}"),
    }
    assert_eq!(polls.get(), 2, "the fixture polled exactly twice");
}

#[test]
fn a_pending_future_completes_under_wake_driven_polling_and_keeps_material() {
    let polls = Rc::new(Cell::new(0));
    let slot = YieldingMemory {
        polls: Rc::clone(&polls),
    };
    let materials = ["alpha kept across pending", "beta unrelated"];
    let (polls_observed, wakes, out) = drive(slot.retrieve(MemoryBaseRequest {
        materials: &materials,
        query: "kept",
    }));
    assert_eq!(polls_observed, 2, "one Pending then Ready");
    assert!(wakes >= 1, "the Pending result was accompanied by a wake");
    assert_eq!(polls.get(), 2);
    assert_eq!(
        out.expect("yielded result"),
        vec!["alpha kept across pending".to_string()]
    );
}

#[test]
fn implementation_can_borrow_local_state_and_local_material_across_pending() {
    // Both the implementation's own state and the request material are borrowed from this
    // stack frame: no owned copy, no leak and no `'static` value is used to dodge the check.
    let calls = Cell::new(0u32);
    let topic = String::from("roof");
    let material = format!("note about the {topic} repair");
    let materials: [&str; 2] = [material.as_str(), "unrelated note"];
    let query = String::from("roof");

    let slot = BorrowingMemory { calls: &calls };
    let as_dyn: &dyn MemoryBase = &slot;
    let (polls_observed, wakes, out) = drive(as_dyn.retrieve(MemoryBaseRequest {
        materials: &materials,
        query: query.as_str(),
    }));

    assert_eq!(polls_observed, 2, "one Pending then Ready");
    assert!(wakes >= 1);
    assert_eq!(out.expect("borrowed result"), vec![material.clone()]);
    assert_eq!(
        calls.get(),
        2,
        "the borrowed counter was used before and after the Pending"
    );
}

#[test]
fn non_send_state_and_thread_safe_state_both_satisfy_the_base_trait() {
    fn assert_send_sync<T: Send + Sync>() {}

    // Thread-safe implementation satisfies the same trait; its counter is internal state
    // asserted separately, while both results come from this call's material.
    assert_send_sync::<AtomicMemory>();
    let calls = Arc::new(AtomicUsize::new(0));
    let atomic = AtomicMemory {
        calls: Arc::clone(&calls),
    };
    let materials = ["atomic material"];
    let (_, _, first) = drive(atomic.retrieve(MemoryBaseRequest {
        materials: &materials,
        query: "atomic",
    }));
    let (_, _, second) = drive(atomic.retrieve(MemoryBaseRequest {
        materials: &materials,
        query: "atomic",
    }));
    assert_eq!(
        first.expect("atomic first"),
        vec!["atomic material".to_string()]
    );
    assert_eq!(
        second.expect("atomic second"),
        vec!["atomic material".to_string()]
    );
    assert_eq!(calls.load(Ordering::SeqCst), 2);

    // Non-Send/non-Sync state is equally acceptable; only the caller's threading model
    // decides whether it may be used there.
    let local = LocalStateMemory {
        seen_queries: Rc::new(RefCell::new(Vec::new())),
    };
    let as_dyn: &dyn MemoryBase = &local;
    let (_, _, out) = drive(as_dyn.retrieve(MemoryBaseRequest {
        materials: &["stack-only material"],
        query: "stack-only",
    }));
    assert_eq!(
        out.expect("local slot"),
        vec!["stack-only material".to_string()]
    );
    assert_eq!(local.seen_queries.borrow().as_slice(), ["stack-only"]);
}

#[test]
fn base_only_bindings_need_no_extension_and_extensions_stay_opt_in() {
    let scored = ScoredMemory;

    // Extension use is an explicit, known-binding choice.
    let extension_value = scored.scored();
    assert_eq!(extension_value, 7);

    // The same value works as a Base-only slot; nothing about Base requires the extension,
    // and its result is still drawn from this call's material.
    let as_base: &dyn MemoryBase = &scored;
    let (_, _, out) = drive(as_base.retrieve(MemoryBaseRequest {
        materials: &["scored material"],
        query: "scored",
    }));
    assert_eq!(out.expect("base call"), vec!["scored material".to_string()]);

    // A Base-only implementation is a complete fixture implementation on its own.
    let plain: &dyn MemoryBase = &KeywordMemory;
    let (_, _, plain_out) = drive(plain.retrieve(MemoryBaseRequest {
        materials: &["plain material"],
        query: "plain",
    }));
    assert_eq!(plain_out.expect("plain base call").len(), 1);
}
