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

use super::base_prompt::LiteralMaterialAssembler;
use super::minimal_role_consumer::assemble_minimal_role_materials;

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
            assemble_minimal_role_materials(self.definition, &LiteralMaterialAssembler, request)
                .await
        })
    }
}
