//! Prompt 组装可替换门面；默认委托 [`PromptBuilder`](super::prompt_builder::PromptBuilder)。

use crate::domain::prompt_builder::{PromptBuilder, PromptInput};
use crate::models::Role;
use std::sync::atomic::{AtomicBool, Ordering};

pub trait PromptAssembler: Send + Sync {
    fn build_prompt(&self, input: &PromptInput<'_>) -> String;
    fn top_topic_hint(&self, role: &Role, scene_id: &str) -> Option<String>;
}

pub struct BuiltinPromptAssembler;

impl PromptAssembler for BuiltinPromptAssembler {
    fn build_prompt(&self, input: &PromptInput<'_>) -> String {
        PromptBuilder::build_prompt(input)
    }

    fn top_topic_hint(&self, role: &Role, scene_id: &str) -> Option<String> {
        PromptBuilder::top_topic_hint(role, scene_id)
    }
}

/// 第二套内置：与 [`BuiltinPromptAssembler`] 相同逻辑，但在正文前追加固定前缀（可测差异）。
pub struct BuiltinPromptAssemblerV2;

const PROMPT_BACKEND_V2_PREFIX: &str = "[oclive:prompt:builtin_v2]\n";

impl PromptAssembler for BuiltinPromptAssemblerV2 {
    fn build_prompt(&self, input: &PromptInput<'_>) -> String {
        format!(
            "{}{}",
            PROMPT_BACKEND_V2_PREFIX,
            PromptBuilder::build_prompt(input)
        )
    }

    fn top_topic_hint(&self, role: &Role, scene_id: &str) -> Option<String> {
        PromptBuilder::top_topic_hint(role, scene_id)
    }
}

pub struct RemotePromptAssemblerPlaceholder {
    inner: BuiltinPromptAssembler,
    warned: AtomicBool,
}

impl RemotePromptAssemblerPlaceholder {
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
            log::warn!(
                target: "oclive_plugin",
                "prompt backend Remote is not connected; using builtin PromptBuilder"
            );
        }
    }
}

impl PromptAssembler for RemotePromptAssemblerPlaceholder {
    fn build_prompt(&self, input: &PromptInput<'_>) -> String {
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
mod tests {
    use super::*;
    use crate::domain::prompt_builder::{effective_reply_quality_anchor, PromptInput};
    use crate::models::{EventType, EvolutionBounds, Memory, PersonalityVector, Role};

    fn minimal_role() -> Role {
        Role {
            id: "t".into(),
            name: "T".into(),
            description: "".into(),
            version: "1".into(),
            author: "".into(),
            core_personality: ".".into(),
            default_personality: crate::models::PersonalityDefaults {
                stubbornness: 0.5,
                clinginess: 0.5,
                sensitivity: 0.5,
                assertiveness: 0.5,
                forgiveness: 0.5,
                talkativeness: 0.5,
                warmth: 0.5,
            },
            evolution_bounds: EvolutionBounds::full_01(),
            user_relations: vec![],
            evolution_config: crate::models::EvolutionConfig::default(),
            memory_config: None,
            default_relation: "friend".into(),
            ollama_model: None,
            identity_binding: crate::models::role::IdentityBinding::default(),
            life_trajectory: None,
            life_schedule: None,
            remote_presence: None,
            autonomous_scene: None,
            interaction_mode: None,
            min_runtime_version: None,
            dev_only: false,
            plugin_backends: crate::models::PluginBackends::default(),
            ui_config: crate::models::UiConfig::default(),
            knowledge_index: None,
            author_pack: None,
            reply_quality_anchor: None,
        }
    }

    #[test]
    fn builtin_v2_prefix_differs_from_builtin() {
        let role = minimal_role();
        let personality = PersonalityVector::zero();
        let memories: Vec<Memory> = vec![];
        let input = PromptInput {
            role: &role,
            personality: &personality,
            memories: &memories,
            user_input: "hi",
            user_emotion: "neutral",
            user_relation_id: "",
            relation_hint: "",
            relation_before: "Stranger",
            favorability_before: 50.0,
            relation_preview: "Stranger",
            favorability_preview: 50.0,
            event_type: &EventType::Ignore,
            impact_factor: 0.0,
            scene_label: "",
            scene_detail: "",
            topic_hint_line: "",
            life_context_line: "",
            worldview_snippet: "",
            mutable_personality: "",
            reply_quality_anchor: effective_reply_quality_anchor(&role),
        };
        let a = BuiltinPromptAssembler.build_prompt(&input);
        let b = BuiltinPromptAssemblerV2.build_prompt(&input);
        assert!(b.starts_with(super::PROMPT_BACKEND_V2_PREFIX));
        assert_eq!(b.len(), a.len() + super::PROMPT_BACKEND_V2_PREFIX.len());
    }
}
