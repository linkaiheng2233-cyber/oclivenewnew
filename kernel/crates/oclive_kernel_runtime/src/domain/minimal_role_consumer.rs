//! Shared minimal-role material preparation for a Host-selected Prompt Base.
//!
//! The Host supplies the current materials, requirements, and capability. This
//! consumer prepares the same persona/material sections as the reference minimal
//! prompt, then makes one call through the existing slot contract. It does not
//! choose other slots, resolve assets, load a legacy `Role`, or own Host state.

use oclive_kernel_contracts::{BaseCallFuture, PromptBase};
use oclive_kernel_types::{MinimalRoleDefinition, PromptBaseRequest};
use oclive_validation::validate_minimal_role_definition;

const PERSONA_HEADING: &str = "【角色设定】\n";
const MATERIAL_HEADING: &str = "\n\n【输入材料】\n";

/// A borrowed consumer that prepares a minimal role for a selected Prompt Base.
///
/// The prepared request adds three fragments: a persona heading, the authored
/// persona, and a material heading. Every current material remains a distinct
/// fragment with its original bytes and order; requirements pass through unchanged.
/// The selected capability owns assembly and requirement handling, so the Host must
/// select an implementation whose published agreement serves the current purpose.
/// Headings are text delimiters, not a downstream prompt-injection guarantee.
///
/// Normal results (including empty text) and complete errors pass through without
/// fallback, retry, or post-processing. No stronger scheduling or cancellation
/// guarantee than the existing local [`BaseCallFuture`] contract is introduced.
pub struct MinimalRolePromptConsumer<'a> {
    definition: &'a MinimalRoleDefinition,
    prompt: &'a dyn PromptBase,
}

impl<'a> MinimalRolePromptConsumer<'a> {
    /// Validate logical role content and borrow the Host's selected capability.
    ///
    /// Asset references are validated as content only; this does not establish
    /// that the referenced assets exist or can be rendered by the Host.
    ///
    /// # Errors
    ///
    /// Returns the shared field/index diagnostics for invalid minimal-role content.
    pub fn new(
        definition: &'a MinimalRoleDefinition,
        prompt: &'a dyn PromptBase,
    ) -> Result<Self, Vec<String>> {
        validate_minimal_role_definition(definition)?;
        Ok(Self { definition, prompt })
    }
}

impl PromptBase for MinimalRolePromptConsumer<'_> {
    fn assemble<'a>(&'a self, request: PromptBaseRequest<'a>) -> BaseCallFuture<'a, String> {
        assemble_minimal_role_materials(self.definition, self.prompt, request)
    }
}

// The legacy prompt keeps its concrete storage (and existing auto traits) while
// sharing preparation with the consumer. Both constructors validate the role.
pub(super) fn assemble_minimal_role_materials<'a>(
    definition: &'a MinimalRoleDefinition,
    prompt: &'a dyn PromptBase,
    request: PromptBaseRequest<'a>,
) -> BaseCallFuture<'a, String> {
    Box::pin(async move {
        let mut fragments = Vec::with_capacity(request.materials.len() + 3);
        fragments.push(PERSONA_HEADING);
        fragments.push(definition.persona_prompt.as_str());
        fragments.push(MATERIAL_HEADING);
        fragments.extend_from_slice(request.materials);
        prompt
            .assemble(PromptBaseRequest {
                materials: &fragments,
                requirements: request.requirements,
            })
            .await
    })
}
