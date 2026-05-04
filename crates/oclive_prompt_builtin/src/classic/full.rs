//! 完整经典提示词组装算法（`feature = "classic"`）。

use oclive_kernel_core::models::Memory;
use oclive_kernel_core::prompt::{PromptInput, PromptRolePromptSlice, TopicHintContext};
use oclive_kernel_models::{EventType, PersonalityVector};
use oclive_validation::PersonalitySource;

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
                .role_prompt
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
            input.role_prompt,
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
        if let Some(h) = input
            .complex_emotion_hint
            .map(str::trim)
            .filter(|s| !s.is_empty())
        {
            prompt.push_str("【复杂情感复盘】\n");
            prompt.push_str(h);
            prompt.push_str("\n\n");
        }
        if !input.reply_quality_anchor.trim().is_empty() {
            prompt.push_str(input.reply_quality_anchor.trim());
            prompt.push_str("\n\n");
        }
        prompt.push_str(&format!("用户说: {}", input.user_input));
        prompt.push_str("\n\n");
        prompt.push_str("【回复结构】\n");
        prompt.push_str(
            "- 须与上文【回复质量锚点】一致：先接住用户本句；出现倾诉信号时先回应遭遇与情绪，再视需要延伸或反问；勿与用户本句基本同义的复述式开场。\n",
        );
        prompt.push_str(
            "- 展开程度与篇幅须遵守锚点中的「篇幅与节奏」与「状态延续」：用户极短时勿强行写成长段或重复上文已交代内容。\n",
        );
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
        role: PromptRolePromptSlice<'_>,
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
        state.push_str(&format!("我的心情倾向: {}\n", mood));
        state.push_str(Self::listening_style_hint(personality));
        state
    }

    fn listening_style_hint(personality: &PersonalityVector) -> &'static str {
        if personality.warmth >= 0.65 && personality.sensitivity >= 0.6 {
            "倾诉应对倾向: 先共情安抚，再用一问一答陪伴展开。"
        } else if personality.assertiveness >= 0.65 {
            "倾诉应对倾向: 可直接点评或吐槽，但要先承认对方情绪，再表达立场，避免上来训话。"
        } else if personality.warmth <= 0.35 && personality.sensitivity <= 0.35 {
            "倾诉应对倾向: 可偏克制或冷感，但仍要先回应事实与情绪，不要敷衍转移。"
        } else {
            "倾诉应对倾向: 先接情绪，再追问一个细节，按对方反馈决定是否展开。"
        }
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

    /// 从 [`TopicHintContext`] 的 `topic_weights` 取当前场景下权重最高的话题，用于 prompt 一句提示。
    pub fn top_topic_hint(ctx: &TopicHintContext<'_>, scene_id: &str) -> Option<String> {
        ctx.top_topic_name_for_scene(scene_id)
    }
}
