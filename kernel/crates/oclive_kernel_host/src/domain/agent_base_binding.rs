//! CP-B3-ALL unit A: the Host-private [`AgentBase`] request-binding view over the real builtin
//! agent.
//!
//! [`HostAgentBaseView`] borrows the **real** [`BuiltinReActAgent`] and the Host's own per-call
//! identity (the model of this turn, and the role/session the debug trace belongs to). It has no
//! execution of its own: [`AgentBase::execute`] runs [`BuiltinReActAgent::execute_react`], the same
//! core the legacy product entry ([`AgentProvider::process`](oclive_kernel_contracts::AgentProvider))
//! runs, and projects that one run's branch facts into a text report.
//!
//! # What the Base request can and cannot carry
//!
//! `AgentBaseRequest` carries the delegated `task` and an optional `context`; nothing else. The task
//! text is handed to the core as this call's message, and the identity comes from the caller's own
//! existing values — never from a default model, a placeholder role or a fake session. The binding
//! keeps **no** shared mutable "current model": the identity lives in the value constructed for this
//! call, and the task lives in the request.
//!
//! CP-B3-ALL-R2: this view builds the core's private four-field input (`ReActInput`) directly from
//! the bound identity and `request.task`. It does **not** construct an `AgentInput`, and therefore
//! cannot bring in `AgentInput::default()`'s personality vector, relation state, favour, scene,
//! policy text or protocol version: there is no field for them, so they are neither fabricated nor
//! passed on. The legacy input's other fields (`tools`, `turn_context`, `constraints`, `scene_id`,
//! `protocol_version`) were never consumed by the core — the product entry accepts and ignores them
//! today — and the Base request cannot carry them; the legacy input's acceptance surface is
//! unchanged.
//!
//! # `context`
//!
//! | `request.context` | Result |
//! |---|---|
//! | `None` | executed under the agreement below |
//! | `Some("")` | same as `None`; this is this adapter's stated agreement, not an empty-string rule for every Base implementation |
//! | any non-empty `Some(..)`, including spaces, newlines, tabs or U+3000 | [`BaseCallErrorKind::Unsupported`] **before any execution**: no discovery, no model round, no tool call |
//!
//! The reason is that this adapter does not invent a context-execution mode for the existing agent
//! loop; it is not a claim that the input is invalid or that no Agent implementation could serve it.
//! This limit is a property of this concrete adapter, not of the Agent Base binding in general.
//!
//! # The report is read from the real branches
//!
//! The report text is built from [`ReActFacts`], which the core fills in at the branch that produced
//! each fact. It is never derived from `AgentOutput.handled`, from whether the reply is empty, or
//! from keywords in a reply:
//!
//! | Real branch | What the report says |
//! |---|---|
//! | empty task, or no usable tool schema | **not taken up**, and which of the two it was; no model round and no tool call happened |
//! | a model round produced a final answer | the model generated an answer, on which round, with its content; explicitly **not** a guarantee that the delegated objective was achieved |
//! | a round produced no parseable function call, or the loop bound was reached | the observed process and results only, with the reply the product path returned; explicitly no guarantee that all tasks completed, that no external effect occurred, or that a retry is safe |
//!
//! Observed tool calls and failures are reported with the loop's own observation texts, in the order
//! the loop produced them. A typed authorisation refusal is reported as a refusal of that one call —
//! not as "this implementation does not support the capability" and with no promise about a retry.
//!
//! The report is human-readable only: it provides **no stable Base machine-state protocol** and adds
//! no machine field, so authorization, completion or external effect must not be inferred from report
//! keywords alone. This does not forbid the Host from handling the text of a specific implementation
//! (showing it, or matching the wording of its own adapter); it means such handling is that adapter's
//! private arrangement rather than a Base-level machine contract.
//!
//! # Failures
//!
//! A discovery error or a model error propagates out of the core exactly as it does for the product
//! entry; this binding projects the **typed** error, without parsing its text:
//!
//! - [`AppError::HighRiskCapabilityNotGranted`] → [`BaseCallErrorKind::Unavailable`]: the capability
//!   is supported, but it is not available to this call without the grant the caller withheld, and
//!   nothing here promises that a later attempt will be allowed. It is deliberately not
//!   `Unsupported` (this implementation does support the capability) and not `Failed` (the effect
//!   was refused rather than attempted).
//! - every other typed error → [`BaseCallErrorKind::Failed`], with the original error text kept as
//!   the human-readable `detail`.
//! - `Cancelled` and `TimedOut` are never manufactured. The core has no cancellation or timeout
//!   source of its own, and the product error type carries no typed cancellation/timeout variant, so
//!   a timeout raised inside the LLM transport cannot be recovered here without parsing text — which
//!   this unit does not do. Adding such a variant would change the frozen error-code registry.
//!
//! The Host keeps its own original [`AppError`] on the product path: [`AgentProvider::process`]
//! returns it unchanged, and the fallback wrapper's typed refusal branch
//! ([`crate::domain::fallback_agent`]) still sees it. Nothing is lost by the projection above.
//!
//! # What this does not do
//!
//! - It performs no extra model round, no extra tool call and no retry; one Base call is one real
//!   execution of the shared core, and the report describes that run.
//! - It does not replace the user reply, add persistence, change `process_message`/
//!   `minimal_response`, or change the legacy output, including `handled` and an empty reply.
//! - **No product consumer reads this view today.** The reference Host still binds
//!   `Arc<dyn AgentProvider>` and reads [`AgentOutput`]; that the builtin agent can serve the Base
//!   view is an additive capability of the implementation, not a product migration.
//! - The returned future is the borrowed [`BaseCallFuture`] shape: it adds no `Send`, `Sync`,
//!   `'static` or executor requirement. The Host's own concrete future is `Send` for its own
//!   reasons, and that is asserted separately rather than inferred from this binding.

use oclive_kernel_contracts::{AgentBase, BaseCallFuture};
use oclive_kernel_types::{AgentBaseRequest, BaseCallError, BaseCallErrorKind};

use super::agent::{BuiltinReActAgent, ReActFacts, ReActInput, ReActStop, MAX_ROUNDS};
use crate::error::AppError;

/// The one rejection text for a non-empty `context`.
///
/// It is the human-readable part only: the machine-readable reason is
/// [`BaseCallErrorKind::Unsupported`], and no decision may be recovered from this text.
const CONTEXT_REJECTION: &str =
    "this Host agent adapter does not process additional context; pass `None` (or an empty string) \
     to run the delegated task with the existing agent loop";

/// Reported when the call was not taken up.
const LIMITS_NOT_TAKEN: &str =
    "本报告只说明本次调用没有承接该委托，不构成对外部效果、任务终态或重试安全性的判断。";

/// Reported when a model round produced a final answer.
const LIMITS_ANSWERED: &str =
    "本报告只说明模型生成了回答；不代表委托的客观目标已经达成，也不代表工具效果已回滚。";

/// Reported when the run ended without a final answer.
const LIMITS_OBSERVED: &str =
    "本报告只陈述本次真实执行已观察到的过程与结果，不代表全部任务已完成、不代表没有产生外部效果、\
     也不代表可以安全重试。";

/// The Host's per-call agent identity, borrowed from the caller's existing values.
///
/// Each binding value owns its own identity; this file has no shared mutable "current model", and
/// the identity is what reaches the debug trace and the model call of the run it starts.
#[derive(Debug, Clone, Copy)]
pub(crate) struct AgentTurnIdentity<'a> {
    /// The model this turn uses (the core passes it to the LLM port unchanged).
    pub(crate) model: &'a str,
    /// The role id recorded in this turn's debug trace.
    pub(crate) role_id: &'a str,
    /// The session namespace recorded in this turn's debug trace.
    pub(crate) session_namespace: &'a str,
}

/// The Host-private [`AgentBase`] view over a real [`BuiltinReActAgent`].
///
/// See the [module documentation](self) for the exact agreement, the `context` branches, how the
/// report is read from the real branches, and the failure projection.
pub(crate) struct HostAgentBaseView<'a> {
    agent: &'a BuiltinReActAgent,
    identity: AgentTurnIdentity<'a>,
}

impl<'a> HostAgentBaseView<'a> {
    /// Binds the Base view to a real agent and the Host's own per-call identity.
    #[must_use]
    pub(crate) fn new(agent: &'a BuiltinReActAgent, identity: AgentTurnIdentity<'a>) -> Self {
        Self { agent, identity }
    }
}

/// Projects a typed product error onto the Base failure carrier, without reading its text.
///
/// See the module documentation for why the refusal is `Unavailable`, why everything else is
/// `Failed`, and why no cancellation/timeout kind is manufactured here.
pub(crate) fn project_error(error: &AppError) -> BaseCallError {
    let kind = match error {
        AppError::HighRiskCapabilityNotGranted { .. } => BaseCallErrorKind::Unavailable,
        _ => BaseCallErrorKind::Failed,
    };
    BaseCallError {
        kind,
        detail: Some(error.to_string()),
    }
}

/// The observed process lines: the loop's own observation texts, in the order the loop produced them.
fn observed_process(facts: &ReActFacts) -> String {
    if facts.observations.is_empty() {
        return "（本次执行没有工具调用记录）".to_string();
    }
    facts.observations.join("；")
}

/// Adds the refusal line when this run contained a typed authorisation refusal.
fn push_refusal(report: &mut String, facts: &ReActFacts) {
    if let Some(refusal) = &facts.refusal {
        report.push_str(&format!(
            "\n被拒绝的调用：{refusal}（该次调用的授权决定，不是本实现不支持该能力；本报告不承诺重试结果）"
        ));
    }
}

/// Builds the report of one real run from the branch facts that run produced.
///
/// The `product_reply` argument is the reply the shared core returned for this same run; it is
/// reported as the product path's returned text, and it never decides which branch is described.
///
/// Crate-private so the unit tests can project **the same** carrier one execution produced, instead
/// of running a second execution to obtain a second view.
pub(crate) fn build_report(facts: &ReActFacts, product_reply: &str) -> String {
    let mut report = match facts.stop {
        ReActStop::EmptyTask => {
            "未承接：本次委托文本为空（trim 后没有字符）；本次调用没有发现工具，没有调用模型，\
             也没有调用任何工具。"
                .to_string()
        }
        ReActStop::NoTools => {
            "未承接：当前没有可用的工具 schema；本次调用没有调用模型，也没有调用任何工具。"
                .to_string()
        }
        ReActStop::FinalAnswer => {
            format!("模型在第 {} 轮生成了回答：{}", facts.rounds, product_reply)
        }
        ReActStop::NoFunctionCall => format!(
            "第 {} 轮模型输出没有可解析的 function call，循环结束；已观察到的过程：{}；\
             产品路径返回的回复：{}",
            facts.rounds,
            observed_process(facts),
            product_reply
        ),
        ReActStop::LoopExhausted => format!(
            "已用满循环上限 {MAX_ROUNDS} 轮仍未获得最终回答；已观察到的过程：{}；\
             产品路径返回的回复：{}",
            observed_process(facts),
            product_reply
        ),
    };
    let limits = match facts.stop {
        ReActStop::EmptyTask | ReActStop::NoTools => LIMITS_NOT_TAKEN,
        ReActStop::FinalAnswer => LIMITS_ANSWERED,
        ReActStop::NoFunctionCall | ReActStop::LoopExhausted => LIMITS_OBSERVED,
    };
    report.push('\n');
    report.push_str(limits);
    push_refusal(&mut report, facts);
    report
}

impl AgentBase for HostAgentBaseView<'_> {
    /// Runs the delegated `request.task` through the shared execution core and reports what happened.
    ///
    /// # Errors
    ///
    /// Returns [`BaseCallErrorKind::Unsupported`] for any non-empty `request.context`, before any
    /// execution. A discovery or model error of the shared core is projected as documented on this
    /// module; this implementation adds no failure source of its own.
    fn execute<'a>(&'a self, request: AgentBaseRequest<'a>) -> BaseCallFuture<'a, String> {
        // Copy the borrowed identity out of `self`: it is per-call data, and the future borrows the
        // request for this call rather than any shared state.
        let agent = self.agent;
        let identity = self.identity;
        Box::pin(async move {
            if request.context.is_some_and(|context| !context.is_empty()) {
                return Err(BaseCallError {
                    kind: BaseCallErrorKind::Unsupported,
                    detail: Some(CONTEXT_REJECTION.to_string()),
                });
            }
            // CP-B3-ALL-R2: the core's private four-field input, built from the bound identity and
            // this call's own task text. No `AgentInput`, no `Default::default()`, and therefore no
            // fabricated personality, relation or session fact.
            let input = ReActInput {
                message: request.task,
                model: identity.model,
                role_id: identity.role_id,
                session_namespace: identity.session_namespace,
            };
            match agent.execute_react(input).await {
                Ok((output, facts)) => Ok(build_report(&facts, output.reply.as_str())),
                Err(error) => Err(project_error(&error)),
            }
        })
    }
}
