//! 提示词构建：[`PromptInput`] 等契约在 [`oclive_kernel_core::prompt`]；**开启** **`default-prompt-providers`** 时正文实现来自 **`oclive_prompt_builtin`**。

pub use oclive_kernel_core::prompt::{
    effective_reply_quality_anchor, PromptInput, PromptRolePromptSlice,
    DEFAULT_REPLY_QUALITY_ANCHOR,
};

#[cfg(feature = "default-prompt-providers")]
pub use oclive_prompt_builtin::PromptBuilder;

#[cfg(all(test, feature = "default-prompt-providers"))]
mod tests {
    use super::*;
    use crate::models::EventType;
    use crate::models::EvolutionBounds;
    use crate::models::PersonalitySource;
    use crate::models::Role;
    use chrono::Utc;

    fn create_test_role() -> Role {
        Role {
            id: "test".to_string(),
            name: "Test Role".to_string(),
            description: "A test role".to_string(),
            version: "1.0".to_string(),
            author: "Test".to_string(),
            core_personality: "Friendly and helpful".to_string(),
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
            default_relation: "friend".to_string(),
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

    fn create_test_personality() -> crate::models::PersonalityVector {
        crate::models::PersonalityVector {
            stubbornness: 0.4,
            clinginess: 0.6,
            sensitivity: 0.7,
            assertiveness: 0.5,
            forgiveness: 0.6,
            talkativeness: 0.6,
            warmth: 0.8,
        }
    }

    fn create_test_memory() -> crate::models::Memory {
        crate::models::Memory {
            id: "1".to_string(),
            role_id: "test".to_string(),
            content: "User likes coffee".to_string(),
            importance: 0.8,
            weight: 1.0,
            created_at: Utc::now(),
            scene_id: None,
        }
    }

    #[test]
    fn test_build_prompt() {
        let role = create_test_role();
        let personality = create_test_personality();
        let memories = vec![create_test_memory()];

        let prompt = PromptBuilder::build_prompt(&PromptInput {
            role_any: &role as &dyn std::any::Any,
            role_prompt: role.prompt_slice(),
            personality: &personality,
            memories: &memories,
            user_input: "Hello",
            user_emotion: "happy",
            user_relation_id: "friend",
            relation_hint: "你们是朋友。",
            relation_before: "Friend",
            favorability_before: 55.0,
            relation_preview: "CloseFriend",
            favorability_preview: 60.0,
            event_type: &EventType::Praise,
            impact_factor: 0.7,
            scene_label: "家",
            scene_detail: "客厅灯暖洋洋的，适合闲聊。",
            topic_hint_line: "在「家」下，你们可能会多聊日常。",
            life_context_line: "",
            worldview_snippet: "",
            mutable_personality: "",
            reply_quality_anchor: effective_reply_quality_anchor(role.prompt_slice()),
            complex_emotion_hint: None,
        });

        assert!(prompt.contains("Test Role"));
        assert!(prompt.contains("Hello"));
        assert!(prompt.contains("User likes coffee"));
        assert!(prompt.contains("用户身份"));
        assert!(prompt.contains("身份语气要点"));
        assert!(prompt.contains("当前关系"));
        assert!(!prompt.contains("家人/长辈场景补充"));
        assert!(prompt.contains("朋友"));
        assert!(prompt.contains("本轮事件与关系状态机"));
        assert!(prompt.contains("Friend"));
        assert!(prompt.contains("Praise"));
        assert!(prompt.contains("场景设定"));
        assert!(prompt.contains("客厅灯暖洋洋"));
        assert!(prompt.contains("用户语气线索"));
        assert!(prompt.contains("happy"));
        assert!(prompt.contains("【回复质量锚点】"));
        assert!(prompt.contains("禁止复述用户"));
        assert!(prompt.contains("状态延续"));
        assert!(prompt.contains("篇幅与节奏"));
        assert!(prompt.contains("倾诉优先"));
        assert!(prompt.contains("倾诉应对倾向"));
    }

    #[test]
    fn test_build_prompt_family_includes_guardrail_supplement() {
        let role = create_test_role();
        let personality = create_test_personality();
        let prompt = PromptBuilder::build_prompt(&PromptInput {
            role_any: &role as &dyn std::any::Any,
            role_prompt: role.prompt_slice(),
            personality: &personality,
            memories: &[],
            user_input: "嗯",
            user_emotion: "neutral",
            user_relation_id: "family",
            relation_hint: "以家人身份自然相处。",
            relation_before: "Friend",
            favorability_before: 50.0,
            relation_preview: "Friend",
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
        });

        assert!(prompt.contains("家人/长辈场景补充"));
        assert!(prompt.contains("身份语气要点"));
        assert!(prompt.contains("当前关系"));
    }

    #[test]
    fn test_build_simple_prompt() {
        let prompt = PromptBuilder::build_simple_prompt("TestBot", "Hi");
        assert!(prompt.contains("TestBot"));
        assert!(prompt.contains("Hi"));
    }

    #[test]
    fn test_build_system_prompt() {
        let prompt = PromptBuilder::build_system_prompt("TestBot");
        assert!(prompt.contains("TestBot"));
        assert!(prompt.contains("AI角色"));
    }

    #[test]
    fn test_build_guidance_prompt() {
        let prompt = PromptBuilder::build_guidance_prompt("Friendly");
        assert!(prompt.contains("Friendly"));
    }

    #[test]
    fn test_prompt_contains_personality() {
        let role = create_test_role();
        let personality = create_test_personality();
        let prompt = PromptBuilder::build_prompt(&PromptInput {
            role_any: &role as &dyn std::any::Any,
            role_prompt: role.prompt_slice(),
            personality: &personality,
            memories: &[],
            user_input: "test",
            user_emotion: "neutral",
            user_relation_id: "",
            relation_hint: "",
            relation_before: "Stranger",
            favorability_before: 0.0,
            relation_preview: "Stranger",
            favorability_preview: 0.0,
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
        });

        assert!(prompt.contains("倔强"));
        assert!(prompt.contains("温暖"));
    }

    #[test]
    fn test_prompt_without_memories() {
        let role = create_test_role();
        let personality = create_test_personality();
        let prompt = PromptBuilder::build_prompt(&PromptInput {
            role_any: &role as &dyn std::any::Any,
            role_prompt: role.prompt_slice(),
            personality: &personality,
            memories: &[],
            user_input: "test",
            user_emotion: "neutral",
            user_relation_id: "",
            relation_hint: "",
            relation_before: "Stranger",
            favorability_before: 0.0,
            relation_preview: "Stranger",
            favorability_preview: 0.0,
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
        });

        assert!(prompt.contains("用户说"));
        assert!(!prompt.contains("关于用户的记忆"));
    }

    #[test]
    fn boundary_tone_low_stage_high_constraint_contains_slow_warm_guidance() {
        let role = create_test_role();
        let cautious = crate::models::PersonalityVector {
            stubbornness: 0.1,
            clinginess: 0.1,
            sensitivity: 0.1,
            assertiveness: 0.1,
            forgiveness: 0.1,
            talkativeness: 0.1,
            warmth: 0.1,
        };
        let prompt = PromptBuilder::build_prompt(&PromptInput {
            role_any: &role as &dyn std::any::Any,
            role_prompt: role.prompt_slice(),
            personality: &cautious,
            memories: &[],
            user_input: "test",
            user_emotion: "neutral",
            user_relation_id: "",
            relation_hint: "",
            relation_before: "Acquaintance",
            favorability_before: 35.0,
            relation_preview: "Friend",
            favorability_preview: 41.0,
            event_type: &EventType::Praise,
            impact_factor: 0.3,
            scene_label: "",
            scene_detail: "",
            topic_hint_line: "",
            life_context_line: "",
            worldview_snippet: "",
            mutable_personality: "",
            reply_quality_anchor: effective_reply_quality_anchor(role.prompt_slice()),
            complex_emotion_hint: None,
        });

        assert!(prompt.contains("边界语气控制指引"));
        assert!(prompt.contains("慢热、谨慎"));
    }

    #[test]
    fn boundary_tone_low_stage_low_constraint_not_overly_stiff() {
        let role = create_test_role();
        let warm = crate::models::PersonalityVector {
            stubbornness: 0.9,
            clinginess: 0.9,
            sensitivity: 0.9,
            assertiveness: 0.9,
            forgiveness: 0.9,
            talkativeness: 0.9,
            warmth: 0.9,
        };
        let prompt = PromptBuilder::build_prompt(&PromptInput {
            role_any: &role as &dyn std::any::Any,
            role_prompt: role.prompt_slice(),
            personality: &warm,
            memories: &[],
            user_input: "test",
            user_emotion: "neutral",
            user_relation_id: "",
            relation_hint: "",
            relation_before: "Stranger",
            favorability_before: 10.0,
            relation_preview: "Stranger",
            favorability_preview: 12.0,
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
        });

        assert!(prompt.contains("边界语气控制指引"));
        assert!(prompt.contains("保持自然友好"));
        assert!(!prompt.contains("慢热、谨慎"));
    }

    #[test]
    fn boundary_tone_high_stage_not_hard_limited() {
        let role = create_test_role();
        let personality = create_test_personality();
        let prompt = PromptBuilder::build_prompt(&PromptInput {
            role_any: &role as &dyn std::any::Any,
            role_prompt: role.prompt_slice(),
            personality: &personality,
            memories: &[],
            user_input: "test",
            user_emotion: "neutral",
            user_relation_id: "",
            relation_hint: "",
            relation_before: "Friend",
            favorability_before: 50.0,
            relation_preview: "CloseFriend",
            favorability_preview: 66.0,
            event_type: &EventType::Praise,
            impact_factor: 0.5,
            scene_label: "",
            scene_detail: "",
            topic_hint_line: "",
            life_context_line: "",
            worldview_snippet: "",
            mutable_personality: "",
            reply_quality_anchor: effective_reply_quality_anchor(role.prompt_slice()),
            complex_emotion_hint: None,
        });

        assert!(!prompt.contains("边界语气控制指引"));
    }

    #[test]
    fn profile_mode_shows_mutable_and_summary_header() {
        let mut role = create_test_role();
        role.evolution_config.personality_source = PersonalitySource::Profile;
        let personality = create_test_personality();
        let prompt = PromptBuilder::build_prompt(&PromptInput {
            role_any: &role as &dyn std::any::Any,
            role_prompt: role.prompt_slice(),
            personality: &personality,
            memories: &[],
            user_input: "hi",
            user_emotion: "neutral",
            user_relation_id: "",
            relation_hint: "",
            relation_before: "Stranger",
            favorability_before: 0.0,
            relation_preview: "Stranger",
            favorability_preview: 0.0,
            event_type: &EventType::Ignore,
            impact_factor: 0.0,
            scene_label: "",
            scene_detail: "",
            topic_hint_line: "",
            life_context_line: "",
            worldview_snippet: "",
            mutable_personality: "最近更黏人了。",
            reply_quality_anchor: effective_reply_quality_anchor(role.prompt_slice()),
            complex_emotion_hint: None,
        });
        assert!(prompt.contains("【可变性格档案】"));
        assert!(prompt.contains("更黏人"));
        assert!(prompt.contains("【七维视图】"));
        assert!(prompt.contains("核心性格档案（创作者与用户设定"));
    }

    #[test]
    fn reply_quality_anchor_custom_overrides_default() {
        let mut role = create_test_role();
        role.reply_quality_anchor = Some("【包级质量锚点】仅测试覆盖用。".to_string());
        let personality = create_test_personality();
        let prompt = PromptBuilder::build_prompt(&PromptInput {
            role_any: &role as &dyn std::any::Any,
            role_prompt: role.prompt_slice(),
            personality: &personality,
            memories: &[],
            user_input: "hi",
            user_emotion: "neutral",
            user_relation_id: "",
            relation_hint: "",
            relation_before: "Stranger",
            favorability_before: 0.0,
            relation_preview: "Stranger",
            favorability_preview: 0.0,
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
        });
        assert!(prompt.contains("【包级质量锚点】仅测试覆盖用。"));
        assert!(!prompt.contains("【回复质量锚点】（每轮须遵守）"));
    }
}
