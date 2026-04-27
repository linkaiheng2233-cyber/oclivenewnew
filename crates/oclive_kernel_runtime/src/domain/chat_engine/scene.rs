//! 场景位移意图：纯规则与输出解析（迁移中）。

use crate::utils::json_loose::extract_json_object;
use serde_json::Value;

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

pub fn together_travel_intent_by_rules(user_message: &str) -> bool {
    let msg = user_message.trim();
    if msg.is_empty() {
        return false;
    }
    TOGETHER_INVITE_PHRASES.iter().any(|p| msg.contains(p))
}

/// `movement_intent` 为 true 时，拆成「选目的地条」与「邀请同行确认」二选一（同行优先）。
pub fn movement_ui_flags(movement_intent: bool, user_message: &str) -> (bool, bool) {
    if !movement_intent {
        return (false, false);
    }
    if together_travel_intent_by_rules(user_message) {
        return (false, true);
    }
    (true, false)
}

/// 规则：位移动词 + 任一其它场景在 keywords/events 上有命中（不使用 scene_id/展示名 宽泛匹配）
pub fn movement_intent_by_rules(
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

pub fn parse_movement_intent_ai_output(raw: &str) -> Option<(bool, f64)> {
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
