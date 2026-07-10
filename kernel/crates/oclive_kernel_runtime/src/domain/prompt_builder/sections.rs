//! Prompt section builders (`build_core_hard_constraint`, tone blocks, etc.).
//!
//! **Scope note (K-DOC-16):** Chinese literals here are product prompt text, not comments.

use super::*;

use crate::models::{Memory, PersonalitySource, Role};

impl PromptBuilder {
    #[must_use]
    pub(super) fn build_core_hard_constraint(
        role: &Role,
        persona_override: Option<&str>,
    ) -> String {
        let mut core = String::new();
        core.push_str(&format!("你是{}。\n", role.name));
        core.push_str("【核心设定·不可违背】以下是你不可违背的核心设定\n");
        let persona_text = persona_override
            .filter(|s| !s.trim().is_empty())
            .or_else(|| {
                if role.core_personality.trim().is_empty() {
                    None
                } else {
                    Some(role.core_personality.as_str())
                }
            });
        if let Some(text) = persona_text {
            if role.evolution_config.personality_source == PersonalitySource::Profile {
                core.push_str(&format!(
                    "核心性格档案（创作者与用户设定，运行时 AI 不得改写；与可变档案冲突时以本段为准）:\n{}\n",
                    text.trim()
                ));
            } else {
                core.push_str(&format!("核心人设:\n{}\n", text.trim()));
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
    pub(super) fn build_ephemeral_archive_block(ephemeral_personality: &str) -> String {
        let text = ephemeral_personality.trim();
        if text.is_empty() {
            return String::new();
        }
        format!("【局面摘要】（临时状态，会过期；与核心/可变档案冲突以核心为准）\n{text}\n\n")
    }

    /// Personality supplement is now text-first: persona comes from the core hard
    /// constraint plus the mutable personality archive narrative. Numeric seven-dim
    /// values are display-only and must not be rendered into the prompt.
    #[must_use]
    pub(super) fn build_personality_supplement(role: &Role, mutable_personality: &str) -> String {
        let mut supplement = String::new();
        if !role.description.trim().is_empty() {
            supplement.push_str(&format!("描述: {}\n", role.description));
        }
        let m = mutable_personality.trim();
        if !m.is_empty() {
            supplement.push_str(
                "【可变性格档案】（由模型在规则内根据对话维护，用于把握相处中的有限变化；创作者不可手写本条；与核心档案冲突时以核心为准）\n",
            );
            supplement.push_str(m);
            supplement.push('\n');
        }
        supplement
    }

    #[must_use]
    pub(super) fn build_character_status_summary(input: &PromptInput<'_>) -> String {
        let mut parts = Vec::new();
        if !input.user_emotion.trim().is_empty() {
            parts.push(format!("用户语气线索：{}。", input.user_emotion.trim()));
        }
        if !input.scene_label.is_empty() {
            parts.push(format!("场景为 {}。", input.scene_label));
        }
        if !input.host_state_expression_hint.trim().is_empty() {
            parts.push(input.host_state_expression_hint.trim().to_string());
        }
        if parts.is_empty() {
            return String::new();
        }
        format!("【角色当前状态】{}\n", parts.join(""))
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

    /// Static authenticity guardrail (no numeric state): replaces the former
    /// favor/relation/event-tone injection. Tone is driven by the persona text and
    /// mutable personality archive, not by relation stage or seven-dim numbers.
    pub(super) fn build_authenticity_constraint() -> &'static str {
        "【真实性约束】不要编造系统状态：不要虚构未发生的关系跳变、共同经历或历史事件。\n"
    }

    /// Care-package keywords used to detect templated repeat concern lists.
    const CARE_PACKAGE_KEYWORDS: &'static [&'static str] = &[
        "出门",
        "晒太阳",
        "作业",
        "早睡",
        "熬夜",
        "热水",
        "暖手",
        "喝水",
        "记得",
        "注意安全",
        "早点睡",
        "写完",
        "多穿",
    ];

    fn care_package_keyword_hits(text: &str) -> usize {
        Self::CARE_PACKAGE_KEYWORDS
            .iter()
            .filter(|w| text.contains(**w))
            .count()
    }

    /// When the previous assistant reply is known, inject a footer constraint before the quality anchor.
    #[must_use]
    pub(super) fn build_previous_reply_constraint(prev: &str) -> Option<String> {
        let prev = prev.trim();
        if prev.is_empty() {
            return None;
        }
        let mut block = String::from("【上一轮回复约束】\n");
        block.push_str("- 勿大段复述上一轮助手回复；用户短确认时勿重新展开已说过的关心清单。\n");
        if Self::care_package_keyword_hits(prev) >= 2 {
            block.push_str(
                "- 上一轮已出现「出门/作业/早睡/热水」类叮嘱，本轮禁止原样复读或打包再问。\n",
            );
        }
        Some(block)
    }
}
