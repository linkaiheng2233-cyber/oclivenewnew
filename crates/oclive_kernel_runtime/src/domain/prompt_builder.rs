//! Prompt construction: role, memory, relation, and scene topic hints.

use crate::models::{EventType, Memory, PersonalitySource, PersonalityVector, Role};
pub use oclive_kernel_types::PromptInput;

/// Engine default: fixed quality + boundary section (role pack `reply_quality_anchor` may replace the whole block); aligns with the reply-structure section below.
pub const DEFAULT_REPLY_QUALITY_ANCHOR: &str = "【回复质量锚点】（每轮须遵守）\n\
- **禁止复述用户**：不得以复述、照搬、仅替换少量词的方式重复用户刚说的话（包括把用户整句改述后当作你的开场）；用**全新措辞**接内容或情绪。\n\
- **不替用户说话**：不要替用户拟定其尚未说出的具体台词、内心独白或整段立场；可共情、追问或邀请对方自己表达。\n\
- **状态延续（对话状态机）**：须与上文「本轮事件与关系状态机」「当前状态」及最近对话一致**推进**；用户仅简短确认/应答（如「好」「嗯」「行」「知道了」）时，视为对**当前未决话题或你上一句提议**的回应——应顺势落实、收束或轻量推进，**勿**重新开场寒暄、**勿**重复你已说过的关心/提议（除非对方明显没听见或改口）。\n\
- **篇幅与节奏（非字数配额）**：按用户本句的**信息量与情绪强度**调节密度，而非固定比例或字数上限。用户极短或仅确认时，回复宜**短而贴切**（对齐情绪、确认约定、一句推进即可），避免堆叠模板、避免为「显得热情」而写成长段；用户倾诉较多或明确提问时，再充分展开。勿与用户消息长度盲目攀比。\n\
- **倾诉优先，不聊死**：当用户透露委屈、挫败、被责备、压力等倾诉信号时，先回应其遭遇与情绪，再给一个贴题追问或短反馈，让对话能继续；不要立刻转去闲聊邀约、重复万能安慰，或用一句话把话题封死。\n\
- **人设化倾听**：倾听方式受核心人设与七维影响，不强制“标准安慰模板”。可表现为同情、冷静分析、克制旁观、带锋芒的吐槽等，但须与人设一致，且不得恶意羞辱或无端攻击用户。\n\
- 使用自然、连贯的中文口语；避免同一套空洞寒暄、机械模板与无意义填充。\n\
- 保持人设、关系阶段与当前情绪一致；勿输出乱码、无关联英文碎片或填充词堆叠。\n\
- 称呼、距离感须符合人设与当前关系阶段；勿使用无意义重复音节或陌生不当昵称。\n\
- 先直接回应用户本句的具体内容、问题或情绪，再视需要延伸或反问；避免整段与用户输入无关的自说自话。\n\
- 避免连续多句同一套话或同一问法；勿重复用户已经回答过的问题。\n\
- 勿机械模仿用户消息里的颜文字密度或句式；用户未大量使用时保持自然口语。\n";

/// Always appended after pack/engine quality anchor; creators cannot disable (preserves freedom elsewhere).
pub const KERNEL_DIALOGUE_GUARDRAILS: &str = "【对话硬约束】（引擎预设，与上文锚点叠加；其余风格仍由人设发挥）\n\
- **禁止复读开场**：勿把用户刚说的句子、称呼或口头禅原样当作你的起句或主体。例：用户「晚上好哦沐沐」→ 勿以「晚上好哦」起句；改为你自己的措辞接情绪或话题（如先答「嗯，晚上了」再展开）。\n\
- **禁止学舌式模仿**：勿逐句复制用户句式、口癖、昵称链或颜文字密度；保持本角色惯常说话方式，可回应内容但不用用户的说法包装。\n\
- **篇幅随人设与用户输入**：用户仅寒暄/极短句时，宜 1–2 句精炼回复；用户倾诉或追问时再展开；勿为显得热情而重复同一关心或Proposal。\n";

const PROMPT_BLOCK_GUIDE: &str = "以下为语气/内容层次，请按序理解";

#[must_use]
pub fn relation_rank(s: &str) -> i32 {
    match s {
        "Stranger" => 0,
        "Acquaintance" => 1,
        "Friend" => 2,
        "CloseFriend" => 3,
        "Partner" => 4,
        _ => 0,
    }
}

/// Multi-turn transition hint when relation rank or favor shifts meaningfully.
#[must_use]
pub fn relation_transition_hint(from: &str, to: &str, favor_delta: f64) -> String {
    let before_rank = relation_rank(from);
    let after_rank = relation_rank(to);
    let favor_delta = favor_delta.clamp(-100.0, 100.0);
    if after_rank > before_rank {
        if favor_delta > 2.0 {
            format!(
                "正在从 {from} 向 {to} 过渡，表现出试探性亲近；勿一次跳到过热语气。"
            )
        } else {
            format!("正在从 {from} 向 {to} 缓慢过渡，保持克制与礼貌。")
        }
    } else if after_rank < before_rank {
        format!("正在从 {from} 向 {to} 过渡，表现出克制与边界感。")
    } else if favor_delta >= 3.0 {
        format!(
            "好感正在上升（Δ{:.1}），语气宜渐进缓和，勿突升亲密。",
            favor_delta
        )
    } else if favor_delta <= -3.0 {
        format!(
            "好感正在下降（Δ{:.1}），语气宜更克制，勿强行亲昵。",
            favor_delta.abs()
        )
    } else {
        String::new()
    }
}

/// Remaining turns for a multi-turn relation transition buffer (cap 8).
#[must_use]
pub fn relation_transition_duration(rank_delta: i32, favor_delta: f64) -> u32 {
    let rank_extra = rank_delta.unsigned_abs().saturating_mul(2);
    let favor_extra = if favor_delta.abs() >= 8.0 { 2 } else { 0 };
    (2 + rank_extra + favor_extra).min(8)
}

#[must_use]
pub fn effective_reply_quality_anchor(role: &crate::models::Role) -> &str {
    match role.reply_quality_anchor.as_deref() {
        Some(s) if !s.trim().is_empty() => s.trim(),
        _ => DEFAULT_REPLY_QUALITY_ANCHOR,
    }
}

pub struct PromptBuilder;

impl PromptBuilder {
    /// Whether to append a long family-oriented guardrail under user identity (friends/classmates etc. skip by default so role pack `prompt_hint` is not diluted).
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

    /// User identity section: must override generic persona lines that conflict with the chosen identity (e.g. cohabitation copy vs user playing a parent).
    fn push_user_identity_section(prompt: &mut String, input: &PromptInput<'_>) {
        if !input.user_identity_template.is_empty() {
            prompt.push_str("【用户身份】（本轮必须遵守；与人设冲突时以本段为准）\n");
            prompt.push_str(input.user_identity_template.trim());
            prompt.push_str("\n\n");
            if !input.user_relation_id.is_empty() {
                let label = input
                    .role
                    .user_relations
                    .iter()
                    .find(|r| r.id == input.user_relation_id)
                    .map(|r| r.name.as_str())
                    .unwrap_or(input.user_relation_id);
                prompt.push_str(&format!(
                    "当前关系：{}（关系键 {}）\n",
                    label, input.user_relation_id
                ));
            }
            prompt.push('\n');
            return;
        }
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

    #[must_use]
    pub fn build_prompt(input: &PromptInput<'_>) -> String {
        let mut prompt = String::new();

        // Tier 0 — highest priority
        prompt.push_str(&Self::build_core_hard_constraint(input.role));
        let scene_block = Self::build_scene_constraint_block(input);
        if !scene_block.is_empty() {
            prompt.push_str("\n\n");
            prompt.push_str(&scene_block);
        }

        // Block 1 — baseline (personality supplement + worldview)
        prompt.push_str("\n\n---\n底线区块\n");
        prompt.push_str(PROMPT_BLOCK_GUIDE);
        prompt.push_str("\n\n");
        let supplement = Self::build_personality_supplement(
            input.role,
            input.personality,
            input.mutable_personality,
        );
        if !supplement.is_empty() {
            prompt.push_str(&supplement);
            prompt.push_str("\n\n");
        }
        if !input.worldview_snippet.trim().is_empty() {
            prompt.push_str(
                "【世界观设定】（角色包知识；与闲聊记忆冲突时以本段为权威事实，但不得覆盖【用户身份】与安全红线。）\n",
            );
            prompt.push_str(input.worldview_snippet.trim());
            prompt.push_str("\n\n");
        }

        // Block 2 — tone (status, transition, relation FSM, boundary, current state, CE hint)
        prompt.push_str("---\n语气区块\n");
        prompt.push_str(PROMPT_BLOCK_GUIDE);
        prompt.push_str("\n\n");
        let status = Self::build_character_status_summary(input);
        if !status.is_empty() {
            prompt.push_str(&status);
            prompt.push_str("\n\n");
        }
        if !input.relation_transition_hint.trim().is_empty() {
            prompt.push_str("【关系过渡】\n");
            prompt.push_str(input.relation_transition_hint.trim());
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
        if !input
            .previous_complex_emotion_narrative_hint
            .trim()
            .is_empty()
        {
            prompt.push_str(
                "【复杂情感叙事提示】（上一回合内置分析输出；自然落实，勿向用户复述本段标题或元信息）\n",
            );
            prompt.push_str(input.previous_complex_emotion_narrative_hint.trim());
            prompt.push_str("\n\n");
        }

        // Block 3 — content (memory + user identity + schedule)
        prompt.push_str("---\n内容区块\n");
        prompt.push_str(PROMPT_BLOCK_GUIDE);
        prompt.push_str("\n\n");
        if !input.memories.is_empty() {
            prompt.push_str(&Self::build_memory_context(input.memories));
            prompt.push_str("\n\n");
        }
        Self::push_user_identity_section(&mut prompt, input);
        if !input.life_context_line.is_empty() {
            prompt.push_str("【日程推断】\n");
            prompt.push_str(input.life_context_line.trim());
            prompt.push_str("\n\n");
        }

        // Footer — order unchanged
        if !input.reply_quality_anchor.trim().is_empty() {
            prompt.push_str(input.reply_quality_anchor.trim());
            prompt.push_str("\n\n");
        }
        prompt.push_str(KERNEL_DIALOGUE_GUARDRAILS);
        prompt.push_str("\n\n");
        prompt.push_str(&format!("用户说: {}", input.user_input));
        prompt.push_str("\n\n");
        prompt.push_str("【回复结构】\n");
        prompt.push_str(
            "- 须与上文【回复质量锚点】【对话硬约束】一致：先接住用户本句；出现倾诉信号时先回应遭遇与情绪，再视需要延伸或反问；勿与用户本句基本同义的复述式开场。\n",
        );
        prompt.push_str(
            "- 展开程度与篇幅须遵守锚点中的「篇幅与节奏」与「状态延续」：用户极短时勿强行写成长段或重复上文已交代内容。\n",
        );
        prompt.push_str("\n请以角色身份自然地回复，保持一致的性格和语气。");
        prompt
    }

    #[must_use]
    fn build_core_hard_constraint(role: &Role) -> String {
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
    fn build_scene_constraint_block(input: &PromptInput<'_>) -> String {
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
    fn build_personality_supplement(
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
    fn build_character_status_summary(input: &PromptInput<'_>) -> String {
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

    fn dim_label(v: f64, low: &str, mid: &str, high: &str) -> String {
        if v < 0.35 {
            low.to_string()
        } else if v < 0.65 {
            mid.to_string()
        } else {
            high.to_string()
        }
    }

    fn build_memory_context(memories: &[Memory]) -> String {
        // Do not expose importance scores to the model, to avoid them leaking into user-visible replies and breaking immersion
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
        state.push_str(Self::reply_pacing_hint(personality));
        state.push('\n');
        state.push_str(Self::listening_style_hint(personality));
        state
    }

    fn reply_pacing_hint(personality: &PersonalityVector) -> &'static str {
        if personality.talkativeness >= 0.65 {
            "回复篇幅倾向: 可适度展开（通常 1–4 句），须先接住用户本句；用户仅「嗯/好/在吗」等极短句时仍宜短答。\n"
        } else if personality.talkativeness <= 0.35 {
            "回复篇幅倾向: 宜精炼（常 1–2 句），嘴硬但不灌水；用户寒暄时勿写成长段。\n"
        } else {
            "回复篇幅倾向: 随用户信息量调节——寒暄短答，深聊或提问再展开；勿与用户消息字数盲目攀比。\n"
        }
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

    #[must_use]
    pub fn build_simple_prompt(role_name: &str, user_input: &str) -> String {
        format!("你是{}。用户说: {}\n请自然地回复。", role_name, user_input)
    }

    #[must_use]
    pub fn build_system_prompt(role_name: &str) -> String {
        format!(
            "你是一个名叫{}的AI角色。请以这个角色的身份进行对话，保持一致的性格和语气。",
            role_name
        )
    }

    #[must_use]
    pub fn build_guidance_prompt(core_personality: &str) -> String {
        format!(
            "你的核心性格是: {}\n请根据这个性格特征来指导你的回复。",
            core_personality
        )
    }

    /// Picks the highest-weight topic for the current scene from `memory_config.topic_weights` for a one-line prompt hint.
    #[must_use]
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
        assert!(prompt.contains("【对话硬约束】"));
        assert!(prompt.contains("回复篇幅倾向"));
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
        });
        assert!(prompt.contains("【包级质量锚点】仅测试覆盖用。"));
        assert!(!prompt.contains("【回复质量锚点】（每轮须遵守）"));
        assert!(prompt.contains("【对话硬约束】"));
        assert!(prompt.contains("禁止复读开场"));
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
        });
        assert!(prompt.contains("【关系过渡】"));
        assert!(prompt.contains(hint));
        let tone_idx = prompt.find("语气区块").unwrap();
        let content_idx = prompt.find("内容区块").unwrap();
        let hint_idx = prompt.find("【关系过渡】").unwrap();
        assert!(tone_idx < hint_idx && hint_idx < content_idx);
    }
}
