//! Prompt construction: role, memory, relation, and scene topic hints.
//!
//! **Scope note (K-DOC-16):** Chinese string literals in this module and `sections.rs` are
//! intentional product prompt content (anchors, guardrails, section headers). Do not
//! English-ize them as code comments; contract-layer `//` comments stay English.

use crate::models::Role;
pub use oclive_kernel_types::PromptInput;

/// Engine default quality anchor (role pack `reply_quality_anchor` may replace the whole block).
/// General dialogue discipline lives in `KERNEL_DIALOGUE_GUARDRAILS` and cannot be overridden by pack anchors.
pub const DEFAULT_REPLY_QUALITY_ANCHOR: &str = "【回复质量锚点】（每轮须遵守）\n\
- **用全新措辞接住用户本句的内容或情绪**，勿照搬其原句起笔。\n\
- **只写角色台词**；可共情追问，勿代写用户内心或立场。\n\
- **人设化倾听**：倾听方式受核心人设与七维影响，不强制“标准安慰模板”。可表现为同情、冷静分析、克制旁观、带锋芒的吐槽等，但须与人设一致，且不得恶意羞辱或无端攻击用户。\n\
- 使用自然、连贯的中文口语；避免同一套空洞寒暄、机械模板与无意义填充。\n\
- 保持人设、关系阶段与当前情绪一致。\n\
- 称呼、距离感须符合人设与当前关系阶段。\n\
- 先直接回应用户本句的具体内容、问题或情绪，再视需要延伸或反问。\n\
- 勿输出乱码、无关联英文碎片或填充词堆叠。\n\
- 避免连续多句同一套话或同一问法；勿重复用户已经回答过的问题。\n\
- 勿机械模仿用户消息里的颜文字密度或句式；用户未大量使用时保持自然口语。\n";

/// Always appended after pack/engine quality anchor; creators cannot disable or replace.
pub const KERNEL_DIALOGUE_GUARDRAILS: &str = "【对话硬约束】（引擎预设，不可被包级锚点替换；其余风格仍由人设发挥）\n\
- **状态延续**：用户「嗯/好/知道了」等短确认 → 顺势落实当前话题或你上一句提议，勿重新寒暄、勿重复已说过的关心。\n\
- **倾诉优先**：用户透露委屈、挫败、被责备、压力时，先回应其遭遇与情绪，再给一个贴题追问或短反馈，让对话能继续；勿转去闲聊邀约或用一句话把话题封死。\n\
- **禁止复读开场**：勿把用户刚说的句子、称呼或口头禅原样当作你的起句或主体。例：用户「晚上好哦沐沐」→ 勿以「晚上好哦」起句；改为你自己的措辞接情绪或话题。\n\
- **禁止学舌式模仿**：勿逐句复制用户句式、口癖、昵称链或颜文字密度；保持本角色惯常说话方式。\n\
- **篇幅随输入**：按用户本句的信息量与情绪强度调节密度。用户极短或仅确认时，宜 1–2 句精炼回复；用户倾诉或追问时再展开；勿为显得热情而重复同一关心或写成长段。\n";

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
            format!("正在从 {from} 向 {to} 过渡，表现出试探性亲近；勿一次跳到过热语气。")
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
        prompt.push_str("---\n语气区块\n\n");
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
        prompt.push_str("---\n内容区块\n\n");
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

        // Footer — extra sections → anchor → guardrails → user input → closing line
        for section in input.extra_sections {
            if section.title.trim().is_empty() && section.body.trim().is_empty() {
                continue;
            }
            if !section.title.trim().is_empty() {
                prompt.push('【');
                prompt.push_str(section.title.trim());
                prompt.push_str("】\n");
            }
            if !section.body.trim().is_empty() {
                prompt.push_str(section.body.trim());
                prompt.push_str("\n\n");
            }
        }
        if !input.reply_quality_anchor.trim().is_empty() {
            prompt.push_str(input.reply_quality_anchor.trim());
            prompt.push_str("\n\n");
        }
        prompt.push_str(KERNEL_DIALOGUE_GUARDRAILS);
        prompt.push_str("\n\n");
        prompt.push_str(&format!("用户说: {}", input.user_input));
        prompt.push_str("\n\n请以角色身份自然地回复，保持一致的性格和语气。");
        prompt
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

mod sections;

#[cfg(test)]
mod tests;
