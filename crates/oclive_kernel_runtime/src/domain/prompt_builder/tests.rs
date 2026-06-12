use super::*;
use crate::models::EventType;
use crate::models::EvolutionBounds;
use crate::models::PersonalitySource;
use crate::models::{Memory, PersonalityVector, Role};
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
        featured: false,
        preset_order: 0,
        plugin_backends: std::sync::Arc::new(crate::models::PluginBackends::default()),
        slot_registry: None,
        slot_groups: None,
        ui_config: crate::models::UiConfig::default(),
        knowledge_index: None,
        author_pack: None,
        reply_quality_anchor: None,
        time_config: Default::default(),
        pack_memory_config: Default::default(),
        pack_relation_config: Default::default(),
        pack_evolution_config: Default::default(),
        pack_chat_storage_config: Default::default(),
        pack_portrait_catalog: Default::default(),
        portrait_catalog: None,
        pack_visual_presentation_config: Default::default(),
        pack_reply_post_processor_config: Default::default(),
        user_identity_catalog: None,
        runtime_config: None,
        pipeline_experimental: None,
        scene_ids: std::sync::Arc::from(Vec::<String>::new()),
        scene_config_cache: std::sync::Arc::new(parking_lot::RwLock::new(
            std::collections::HashMap::new(),
        )),
        scene_text_cache: std::sync::Arc::new(parking_lot::RwLock::new(
            std::collections::HashMap::new(),
        )),
    }
}

fn create_test_personality() -> PersonalityVector {
    PersonalityVector {
        stubbornness: 0.4,
        clinginess: 0.6,
        sensitivity: 0.7,
        assertiveness: 0.5,
        forgiveness: 0.6,
        talkativeness: 0.6,
        warmth: 0.8,
    }
}

fn create_test_memory() -> Memory {
    Memory {
        id: "1".to_string(),
        role_id: "test".to_string(),
        content: "User likes coffee".to_string(),
        importance: 0.8,
        weight: 1.0,
        created_at: Utc::now(),
        scene_id: None,
        mention_count: 1,
        accessed_at: None,
    }
}

#[test]
fn test_build_prompt() {
    let role = create_test_role();
    let personality = create_test_personality();
    let memories = vec![create_test_memory()];

    let prompt = PromptBuilder::build_prompt(&PromptInput {
        role: &role,
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
        reply_quality_anchor: effective_reply_quality_anchor(&role),
        previous_complex_emotion_narrative_hint: "",
        user_identity_template: "",
        user_identity_id: "",
        host_prompt_overlay: "",
        host_state_expression_hint: "",
        relation_transition_hint: "",
        extra_sections: &[],
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
    assert!(prompt.contains("赞美"));
    assert!(!prompt.contains("Praise"));
    assert!(prompt.contains("场景设定"));
    assert!(prompt.contains("客厅灯暖洋洋"));
    assert!(prompt.contains("用户语气线索"));
    assert!(prompt.contains("happy"));
    assert!(prompt.contains("【回复质量锚点】"));
    assert!(prompt.contains("用全新措辞接住用户本句的内容或情绪"));
    assert!(prompt.contains("状态延续"));
    assert!(prompt.contains("篇幅随输入"));
    assert!(!prompt.contains("篇幅与节奏"));
    assert!(prompt.contains("倾诉优先"));
    assert!(prompt.contains("倾诉应对倾向"));
    assert!(prompt.contains("【对话硬约束】"));
    assert!(prompt.contains("回复篇幅倾向"));
    assert!(!prompt.contains("【回复结构】"));
    assert!(!prompt.contains("影响因子(已归一)"));
    assert!(!prompt.contains("warmup_level="));
}

#[test]
fn test_build_prompt_family_includes_guardrail_supplement() {
    let role = create_test_role();
    let personality = create_test_personality();
    let prompt = PromptBuilder::build_prompt(&PromptInput {
        role: &role,
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
        reply_quality_anchor: effective_reply_quality_anchor(&role),
        previous_complex_emotion_narrative_hint: "",
        user_identity_template: "",
        user_identity_id: "",
        host_prompt_overlay: "",
        host_state_expression_hint: "",
        relation_transition_hint: "",
        extra_sections: &[],
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
        role: &role,
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
        reply_quality_anchor: effective_reply_quality_anchor(&role),
        previous_complex_emotion_narrative_hint: "",
        user_identity_template: "",
        user_identity_id: "",
        host_prompt_overlay: "",
        host_state_expression_hint: "",
        relation_transition_hint: "",
        extra_sections: &[],
    });

    assert!(prompt.contains("倔强"));
    assert!(prompt.contains("温暖"));
}

#[test]
fn test_prompt_without_memories() {
    let role = create_test_role();
    let personality = create_test_personality();
    let prompt = PromptBuilder::build_prompt(&PromptInput {
        role: &role,
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
        reply_quality_anchor: effective_reply_quality_anchor(&role),
        previous_complex_emotion_narrative_hint: "",
        user_identity_template: "",
        user_identity_id: "",
        host_prompt_overlay: "",
        host_state_expression_hint: "",
        relation_transition_hint: "",
        extra_sections: &[],
    });

    assert!(prompt.contains("用户说"));
    assert!(!prompt.contains("关于用户的记忆"));
}

#[test]
fn boundary_tone_low_stage_high_constraint_contains_slow_warm_guidance() {
    let role = create_test_role();
    let cautious = PersonalityVector {
        stubbornness: 0.1,
        clinginess: 0.1,
        sensitivity: 0.1,
        assertiveness: 0.1,
        forgiveness: 0.1,
        talkativeness: 0.1,
        warmth: 0.1,
    };
    let prompt = PromptBuilder::build_prompt(&PromptInput {
        role: &role,
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
        reply_quality_anchor: effective_reply_quality_anchor(&role),
        previous_complex_emotion_narrative_hint: "",
        user_identity_template: "",
        user_identity_id: "",
        host_prompt_overlay: "",
        host_state_expression_hint: "",
        relation_transition_hint: "",
        extra_sections: &[],
    });

    assert!(prompt.contains("边界语气控制指引"));
    assert!(prompt.contains("慢热、谨慎"));
}

#[test]
fn boundary_tone_low_stage_low_constraint_not_overly_stiff() {
    let role = create_test_role();
    let warm = PersonalityVector {
        stubbornness: 0.9,
        clinginess: 0.9,
        sensitivity: 0.9,
        assertiveness: 0.9,
        forgiveness: 0.9,
        talkativeness: 0.9,
        warmth: 0.9,
    };
    let prompt = PromptBuilder::build_prompt(&PromptInput {
        role: &role,
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
        reply_quality_anchor: effective_reply_quality_anchor(&role),
        previous_complex_emotion_narrative_hint: "",
        user_identity_template: "",
        user_identity_id: "",
        host_prompt_overlay: "",
        host_state_expression_hint: "",
        relation_transition_hint: "",
        extra_sections: &[],
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
        role: &role,
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
        reply_quality_anchor: effective_reply_quality_anchor(&role),
        previous_complex_emotion_narrative_hint: "",
        user_identity_template: "",
        user_identity_id: "",
        host_prompt_overlay: "",
        host_state_expression_hint: "",
        relation_transition_hint: "",
        extra_sections: &[],
    });

    assert!(!prompt.contains("边界语气控制指引"));
}

#[test]
fn profile_mode_shows_mutable_and_summary_header() {
    let mut role = create_test_role();
    role.evolution_config.personality_source = PersonalitySource::Profile;
    let personality = create_test_personality();
    let prompt = PromptBuilder::build_prompt(&PromptInput {
        role: &role,
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
        reply_quality_anchor: effective_reply_quality_anchor(&role),
        previous_complex_emotion_narrative_hint: "",
        user_identity_template: "",
        user_identity_id: "",
        host_prompt_overlay: "",
        host_state_expression_hint: "",
        relation_transition_hint: "",
        extra_sections: &[],
    });
    assert!(prompt.contains("【可变性格档案】"));
    assert!(prompt.contains("更黏人"));
    assert!(prompt.contains("【七维视图】"));
    assert!(prompt.contains("核心性格档案（创作者与用户设定"));
}

#[test]
fn default_reply_quality_anchor_and_guardrails_constants_present() {
    assert!(DEFAULT_REPLY_QUALITY_ANCHOR.contains("【回复质量锚点】"));
    assert!(DEFAULT_REPLY_QUALITY_ANCHOR.contains("用全新措辞接住用户本句的内容或情绪"));
    assert!(!DEFAULT_REPLY_QUALITY_ANCHOR.contains("状态延续"));
    assert!(!DEFAULT_REPLY_QUALITY_ANCHOR.contains("倾诉优先"));
    assert!(KERNEL_DIALOGUE_GUARDRAILS.contains("【对话硬约束】"));
    assert!(KERNEL_DIALOGUE_GUARDRAILS.contains("禁止复读开场"));
    assert!(KERNEL_DIALOGUE_GUARDRAILS.contains("状态延续"));
    assert!(KERNEL_DIALOGUE_GUARDRAILS.contains("倾诉优先"));
}

#[test]
fn reply_quality_anchor_custom_overrides_default() {
    let mut role = create_test_role();
    role.reply_quality_anchor = Some("【包级质量锚点】仅测试覆盖用。".to_string());
    let personality = create_test_personality();
    let prompt = PromptBuilder::build_prompt(&PromptInput {
        role: &role,
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
        reply_quality_anchor: effective_reply_quality_anchor(&role),
        previous_complex_emotion_narrative_hint: "",
        user_identity_template: "",
        user_identity_id: "",
        host_prompt_overlay: "",
        host_state_expression_hint: "",
        relation_transition_hint: "",
        extra_sections: &[],
    });
    assert!(prompt.contains("【包级质量锚点】仅测试覆盖用。"));
    assert!(!prompt.contains("【回复质量锚点】（每轮须遵守）"));
    assert!(prompt.contains("【对话硬约束】"));
    assert!(prompt.contains("禁止复读开场"));
    assert!(prompt.contains("状态延续"));
    assert!(prompt.contains("倾诉优先"));
}

#[test]
fn empty_narrative_hint_skips_section() {
    let role = create_test_role();
    let personality = create_test_personality();
    let prompt = PromptBuilder::build_prompt(&PromptInput {
        role: &role,
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
        reply_quality_anchor: effective_reply_quality_anchor(&role),
        previous_complex_emotion_narrative_hint: "   \n  ",
        user_identity_template: "",
        user_identity_id: "",
        host_prompt_overlay: "",
        host_state_expression_hint: "",
        relation_transition_hint: "",
        extra_sections: &[],
    });
    assert!(!prompt.contains("【复杂情感叙事提示】"));
    assert!(prompt.contains("用户说: hi"));
}

#[test]
fn special_chars_in_narrative_hint_preserve_prompt_structure() {
    let role = create_test_role();
    let personality = create_test_personality();
    let hint = "引号\"与\n换行\n**markdown** `_未闭合";
    let prompt = PromptBuilder::build_prompt(&PromptInput {
        role: &role,
        personality: &personality,
        memories: &[],
        user_input: "after",
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
        reply_quality_anchor: effective_reply_quality_anchor(&role),
        previous_complex_emotion_narrative_hint: hint,
        user_identity_template: "",
        user_identity_id: "",
        host_prompt_overlay: "",
        host_state_expression_hint: "",
        relation_transition_hint: "",
        extra_sections: &[],
    });
    assert!(prompt.contains("【复杂情感叙事提示】"));
    assert!(prompt.contains("引号\""));
    assert!(prompt.contains("**markdown**"));
    let user_idx = prompt.find("用户说: after").expect("user section");
    let section_idx = prompt.find("【复杂情感叙事提示】").expect("section");
    assert!(
        section_idx < user_idx,
        "narrative section must precede user line"
    );
}

#[test]
fn prompt_section_order_core_first() {
    let role = create_test_role();
    let personality = create_test_personality();
    let prompt = PromptBuilder::build_prompt(&PromptInput {
        role: &role,
        personality: &personality,
        memories: &[],
        user_input: "hi",
        user_emotion: "neutral",
        user_relation_id: "friend",
        relation_hint: "朋友",
        relation_before: "Friend",
        favorability_before: 55.0,
        relation_preview: "Friend",
        favorability_preview: 55.0,
        event_type: &EventType::Ignore,
        impact_factor: 0.0,
        scene_label: "家",
        scene_detail: "客厅",
        topic_hint_line: "话题",
        life_context_line: "",
        worldview_snippet: "",
        mutable_personality: "",
        reply_quality_anchor: effective_reply_quality_anchor(&role),
        previous_complex_emotion_narrative_hint: "",
        user_identity_template: "",
        user_identity_id: "",
        host_prompt_overlay: "",
        host_state_expression_hint: "",
        relation_transition_hint: "",
        extra_sections: &[],
    });
    let core_idx = prompt
        .find("【核心设定·不可违背】")
        .expect("core section");
    for title in [
        "【当前场景约束】",
        "【用户身份】",
        "【角色当前状态】",
        "【回复质量锚点】",
    ] {
        if let Some(idx) = prompt.find(title) {
            assert!(
                core_idx < idx,
                "{title} must follow core; core={core_idx} other={idx}"
            );
        }
    }
}

#[test]
fn prompt_three_blocks_present() {
    let role = create_test_role();
    let personality = create_test_personality();
    let prompt = PromptBuilder::build_prompt(&PromptInput {
        role: &role,
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
        reply_quality_anchor: effective_reply_quality_anchor(&role),
        previous_complex_emotion_narrative_hint: "",
        user_identity_template: "",
        user_identity_id: "",
        host_prompt_overlay: "",
        host_state_expression_hint: "",
        relation_transition_hint: "",
        extra_sections: &[],
    });
    let bottom = prompt.find("底线区块").expect("bottom block");
    let tone = prompt.find("语气区块").expect("tone block");
    let content = prompt.find("内容区块").expect("content block");
    assert!(bottom < tone && tone < content);
}

#[test]
fn prompt_scene_constraint_after_core() {
    let role = create_test_role();
    let personality = create_test_personality();
    let prompt = PromptBuilder::build_prompt(&PromptInput {
        role: &role,
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
        scene_label: "VS Code",
        scene_detail: "结对编程",
        topic_hint_line: "",
        life_context_line: "",
        worldview_snippet: "",
        mutable_personality: "",
        reply_quality_anchor: effective_reply_quality_anchor(&role),
        previous_complex_emotion_narrative_hint: "",
        user_identity_template: "",
        user_identity_id: "",
        host_prompt_overlay: "",
        host_state_expression_hint: "",
        relation_transition_hint: "",
        extra_sections: &[],
    });
    let core_idx = prompt.find("【核心设定·不可违背】").unwrap();
    let scene_idx = prompt.find("【当前场景约束】").unwrap();
    let bottom_idx = prompt.find("底线区块").unwrap();
    assert!(core_idx < scene_idx && scene_idx < bottom_idx);
    assert!(prompt.contains("结对编程"));
}

#[test]
fn prompt_concise_overlay_in_scene_block() {
    let role = create_test_role();
    let personality = create_test_personality();
    let overlay = "【发行版简洁模式】回复宜短。";
    let prompt = PromptBuilder::build_prompt(&PromptInput {
        role: &role,
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
        scene_label: "VS Code",
        scene_detail: "",
        topic_hint_line: "",
        life_context_line: "",
        worldview_snippet: "",
        mutable_personality: "",
        reply_quality_anchor: effective_reply_quality_anchor(&role),
        previous_complex_emotion_narrative_hint: "",
        user_identity_template: "",
        user_identity_id: "",
        host_prompt_overlay: overlay,
        host_state_expression_hint: "",
        relation_transition_hint: "",
        extra_sections: &[],
    });
    assert!(!prompt.starts_with(overlay));
    let scene_idx = prompt.find("【当前场景约束】").unwrap();
    let overlay_idx = prompt.find(overlay).unwrap();
    assert!(scene_idx < overlay_idx);
}

#[test]
fn relation_transition_duration_respects_cap_and_rank() {
    assert_eq!(relation_transition_duration(0, 2.0), 2);
    assert_eq!(relation_transition_duration(1, 2.0), 4);
    assert_eq!(relation_transition_duration(2, 9.0), 8);
    assert_eq!(relation_transition_duration(0, 9.0), 4);
}

#[test]
fn build_character_status_summary_includes_scene_and_host_hint() {
    let role = create_test_role();
    let personality = create_test_personality();
    let prompt = PromptBuilder::build_prompt(&PromptInput {
        role: &role,
        personality: &personality,
        memories: &[],
        user_input: "hi",
        user_emotion: "低落",
        user_relation_id: "",
        relation_hint: "",
        relation_before: "Friend",
        favorability_before: 62.0,
        relation_preview: "Friend",
        favorability_preview: 62.0,
        event_type: &EventType::Ignore,
        impact_factor: 0.0,
        scene_label: "VS Code 结对编程",
        scene_detail: "",
        topic_hint_line: "",
        life_context_line: "",
        worldview_snippet: "",
        mutable_personality: "",
        reply_quality_anchor: effective_reply_quality_anchor(&role),
        previous_complex_emotion_narrative_hint: "",
        user_identity_template: "",
        user_identity_id: "",
        host_prompt_overlay: "",
        host_state_expression_hint: "更信任用户的技术判断，少寒暄",
        relation_transition_hint: "",
        extra_sections: &[],
    });
    assert!(prompt.contains("【角色当前状态】"));
    assert!(prompt.contains("好感约 62/100"));
    assert!(prompt.contains("Friend"));
    assert!(prompt.contains("低落"));
    assert!(prompt.contains("VS Code 结对编程"));
    assert!(prompt.contains("更信任用户的技术判断"));
}

#[test]
fn relation_transition_hint_in_tone_block() {
    let role = create_test_role();
    let personality = create_test_personality();
    let hint = "正在从 Acquaintance 向 Friend 过渡";
    let prompt = PromptBuilder::build_prompt(&PromptInput {
        role: &role,
        personality: &personality,
        memories: &[],
        user_input: "hi",
        user_emotion: "neutral",
        user_relation_id: "",
        relation_hint: "",
        relation_before: "Acquaintance",
        favorability_before: 42.0,
        relation_preview: "Friend",
        favorability_preview: 45.0,
        event_type: &EventType::Praise,
        impact_factor: 0.5,
        scene_label: "",
        scene_detail: "",
        topic_hint_line: "",
        life_context_line: "",
        worldview_snippet: "",
        mutable_personality: "",
        reply_quality_anchor: effective_reply_quality_anchor(&role),
        previous_complex_emotion_narrative_hint: "",
        user_identity_template: "",
        user_identity_id: "",
        host_prompt_overlay: "",
        host_state_expression_hint: "",
        relation_transition_hint: hint,
        extra_sections: &[],
    });
    assert!(prompt.contains("【关系过渡】"));
    assert!(prompt.contains(hint));
    let tone_idx = prompt.find("语气区块").unwrap();
    let content_idx = prompt.find("内容区块").unwrap();
    let hint_idx = prompt.find("【关系过渡】").unwrap();
    assert!(tone_idx < hint_idx && hint_idx < content_idx);
}

#[test]
fn custom_anchor_still_has_guardrails_state_and_vent() {
    let mut role = create_test_role();
    role.reply_quality_anchor = Some("【包级锚点】仅人设差异。".to_string());
    let personality = create_test_personality();
    let prompt = PromptBuilder::build_prompt(&PromptInput {
        role: &role,
        personality: &personality,
        memories: &[],
        user_input: "好",
        user_emotion: "neutral",
        user_relation_id: "",
        relation_hint: "",
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
        reply_quality_anchor: effective_reply_quality_anchor(&role),
        previous_complex_emotion_narrative_hint: "",
        user_identity_template: "",
        user_identity_id: "",
        host_prompt_overlay: "",
        host_state_expression_hint: "",
        relation_transition_hint: "",
        extra_sections: &[],
    });
    assert!(prompt.contains("【包级锚点】仅人设差异。"));
    let guard_idx = prompt.find("【对话硬约束】").expect("guardrails");
    assert!(prompt[guard_idx..].contains("状态延续"));
    assert!(prompt[guard_idx..].contains("倾诉优先"));
}

#[test]
fn prompt_user_input_before_closing_line() {
    let role = create_test_role();
    let personality = create_test_personality();
    let prompt = PromptBuilder::build_prompt(&PromptInput {
        role: &role,
        personality: &personality,
        memories: &[],
        user_input: "今天被老板骂了",
        user_emotion: "sad",
        user_relation_id: "",
        relation_hint: "",
        relation_before: "Friend",
        favorability_before: 50.0,
        relation_preview: "Friend",
        favorability_preview: 50.0,
        event_type: &EventType::Complaint,
        impact_factor: -0.3,
        scene_label: "",
        scene_detail: "",
        topic_hint_line: "",
        life_context_line: "",
        worldview_snippet: "",
        mutable_personality: "",
        reply_quality_anchor: effective_reply_quality_anchor(&role),
        previous_complex_emotion_narrative_hint: "",
        user_identity_template: "",
        user_identity_id: "",
        host_prompt_overlay: "",
        host_state_expression_hint: "",
        relation_transition_hint: "",
        extra_sections: &[],
    });
    let user_idx = prompt.find("用户说: 今天被老板骂了").expect("user line");
    let closing_idx = prompt
        .find("请以角色身份自然地回复，保持一致的性格和语气。")
        .expect("closing");
    assert!(user_idx < closing_idx);
    assert!(!prompt.contains("【回复结构】"));
    assert_eq!(prompt.matches("用户说:").count(), 1);
}

#[test]
fn event_relation_block_no_impact_factor_jargon() {
    let role = create_test_role();
    let personality = create_test_personality();
    let prompt = PromptBuilder::build_prompt(&PromptInput {
        role: &role,
        personality: &personality,
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
        reply_quality_anchor: effective_reply_quality_anchor(&role),
        previous_complex_emotion_narrative_hint: "",
        user_identity_template: "",
        user_identity_id: "",
        host_prompt_overlay: "",
        host_state_expression_hint: "",
        relation_transition_hint: "",
        extra_sections: &[],
    });
    assert!(!prompt.contains("影响因子(已归一)"));
    assert!(!prompt.contains("warmup_level="));
    assert!(!prompt.contains("boundary_tone_level="));
    assert!(prompt.contains("本轮事件类型: 赞美"));
}

#[test]
fn prompt_block_guide_not_triplicated() {
    let role = create_test_role();
    let personality = create_test_personality();
    let prompt = PromptBuilder::build_prompt(&PromptInput {
        role: &role,
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
        reply_quality_anchor: effective_reply_quality_anchor(&role),
        previous_complex_emotion_narrative_hint: "",
        user_identity_template: "",
        user_identity_id: "",
        host_prompt_overlay: "",
        host_state_expression_hint: "",
        relation_transition_hint: "",
        extra_sections: &[],
    });
    assert_eq!(
        prompt.matches("以下为语气/内容层次，请按序理解").count(),
        1
    );
}

#[test]
fn extra_sections_render_before_reply_quality_anchor() {
    use oclive_kernel_types::PromptExtraSection;

    let role = create_test_role();
    let personality = create_test_personality();
    let sections = [PromptExtraSection {
        title: "插件扩展",
        body: "请保持角色口吻，同时留意附加约束。",
    }];
    let prompt = PromptBuilder::build_prompt(&PromptInput {
        role: &role,
        personality: &personality,
        memories: &[],
        user_input: "你好",
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
        previous_complex_emotion_narrative_hint: "",
        user_identity_template: "",
        user_identity_id: "",
        host_prompt_overlay: "",
        host_state_expression_hint: "",
        relation_transition_hint: "",
        extra_sections: &sections,
    });
    let anchor_idx = prompt.find("【回复质量锚点】").unwrap_or_else(|| {
        prompt
            .find(DEFAULT_REPLY_QUALITY_ANCHOR.trim())
            .expect("anchor")
    });
    let extra_idx = prompt.find("【插件扩展】").expect("extra section title");
    assert!(extra_idx < anchor_idx);
    assert!(prompt.contains("请保持角色口吻，同时留意附加约束。"));
}

