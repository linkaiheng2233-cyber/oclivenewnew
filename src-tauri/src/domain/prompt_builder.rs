//! 提示词构建：角色、记忆、关系与场景话题提示

use crate::models::{EventType, Memory, PersonalitySource, PersonalityVector, Role};

/// 主对话 `build_prompt` 的输入，避免长参数列表与调用处错位。
pub struct PromptInput<'a> {
    pub role: &'a Role,
    pub personality: &'a PersonalityVector,
    pub memories: &'a [Memory],
    pub user_input: &'a str,
    pub user_emotion: &'a str,
    /// 当前用户身份键（与 manifest `user_relations`、DB 一致）；空则跳过【用户身份】整段。
    pub user_relation_id: &'a str,
    pub relation_hint: &'a str,
    pub relation_before: &'a str,
    pub favorability_before: f64,
    pub relation_preview: &'a str,
    pub favorability_preview: f64,
    pub event_type: &'a EventType,
    pub impact_factor: f64,
    pub scene_label: &'a str,
    /// 来自角色包 `description.txt` 或 `scene.json` 的自动拼装，新场景无需改代码。
    pub scene_detail: &'a str,
    pub topic_hint_line: &'a str,
    /// 虚拟时间日程推断一行；空则跳过（不改变无配置时的对话行为）
    pub life_context_line: &'a str,
    /// 本回合检索到的世界观知识片段；空则跳过【世界观设定】段
    pub worldview_snippet: &'a str,
    /// 人设优先模式下 DB 中的「可变性格档案」全文；`vector` 模式传空串即可。
    pub mutable_personality: &'a str,
}

pub struct PromptBuilder;

impl PromptBuilder {
    /// 是否在【用户身份】中追加家人向长约束（好友/同学等默认不注入，以免冲淡角色包 `prompt_hint`）。
    fn should_inject_family_long_guardrail(user_relation_id: &str, relation_hint: &str) -> bool {
        let family_id = user_relation_id.eq_ignore_ascii_case("family")
            || user_relation_id.eq_ignore_ascii_case("parent")
            || user_relation_id.eq_ignore_ascii_case("parents")
            || user_relation_id.eq_ignore_ascii_case("guardian");
        let hint_suggests_family = relation_hint.contains("父母")
            || relation_hint.contains("长辈")
            || relation_hint.contains("家长");
        family_id || hint_suggests_family
    }

    /// 【用户身份】：须优先于人设中与身份冲突的笼统描述（如同居文案 vs 用户扮演父母）。
    fn push_user_identity_section(prompt: &mut String, input: &PromptInput<'_>) {
        if !input.user_relation_id.is_empty() {
            let label = input
                .role
                .user_relations
                .iter()
                .find(|r| r.id == input.user_relation_id)
                .map(|r| r.name.as_str())
                .unwrap_or(input.user_relation_id);
            prompt.push_str("【用户身份】（本轮必须遵守；与人设冲突时以本段为准）\n");
            if !input.relation_hint.is_empty() {
                prompt.push_str("身份语气要点（角色包配置，须落实）：\n");
                prompt.push_str(input.relation_hint.trim());
                prompt.push_str("\n\n");
            }
            prompt.push_str(&format!(
                "当前关系：{}（关系键 {}）\n",
                label, input.user_relation_id
            ));
            prompt.push_str(
                "约束（通用）：称呼、距离感与话题分寸须与当前关系一致；若上文有身份语气要点，须一并落实，勿与人设或本段矛盾。\n",
            );
            if Self::should_inject_family_long_guardrail(
                input.user_relation_id,
                input.relation_hint,
            ) {
                prompt.push_str(
                    "（家人/长辈场景补充）你必须按上述身份理解用户。若用户以父母、长辈或家人身份自居，你须以子女、晚辈或对应家人身份回应，称呼与态度须匹配；不得用「才不是」「你逗我」等话否认用户的家长或长辈身份。若人设中与当前身份冲突，以本段为准调整语气；禁止在明知用户扮演长辈时仍以同龄暧昧口吻（如反复「大笨蛋」调情）主导回复。\n",
                );
            }
            prompt.push('\n');
        } else if !input.relation_hint.is_empty() {
            prompt.push_str("【用户身份】\n");
            prompt.push_str(input.relation_hint);
            prompt.push_str("\n\n");
        }
    }

    pub fn build_prompt(input: &PromptInput<'_>) -> String {
        let mut prompt = String::new();
        prompt.push_str(&Self::build_role_definition(
            input.role,
            input.personality,
            input.mutable_personality,
        ));
        prompt.push_str("\n\n");
        Self::push_user_identity_section(&mut prompt, input);
        if !input.scene_label.is_empty()
            || !input.scene_detail.is_empty()
            || !input.topic_hint_line.is_empty()
        {
            prompt.push_str("【场景与话题】\n");
            if !input.scene_label.is_empty() {
                prompt.push_str(&format!("当前场景：{}\n", input.scene_label));
            }
            if !input.scene_detail.trim().is_empty() {
                prompt.push_str("场景设定（来自角色包，请在此氛围内自然发挥）：\n");
                prompt.push_str(input.scene_detail.trim());
                prompt.push_str("\n\n");
            }
            if !input.topic_hint_line.is_empty() {
                prompt.push_str(input.topic_hint_line);
                prompt.push('\n');
            }
            prompt.push('\n');
        }
        if !input.life_context_line.is_empty() {
            prompt.push_str("【日程推断】\n");
            prompt.push_str(input.life_context_line.trim());
            prompt.push_str("\n\n");
        }
        if !input.worldview_snippet.trim().is_empty() {
            prompt.push_str("【世界观设定】（角色包知识；与闲聊记忆冲突时以本段为权威事实，但不得覆盖【用户身份】与安全红线。）\n");
            prompt.push_str(input.worldview_snippet.trim());
            prompt.push_str("\n\n");
        }
        if !input.memories.is_empty() {
            prompt.push_str(&Self::build_memory_context(input.memories));
            prompt.push_str("\n\n");
        }
        prompt.push_str(&Self::build_event_relation_state(
            input.relation_before,
            input.favorability_before,
            input.relation_preview,
            input.favorability_preview,
            input.event_type,
            input.impact_factor,
        ));
        prompt.push_str("\n\n");
        if let Some(boundary_guide) = Self::build_boundary_tone_guideline(
            input.personality,
            input.relation_before,
            input.relation_preview,
        ) {
            prompt.push_str(&boundary_guide);
            prompt.push_str("\n\n");
        }
        prompt.push_str(&Self::build_current_state(
            input.personality,
            input.user_emotion,
        ));
        prompt.push_str("\n\n");
        prompt.push_str(&format!("用户说: {}", input.user_input));
        prompt.push_str("\n\n");
        prompt.push_str("【回复结构】\n");
        prompt.push_str(
            "- 先直接回应用户本句的具体内容、问题或情绪，再自然延伸或反问；整体篇幅建议约一半紧扣用户输入，一半为相关延伸，避免整段与用户发言无关的自说自话。\n",
        );
        prompt.push_str(
            "- 不要使用无意义的重复音节、乱码式英文碎片或陌生昵称；称呼须符合人设与当前关系阶段。\n",
        );
        prompt.push_str("- 避免连续多句同一套话或同一问法；勿重复用户已经回答过的问题。\n");
        prompt
            .push_str("- 勿机械模仿用户消息里的颜文字密度或句式；用户未大量使用时保持自然口语。\n");
        prompt.push_str("\n请以角色身份自然地回复，保持一致的性格和语气。");
        prompt
    }

    fn dim_label(v: f64, low: &str, mid: &str, high: &str) -> String {
        if v < 0.35 {
            low.to_string()
        } else if v < 0.65 {
            mid.to_string()
        } else {
            high.to_string()
        }
    }

    fn build_role_definition(
        role: &Role,
        personality: &PersonalityVector,
        mutable_personality: &str,
    ) -> String {
        let profile_primary =
            role.evolution_config.personality_source == PersonalitySource::Profile;
        let mut definition = String::new();
        definition.push_str(&format!("你是{}。\n", role.name));
        definition.push_str(&format!("描述: {}\n", role.description));
        if profile_primary {
            if !role.core_personality.trim().is_empty() {
                definition.push_str(&format!(
                    "核心性格档案（创作者与用户设定，运行时 AI 不得改写；与可变档案冲突时以本段为准）:\n{}\n",
                    role.core_personality.trim()
                ));
            }
            let m = mutable_personality.trim();
            if !m.is_empty() {
                definition.push_str(
                    "【可变性格档案】（由模型在规则内根据对话维护，用于抓住相处中的有限变化；创作者不可手写本条；与核心档案冲突时以核心为准）\n",
                );
                definition.push_str(m);
                definition.push_str("\n\n");
            }
            definition.push_str(
                "【七维视图】（仅由「核心 + 可变档案」正文经规则归纳的辅助读数，帮助把握语气松紧；**不是**性格主数据源；与上文档案冲突时以档案正文为准）\n",
            );
        } else if !role.core_personality.trim().is_empty() {
            definition.push_str(&format!("核心人设:\n{}\n", role.core_personality.trim()));
        }
        if !profile_primary {
            definition.push_str("\n当前性格（自然语言）:\n");
        }
        definition.push_str(&format!(
            "- 倔强: {}\n",
            Self::dim_label(personality.stubbornness, "偏低", "一般", "偏高")
        ));
        definition.push_str(&format!(
            "- 黏人: {}\n",
            Self::dim_label(personality.clinginess, "偏低", "一般", "偏高")
        ));
        definition.push_str(&format!(
            "- 敏感: {}\n",
            Self::dim_label(personality.sensitivity, "偏低", "一般", "偏高")
        ));
        definition.push_str(&format!(
            "- 强势: {}\n",
            Self::dim_label(personality.assertiveness, "偏低", "一般", "偏高")
        ));
        definition.push_str(&format!(
            "- 宽容: {}\n",
            Self::dim_label(personality.forgiveness, "偏低", "一般", "偏高")
        ));
        definition.push_str(&format!(
            "- 话多: {}\n",
            Self::dim_label(personality.talkativeness, "偏低", "一般", "偏高")
        ));
        definition.push_str(&format!(
            "- 温暖: {}",
            Self::dim_label(personality.warmth, "偏低", "一般", "偏高")
        ));
        definition
    }

    fn build_memory_context(memories: &[Memory]) -> String {
        // 不向模型暴露 importance 数值，避免被复述进用户可见回复造成「脱戏」
        let mut context = String::from(
            "关于用户的记忆（已按相关性排序；请勿在回复中复述编号、括号或「重要性」等系统字样）:\n",
        );
        for (i, memory) in memories.iter().enumerate() {
            context.push_str(&format!("{}. {}\n", i + 1, memory.content.trim()));
        }
        context
    }

    fn build_current_state(personality: &PersonalityVector, user_emotion: &str) -> String {
        let mut state = String::from("当前状态:\n");
        state.push_str("用户语气线索（内置情感引擎；请先对齐再编内容）:\n");
        state.push_str(user_emotion.trim());
        state.push('\n');
        let balance = (personality.forgiveness + personality.warmth) / 2.0;
        let mood = if balance > 0.65 {
            "偏温柔、好说话"
        } else if balance > 0.35 {
            "平常"
        } else {
            "偏硬、易较真"
        };
        state.push_str(&format!("我的心情倾向: {}", mood));
        state
    }

    fn build_event_relation_state(
        relation_before: &str,
        favorability_before: f64,
        relation_preview: &str,
        favorability_preview: f64,
        event_type: &EventType,
        impact_factor: f64,
    ) -> String {
        let mut s = String::from("【本轮事件与关系状态机】\n");
        s.push_str(&format!("当前关系阶段: {}\n", relation_before));
        s.push_str(&format!(
            "当前好感度: {:.1}/100\n",
            favorability_before.clamp(0.0, 100.0)
        ));
        s.push_str(&format!(
            "本轮关系预览: {} -> {}（预计好感 {:.1}/100）\n",
            relation_before,
            relation_preview,
            favorability_preview.clamp(0.0, 100.0)
        ));
        s.push_str(&format!("本轮事件类型: {:?}\n", event_type));
        s.push_str(&format!(
            "本轮影响因子(已归一): {:.3} (范围 -1.0 ~ 1.0)\n",
            impact_factor.clamp(-1.0, 1.0)
        ));
        s.push_str("\n硬约束（必须遵守）：\n");
        s.push_str("- 关系阶段与好感决定亲密度：低阶段/低好感时不要突然使用过度亲昵称呼、不要突然表白或承诺长期关系。\n");
        s.push_str("- 若事件为 Quarrel 或影响因子 < 0：语气应更克制、防御或冷静，不要立刻甜蜜撒娇、不要“当作没吵过”。\n");
        s.push_str("- 若事件为 Praise/Apology 或影响因子 > 0：允许缓和、更温柔，但仍需服从当前关系阶段。\n");
        s.push_str(
            "- 请把语气对齐到「本轮关系预览」：若仅小幅缓和，请用过渡口吻，避免语气突然升阶。\n",
        );
        s.push_str("- 不要编造系统状态：不要虚构未发生的关系跳变、共同经历或历史事件。\n");
        s.push_str(&Self::build_transition_tone_line(
            relation_before,
            relation_preview,
            favorability_before,
            favorability_preview,
            impact_factor,
        ));
        s
    }

    fn build_transition_tone_line(
        relation_before: &str,
        relation_preview: &str,
        favorability_before: f64,
        favorability_preview: f64,
        impact_factor: f64,
    ) -> String {
        let before_rank = Self::relation_rank(relation_before);
        let preview_rank = Self::relation_rank(relation_preview);
        let favor_delta = (favorability_preview - favorability_before).clamp(-100.0, 100.0);
        let impact = impact_factor.clamp(-1.0, 1.0);
        let line = if preview_rank > before_rank {
            if favor_delta > 2.0 || impact > 0.45 {
                "本轮过渡语气：可轻微升温，但先用试探/确认式表达，再进入更亲近语气。"
            } else {
                "本轮过渡语气：关系有改善预览，但请维持慢热，只做一句轻度缓和。"
            }
        } else if preview_rank < before_rank || impact < -0.2 {
            "本轮过渡语气：关系收紧，优先克制与边界，不使用亲密化措辞。"
        } else {
            "本轮过渡语气：延续当前阶段语气，避免突然升阶或突然疏离。"
        };
        format!("{line}\n")
    }

    fn relation_rank(s: &str) -> i32 {
        match s {
            "Stranger" => 0,
            "Acquaintance" => 1,
            "Friend" => 2,
            "CloseFriend" => 3,
            "Partner" => 4,
            _ => 0,
        }
    }

    fn seven_dim_equal_weight_score(personality: &PersonalityVector) -> f64 {
        let sum = personality.stubbornness
            + personality.clinginess
            + personality.sensitivity
            + personality.assertiveness
            + personality.forgiveness
            + personality.talkativeness
            + personality.warmth;
        (sum / 7.0).clamp(0.0, 1.0)
    }

    fn build_boundary_tone_guideline(
        personality: &PersonalityVector,
        relation_before: &str,
        relation_preview: &str,
    ) -> Option<String> {
        let before_rank = Self::relation_rank(relation_before);
        let preview_rank = Self::relation_rank(relation_preview);
        let is_low_stage = before_rank <= 1 || preview_rank <= 1;
        let is_low_to_friend_boundary = before_rank <= 1 && preview_rank == 2;
        if !(is_low_stage || is_low_to_friend_boundary) {
            return None;
        }

        let warmup_level = Self::seven_dim_equal_weight_score(personality);
        let stage_weight = if is_low_to_friend_boundary {
            0.95
        } else if is_low_stage {
            0.65
        } else {
            0.0
        };
        let boundary_tone_level = (stage_weight * (1.0 - warmup_level * 0.45)).clamp(0.0, 1.0);

        let mut s = String::from("【边界语气控制指引】\n");
        s.push_str(&format!(
            "7维等权连续分数 warmup_level={:.3}，边界约束强度 boundary_tone_level={:.3}。\n",
            warmup_level, boundary_tone_level
        ));
        if boundary_tone_level >= 0.7 {
            s.push_str("- 当前处于低阶段或升阶边界，语气请慢热、谨慎、先建立安全感；避免突然亲昵称呼或强承诺。\n");
        } else if boundary_tone_level >= 0.4 {
            s.push_str("- 当前建议渐进升温：保持友好与礼貌，可轻微拉近距离，但避免语气突然变得过度亲密。\n");
        } else {
            s.push_str(
                "- 当前仅需轻度边界控制：保持自然友好，不必刻意生硬，但仍避免突升亲密语气。\n",
            );
        }
        Some(s)
    }

    pub fn build_simple_prompt(role_name: &str, user_input: &str) -> String {
        format!("你是{}。用户说: {}\n请自然地回复。", role_name, user_input)
    }

    pub fn build_system_prompt(role_name: &str) -> String {
        format!(
            "你是一个名叫{}的AI角色。请以这个角色的身份进行对话，保持一致的性格和语气。",
            role_name
        )
    }

    pub fn build_guidance_prompt(core_personality: &str) -> String {
        format!(
            "你的核心性格是: {}\n请根据这个性格特征来指导你的回复。",
            core_personality
        )
    }

    /// 从 `memory_config.topic_weights` 取当前场景下权重最高的话题，用于 prompt 一句提示
    pub fn top_topic_hint(role: &Role, scene_id: &str) -> Option<String> {
        let mc = role.memory_config.as_ref()?;
        let tw = mc.topic_weights.get(scene_id)?;
        tw.iter()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(name, _)| name.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::EventType;
    use crate::models::EvolutionBounds;
    use crate::models::PersonalitySource;
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
        });
        assert!(prompt.contains("【可变性格档案】"));
        assert!(prompt.contains("更黏人"));
        assert!(prompt.contains("【七维视图】"));
        assert!(prompt.contains("核心性格档案（创作者与用户设定"));
    }
}
