//! Independent six-slot Base traits (B1 Rust binding).
//!
//! These six traits are the **call half** of the independent Base surface described in
//! `handoff/MODULE_MAP_AND_HANDOFF.md` §0.4–§0.9. They are added **alongside** the
//! reference-runtime ports documented in this crate root: the existing port table, the
//! legacy slot traits, the wire shapes and the plugin protocol are unchanged, and no
//! legacy implementation is claimed to satisfy these traits. This binding is not wired
//! into the reference Host.
//!
//! Each trait has exactly one method and returns [`BaseCallFuture`]. What that binding
//! does and does not promise to implementations and hosts is documented once, on
//! [`BaseCallFuture`]; each trait below keeps only its own capability notes.
//!
//! A normal result may be empty: `Ok(Vec::new())` for Memory, `Ok(None)` for
//! Emotion/Event, and `Ok(String::new())` for Prompt/LLM/Agent. An empty value means
//! "no applicable content in this call's scope" at most: it does not mean the slot was
//! never called, it does not by itself establish that a call completed normally, and it
//! is not a [`BaseCallError`].
//!
//! # Example: a Base-only implementation, used through `dyn`
//!
//! ```no_run
//! use oclive_kernel_contracts::{BaseCallFuture, MemoryBase};
//! use oclive_kernel_types::MemoryBaseRequest;
//!
//! struct KeywordMemory;
//!
//! impl MemoryBase for KeywordMemory {
//!     fn retrieve<'a>(&'a self, request: MemoryBaseRequest<'a>) -> BaseCallFuture<'a, Vec<String>> {
//!         Box::pin(async move {
//!             let hits: Vec<String> = request
//!                 .materials
//!                 .iter()
//!                 .filter(|item| item.contains(request.query))
//!                 .map(|item| (*item).to_string())
//!                 .collect();
//!             Ok(hits)
//!         })
//!     }
//! }
//!
//! fn pick_slot(memory: &dyn MemoryBase) -> &dyn MemoryBase {
//!     memory
//! }
//! ```
//!
//! # Example: a Base-only binding cannot call a test-local extension
//!
//! ```compile_fail
//! use oclive_kernel_contracts::MemoryBase;
//!
//! trait MemoryScoredExtension {
//!     fn scored(&self) -> u32;
//! }
//!
//! fn use_extension(slot: &dyn MemoryScoredExtension) -> u32 {
//!     slot.scored()
//! }
//!
//! fn only_base(slot: &dyn MemoryBase) -> u32 {
//!     // `dyn MemoryBase` does not carry the extension method.
//!     use_extension(slot)
//! }
//! ```

use std::future::Future;
use std::pin::Pin;

use oclive_kernel_types::{
    AgentBaseRequest, BaseCallError, EmotionBaseRequest, EventBaseRequest, LlmBaseRequest,
    MemoryBaseRequest, PromptBaseRequest,
};

/// The boxed local future every Base method returns.
///
/// `T` is the slot's normal result. This is the shared binding description for all six
/// Base traits; each trait below adds only its own capability notes.
///
/// # What this binding is
///
/// - It is the **independent B1 binding**: it does not replace the reference-runtime
///   ports of this crate and is **not** wired into the reference Host. No legacy
///   implementation is claimed to satisfy it.
/// - Every method returns this explicitly boxed local future, which keeps the traits
///   usable through `dyn`: a caller can hold several implementations of one slot and
///   choose among them, without a registry, a negotiation system or a macro dependency.
///
/// # Costs and limits
///
/// - Boxing and dynamic dispatch cost an allocation in the general case: **no
///   zero-allocation guarantee** is made.
/// - Implementations are **not** required to be `Send`, `Sync` or `'static`. The traits
///   declare no such supertrait, requests and implementations only need to live for the
///   borrow period of the returned future, and the future itself is **not** `Send`.
/// - Erasing the future hides any `Send` property of a concrete implementation: adding
///   `Send + Sync` to an object type does **not** restore `Send` on the erased future. A
///   host that must move futures across threads needs a stronger binding or an adapter,
///   never an unsafe cast.
/// - The host drives this future itself. `&self` promises no re-entrancy, thread safety,
///   preemptibility or non-blocking behaviour, and no arbitrary concurrent re-entry is
///   guaranteed.
/// - Dropping a future is **not** a general cancellation protocol: it does not prove
///   that physical execution ended, that an effect was rolled back, or that a retry is
///   safe. The binding carries no invocation identifier or lifetime object, and no
///   method here decides whether an invocation has ended — a capability return does not
///   by itself determine that an invocation is terminal.
/// - `async fn` in traits is deliberately not the default here, so that direct `dyn`
///   assembly is verified first. That is a choice for this slice, not a prohibition on
///   native `async fn`, GATs or paired local/send traits later.
///
/// Returned text is owned by the caller, and its later reuse still obeys the applicable
/// usage constraints.
pub type BaseCallFuture<'a, T> = Pin<Box<dyn Future<Output = Result<T, BaseCallError>> + 'a>>;

/// Memory: select the material relevant to this call's retrieval need.
///
/// The result is a text selection drawn from [`MemoryBaseRequest::materials`]; an empty
/// vector means "no applicable content in this call's scope". Faithful selection,
/// reordering, paraphrase and compression are allowed, while subject, time, negation and
/// uncertainty must be preserved, and added inference must not be presented as retrieved
/// material.
///
/// Base retrieval, and the result it returns, do **not** themselves grant Host memory
/// write-back, domain commit or any other external operation: completing a retrieval is
/// not authorisation to perform them. This says nothing about an implementation's own
/// internal caching or diagnostics performed within its existing legitimate resource
/// permissions — it neither exempts such behaviour from authorisation nor lets a cache
/// change domain state under that name.
///
/// Binding shape and its costs: [`BaseCallFuture`].
///
/// # Example
///
/// ```compile_fail
/// use oclive_kernel_contracts::{BaseCallFuture, MemoryBase};
/// use oclive_kernel_types::{EmotionBaseRequest, MemoryBaseRequest};
///
/// struct OnlyMemory;
///
/// impl MemoryBase for OnlyMemory {
///     fn retrieve<'a>(&'a self, _request: MemoryBaseRequest<'a>) -> BaseCallFuture<'a, Vec<String>> {
///         Box::pin(async move { Ok(Vec::new()) })
///     }
/// }
///
/// let slot = OnlyMemory;
/// // A request belonging to another slot must not be interchangeable.
/// let _ = slot.retrieve(EmotionBaseRequest {
///     material: "not a memory query",
///     context: None,
/// });
/// ```
pub trait MemoryBase {
    /// Retrieves the material relevant to `request`.
    fn retrieve<'a>(&'a self, request: MemoryBaseRequest<'a>) -> BaseCallFuture<'a, Vec<String>>;
}

/// Emotion: analyse the emotion expressed by the given text.
///
/// `Ok(None)` means this call formed no applicable analysis — it is not "neutral", not a
/// failure, and not evidence that the slot was never called. An implementation does not
/// decide domain state, and a mentioned emotion must not be silently reattributed to the
/// current speaker.
///
/// Binding shape and its costs: [`BaseCallFuture`].
pub trait EmotionBase {
    /// Analyses `request.material`; `None` means no applicable analysis was formed.
    fn analyze<'a>(&'a self, request: EmotionBaseRequest<'a>)
        -> BaseCallFuture<'a, Option<String>>;
}

/// Event: analyse the meaning or possible impact of the described event.
///
/// `Ok(None)` means no applicable analysis, not "nothing happened" and not "impact is
/// exactly zero". Analysis may consume an Emotion result when the caller supplies one,
/// but does not require that slot; conditions, negation, hypotheses and plans in the
/// material must not be rewritten as facts, and nothing is published.
///
/// Binding shape and its costs: [`BaseCallFuture`].
pub trait EventBase {
    /// Analyses `request.material`; `None` means no applicable analysis was formed.
    fn analyze<'a>(&'a self, request: EventBaseRequest<'a>) -> BaseCallFuture<'a, Option<String>>;
}

/// Prompt: organise the supplied material into model input.
///
/// Passing the material through unchanged is a valid assembly when it already satisfies
/// the stated requirements. Material and requirements stay distinguishable, and
/// instructions found inside reference material do not become extra authority.
///
/// Binding shape and its costs: [`BaseCallFuture`].
pub trait PromptBase {
    /// Assembles model input for `request`.
    fn assemble<'a>(&'a self, request: PromptBaseRequest<'a>) -> BaseCallFuture<'a, String>;
}

/// LLM: generate text from input that the caller has already prepared.
///
/// The input need not come from a Prompt slot. A generation that completed normally may
/// return empty text; emptiness alone cannot establish that a call completed normally,
/// and text already produced cannot erase a later failure. When the current consumer
/// explicitly requires non-empty output, that is judged by the applicable agreement, not
/// by this binding. Generated text grants no tool or domain authority, and normal
/// completion does not by itself end an invocation.
///
/// Binding shape and its costs: [`BaseCallFuture`].
///
/// # Example
///
/// ```
/// use std::future::Future;
/// use std::task::{Context, Poll, Waker};
///
/// use oclive_kernel_contracts::{BaseCallFuture, LlmBase};
/// use oclive_kernel_types::LlmBaseRequest;
///
/// struct Echo;
///
/// impl LlmBase for Echo {
///     fn generate<'a>(&'a self, request: LlmBaseRequest<'a>) -> BaseCallFuture<'a, String> {
///         Box::pin(async move { Ok(request.input.to_string()) })
///     }
/// }
///
/// let slot = Echo;
/// let mut future = slot.generate(LlmBaseRequest { input: "hello" });
/// let waker = Waker::noop();
/// let mut cx = Context::from_waker(waker);
/// // This fixture implementation is ready on first poll; the binding stays asynchronous.
/// assert!(matches!(
///     future.as_mut().poll(&mut cx),
///     Poll::Ready(Ok(text)) if text == "hello"
/// ));
/// ```
pub trait LlmBase {
    /// Generates text for `request.input`.
    fn generate<'a>(&'a self, request: LlmBaseRequest<'a>) -> BaseCallFuture<'a, String>;
}

/// Agent: process a delegated task and report the outcome in text.
///
/// The report carries the result and a truthful completion statement; `handled`-style
/// control fields are not required, and neither `Ok`, nor an empty body, nor any
/// keyword may be read as task achievement. Distinguish "not taken up", "no action
/// needed", and "failed": they are different facts, and none of them proves that no
/// external effect occurred or that a retry is safe.
///
/// Binding shape and its costs: [`BaseCallFuture`].
pub trait AgentBase {
    /// Executes the delegated `request.task` and returns its text report.
    fn execute<'a>(&'a self, request: AgentBaseRequest<'a>) -> BaseCallFuture<'a, String>;
}
