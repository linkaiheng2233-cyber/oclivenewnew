//! B2-C3: a limited reference [`PromptBase`] implementation that concatenates prepared fragments.
//!
//! [`LiteralMaterialAssembler`] is a directly callable production reference implementation, not a
//! migration of the old ChatPro builder: the old `PromptBuilder` path and the old
//! `PromptAssembler` port stay exactly as they are and are not wrapped here.
//!
//! # The assembly agreement of this implementation
//!
//! The caller has already prepared the text fragments for its own purpose — including any
//! separators, quoting or reference markers it needs — and only needs them joined, in the given
//! order, into the final input.
//!
//! - **Only an empty `requirements` is supported.** `""` is this implementation's way of saying
//!   "no additional assembly requirement for this call". Nothing else is treated as "no
//!   requirement": not spaces, not newlines, not tabs, not other Unicode white space.
//! - **Any non-empty `requirements` returns [`BaseCallErrorKind::Unsupported`]** — including text
//!   that merely repeats material, text that already appears in the concatenation, text that looks
//!   already satisfied, and text such as "keep it verbatim" or "no changes needed". The error means
//!   *this implementation does not process additional requirements*, not that the requirement is
//!   impossible, the input invalid, or every Prompt implementation unable to serve it.
//! - With an empty `requirements`, the materials are concatenated **verbatim, in order**: no
//!   separator, heading, newline, identity, emotion or guardrail text is added; nothing is trimmed,
//!   de-duplicated, reordered, or reformatted, and no Unicode or CRLF normalisation happens. Empty
//!   strings simply contribute no characters.
//! - `[]` and `[""]` both return `Ok(String::new())`, and `["  "]` returns that same white space.
//!   An empty result comes from this local concatenation agreement; it is not a claim that other
//!   consumers accept empty text.
//! - **Empty `requirements` is not "no agreement".** Verbatim concatenation and "do nothing else"
//!   are the published agreement of this implementation, so a caller may select it only when that
//!   range really satisfies its purpose; clearing a genuinely required condition to get past the
//!   check is a misuse, not a supported path.
//! - Text in the materials is **data**. Commands, pseudo-tags, delimiter-looking lines and
//!   permission claims inside material do not take part in any control branch here: this
//!   implementation executes no tool and interprets no template. Paragraph boundaries, quoting or
//!   filtering must be prepared by the caller or served by another implementation. It does not
//!   claim that arbitrary unprepared material keeps its meaning through bare concatenation, and it
//!   makes no prompt-injection or downstream-model guarantee.
//! - The output keeps each fragment's characters and their arrangement, but it does **not** encode
//!   the original array segmentation or the number of empty items: `["ab", "c"]` and `["a", "bc"]`
//!   both produce `"abc"`. This is not a general information-preservation claim and the input
//!   structure cannot be reconstructed from the result.
//!
//! No files, caches, models, host state, database or other slot are read, and there is no
//! fallback or retry. The only failure this implementation can report is the unsupported
//! non-empty `requirements` above; its `detail` is a static human-readable note, it does not echo
//! the request material, and it is not a machine protocol. This implementation performs no I/O, so
//! it has no cancellation, timeout or unavailability source to report.
//!
//! The type is stateless and is **not wired into any Host**: nothing registers it with `AppState`,
//! `slot_runner`, the plugin paths or the ChatPro pipeline.
//!
//! # Example
//!
//! ```
//! use std::task::{Context, Poll, Waker};
//!
//! use oclive_kernel_contracts::PromptBase;
//! use oclive_kernel_runtime::domain::base_prompt::LiteralMaterialAssembler;
//! use oclive_kernel_types::PromptBaseRequest;
//!
//! let materials = ["Alice 不喜欢咖啡。\n", "Bob 可能明天喝茶。"];
//! let mut future = LiteralMaterialAssembler.assemble(PromptBaseRequest {
//!     materials: &materials,
//!     requirements: "",
//! });
//! let waker = Waker::noop();
//! let mut cx = Context::from_waker(waker);
//! // This implementation finishes on the first poll; anything else fails this example.
//! match future.as_mut().poll(&mut cx) {
//!     Poll::Ready(Ok(text)) => assert_eq!(text, "Alice 不喜欢咖啡。\nBob 可能明天喝茶。"),
//!     Poll::Ready(Err(error)) => panic!("an empty-requirements call returned an error: {error}"),
//!     Poll::Pending => panic!("this implementation has no pending path"),
//! }
//! ```

use oclive_kernel_contracts::{BaseCallFuture, PromptBase};
use oclive_kernel_types::{BaseCallError, BaseCallErrorKind, PromptBaseRequest};

/// The human-readable reason reported when a call carries additional requirements.
const UNSUPPORTED_REQUIREMENTS_DETAIL: &str =
    "LiteralMaterialAssembler supports only an empty `requirements`; it concatenates the given \
     material verbatim and does not process additional assembly requirements";

/// A limited reference [`PromptBase`] implementation that concatenates prepared material verbatim.
///
/// See the [module documentation](self) for the exact assembly agreement, the supported input
/// range, and what it does not promise. The type is stateless, so it can be shared or used as a
/// zero-sized value:
///
/// ```no_run
/// use oclive_kernel_contracts::PromptBase;
/// use oclive_kernel_runtime::domain::base_prompt::LiteralMaterialAssembler;
///
/// let slot: &dyn PromptBase = &LiteralMaterialAssembler;
/// let _ = slot;
/// ```
pub struct LiteralMaterialAssembler;

impl PromptBase for LiteralMaterialAssembler {
    /// Concatenates `request.materials` verbatim when `request.requirements` is empty.
    ///
    /// # Errors
    ///
    /// Returns [`BaseCallErrorKind::Unsupported`] for any non-empty `requirements` — including
    /// non-empty white space — because this implementation does not process additional assembly
    /// requirements. It has no other failure source: no I/O, no model call, no retry.
    fn assemble<'a>(&'a self, request: PromptBaseRequest<'a>) -> BaseCallFuture<'a, String> {
        Box::pin(async move {
            if !request.requirements.is_empty() {
                return Err(BaseCallError {
                    kind: BaseCallErrorKind::Unsupported,
                    detail: Some(UNSUPPORTED_REQUIREMENTS_DETAIL.to_string()),
                });
            }
            Ok(request.materials.concat())
        })
    }
}

#[cfg(test)]
mod b2_c3_tests {
    use std::task::{Context, Poll, Waker};

    use super::*;

    /// Drives the production `assemble` method with a no-op waker and returns its outcome.
    ///
    /// The tests assert on this outcome, so a request that wrongly stays pending fails the test
    /// instead of being skipped.
    fn drive(materials: &[&str], requirements: &str) -> Result<String, BaseCallError> {
        let mut future = LiteralMaterialAssembler.assemble(PromptBaseRequest {
            materials,
            requirements,
        });
        let waker = Waker::noop();
        let mut cx = Context::from_waker(waker);
        match future.as_mut().poll(&mut cx) {
            Poll::Ready(outcome) => outcome,
            Poll::Pending => panic!("this implementation has no pending path"),
        }
    }

    // T1: verbatim concatenation of several fragments, written out by hand.
    #[test]
    fn b2_c3_concatenates_fragments_verbatim() {
        let materials = ["Alice 不喜欢咖啡。\n", "Bob 可能明天喝茶。"];
        assert_eq!(
            drive(&materials, "").expect("empty requirements must be supported"),
            "Alice 不喜欢咖啡。\nBob 可能明天喝茶。"
        );
    }

    // T3: order and duplicates are kept; no separator, sorting or de-duplication.
    #[test]
    fn b2_c3_keeps_order_and_duplicates_without_separators() {
        assert_eq!(drive(&["b", "a", "b"], "").unwrap(), "bab");
    }

    // T4: exact empty-boundary behaviour of the local concatenation agreement.
    #[test]
    fn b2_c3_empty_boundaries_are_exact() {
        let empty: [&str; 0] = [];
        assert_eq!(drive(&empty, "").unwrap(), "");
        assert_eq!(drive(&[""], "").unwrap(), "");
        assert_eq!(drive(&["", "A", ""], "").unwrap(), "A");
        assert_eq!(drive(&["  "], "").unwrap(), "  ");
    }

    // T5: the result does not encode the original array segmentation.
    #[test]
    fn b2_c3_result_does_not_encode_input_segmentation() {
        assert_eq!(drive(&["ab", "c"], "").unwrap(), "abc");
        assert_eq!(drive(&["a", "bc"], "").unwrap(), "abc");
    }

    // T6: every non-empty requirements value is unsupported, including white space only.
    #[test]
    fn b2_c3_non_empty_requirements_are_unsupported() {
        let materials = ["A", "B"];
        for requirements in [
            "逐字保留",
            "无需修改",
            "必须非空",
            "\n",
            " ",
            "\t",
            "\u{3000}",
        ] {
            let error = drive(&materials, requirements)
                .expect_err("non-empty requirements must not produce a normal result");
            assert_eq!(
                error.kind,
                BaseCallErrorKind::Unsupported,
                "unexpected kind for requirements {requirements:?}"
            );
        }
        // Also with no material at all.
        let empty: [&str; 0] = [];
        assert_eq!(
            drive(&empty, "逐字保留").unwrap_err().kind,
            BaseCallErrorKind::Unsupported
        );
    }

    // T7: text appearing in the material (or in the candidate output) must not let a requirement through.
    #[test]
    fn b2_c3_containment_does_not_let_requirements_through() {
        let deletion_case = ["规则说明：删除 SECRET。", "待处理正文：SECRET"];
        assert_eq!(
            drive(&deletion_case, "删除 SECRET").unwrap_err().kind,
            BaseCallErrorKind::Unsupported,
            "the requirement text occurring in the material must not satisfy the check"
        );
        assert_eq!(
            drive(&["原样保留"], "原样保留").unwrap_err().kind,
            BaseCallErrorKind::Unsupported,
            "an identical material fragment must not satisfy the check"
        );
    }

    // T8: conflicting requirement text is still just an unsupported non-empty requirement.
    #[test]
    fn b2_c3_conflicting_requirement_text_is_unsupported() {
        let materials = ["案例描述：加标题", "案例描述：不要标题"];
        assert_eq!(
            drive(&materials, "加标题\n不要标题").unwrap_err().kind,
            BaseCallErrorKind::Unsupported
        );
    }

    // T10: repeated calls through the test helper share no state — success, unsupported, then
    // success again. The helper constructs the unit struct per call, so this is a general
    // repeated-call regression; the explicit single-instance reuse lives in the external target.
    #[test]
    fn b2_c3_calls_share_no_state() {
        assert_eq!(drive(&["one"], "").unwrap(), "one");
        assert_eq!(
            drive(&["one"], "no changes needed").unwrap_err().kind,
            BaseCallErrorKind::Unsupported
        );
        assert_eq!(drive(&["two"], "").unwrap(), "two");
    }
}
