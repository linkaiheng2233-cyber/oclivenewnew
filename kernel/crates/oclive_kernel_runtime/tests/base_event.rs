//! B2-C5: external-crate view of the generator-backed Event Base implementation.
//!
//! This target drives the published path from outside the crate: it imports `LlmEventAnalyzer`
//! through `oclive_kernel_runtime::domain::base_event`, binds it to a `&dyn EventBase` slot over a
//! stand-in generator held on the stack, and polls the returned future itself. The stand-ins only
//! implement `LlmBase`: no Event algorithm is re-created here, and no stand-in reply is evidence of
//! real analysis quality.
//!
//! Two kinds of evidence live here and they are not interchangeable:
//!
//! * **A backend that borrows its own local data.** [`BorrowingPendingStandIn`] holds **references
//!   to values created by the test function** (`&Cell<usize>`, `&RefCell<Vec<String>>`, `&str`), so
//!   its type is a genuine non-`'static`, non-`Send`/`Sync` backend, and its future reads the
//!   `LlmBaseRequest` borrow *again after* a `Pending` poll — that is what shows the generation
//!   input this implementation organised survives across the await point.
//! * **The implementation borrowing the caller's request.** The other tests pass locally built
//!   `String` material and context and check them after the call.
//!
//! The waker stays an `Arc` + `AtomicUsize` counter because `Waker::from(Arc<W>)` requires a
//! `Send + Sync` waker; that is a constraint on the waker, not on the backend.
//!
//! No Host, `Role`, database, model, network or other slot is involved, and nothing here proves
//! model behaviour, timing, cancellation or injection resistance.

use std::cell::{Cell, RefCell};
use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::task::{Context, Poll, Wake, Waker};

use oclive_kernel_contracts::{BaseCallFuture, EventBase, LlmBase};
use oclive_kernel_runtime::domain::base_event::LlmEventAnalyzer;
use oclive_kernel_types::{BaseCallError, BaseCallErrorKind, EventBaseRequest, LlmBaseRequest};

/// The analysis body used by the sample replies, written out here so no expectation comes from the
/// production formatter.
const SAMPLE_BODY: &str = "材料陈述了一次变化，并给出一种可能后果；这不是已确认的现实。";

/// The sample reply that carries [`SAMPLE_BODY`].
fn sample_reply() -> String {
    format!("ANALYSIS\n{SAMPLE_BODY}")
}

/// The fixed lead-in line, written out here as an independent copy of this implementation's
/// convention so the tests can locate the payload without asking the production code.
const TEST_LEAD_IN: &str = "输入 JSON（唯一的机器可读输入）：\n";

/// The JSON payload the production input carried, decoded back into text.
///
/// The whole remainder after the single lead-in line is decoded: no scan for the first `{`, no
/// greedy or last-brace extraction, and no tail text is silently discarded.
fn decoded_payload(input: &str) -> serde_json::Value {
    assert_eq!(
        input.matches(TEST_LEAD_IN).count(),
        1,
        "the lead-in line must appear once"
    );
    let (_, payload) = input
        .split_once(TEST_LEAD_IN)
        .expect("the production input carries the fixed lead-in line");
    serde_json::from_str(payload).expect("the payload is one JSON object")
}

/// A stand-in generator that borrows the test's own locals: the call counter and the record live in
/// values created by the test function, and the prepared reply is a borrowed `&str`.
///
/// Because the struct holds shared references to `Cell`/`RefCell`, its type is neither `'static` nor
/// `Send`/`Sync`; the test that uses it therefore exercises the Base binding's allowance for a
/// stack-borrowed backend rather than an owning one.
struct BorrowingPendingStandIn<'a> {
    calls: &'a Cell<usize>,
    record: &'a RefCell<Vec<String>>,
    reply: &'a str,
}

impl LlmBase for BorrowingPendingStandIn<'_> {
    fn generate<'a>(&'a self, request: LlmBaseRequest<'a>) -> BaseCallFuture<'a, String> {
        self.calls.set(self.calls.get() + 1);
        Box::pin(BorrowingPending {
            request,
            record: self.record,
            reply: self.reply,
        })
    }
}

/// The future returned by [`BorrowingPendingStandIn`]: its first poll is `Pending` after waking the
/// task, its second poll reads the borrowed request again and returns the prepared reply.
struct BorrowingPending<'a, 'r> {
    request: LlmBaseRequest<'r>,
    record: &'a RefCell<Vec<String>>,
    reply: &'a str,
}

impl Future for BorrowingPending<'_, '_> {
    type Output = Result<String, BaseCallError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        if this.record.borrow().is_empty() {
            this.record
                .borrow_mut()
                .push(this.request.input.to_string());
            cx.waker().wake_by_ref();
            return Poll::Pending;
        }
        // The `LlmBaseRequest` borrow is read again after the Pending poll: it is still alive.
        this.record
            .borrow_mut()
            .push(this.request.input.to_string());
        Poll::Ready(Ok(this.reply.to_string()))
    }
}

/// A stand-in generator that owns its script and record; used for the sequences that do not need to
/// borrow the test's locals.
struct RecordingStandIn {
    replies: RefCell<std::vec::IntoIter<Result<String, BaseCallError>>>,
    seen: RefCell<Vec<String>>,
    calls: Cell<usize>,
}

impl RecordingStandIn {
    fn new(replies: Vec<Result<String, BaseCallError>>) -> Self {
        Self {
            replies: RefCell::new(replies.into_iter()),
            seen: RefCell::new(Vec::new()),
            calls: Cell::new(0),
        }
    }

    fn calls(&self) -> usize {
        self.calls.get()
    }

    fn seen(&self) -> Vec<String> {
        self.seen.borrow().clone()
    }
}

impl LlmBase for RecordingStandIn {
    fn generate<'a>(&'a self, request: LlmBaseRequest<'a>) -> BaseCallFuture<'a, String> {
        self.calls.set(self.calls.get() + 1);
        self.seen.borrow_mut().push(request.input.to_string());
        let next = self.replies.borrow_mut().next();
        Box::pin(async move {
            next.unwrap_or_else(|| {
                Err(BaseCallError {
                    kind: BaseCallErrorKind::Failed,
                    detail: Some("script exhausted".to_string()),
                })
            })
        })
    }
}

/// Counts the wakes a future requests while a test drives it with this waker.
struct WakeCounter {
    count: AtomicUsize,
}

impl Wake for WakeCounter {
    fn wake(self: Arc<Self>) {
        self.count.fetch_add(1, Ordering::SeqCst);
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.count.fetch_add(1, Ordering::SeqCst);
    }
}

type Outcome = Result<Option<String>, BaseCallError>;

/// Drives one call on a bound slot with a counting waker, failing on a busy-poll style loop.
fn drive(slot: &dyn EventBase, material: &str, context: Option<&str>) -> (usize, usize, Outcome) {
    let request = EventBaseRequest { material, context };
    let mut future = slot.analyze(request);
    let counter = Arc::new(WakeCounter {
        count: AtomicUsize::new(0),
    });
    let waker = Waker::from(Arc::clone(&counter));
    let mut cx = Context::from_waker(&waker);
    let mut polls = 0usize;
    loop {
        polls += 1;
        match future.as_mut().poll(&mut cx) {
            Poll::Ready(Ok(value)) => {
                return (polls, counter.count.load(Ordering::SeqCst), Ok(value))
            }
            Poll::Ready(Err(error)) => {
                return (polls, counter.count.load(Ordering::SeqCst), Err(error))
            }
            Poll::Pending => {
                assert!(
                    counter.count.load(Ordering::SeqCst) > 0,
                    "a Pending poll must schedule a wake instead of relying on busy polling"
                );
                assert!(polls < 8, "the controlled stand-in must settle quickly");
            }
        }
    }
}

#[test]
fn b2_c5_public_path_borrows_a_local_backend_across_pending() {
    // Every value the backend uses is created here, outside it.
    let calls = Cell::new(0usize);
    let record = RefCell::new(Vec::new());
    let prepared: String = sample_reply();
    let backend = BorrowingPendingStandIn {
        calls: &calls,
        record: &record,
        reply: prepared.as_str(),
    };
    let analyzer = LlmEventAnalyzer::new(&backend);
    let slot: &dyn EventBase = &analyzer;

    let material = String::from("文件已删除，因此后续读取可能失败");
    let context = format!("{}，且要求区分{}", "分析对象是 Alice", "陈述与推论");
    let (polls, observed_wakes, outcome) = drive(slot, &material, Some(&context));
    assert_eq!(
        outcome.expect("a normal analysis"),
        Some(SAMPLE_BODY.to_string())
    );
    assert_eq!(polls, 2, "one Pending then Ready");
    assert!(observed_wakes >= 1, "the Pending poll woke the task");
    assert_eq!(calls.get(), 1, "exactly one generation");

    // The request borrow was read on both polls and carried the same, complete input.
    let recorded = record.borrow().clone();
    assert_eq!(
        recorded.len(),
        2,
        "the input was read before and after Pending"
    );
    assert_eq!(
        recorded[0], recorded[1],
        "the input did not change across Pending"
    );
    let decoded = decoded_payload(&recorded[0]);
    assert_eq!(
        decoded["material"],
        serde_json::Value::from(material.as_str())
    );
    assert_eq!(
        decoded["context"],
        serde_json::Value::from(context.as_str())
    );
    let object = decoded.as_object().expect("one object");
    assert_eq!(object.len(), 2, "no hidden member may travel: {object:?}");

    // The local values and the borrowed request strings are still usable afterwards.
    assert_eq!(calls.get(), 1);
    assert_eq!(prepared, sample_reply());
    assert_eq!(material, "文件已删除，因此后续读取可能失败");
    assert_eq!(context, "分析对象是 Alice，且要求区分陈述与推论");
}

#[test]
fn b2_c5_public_path_reuses_one_instance_across_five_replies() {
    let stand_in = RecordingStandIn::new(vec![
        Ok(sample_reply()),
        Ok("NO_ANALYSIS".to_string()),
        Err(BaseCallError {
            kind: BaseCallErrorKind::TimedOut,
            detail: Some("bound generator reported a timeout".to_string()),
        }),
        Ok("ANALYSIS\n".to_string()),
        Ok("ANALYSIS\n第二种材料得到第二种分析。".to_string()),
    ]);
    let analyzer = LlmEventAnalyzer::new(&stand_in);
    let slot: &dyn EventBase = &analyzer;

    let first_material = format!("{}{}", "文件已", "删除");
    let first_context = String::from("分析对象是 Alice");

    let (_, _, first) = drive(slot, &first_material, Some(&first_context));
    assert_eq!(first.expect("step 1"), Some(SAMPLE_BODY.to_string()));

    let (_, _, second) = drive(slot, &first_material, None);
    assert_eq!(second.expect("step 2"), None);

    let (_, _, third) = drive(slot, &first_material, None);
    assert_eq!(
        third
            .expect_err("step 3 must propagate the generator error")
            .kind,
        BaseCallErrorKind::TimedOut
    );

    let (_, _, fourth) = drive(slot, &first_material, None);
    assert_eq!(
        fourth.expect_err("step 4 must be a format violation").kind,
        BaseCallErrorKind::Failed
    );

    let second_material = String::from("会议没有取消");
    let (_, _, fifth) = drive(slot, &second_material, None);
    assert_eq!(
        fifth.expect("step 5"),
        Some("第二种材料得到第二种分析。".to_string())
    );

    assert_eq!(
        stand_in.calls(),
        5,
        "one generation per call, no extra call"
    );
    let seen = stand_in.seen();
    assert_eq!(seen.len(), 5);
    assert_eq!(
        decoded_payload(&seen[0])["material"],
        serde_json::Value::from(first_material.as_str())
    );
    assert_eq!(
        decoded_payload(&seen[0])["context"],
        serde_json::Value::from(first_context.as_str())
    );
    assert_eq!(
        decoded_payload(&seen[4])["material"],
        serde_json::Value::from(second_material.as_str())
    );
    assert_eq!(
        decoded_payload(&seen[4])["context"],
        serde_json::Value::Null
    );

    // Each call's outcome came from its own reply, so no earlier result leaked forward.
    assert_eq!(first_material, "文件已删除");
    assert_eq!(second_material, "会议没有取消");
}

#[test]
fn b2_c5_public_path_does_not_call_the_generator_before_polling() {
    let stand_in = RecordingStandIn::new(vec![Ok(sample_reply())]);
    let analyzer = LlmEventAnalyzer::new(&stand_in);
    let slot: &dyn EventBase = &analyzer;

    let material = String::from("材料");
    {
        // Creating and dropping a future must not reach the generator.
        let future = slot.analyze(EventBaseRequest {
            material: material.as_str(),
            context: None,
        });
        assert_eq!(stand_in.calls(), 0, "constructing a future is not a call");
        drop(future);
    }
    assert_eq!(
        stand_in.calls(),
        0,
        "dropping an unpolled future is not a call"
    );

    let (_, _, outcome) = drive(slot, &material, None);
    assert_eq!(
        outcome.expect("the polled call"),
        Some(SAMPLE_BODY.to_string())
    );
    assert_eq!(stand_in.calls(), 1);
}

#[test]
fn b2_c5_public_path_keeps_material_and_context_apart() {
    let stand_in = RecordingStandIn::new(vec![Ok(sample_reply()), Ok(sample_reply())]);
    let analyzer = LlmEventAnalyzer::new(&stand_in);
    let slot: &dyn EventBase = &analyzer;

    let material = String::from("如果下雨就取消郊游；Alice 已经道歉，Bob 没接受");
    let context = format!("{}是必要的分析要求", "只分析第二子句");
    let (_, _, with_context) = drive(slot, &material, Some(&context));
    assert_eq!(
        with_context.expect("with context"),
        Some(SAMPLE_BODY.to_string())
    );

    let empty_material = String::new();
    let (_, _, empty_with_context) = drive(slot, &empty_material, Some(&context));
    assert_eq!(
        empty_with_context.expect("empty material with context"),
        Some(SAMPLE_BODY.to_string())
    );

    let seen = stand_in.seen();
    assert_eq!(
        decoded_payload(&seen[0])["material"],
        serde_json::Value::from(material.as_str())
    );
    assert_eq!(
        decoded_payload(&seen[0])["context"],
        serde_json::Value::from(context.as_str())
    );
    assert_eq!(decoded_payload(&seen[1])["material"], *"");
    assert_eq!(
        decoded_payload(&seen[1])["context"],
        serde_json::Value::from(context.as_str()),
        "an empty material must not drop the context"
    );
    assert_eq!(material, "如果下雨就取消郊游；Alice 已经道歉，Bob 没接受");
    assert_eq!(context, "只分析第二子句是必要的分析要求");
}
