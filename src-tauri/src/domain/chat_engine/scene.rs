//! 场景位移意图：规则命中 → 可选 LLM 判定（仅判断是否「要去/前往某地」，不解析目标 scene_id）

use crate::infrastructure::llm::LlmClient;
use crate::state::AppState;
use crate::utils::json_loose::extract_json_object;
use chrono::Utc;
use serde_json::Value;
use std::sync::Arc;

const MOVE_VERBS: &[&str] = &[
    "去", "来", "回", "到", "进", "出", "逛", "前往", "回到", "来到",
];

/// 用户明确邀请角色「同行」时的常见短语（规则命中，不解析目的地）
const TOGETHER_INVITE_PHRASES: &[&str] = &[
    "一起",
    "同行",
    "跟我来",
    "跟我去",
    "带上你",
    "咱们",
    "陪我",
    "你也来",
];

pub(super) fn together_travel_intent_by_rules(user_message: &str) -> bool {
    let msg = user_message.trim();
    if msg.is_empty() {
        return false;
    }
    TOGETHER_INVITE_PHRASES.iter().any(|p| msg.contains(p))
}

/// `detect_movement_intent` 为 true 时，拆成「选目的地条」与「邀请同行确认」二选一（同行优先）。
pub(super) fn movement_ui_flags(movement_intent: bool, user_message: &str) -> (bool, bool) {
    if !movement_intent {
        return (false, false);
    }
    if together_travel_intent_by_rules(user_message) {
        return (false, true);
    }
    (true, false)
}

/// 规则：位移动词 + 任一其它场景在 keywords/events 上有命中（不使用 scene_id/展示名 宽泛匹配）
pub(super) fn movement_intent_by_rules(
    user_message: &str,
    current_scene_id: &str,
    candidates: &[(String, String, Vec<String>, Vec<String>)],
) -> bool {
    let msg = user_message.trim();
    if msg.is_empty() {
        return false;
    }
    if !MOVE_VERBS.iter().any(|v| msg.contains(v)) {
        return false;
    }
    for (scene_id, _label, keywords, events) in candidates {
        if scene_id == current_scene_id {
            continue;
        }
        let mut score = 0i32;
        for kw in keywords {
            if msg.contains(kw.as_str()) {
                score += 2;
            }
        }
        for ev in events {
            if msg.contains(ev.as_str()) {
                score += 1;
            }
        }
        if score > 0 {
            return true;
        }
    }
    false
}

pub(super) fn parse_movement_intent_ai_output(raw: &str) -> Option<(bool, f64)> {
    let direct = serde_json::from_str::<Value>(raw.trim());
    let val = direct
        .ok()
        .or_else(|| extract_json_object(raw).and_then(|s| serde_json::from_str::<Value>(s).ok()))?;
    let intent = val.get("movement_intent").and_then(|v| {
        if let Some(b) = v.as_bool() {
            Some(b)
        } else if let Some(s) = v.as_str() {
            match s.trim().to_ascii_lowercase().as_str() {
                "true" | "yes" | "1" => Some(true),
                "false" | "no" | "0" => Some(false),
                _ => None,
            }
        } else {
            None
        }
    })?;
    let confidence = match val.get("confidence") {
        Some(Value::Number(n)) => n.as_f64().unwrap_or(0.0),
        Some(Value::String(s)) => s.trim().parse::<f64>().ok().unwrap_or(0.0),
        _ => 0.0,
    };
    Some((intent, confidence.clamp(0.0, 1.0)))
}

/// 是否应向前端提供「选目的地」条：不写入 DB、不解析 `scene_id`。
pub(super) async fn detect_movement_intent(
    state: &AppState,
    llm: &Arc<dyn LlmClient>,
    role_id: &str,
    scene_id: &str,
    scenes: &[String],
    user_message: &str,
    ollama_model: &str,
) -> bool {
    if scenes.len() <= 1 {
        return false;
    }

    let virtual_time_ms = state
        .db_manager
        .get_virtual_time_ms(role_id)
        .await
        .ok()
        .flatten()
        .unwrap_or_else(|| Utc::now().timestamp_millis());

    let mut candidate_scenes: Vec<(String, String, Vec<String>, Vec<String>)> = Vec::new();
    for sid in scenes {
        if sid == scene_id {
            continue;
        }
        if !state
            .storage
            .is_scene_time_allowed(role_id, sid.as_str(), virtual_time_ms)
        {
            continue;
        }
        candidate_scenes.push((
            sid.clone(),
            state.storage.scene_display_name(role_id, sid),
            state.storage.scene_keywords(role_id, sid),
            state.storage.scene_events(role_id, sid),
        ));
    }

    if candidate_scenes.is_empty() {
        return false;
    }

    if movement_intent_by_rules(user_message, scene_id, &candidate_scenes) {
        return true;
    }

    let candidate_lines = candidate_scenes
        .iter()
        .map(|(sid, label, kws, _)| {
            let hint = state.storage.scene_switch_hint_line(role_id, sid);
            format!(
                "- id={} 名称={} keywords={} 摘要={}",
                sid,
                label,
                kws.join("、"),
                hint
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    let prompt = format!(
        r#"你是位移意图判定器。判断用户是否表达「要去/来/回/前往某地」等与空间移动相关的意图。
仅输出 JSON：{{"movement_intent":true或false,"confidence":0~1}}

规则：
1) 仅当用户明显表达想去另一个地方、出门、前往、回到某处等位移语义时 movement_intent 为 true。
2) 闲聊、情绪表达、无位移含义时 movement_intent 为 false，confidence 可低于 0.5。
3) 不要猜测具体目的地；只判断是否有位移意图。

当前场景: {current_scene}
可前往的其它场景（供理解语义，不要求你输出 id）:
{candidate_lines}
用户消息: {msg}"#,
        current_scene = scene_id,
        candidate_lines = candidate_lines,
        msg = user_message
    );
    if let Ok(raw) = llm.generate_tag(ollama_model, &prompt).await {
        if let Some((intent, conf)) = parse_movement_intent_ai_output(&raw) {
            // 问卷「平衡」：略宽于保守阈值，减少漏检；仍要求 intent=true
            if intent && conf >= 0.63 {
                return true;
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn movement_ui_flags_prefers_together_offer() {
        assert_eq!(movement_ui_flags(true, "我们去教室自习"), (true, false));
        assert_eq!(movement_ui_flags(true, "我们一起去教室自习"), (false, true));
    }

    #[test]
    fn movement_intent_rules_requires_verb_and_keyword() {
        let c = vec![(
            "school".to_string(),
            "学校".to_string(),
            vec!["教室".to_string()],
            vec![],
        )];
        assert!(!movement_intent_by_rules("今天好累", "home", &c));
        assert!(!movement_intent_by_rules("去学校", "home", &c));
        assert!(movement_intent_by_rules("去教室自习", "home", &c));
    }

    #[test]
    fn parse_movement_intent_ai_output_accepts_json() {
        let r = parse_movement_intent_ai_output(r#"{"movement_intent":true,"confidence":0.9}"#);
        assert_eq!(r, Some((true, 0.9)));
        let r2 = parse_movement_intent_ai_output(r#"{"movement_intent":false,"confidence":0.2}"#);
        assert_eq!(r2, Some((false, 0.2)));
    }
}
