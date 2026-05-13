//! Prompt 组装可替换门面；**`PromptAssembler`** 定义于 [`oclive_kernel_core::prompt`]；
//! 开启 **`default-prompt-providers`** 时内置实现来自 **`oclive_prompt_builtin`**。

pub use oclive_kernel_core::prompt::PromptAssembler;

#[cfg(not(feature = "default-prompt-providers"))]
use crate::domain::disabled_default_providers::DisabledPromptAssembler;
use crate::domain::prompt_builder::PromptInput;
use oclive_kernel_core::prompt::TopicHintContext;
#[cfg(feature = "default-prompt-providers")]
pub use oclive_prompt_builtin::{
    BuiltinPromptAssembler, BuiltinPromptAssemblerV2, PROMPT_BACKEND_V2_PREFIX,
};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

#[must_use]
pub fn default_prompt_slot_v1() -> Arc<dyn PromptAssembler> {
    #[cfg(feature = "default-prompt-providers")]
    {
        Arc::new(BuiltinPromptAssembler)
    }
    #[cfg(not(feature = "default-prompt-providers"))]
    {
        Arc::new(DisabledPromptAssembler)
    }
}

#[must_use]
pub fn default_prompt_slot_v2() -> Arc<dyn PromptAssembler> {
    #[cfg(feature = "default-prompt-providers")]
    {
        Arc::new(BuiltinPromptAssemblerV2)
    }
    #[cfg(not(feature = "default-prompt-providers"))]
    {
        Arc::new(DisabledPromptAssembler)
    }
}

pub struct RemotePromptAssemblerPlaceholder {
    inner: Arc<dyn PromptAssembler>,
    warned: AtomicBool,
}

impl RemotePromptAssemblerPlaceholder {
    pub fn new() -> Self {
        Self {
            inner: default_prompt_slot_v1(),
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

    fn top_topic_hint(&self, ctx: &TopicHintContext<'_>, scene_id: &str) -> Option<String> {
        self.warn_once();
        self.inner.top_topic_hint(ctx, scene_id)
    }
}

impl Default for RemotePromptAssemblerPlaceholder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(all(test, feature = "default-prompt-providers"))]
mod tests {
    use super::*;
    use crate::domain::prompt_builder::{effective_reply_quality_anchor, PromptInput};
    use crate::models::{EventType, EvolutionBounds, Memory, PersonalityVector, Role};
    use std::any::Any;

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
            creator_message_to_downloader: None,
        }
    }

    #[test]
    fn builtin_v2_prefix_differs_from_builtin() {
        let role = minimal_role();
        let personality = PersonalityVector::zero();
        let memories: Vec<Memory> = vec![];
        let input = PromptInput {
            role_any: &role as &dyn Any,
            role_prompt: role.prompt_slice(),
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
            reply_quality_anchor: effective_reply_quality_anchor(role.prompt_slice()),
            complex_emotion_hint: None,
        };
        let a = BuiltinPromptAssembler.build_prompt(&input);
        let b = BuiltinPromptAssemblerV2.build_prompt(&input);
        assert!(b.starts_with(PROMPT_BACKEND_V2_PREFIX));
        assert_eq!(b.len(), a.len() + PROMPT_BACKEND_V2_PREFIX.len());
    }
}
