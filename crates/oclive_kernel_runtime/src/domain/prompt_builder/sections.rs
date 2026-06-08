use super::*;

use crate::models::{EventType, Memory, PersonalitySource, PersonalityVector, Role};

impl PromptBuilder {
    #[must_use]
    pub(super) fn build_core_hard_constraint(role: &Role) -> String {
        let mut core = String::new();
        core.push_str(&format!("你是{}。\n", role.name));
        core.push_str("【核心设定·不可违背】以下是你不可违背的核心设定\n");
        if !role.core_personality.trim().is_empty() {
            if role.evolution_config.personality_source == PersonalitySource::Profile {
                core.push_str(&format!(
                    "核心性格档案（创作者与用户设定，运行时 AI 不得改写；与可变档案冲突时以本段为准）:\n{}\n",
                    role.core_personality.trim()
                ));
            } else {
                core.push_str(&format!("核心人设:\n{}\n", role.core_personality.trim()));
            }
        }
        core
    }

    #[must_use]
    pub(super) fn build_scene_constraint_block(input: &PromptInput<'_>) -> String {
        let has_scene = !input.scene_label.is_empty()
            || !input.scene_detail.is_empty()
            || !input.topic_hint_line.is_empty()
            || !input.host_prompt_overlay.trim().is_empty();
        if !has_scene {
            return String::new();
        }
        let mut block = String::from("【当前场景约束】（当前在什么场景、应如何表现）\n");
        if !input.host_prompt_overlay.trim().is_empty() {
            block.push_str(input.host_prompt_overlay.trim());
            block.push_str("\n\n");
        }
        if !input.scene_label.is_empty() {
            block.push_str(&format!("当前场景：{}\n", input.scene_label));
        }
        if !input.scene_detail.trim().is_empty() {
            block.push_str("场景设定（来自角色包，请在此氛围内自然发挥）：\n");
            block.push_str(input.scene_detail.trim());
            block.push_str("\n\n");
        }
        if !input.topic_hint_line.is_empty() {
            block.push_str(input.topic_hint_line);
            block.push('\n');
        }
        block
    }

    #[must_use]
    pub(super) fn build_personality_supplement(
        role: &Role,
        personality: &PersonalityVector,
        mutable_personality: &str,
    ) -> String {
        let profile_primary =
            role.evolution_config.personality_source == PersonalitySource::Profile;
        let mut supplement = String::new();
        if !role.description.trim().is_empty() {
            supplement.push_str(&format!("描述: {}\n", role.description));
        }
        if profile_primary {
            let m = mutable_personality.trim();
            if !m.is_empty() {
                supplement.push_str(
                    "【可变性格档案】（由模型在规则内根据对话维护，用于抓住相处中的有限变化；创作者不可手写本条；与核心档案冲突时以核心为准）\n",
                );
                supplement.push_str(m);
                supplement.push_str("\n\n");
            }
            supplement.push_str(
                "【七维视图】（仅由「核心 + 可变档案」正文经规则归纳的辅助读数，帮助把握语气松紧；**不是**性格主数据源；与上文档案冲突时以档案正文为准）\n",
            );
        } else {
            supplement.push_str("\n当前性格（自然语言）:\n");
        }
        supplement.push_str(&format!(
            "- 倔强: {}\n",
            Self::dim_label(personality.stubbornness, "偏低", "一般", "偏高")
        ));
        supplement.push_str(&format!(
            "- 黏人: {}\n",
            Self::dim_label(personality.clinginess, "偏低", "一般", "偏高")
        ));
        supplement.push_str(&format!(
            "- 敏感: {}\n",
            Self::dim_label(personality.sensitivity, "偏低", "一般", "偏高")
        ));
        supplement.push_str(&format!(
            "- 强势: {}\n",
            Self::dim_label(personality.assertiveness, "偏低", "一般", "偏高")
        ));
        supplement.push_str(&format!(
            "- 宽容: {}\n",
            Self::dim_label(personality.forgiveness, "偏低", "一般", "偏高")
        ));
        supplement.push_str(&format!(
            "- 话多: {}\n",
            Self::dim_label(personality.talkativeness, "偏低", "一般", "偏高")
        ));
        supplement.push_str(&format!(
            "- 温暖: {}",
            Self::dim_label(personality.warmth, "偏低", "一般", "偏高")
        ));
        supplement
    }

    #[must_use]
    pub(super) fn build_character_status_summary(input: &PromptInput<'_>) -> String {
        let mut parts = vec![format!(
            "你当前对用户好感约 {:.0}/100，关系处于 {}。",
            input.favorability_before.clamp(0.0, 100.0),
            input.relation_before
        )];
        if !input.user_emotion.trim().is_empty() {
            parts.push(format!(
                "用户语气线索：{}。",
                input.user_emotion.trim()
            ));
        }
        if !input.scene_label.is_empty() {
            parts.push(format!("场景为 {}。", input.scene_label));
        }
        if !input.host_state_expression_hint.trim().is_empty() {
            parts.push(input.host_state_expression_hint.trim().to_string());
        }
        format!("【角色当前状态】{}\n", parts.join(""))
    }

    pub(super) fn dim_label(v: f64, low: &str, mid: &str, high: &str) -> String {
        if v < 0.35 {
            low.to_string()
        } else if v < 0.65 {
            mid.to_string()
        } else {
            high.to_string()
        }
    }

    pub(super) fn build_memory_context(memories: &[Memory]) -> String {
        // Do not expose importance scores to the model, to avoid them leaking into user-visible replies and breaking immersion
        let mut context = String::from(
            "关于用户的记忆（已按相关性排序；请勿在回复中复述编号、括号或「重要性」等系统字样）:\n",
        );
        for (i, memory) in memories.iter().enumerate() {
            context.push_str(&format!("{}. {}\n", i + 1, memory.content.trim()));
        }
        context
    }

    pub(super) fn build_current_state(personality: &PersonalityVector, user_emotion: &str) -> String {
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
        state.push_str(Self::reply_pacing_hint(personality));
        state.push('\n');
        state.push_str(Self::listening_style_hint(personality));
        state
    }

    pub(super) fn reply_pacing_hint(personality: &PersonalityVector) -> &'static str {
        if personality.talkativeness >= 0.65 {
            "回复篇幅倾向: 可适度展开（通常 1–4 句），须先接住用户本句；用户仅「嗯/好/在吗」等极短句时仍宜短答。\n"
        } else if personality.talkativeness <= 0.35 {
            "回复篇幅倾向: 宜精炼（常 1–2 句），嘴硬但不灌水；用户寒暄时勿写成长段。\n"
        } else {
            "回复篇幅倾向: 随用户信息量调节——寒暄短答，深聊或提问再展开；勿与用户消息字数盲目攀比。\n"
        }
    }

    pub(super) fn listening_style_hint(personality: &PersonalityVector) -> &'static str {
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

    pub(super) fn build_event_relation_state(
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

    pub(super) fn build_transition_tone_line(
        relation_before: &str,
        relation_preview: &str,
        favorability_before: f64,
        favorability_preview: f64,
        impact_factor: f64,
    ) -> String {
        let before_rank = relation_rank(relation_before);
        let preview_rank = relation_rank(relation_preview);
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

    pub(super) fn seven_dim_equal_weight_score(personality: &PersonalityVector) -> f64 {
        let sum = personality.stubbornness
            + personality.clinginess
            + personality.sensitivity
            + personality.assertiveness
            + personality.forgiveness
            + personality.talkativeness
            + personality.warmth;
        (sum / 7.0).clamp(0.0, 1.0)
    }

    pub(super) fn build_boundary_tone_guideline(
        personality: &PersonalityVector,
        relation_before: &str,
        relation_preview: &str,
    ) -> Option<String> {
        let before_rank = relation_rank(relation_before);
        let preview_rank = relation_rank(relation_preview);
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
}
