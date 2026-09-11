// Private builtin compatibility projection only; this is not a Kernel public contract.
use crate::models::PersonalitySource;

use super::PromptInput;

/// Borrowed role values selected for builtin prompt rendering.
pub(super) struct RolePromptContext<'a> {
    name: &'a str,
    persona: Option<&'a str>,
    personality_source: PersonalitySource,
    description: &'a str,
    relation_label: Option<&'a str>,
}

impl<'a> RolePromptContext<'a> {
    pub(super) fn from_input(input: &'a PromptInput<'a>) -> Self {
        let persona = input
            .persona_override
            .filter(|text| !text.trim().is_empty())
            .map(str::trim)
            .or_else(|| {
                (!input.role.core_personality.trim().is_empty())
                    .then_some(input.role.core_personality.trim())
            });
        let relation_label = if input.user_relation_id.is_empty() {
            None
        } else {
            Some(
                input
                    .role
                    .user_relations
                    .iter()
                    .find(|relation| relation.id == input.user_relation_id)
                    .map(|relation| relation.name.as_str())
                    .unwrap_or(input.user_relation_id),
            )
        };

        Self {
            name: input.role.name.as_str(),
            persona,
            personality_source: input.role.evolution_config.personality_source,
            description: input.role.description.as_str(),
            relation_label,
        }
    }

    pub(super) fn name(&self) -> &str {
        self.name
    }

    pub(super) fn persona(&self) -> Option<&str> {
        self.persona
    }

    pub(super) fn personality_source(&self) -> PersonalitySource {
        self.personality_source
    }

    pub(super) fn description(&self) -> &str {
        self.description
    }

    pub(super) fn relation_label(&self) -> Option<&str> {
        self.relation_label
    }
}
