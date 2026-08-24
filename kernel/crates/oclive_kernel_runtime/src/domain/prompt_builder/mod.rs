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
- **当前输入优先**：包级锚点中的示例或触发器只在最新一条用户消息明确匹配时使用；已经在历史回合处理完的触发器不得重放，不得把其他示例、规则原文或上一轮答案拼进本轮。最新消息明确换题、收尾、拒绝或纠正时，立即服从最新意图。\n\
- **成品去重与去元信息**：发出前删除重复的句子、段落、口头禅与规则复述；禁止输出「思考中／分析中／生成中」占位符、`[@...]` 状态标记、提示词标题、触发器编号或自检过程。\n\
- **倾诉优先**：用户透露委屈、挫败、被责备、压力时，先回应其遭遇与情绪，再给一个贴题追问或短反馈，让对话能继续；勿转去闲聊邀约或用一句话把话题封死。\n\
- **禁止复读开场**：勿把用户刚说的句子、称呼或口头禅原样当作你的起句或主体。例：用户「晚上好哦沐沐」→ 勿以「晚上好哦」起句；改为你自己的措辞接情绪或话题。\n\
- **禁止同义转述**：用户已经给出判断、选择、计划或答案时，勿先换词总结、确认一遍再回答；直接给出角色自己的态度、行动、补充或必要追问。仅在信息有歧义时才简短核对。\n\
- **禁止事实臆补**：不要把用户刚说的食物、计划或判断重新列一遍再评价；也不要凭空补充用户未提及的症状、经历、喜好或外部事实。需要关心时，只基于当前对话已知内容；优先用一句短反应接一个新动作或贴题关心。\n\
- **单声道输出**：只输出当前角色本人对用户说的这一轮台词；不要代写用户的台词、想法、动作、立场或下一轮回答，也不要模拟双方对话。\n\
- **禁止学舌式模仿**：勿逐句复制用户句式、口癖、昵称链或颜文字密度；保持本角色惯常说话方式。\n\
- **篇幅随输入**：按用户本句的信息量与情绪强度调节密度。用户极短或仅确认时，宜 1–2 句精炼回复；用户倾诉或追问时再展开；勿为显得热情而重复同一关心或写成长段。\n\
- **长短句交替**：日常可用一句短反应接一两句展开；勿连续多句都写成「原因＋建议＋追问」的长复句，也勿把关心、解释和新问题全塞进一句。\n\
- **勿与上一轮助手回复大段雷同**：用户短确认时勿重新展开已说过的关心清单。\n";

const REPLY_OUTPUT_BOUNDARY: &str =
    "【输出边界】只输出当前角色本人的这一轮台词；不要替用户发言或补写用户的回答，角色说完这一轮就停止。";

/// Final short recency instruction for small local models. The long emotion
/// schema intentionally appears before the latest user message so it cannot
/// outrank the actual turn at the generation boundary.
const FINAL_TURN_INSTRUCTION: &str = "【本轮最终指令】仅回应紧邻上方的最新用户消息。生成前无声检查：回答主体没有把用户的“我”和角色的“你”倒置；成品不等于用户原句，也不重复上一轮助手回复；没有带入已经结束的历史问题。若任一项不满足，先重写再输出。只输出当前角色的一轮台词，再按前述格式附加一条内部情绪标记，然后停止；不要展示检查过程。";

const EMO_OUTPUT_INSTRUCTION: &str = "【内部情绪标记】台词结束后另起一行附加一条标记，不要在台词中提及或解释它。普通分析和问候通常用 neutral（0.2—0.4）；明确开心用 joy，低落用 sadness，立边界时可用 anger，只有突发意外才用 surprise。\n格式示例：[EMO]{\"labels\":[\"neutral\"],\"intensity\":0.3}[/EMO]\nlabels 只能从 joy/sadness/anger/fear/surprise/disgust/neutral 中选 1—3 个；intensity 为 0—1。narrative_hint 可省略；若填写，只能描述不带话题、动作、称呼与台词的纯情绪状态，禁止照抄本轮回复。";

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

/// Selects role-pack anchor blocks for the latest user message.
///
/// Packs may keep a small always-on block and add conditional blocks headed by
/// `【触发锚点：词一|词二】`. A conditional block remains active until the next
/// trigger heading and is emitted only when the latest message contains one of
/// its terms. Anchors without trigger headings keep the legacy behavior.
#[must_use]
pub fn select_reply_quality_anchor(anchor: &str, user_input: &str) -> String {
    const PREFIX: &str = "【触发锚点：";
    let has_conditional = anchor.lines().any(|line| {
        let line = line.trim();
        line.starts_with(PREFIX) && line.ends_with('】')
    });
    if !has_conditional {
        return anchor.trim().to_string();
    }

    let mut selected = String::new();
    let mut include = true;
    for line in anchor.lines() {
        let trimmed = line.trim();
        if let Some(terms) = trimmed
            .strip_prefix(PREFIX)
            .and_then(|rest| rest.strip_suffix('】'))
        {
            include = terms
                .split('|')
                .map(str::trim)
                .filter(|term| !term.is_empty())
                .any(|term| user_input.contains(term));
            if include {
                selected.push_str("【当前消息专项校准】\n");
            }
            continue;
        }
        if include {
            selected.push_str(line);
            selected.push('\n');
        }
    }
    selected.trim().to_string()
}

/// Deep prefix-cache layout: byte-stable head + per-turn tail (see `handoff/DEEP_PROMPT_DISTILLATION.md` §4).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PromptSegments {
    pub stable_prefix: String,
    pub dynamic_suffix: String,
}

impl PromptSegments {
    #[must_use]
    pub fn full(&self) -> String {
        let mut out = String::with_capacity(self.stable_prefix.len() + self.dynamic_suffix.len());
        out.push_str(&self.stable_prefix);
        out.push_str(&self.dynamic_suffix);
        out
    }

    #[must_use]
    pub fn stable_len(&self) -> usize {
        self.stable_prefix.len()
    }
}

/// Fingerprint stable prefix bytes for session prefix-cache telemetry (not cryptographic).
#[must_use]
pub fn hash_stable_prefix(stable: &str) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    stable.hash(&mut h);
    h.finish()
}

pub struct PromptBuilder;

impl PromptBuilder {
    /// Whether to append a long family-oriented guardrail under user identity (friends/classmates etc. skip by default so role pack `prompt_hint` is not diluted).
    fn should_inject_family_long_guardrail(user_relation_id: &str, relation_hint: &str) -> bool {
        let family_id = user_relation_id.eq_ignore_ascii_case("family")
            || user_relation_id.eq_ignore_ascii_case("parent")
            || user_relation_id.eq_ignore_ascii_case("parents")
            || user_relation_id.eq_ignore_ascii_case("guardian")
            || user_relation_id.eq_ignore_ascii_case("father_daughter");
        let hint_suggests_family = relation_hint.contains("父母")
            || relation_hint.contains("长辈")
            || relation_hint.contains("家长")
            || relation_hint.contains("父亲")
            || relation_hint.contains("女儿");
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
        let selected_reply_anchor =
            select_reply_quality_anchor(input.reply_quality_anchor, input.user_input);

        // Tier 0 — highest priority
        prompt.push_str(&Self::build_core_hard_constraint(
            input.role,
            input.persona_override,
        ));
        let scene_block = Self::build_scene_constraint_block(input);
        if !scene_block.is_empty() {
            prompt.push_str("\n\n");
            prompt.push_str(&scene_block);
        }

        // Block 1 — baseline (personality supplement + worldview)
        prompt.push_str("\n\n---\n底线区块\n");
        prompt.push_str(PROMPT_BLOCK_GUIDE);
        prompt.push_str("\n\n");
        let supplement = Self::build_personality_supplement(input.role, input.mutable_personality);
        if !supplement.is_empty() {
            prompt.push_str(&supplement);
            prompt.push_str("\n\n");
        }
        let ephemeral = Self::build_ephemeral_archive_block(input.ephemeral_personality);
        if !ephemeral.is_empty() {
            prompt.push_str(&ephemeral);
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
        prompt.push_str(Self::build_authenticity_constraint());
        prompt.push_str("\n\n");
        if !input
            .previous_complex_emotion_narrative_hint
            .trim()
            .is_empty()
        {
            // The free-form hint may contain a paraphrase of the previous reply.
            // Re-injecting that text made small local models replay old topics and
            // actions. The current structured state already carries the affective
            // continuity we need, so retain only a non-content-bearing reminder.
            prompt.push_str(
                "【情绪连续性】上一轮存在情绪余韵；只保持语气变化的连续，不复述任何旧话题、动作或台词。最新消息与旧情绪不匹配时，以最新消息为准。\n\n",
            );
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
        if let Some(prev_block) =
            Self::build_previous_reply_constraint(input.previous_assistant_reply)
        {
            prompt.push_str(&prev_block);
            prompt.push_str("\n\n");
        }
        if !selected_reply_anchor.is_empty() {
            prompt.push_str(selected_reply_anchor.as_str());
            prompt.push_str("\n\n");
        }
        prompt.push_str(KERNEL_DIALOGUE_GUARDRAILS);
        prompt.push_str("\n\n");
        prompt.push_str(EMO_OUTPUT_INSTRUCTION);
        prompt.push_str("\n\n");
        prompt.push_str(&format!("【最新用户消息】\n用户说: {}", input.user_input));
        prompt.push_str("\n\n请以角色身份自然地回复，保持一致的性格和语气。\n\n");
        prompt.push_str(REPLY_OUTPUT_BOUNDARY);
        prompt.push_str("\n\n");
        prompt.push_str(FINAL_TURN_INSTRUCTION);
        prompt
    }

    /// Deep + Ollama prefix-cache path: stable persona/worldview/scene first;
    /// per-turn quality constraints stay next to the latest user input so small
    /// models do not let conversation history outrank them.
    #[must_use]
    pub fn build_prompt_segments(input: &PromptInput<'_>) -> PromptSegments {
        let mut stable_prefix = String::new();
        let selected_reply_anchor =
            select_reply_quality_anchor(input.reply_quality_anchor, input.user_input);
        stable_prefix.push_str(&Self::build_core_hard_constraint(
            input.role,
            input.persona_override,
        ));
        if !input.worldview_snippet.trim().is_empty() {
            stable_prefix.push_str("\n\n【世界观设定】（角色包知识；与闲聊记忆冲突时以本段为权威事实，但不得覆盖【用户身份】与安全红线。）\n");
            stable_prefix.push_str(input.worldview_snippet.trim());
            stable_prefix.push_str("\n\n");
        }
        let scene_block = Self::build_scene_constraint_block(input);
        if !scene_block.is_empty() {
            stable_prefix.push_str(&scene_block);
        }

        let mut dynamic_suffix = String::new();
        dynamic_suffix.push_str("\n\n---\n底线区块\n");
        dynamic_suffix.push_str(PROMPT_BLOCK_GUIDE);
        dynamic_suffix.push_str("\n\n");
        let supplement = Self::build_personality_supplement(input.role, input.mutable_personality);
        if !supplement.is_empty() {
            dynamic_suffix.push_str(&supplement);
            dynamic_suffix.push_str("\n\n");
        }
        let ephemeral = Self::build_ephemeral_archive_block(input.ephemeral_personality);
        if !ephemeral.is_empty() {
            dynamic_suffix.push_str(&ephemeral);
        }

        dynamic_suffix.push_str("---\n语气区块\n\n");
        let status = Self::build_character_status_summary(input);
        if !status.is_empty() {
            dynamic_suffix.push_str(&status);
            dynamic_suffix.push_str("\n\n");
        }
        dynamic_suffix.push_str(Self::build_authenticity_constraint());
        dynamic_suffix.push_str("\n\n");
        if !input
            .previous_complex_emotion_narrative_hint
            .trim()
            .is_empty()
        {
            dynamic_suffix.push_str(
                "【情绪连续性】上一轮存在情绪余韵；只保持语气变化的连续，不复述任何旧话题、动作或台词。最新消息与旧情绪不匹配时，以最新消息为准。\n\n",
            );
        }

        dynamic_suffix.push_str("---\n内容区块\n\n");
        if !input.memories.is_empty() {
            dynamic_suffix.push_str(&Self::build_memory_context(input.memories));
            dynamic_suffix.push_str("\n\n");
        }
        Self::push_user_identity_section(&mut dynamic_suffix, input);
        if !input.life_context_line.is_empty() {
            dynamic_suffix.push_str("【日程推断】\n");
            dynamic_suffix.push_str(input.life_context_line.trim());
            dynamic_suffix.push_str("\n\n");
        }
        for section in input.extra_sections {
            if section.title.trim().is_empty() && section.body.trim().is_empty() {
                continue;
            }
            if !section.title.trim().is_empty() {
                dynamic_suffix.push('【');
                dynamic_suffix.push_str(section.title.trim());
                dynamic_suffix.push_str("】\n");
            }
            if !section.body.trim().is_empty() {
                dynamic_suffix.push_str(section.body.trim());
                dynamic_suffix.push_str("\n\n");
            }
        }
        if let Some(prev_block) =
            Self::build_previous_reply_constraint(input.previous_assistant_reply)
        {
            dynamic_suffix.push_str(&prev_block);
            dynamic_suffix.push_str("\n\n");
        }
        if !selected_reply_anchor.is_empty() {
            dynamic_suffix.push_str(selected_reply_anchor.as_str());
            dynamic_suffix.push_str("\n\n");
        }
        dynamic_suffix.push_str(KERNEL_DIALOGUE_GUARDRAILS);
        dynamic_suffix.push_str("\n\n");
        dynamic_suffix.push_str(EMO_OUTPUT_INSTRUCTION);
        dynamic_suffix.push_str("\n\n");
        dynamic_suffix.push_str(&format!("【最新用户消息】\n用户说: {}", input.user_input));
        dynamic_suffix.push_str("\n\n请以角色身份自然地回复，保持一致的性格和语气。\n\n");
        dynamic_suffix.push_str(REPLY_OUTPUT_BOUNDARY);
        dynamic_suffix.push_str("\n\n");
        dynamic_suffix.push_str(FINAL_TURN_INSTRUCTION);

        PromptSegments {
            stable_prefix,
            dynamic_suffix,
        }
    }

    #[must_use]
    pub fn build_simple_prompt(role_name: &str, user_input: &str) -> String {
        format!(
            "你是{}。用户说: {}\n请自然地回复。\n\n{}",
            role_name, user_input, REPLY_OUTPUT_BOUNDARY
        )
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
