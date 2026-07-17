//! Portrait director AI: closed-set asset id selection (Phase 3; merges portrait_emotion LLM path).

use crate::domain::portrait_emotion_engine::{
    apply_persona_event_overrides, fallback_base_from_emotion,
};
use crate::domain::portrait_facility::rule::{
    resolve_visual_state_rule, resolve_visual_state_rule_with_intensity, validate_asset_id,
};
use crate::domain::ports::LlmClient;
use crate::error::Result;
use crate::models::{Emotion, Event, PersonalityVector, Role};
use oclive_kernel_types::models::PortraitCatalogFile;
use std::sync::Arc;

pub(crate) fn portrait_director_enabled() -> bool {
    std::env::var("OCLIVE_PORTRAIT_EMOTION_LLM")
        .ok()
        .map(|v| {
            !matches!(
                v.trim().to_ascii_lowercase().as_str(),
                "0" | "false" | "no" | "off"
            )
        })
        .unwrap_or(true)
}

fn parse_asset_id(raw: &str, catalog: &PortraitCatalogFile) -> Option<String> {
    let t = raw
        .trim()
        .trim_matches(|c| c == '`' || c == '"' || c == '\'');
    let first = t.split_whitespace().next().unwrap_or("");
    validate_asset_id(catalog, first)
}

#[allow(clippy::too_many_arguments)]
fn build_director_prompt(
    role: &Role,
    catalog: &PortraitCatalogFile,
    core_personality: &PersonalityVector,
    personality: &PersonalityVector,
    favorability: f64,
    user_message: &str,
    reply: &str,
    user_emotion_str: &str,
    bot_emotion: &Emotion,
    recent_events: &[Event],
    recent_turns: &[(String, String)],
    narrative_hint: Option<&str>,
) -> String {
    let asset_lines: String = catalog
        .assets
        .iter()
        .map(|a| {
            let tags = a.tags.join(",");
            format!("- {} | {} | tags=[{}]", a.id, a.desc, tags)
        })
        .collect::<Vec<_>>()
        .join("\n");

    let ev_line = if recent_events.is_empty() {
        "（无）".to_string()
    } else {
        recent_events
            .iter()
            .take(4)
            .map(|e| format!("{:?}", e.event_type))
            .collect::<Vec<_>>()
            .join(", ")
    };

    let turns = if recent_turns.is_empty() {
        "（首轮或无历史）".to_string()
    } else {
        recent_turns
            .iter()
            .rev()
            .take(3)
            .map(|(u, b)| format!("用户:{} 角色:{}", truncate(u, 80), truncate(b, 80)))
            .collect::<Vec<_>>()
            .join("\n")
    };

    let hint_block = narrative_hint
        .filter(|h| !h.trim().is_empty())
        .map(|h| format!("\n复杂情感叙事提示：{}\n", truncate(h, 200)))
        .unwrap_or_default();

    format!(
        r#"你是角色「{}」的表现导演。从下列**封闭**立绘 id 中选一个最符合此刻对用户态度与关系张力的状态。
只能输出列表中的一个 id，不要解释。

可选 id（id | 描述 | tags）：
{}

原则：
1) 表情符合角色对用户的真实态度，不是复述用户情绪。
2) 综合七维性格与近期事件；争吵未和解时勿轻易选开心 id。
3) 好感度调节关系松紧。

人设摘要：{}
核心性格 倔强{:.2} 黏人{:.2} 敏感{:.2} 强势{:.2} 宽容{:.2} 话多{:.2} 温暖{:.2}
当前有效性格 倔强{:.2} 黏人{:.2} 敏感{:.2} 强势{:.2} 宽容{:.2} 话多{:.2} 温暖{:.2}
好感度：{:.1}
近期事件：{}
最近几轮：
{}{}
本回合：
- 用户说：{}
- 角色回复：{}
- 用户情绪：{}
- 回复粗读情绪（参考）：{}

只输出一个 id。"#,
        role.name,
        asset_lines,
        if role.description.trim().is_empty() {
            role.core_personality.as_str()
        } else {
            role.description.as_str()
        },
        core_personality.stubbornness,
        core_personality.clinginess,
        core_personality.sensitivity,
        core_personality.assertiveness,
        core_personality.forgiveness,
        core_personality.talkativeness,
        core_personality.warmth,
        personality.stubbornness,
        personality.clinginess,
        personality.sensitivity,
        personality.assertiveness,
        personality.forgiveness,
        personality.talkativeness,
        personality.warmth,
        favorability,
        ev_line,
        turns,
        hint_block,
        user_message,
        reply,
        user_emotion_str,
        bot_emotion,
    )
}

fn truncate(s: &str, max: usize) -> String {
    let mut t = s.trim().replace('\n', " ");
    if t.chars().count() > max {
        t = t.chars().take(max).collect::<String>() + "…";
    }
    t
}

fn emotion_tag_from_visual_state(catalog: &PortraitCatalogFile, visual_state_id: &str) -> String {
    catalog
        .assets
        .iter()
        .find(|a| a.id == visual_state_id)
        .and_then(|a| a.tags.first())
        .map(|t| t.to_ascii_lowercase())
        .unwrap_or_else(|| "neutral".to_string())
}

/// Returns `(portrait_emotion_tag, visual_state_id)` when catalog is active.
///
/// # Errors
///
/// Returns `AppError` when the director LLM call fails or the chosen asset id is invalid.
#[allow(clippy::too_many_arguments)]
pub async fn pick_portrait_with_catalog(
    llm: &Arc<dyn LlmClient>,
    ollama_model: &str,
    role: &Role,
    catalog: &PortraitCatalogFile,
    core_personality: &PersonalityVector,
    personality: &PersonalityVector,
    favorability: f64,
    user_message: &str,
    reply: &str,
    user_emotion_str: &str,
    bot_emotion: &Emotion,
    recent_events: &[Event],
    recent_turns: &[(String, String)],
    narrative_hint: Option<&str>,
    intensity: f64,
) -> Result<(String, String)> {
    let mut visual_state_id = if portrait_director_enabled() {
        let prompt = build_director_prompt(
            role,
            catalog,
            core_personality,
            personality,
            favorability,
            user_message,
            reply,
            user_emotion_str,
            bot_emotion,
            recent_events,
            recent_turns,
            narrative_hint,
        );
        match llm.generate_tag(ollama_model, &prompt).await {
            Ok(raw) => parse_asset_id(&raw, catalog).unwrap_or_else(|| {
                resolve_visual_state_rule_with_intensity(
                    catalog,
                    &bot_emotion.to_string(),
                    Some(intensity),
                )
                .unwrap_or_else(|| "neutral_default".to_string())
            }),
            Err(e) => {
                tracing::warn!("portrait_director LLM failed, rule fallback: {}", e);
                resolve_visual_state_rule_with_intensity(
                    catalog,
                    &bot_emotion.to_string(),
                    Some(intensity),
                )
                .unwrap_or_else(|| "neutral_default".to_string())
            }
        }
    } else {
        resolve_visual_state_rule_with_intensity(
            catalog,
            &fallback_base_from_emotion(bot_emotion, recent_turns),
            Some(intensity),
        )
        .unwrap_or_else(|| "neutral_default".to_string())
    };

    // The director may still choose a legacy/default id. If the catalog has
    // an intensity sibling, prefer it so upgraded packs are visible even
    // when the model returns a conservative default.
    if visual_state_id.ends_with("_default") {
        let tag = emotion_tag_from_visual_state(catalog, &visual_state_id);
        if let Some(preferred) =
            resolve_visual_state_rule_with_intensity(catalog, &tag, Some(intensity))
        {
            visual_state_id = preferred;
        }
    }

    if validate_asset_id(catalog, &visual_state_id).is_none() {
        visual_state_id = resolve_visual_state_rule_with_intensity(
            catalog,
            &bot_emotion.to_string(),
            Some(intensity),
        )
        .unwrap_or_else(|| "neutral_default".to_string());
    }

    let selected_portrait_tag = emotion_tag_from_visual_state(catalog, &visual_state_id);
    let mut portrait_tag = selected_portrait_tag.clone();
    portrait_tag =
        apply_persona_event_overrides(portrait_tag, user_emotion_str, recent_events, personality);

    if !portrait_tag.eq_ignore_ascii_case(&selected_portrait_tag) {
        if let Some(rule_id) = resolve_visual_state_rule(catalog, &portrait_tag) {
            visual_state_id = rule_id;
        }
    }

    Ok((portrait_tag, visual_state_id))
}
