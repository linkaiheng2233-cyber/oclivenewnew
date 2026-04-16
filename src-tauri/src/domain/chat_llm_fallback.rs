//! 主对话 LLM 不可用时的应急回复：长度受「话多」维度（talkativeness）影响，略拟人。

use crate::models::{EventType, PersonalityVector, Role};

pub struct FallbackReplyContext<'a> {
    pub relation_before: &'a str,
    pub relation_preview: &'a str,
    pub favorability_before: f64,
    pub event_type: &'a EventType,
    pub impact_factor: f64,
}

/// `talkativeness` 0~1：越低越短句，越高可稍长（约 28~220 字量级）。
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
    let transition_hint = if preview_upward {
        "语气先自然延续、只小幅升温。"
    } else {
        "语气保持当前关系边界，不突兀升阶。"
    };
    let stance = if conflict_mode {
        "我听到了。我们先把这件事说清楚。"
    } else if low_intimacy {
        "我知道了，我们先从这一步说起。"
    } else {
        "我接住你的意思了，我们继续。"
    };
    let base = if t < 0.35 {
        format!(
            "{}：{}{}",
            role.name,
            snippet,
            if conflict_mode { "。" } else { "…" }
        )
    } else if t < 0.55 {
        format!(
            "（有点卡）{}：你说的「{}」，{} {}",
            role.name, snippet, stance, transition_hint
        )
    } else {
        format!(
            "（模型暂时连不上，先这样回你）{}：关于「{}」，{} {}",
            role.name, snippet, stance, transition_hint
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
            plugin_backends: crate::models::PluginBackends::default(),
            ui_config: crate::models::UiConfig::default(),
            knowledge_index: None,
            author_pack: None,
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
