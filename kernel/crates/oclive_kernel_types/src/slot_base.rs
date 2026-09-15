//! Six-slot Base request data and the minimal call-failure carrier (B1 surface).
//!
//! This module is the **data half** of the independent Base interface described in
//! `handoff/MODULE_MAP_AND_HANDOFF.md` §0.4–§0.9. It does not replace anything: the
//! legacy slot traits, reference DTOs, wire shapes, plugin protocol and error-code
//! registry stay as they are and keep their own documentation.
//!
//! Requests carry only what the capability needs for this one call — explicit material
//! and this call's requirement. The Base surface itself requires **no** complete `Role`,
//! identity model, permission handle, six-slot state or other Host data structure, and
//! nothing here makes such structures a request obligation.
//!
//! That is not a ban on explicit domain text: the Host may organise whatever is relevant
//! to the current use into explicit text material or context, and pass it through these
//! fields. Such text is a description, not state ownership, a resource handle or an
//! authorisation credential, and optional context must not become a hidden bundle that
//! every Base implementation is expected to understand. Material may mention a role, past
//! turns or constraints; mentioning them is not by itself illegitimate, and a request
//! field must not be used as a back door for state that Base does not define.
//!
//! A normal result is carried by the call's own return value; a call that completed
//! normally may return empty content. An empty value alone does not establish that the
//! call completed normally, and non-empty content does not erase a failure.
//! [`BaseCallError`] carries the reason a call did **not** complete normally; it is not a
//! terminal state of a session, an invocation, or a product operation.

use std::fmt;

/// Memory: explicit candidate material plus this call's retrieval need.
///
/// `materials` are borrowed text items supplied for this call, not a handle to Host
/// storage: subject, time, quotation and uncertainty stay as the provider wrote them.
/// `query` states the retrieval need and may carry the agreed scope or usage constraint.
///
/// Neither field opens an implicit data source, and returning a selection does not
/// promise that every relevant material was found.
#[derive(Debug, Clone, Copy)]
pub struct MemoryBaseRequest<'a> {
    pub materials: &'a [&'a str],
    pub query: &'a str,
}

/// Emotion: the text whose expressed emotion is analysed, plus optional context.
///
/// `context` may carry applicable situational detail or analysis requirements written as
/// explicit text: for example the surrounding scene, the subject being discussed or a
/// constraint for this analysis. It is not required to come from another slot, it is not
/// an ownership claim over role or slot state, and implementations are not required to
/// understand an undocumented bundle passed through it. Analysing material does not
/// decide any domain state.
#[derive(Debug, Clone, Copy)]
pub struct EmotionBaseRequest<'a> {
    pub material: &'a str,
    pub context: Option<&'a str>,
}

/// Event: the material whose described event is analysed, plus optional context.
///
/// `context` may carry applicable situational detail or analysis requirements; it is
/// not required to be a complete set of upstream slot outputs. Analysis of described
/// events does not assert that they happened, and it publishes nothing.
#[derive(Debug, Clone, Copy)]
pub struct EventBaseRequest<'a> {
    pub material: &'a str,
    pub context: Option<&'a str>,
}

/// Prompt: the material to organise, kept separate from this call's requirements.
///
/// `materials` are reference material for the model input; `requirements` is what this
/// assembly must satisfy. Keeping them separate lets an implementation preserve the
/// difference between instructions and quoted material, but it does not by itself
/// guarantee that a downstream model resists every prompt-injection attempt.
#[derive(Debug, Clone, Copy)]
pub struct PromptBaseRequest<'a> {
    pub materials: &'a [&'a str],
    pub requirements: &'a str,
}

/// LLM: text generation input that has already been prepared by the caller.
///
/// The input may come from any source; it is not required to be produced by a Prompt
/// slot. Applicable generation requirements are carried by the input and the assembly
/// agreement, so they are not repeated here, and no model name, provider or tool
/// definition is required in the request.
#[derive(Debug, Clone, Copy)]
pub struct LlmBaseRequest<'a> {
    pub input: &'a str,
}

/// Agent: the delegated task plus optional material or constraint context.
///
/// The task text and context are not an authorisation credential: permission and
/// resource binding belong to the caller's legitimate authorisation path. Tool tables,
/// session storage and model choice are not required here.
#[derive(Debug, Clone, Copy)]
pub struct AgentBaseRequest<'a> {
    pub task: &'a str,
    pub context: Option<&'a str>,
}

/// Machine-distinguishable reason that a Base call did not complete normally.
///
/// These are reasons, not a terminal state: a call that produced a (possibly empty)
/// normal result reports it through its own return value, and none of these variants
/// describes such a result. [`BaseCallErrorKind`] is `#[non_exhaustive]` so that
/// external consumers keep a branch for reasons added later; an unknown reason is still
/// "did not complete normally" and must not be read as success, retryable, stopped, or
/// free of effects.
///
/// This enumeration does not publish an effect ledger, a retry suggestion, a task
/// terminal state or a permission category, and it does not replace finer-grained
/// refusal or authorisation information that a caller already owns.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BaseCallErrorKind {
    /// The call failed while attempting the capability. It does not swallow a known
    /// cancellation or timeout, and it does not prove that no effect occurred.
    Failed,
    /// The capability is supported but currently unavailable. It does not promise
    /// recovery later, and it does not itself grant a retry.
    Unavailable,
    /// The implementation does not support a commitment this call required. It does not
    /// allow an implementation to pose as Base-compliant by repeatedly answering
    /// `Unsupported`, and lacking an optional extension is not a Base violation.
    Unsupported,
    /// This call is known to have been cancelled before normal completion. It means more
    /// than "a request was received", and it does not prove the remote side stopped or
    /// that the logical invocation ended.
    Cancelled,
    /// This call is known to have timed out before normal completion. It does not prove
    /// that an invocation-wide deadline expired, that the task completed, or that any
    /// effect was rolled back.
    TimedOut,
}

/// A Base call that did not complete normally, with its machine-readable reason.
///
/// `kind` is the machine-readable part. `detail` is human-readable explanation only: it
/// is not a string-parsing protocol, and machine decisions must not be recovered from
/// its text. The type converts to nothing else on purpose — it is not an
/// [`AppError`](crate::AppError), is not mapped into HTTP/SSE payloads, and does not
/// join the error-code registry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BaseCallError {
    pub kind: BaseCallErrorKind,
    pub detail: Option<String>,
}

impl fmt::Display for BaseCallError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let reason = match self.kind {
            BaseCallErrorKind::Failed => "base call failed",
            BaseCallErrorKind::Unavailable => "base capability unavailable",
            BaseCallErrorKind::Unsupported => "base commitment unsupported",
            BaseCallErrorKind::Cancelled => "base call cancelled",
            BaseCallErrorKind::TimedOut => "base call timed out",
        };
        match self.detail.as_deref() {
            Some(detail) if !detail.is_empty() => write!(f, "{reason}: {detail}"),
            _ => f.write_str(reason),
        }
    }
}

impl std::error::Error for BaseCallError {}

#[cfg(test)]
mod tests {
    use super::{
        AgentBaseRequest, BaseCallError, BaseCallErrorKind, EmotionBaseRequest, EventBaseRequest,
        LlmBaseRequest, MemoryBaseRequest, PromptBaseRequest,
    };

    fn all_kinds() -> [BaseCallErrorKind; 5] {
        [
            BaseCallErrorKind::Failed,
            BaseCallErrorKind::Unavailable,
            BaseCallErrorKind::Unsupported,
            BaseCallErrorKind::Cancelled,
            BaseCallErrorKind::TimedOut,
        ]
    }

    #[test]
    fn requests_are_borrowed_views_over_caller_material() {
        let materials = ["alpha", "beta"];
        let memory = MemoryBaseRequest {
            materials: &materials,
            query: "alpha",
        };
        let emotion = EmotionBaseRequest {
            material: "tired but calm",
            context: None,
        };
        let event = EventBaseRequest {
            material: "she said she may move next week",
            context: Some("current conversation"),
        };
        let prompt = PromptBaseRequest {
            materials: &materials,
            requirements: "reply in her voice",
        };
        let llm = LlmBaseRequest { input: "hi" };
        let agent = AgentBaseRequest {
            task: "summarise the last three turns",
            context: None,
        };

        assert_eq!(memory.materials.len(), 2);
        assert_eq!(memory.query, "alpha");
        assert_eq!(emotion.material, "tired but calm");
        assert!(emotion.context.is_none());
        assert_eq!(event.material, "she said she may move next week");
        assert_eq!(event.context, Some("current conversation"));
        assert_eq!(prompt.requirements, "reply in her voice");
        assert_eq!(llm.input, "hi");
        assert_eq!(agent.task, "summarise the last three turns");
        assert!(agent.context.is_none());

        // Requests are borrow-only views: copying one must not move the material.
        let copied = memory;
        assert_eq!(copied.query, memory.query);
        assert_eq!(materials[1], "beta");
    }

    #[test]
    fn every_known_reason_is_distinguishable() {
        let kinds = all_kinds();
        for (i, left) in kinds.iter().enumerate() {
            for (j, right) in kinds.iter().enumerate() {
                if i == j {
                    assert_eq!(left, right);
                } else {
                    assert_ne!(left, right, "distinct reasons must not compare equal");
                }
            }
        }
    }

    #[test]
    fn display_names_the_reason_without_carrying_machine_state() {
        for kind in all_kinds() {
            let plain = BaseCallError { kind, detail: None };
            let described = BaseCallError {
                kind,
                detail: Some("any text at all".to_string()),
            };
            let empty_detail = BaseCallError {
                kind,
                detail: Some(String::new()),
            };
            assert_eq!(described.kind, kind);
            assert!(!plain.to_string().contains("any text at all"));
            // A detail string never changes the machine-readable reason.
            assert!(described.to_string().contains("any text at all"));
            assert_eq!(empty_detail.to_string(), plain.to_string());
        }

        let failed = BaseCallError {
            kind: BaseCallErrorKind::Failed,
            detail: None,
        };
        let cancelled = BaseCallError {
            kind: BaseCallErrorKind::Cancelled,
            detail: None,
        };
        let timed_out = BaseCallError {
            kind: BaseCallErrorKind::TimedOut,
            detail: None,
        };
        assert_ne!(failed.to_string(), cancelled.to_string());
        assert_ne!(failed.to_string(), timed_out.to_string());
        assert_ne!(cancelled.to_string(), timed_out.to_string());
    }

    #[test]
    fn known_cancellation_and_timeout_are_not_reported_as_plain_failure() {
        let cancelled = BaseCallError {
            kind: BaseCallErrorKind::Cancelled,
            detail: None,
        };
        let timed_out = BaseCallError {
            kind: BaseCallErrorKind::TimedOut,
            detail: None,
        };
        assert_ne!(cancelled.kind, BaseCallErrorKind::Failed);
        assert_ne!(timed_out.kind, BaseCallErrorKind::Failed);
        assert!(std::error::Error::source(&cancelled).is_none());
    }

    #[test]
    fn known_reasons_are_reported_verbatim() {
        // Inside the defining crate `#[non_exhaustive]` has no effect, so this match is
        // exhaustive on purpose: adding a reason must be a deliberate, visible change.
        // External consumers do need a wildcard branch — the contracts fixture covers that.
        fn describe(kind: BaseCallErrorKind) -> &'static str {
            match kind {
                BaseCallErrorKind::Failed => "failed",
                BaseCallErrorKind::Unavailable => "unavailable",
                BaseCallErrorKind::Unsupported => "unsupported",
                BaseCallErrorKind::Cancelled => "cancelled",
                BaseCallErrorKind::TimedOut => "timed out",
            }
        }
        assert_eq!(describe(BaseCallErrorKind::Cancelled), "cancelled");
        assert_eq!(describe(BaseCallErrorKind::TimedOut), "timed out");
        assert_eq!(describe(BaseCallErrorKind::Failed), "failed");
    }
}
