//! Emergency reply when the main chat LLM is unavailable: length is influenced by the "talkativeness" dimension, slightly anthropomorphized.

use crate::models::{EventType, PersonalityVector, Role};

pub struct FallbackReplyContext<'a> {
    pub relation_before: &'a str,
    pub relation_preview: &'a str,
    pub favorability_before: f64,
    pub event_type: &'a EventType,
    pub impact_factor: f64,
}

/// `talkativeness` 0~1: lower means shorter sentences, higher allows slightly longer (roughly 28~220 characters).
#[must_use]
pub fn fallback_reply_for_llm_failure(
    role: &Role,
    personality: &PersonalityVector,
    user_message: &str,
    ctx: &FallbackReplyContext<'_>,
) -> String {
    let t = personality.talkativeness.clamp(0.0, 1.0);
    let max_chars = (28.0 + t * 190.0).round() as usize;
    let um = user_message.trim();
    let snippet = if um.chars().count() > 96 {
        um.chars().take(96).collect::<String>() + "…"
    } else {
        um.to_string()
    };
    let conflict_mode = matches!(ctx.event_type, EventType::Quarrel) || ctx.impact_factor < 0.0;
    let low_intimacy = ctx.favorability_before < 35.0
        || matches!(ctx.relation_before, "Stranger" | "Acquaintance" | "Friend");
    let preview_upward = ctx.relation_before != ctx.relation_preview;
    // Dialogue only; no behind-the-scenes hints like "tone/escalation" (otherwise it would leak into the chat like a system prompt).
    let body = if conflict_mode {
        "我们先把这件事说清楚，你慢慢讲。"
    } else if low_intimacy {
        if preview_upward {
            "我听到了。你想聊什么都可以，我们一步一步来。"
        } else {
            "嗯，我听到了。你接着说。"
        }
    } else if preview_upward {
        "好，我懂你的意思了，我们继续。"
    } else {
        "嗯，我在听，你继续。"
    };
    let base = if t < 0.35 {
        format!(
            "{}：{}{}",
            role.name,
            snippet,
            if conflict_mode { "。" } else { "…" }
        )
    } else if t < 0.55 {
        format!("（有点卡）{}：你刚说的「{}」，{}", role.name, snippet, body)
    } else {
        format!(
            "（模型暂时连不上，先这样回你）{}：关于「{}」，{}",
            role.name, snippet, body
        )
    };
    let count = base.chars().count();
    if count > max_chars {
        base.chars()
            .take(max_chars.saturating_sub(1))
            .collect::<String>()
            + "…"
    } else {
        base
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{EvolutionBounds, PersonalityDefaults};

    fn role() -> Role {
        Role {
            id: "r".to_string(),
            name: "测试".to_string(),
            description: String::new(),
            version: "1".to_string(),
            author: "t".to_string(),
            core_personality: String::new(),
            default_personality: PersonalityDefaults {
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
            deep_capsule_enabled: false,
            deep_capsule: None,
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
            pack_turn_thinking_config: None,
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
            source_dir: None,
        }
    }

    #[test]
    fn low_talk_short() {
        let r = role();
        let mut p = PersonalityVector::from(&r.default_personality);
        p.talkativeness = 0.2;
        let s = fallback_reply_for_llm_failure(
            &r,
            &p,
            "你好啊今天天气不错",
            &FallbackReplyContext {
                relation_before: "Stranger",
                relation_preview: "Stranger",
                favorability_before: 10.0,
                event_type: &EventType::Ignore,
                impact_factor: 0.0,
            },
        );
        assert!(s.chars().count() < 40, "{}", s);
    }

    #[test]
    fn high_talk_longer() {
        let r = role();
        let mut p = PersonalityVector::from(&r.default_personality);
        p.talkativeness = 0.95;
        let s = fallback_reply_for_llm_failure(
            &r,
            &p,
            "你好",
            &FallbackReplyContext {
                relation_before: "Friend",
                relation_preview: "CloseFriend",
                favorability_before: 62.0,
                event_type: &EventType::Praise,
                impact_factor: 0.4,
            },
        );
        assert!(s.chars().count() > 20, "{}", s);
    }
}
