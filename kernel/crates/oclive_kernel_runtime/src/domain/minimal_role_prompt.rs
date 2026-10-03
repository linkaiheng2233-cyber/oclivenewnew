//! A Prompt Base capability configured from a validated local minimal-role snapshot.
//!
//! This is an additive capability implementation, not the reference Host's role
//! activation path. The caller owns the snapshot, the current invocation, and any
//! choice to use this slot. Visual bytes remain available through that snapshot;
//! this text-only slot neither decodes nor renders them.

use oclive_kernel_contracts::{BaseCallFuture, PromptBase};
use oclive_kernel_types::{BaseCallError, BaseCallErrorKind, PromptBaseRequest};
use oclive_validation::minimal_role_local_file::LocalMinimalRoleSnapshot;

use super::base_prompt::concat_prepared_text;

const PERSONA_HEADING: &str = "【角色设定】\n";
const MATERIAL_HEADING: &str = "\n\n【输入材料】\n";

/// A borrowed Prompt Base implementation for one prepared local minimal role.
///
/// The local loader already checked the logical definition and snapshotted its
/// nonempty visual assets. No legacy `Role`, relation, personality vector, display
/// name, or slot registry is constructed. The caller must keep the snapshot alive.
pub struct LocalMinimalRolePrompt<'a> {
    snapshot: &'a LocalMinimalRoleSnapshot,
}

impl<'a> LocalMinimalRolePrompt<'a> {
    #[must_use]
    pub fn new(snapshot: &'a LocalMinimalRoleSnapshot) -> Self {
        Self { snapshot }
    }
}

impl PromptBase for LocalMinimalRolePrompt<'_> {
    /// Preserves the authored persona and material bytes in distinct text sections.
    ///
    /// The persona is configuration of this slot, not reference material supplied
    /// by the current call. Additional nonempty requirements are rejected because
    /// this implementation cannot prove that it satisfies them. Headings are only
    /// text delimiters; they do not guarantee downstream prompt-injection resistance.
    fn assemble<'a>(&'a self, request: PromptBaseRequest<'a>) -> BaseCallFuture<'a, String> {
        Box::pin(async move {
            if !request.requirements.is_empty() {
                return Err(BaseCallError {
                    kind: BaseCallErrorKind::Unsupported,
                    detail: Some(
                        "local minimal role prompt does not process additional requirements".into(),
                    ),
                });
            }
            let mut fragments = Vec::with_capacity(request.materials.len() + 3);
            fragments.push(PERSONA_HEADING);
            fragments.push(self.snapshot.definition().persona_prompt.as_str());
            fragments.push(MATERIAL_HEADING);
            fragments.extend_from_slice(request.materials);
            Ok(concat_prepared_text(&fragments))
        })
    }
}
