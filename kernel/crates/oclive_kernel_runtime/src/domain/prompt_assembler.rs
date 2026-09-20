//! Swappable prompt-assembler facade; default delegates to [`PromptBuilder`](super::prompt_builder::PromptBuilder).

use crate::domain::prompt_builder::{PromptBuilder, PromptInput};
use crate::error::Result;
use crate::models::Role;
use oclive_kernel_contracts::{BaseCallFuture, PromptBase};
use oclive_kernel_types::PromptBaseRequest;
use std::sync::atomic::{AtomicBool, Ordering};

pub use oclive_kernel_contracts::PromptAssembler;

/// The builtin product assembler.
///
/// CP-B3-ALL unit P: besides the legacy [`PromptAssembler`] entry it also serves the Base view
/// through the crate's one literal-assembly core
/// (`super::base_prompt::assemble_literal_material`) — the same core
/// [`LiteralMaterialAssembler`](super::base_prompt::LiteralMaterialAssembler) uses, and the same
/// connection rule the product layout now applies to its prepared blocks. The Base view accepts
/// only an empty `requirements` and concatenates what it is given, so the rich product preparation
/// (`Role`, persona, relation, guardrails, segments, prefix cache) stays in the product entries and
/// is neither replaced nor re-derived here.
pub struct BuiltinPromptAssembler;

impl PromptAssembler for BuiltinPromptAssembler {
    fn build_prompt(&self, input: &PromptInput<'_>) -> Result<String> {
        Ok(PromptBuilder::build_prompt(input))
    }

    fn top_topic_hint(&self, role: &Role, scene_id: &str) -> Option<String> {
        PromptBuilder::top_topic_hint(role, scene_id)
    }
}

impl PromptBase for BuiltinPromptAssembler {
    /// Concatenates the supplied material verbatim when `requirements` is empty.
    ///
    /// # Errors
    ///
    /// Returns [`oclive_kernel_types::BaseCallErrorKind::Unsupported`] for any non-empty
    /// `requirements` — including white space only — because this Base view does not process
    /// additional assembly requirements. It has no other failure source: no I/O, no model call, no
    /// retry, and it does not read the product configuration.
    fn assemble<'a>(&'a self, request: PromptBaseRequest<'a>) -> BaseCallFuture<'a, String> {
        Box::pin(async move {
            crate::domain::base_prompt::assemble_literal_material(
                request.materials,
                request.requirements,
            )
        })
    }
}

pub struct RemotePromptAssemblerPlaceholder {
    inner: BuiltinPromptAssembler,
    warned: AtomicBool,
}

impl RemotePromptAssemblerPlaceholder {
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: BuiltinPromptAssembler,
            warned: AtomicBool::new(false),
        }
    }

    fn warn_once(&self) {
        if self
            .warned
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
        {
            tracing::warn!(
                target: "oclive_plugin",
                "prompt backend Remote is not connected; using builtin PromptBuilder"
            );
        }
    }
}

impl PromptAssembler for RemotePromptAssemblerPlaceholder {
    fn build_prompt(&self, input: &PromptInput<'_>) -> Result<String> {
        self.warn_once();
        self.inner.build_prompt(input)
    }

    fn top_topic_hint(&self, role: &Role, scene_id: &str) -> Option<String> {
        self.warn_once();
        self.inner.top_topic_hint(role, scene_id)
    }
}

impl Default for RemotePromptAssemblerPlaceholder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod cp_b3_all_prompt_tests {
    use std::task::{Context, Poll, Waker};

    use super::*;
    use crate::domain::base_prompt::LiteralMaterialAssembler;

    fn drive_base(
        slot: &dyn PromptBase,
        materials: &[&str],
        requirements: &str,
    ) -> std::result::Result<String, oclive_kernel_types::BaseCallError> {
        let request = PromptBaseRequest {
            materials,
            requirements,
        };
        let mut future = slot.assemble(request);
        let waker = Waker::noop();
        let mut cx = Context::from_waker(waker);
        match future.as_mut().poll(&mut cx) {
            Poll::Ready(outcome) => outcome,
            Poll::Pending => panic!("this implementation has no pending path"),
        }
    }

    /// CP-B3-ALL unit P: the product assembler serves the Base view through the same core as the
    /// reference literal assembler — same bytes, same requirement agreement, one implementation.
    #[test]
    fn cp_b3_all_prompt_builtin_base_entry_shares_the_reference_core() {
        let builtin = BuiltinPromptAssembler;
        let reference = LiteralMaterialAssembler;
        for materials in [
            &["a"][..],
            &[""][..],
            &["b", "a", "b"][..],
            &["ab", "c"][..],
            &["核心\r\n", "第二行\u{3000}"][..],
        ] {
            assert_eq!(
                drive_base(&builtin, materials, "").expect("empty requirements"),
                drive_base(&reference, materials, "").expect("empty requirements"),
                "{materials:?}: the two Base entries must not drift"
            );
        }
        for requirements in ["逐字保留", " ", "\u{3000}"] {
            let builtin_error = drive_base(&builtin, &["A"], requirements)
                .expect_err("non-empty requirements must not assemble");
            let reference_error = drive_base(&reference, &["A"], requirements)
                .expect_err("non-empty requirements must not assemble");
            assert_eq!(builtin_error.kind, reference_error.kind);
            assert_eq!(builtin_error.detail, reference_error.detail);
        }
        // The legacy entry still needs a real product input; that path is covered by the
        // prompt_builder baseline suite and the external target, not by a fabricated here.
        assert!(drive_base(&builtin, &[], "")
            .expect("empty call")
            .is_empty());
    }
}
