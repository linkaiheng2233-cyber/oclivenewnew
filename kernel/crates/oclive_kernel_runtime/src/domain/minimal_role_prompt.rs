//! A Prompt Base capability configured from portable minimal-role content.
//!
//! This is an additive capability implementation, not the reference Host's role
//! activation path. The reference Host's additive minimal text entry can use it;
//! the caller owns asset resolution, the current invocation, and
//! any choice to use this slot. This text-only slot never reads or renders assets.

use oclive_kernel_contracts::{BaseCallFuture, PromptBase};
use oclive_kernel_types::{
    BaseCallError, BaseCallErrorKind, MinimalRoleDefinition, PromptBaseRequest,
};
use oclive_validation::validate_minimal_role_definition;

use super::base_prompt::concat_prepared_text;

const PERSONA_HEADING: &str = "【角色设定】\n";
const MATERIAL_HEADING: &str = "\n\n【输入材料】\n";

/// A borrowed Prompt Base implementation for one validated logical minimal role.
///
/// The caller may obtain this definition from any adapter; the constructor checks
/// the shared minimal-role content requirements, but cannot establish that assets
/// exist or can be rendered. No legacy `Role`, relation, personality vector,
/// display name, or slot registry is constructed.
pub struct MinimalRolePrompt<'a> {
    definition: &'a MinimalRoleDefinition,
}

impl<'a> MinimalRolePrompt<'a> {
    /// Validate the shared logical contract without loading or decoding assets.
    ///
    /// # Errors
    ///
    /// Returns field/index diagnostics when the prompt or asset references are blank.
    pub fn new(definition: &'a MinimalRoleDefinition) -> Result<Self, Vec<String>> {
        validate_minimal_role_definition(definition)?;
        Ok(Self { definition })
    }
}

impl PromptBase for MinimalRolePrompt<'_> {
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
                        "minimal role prompt does not process additional requirements".into(),
                    ),
                });
            }
            let mut fragments = Vec::with_capacity(request.materials.len() + 3);
            fragments.push(PERSONA_HEADING);
            fragments.push(self.definition.persona_prompt.as_str());
            fragments.push(MATERIAL_HEADING);
            fragments.extend_from_slice(request.materials);
            Ok(concat_prepared_text(&fragments))
        })
    }
}
