//! 事件影响 AI 估计：一次 LLM 输出同时给出事件类型与影响因子。
//! 失败时回退到规则 `EventDetector`，保证稳定性。

use crate::domain::affect_policy::softness_coldness_volatility;
use crate::domain::event_detector::EventDetector;
use crate::domain::personality_engine::PersonalityEngine;
use crate::error::Result;
use crate::infrastructure::llm::LlmClient;
use crate::models::knowledge::KnowledgeEventAugment;
use crate::models::{Emotion, Event, EventType, PersonalityVector};
use crate::utils::json_loose::extract_json_object;
use serde_json::Value;
use std::sync::Arc;

pub fn event_impact_ai_enabled() -> bool {
    std::env::var("OCLIVE_EVENT_IMPACT_LLM")
        .ok()
        .map(|v| {
            !matches!(
                v.trim().to_ascii_lowercase().as_str(),
                "0" | "false" | "no" | "off"
            )
        })
        .unwrap_or(true)
}

fn parse_event_type_ai_token(raw: &str) -> Option<EventType> {
    let t = raw.trim();
    if t.is_empty() {
        return None;
    }
    let lower = t.to_ascii_lowercase();
    match lower.as_str() {
        "quarrel" => Some(EventType::Quarrel),
        "apology" => Some(EventType::Apology),
        "praise" => Some(EventType::Praise),
        "complaint" => Some(EventType::Complaint),
        "confession" => Some(EventType::Confession),
        "joke" => Some(EventType::Joke),
        "ignore" => Some(EventType::Ignore),
        "争吵" | "吵架" => Some(EventType::Quarrel),
        "道歉" | "抱歉" => Some(EventType::Apology),
        "表扬" | "称赞" => Some(EventType::Praise),
        "抱怨" | "不满" => Some(EventType::Complaint),
        "表白" | "告白" => Some(EventType::Confession),
        "笑话" | "玩笑" => Some(EventType::Joke),
        "忽略" | "无视" => Some(EventType::Ignore),
        _ => None,
    }
}

fn parse_impact_factor_ai_value(v: &Value) -> Option<f64> {
    match v {
        Value::Number(n) => n.as_f64(),
        Value::String(s) => s.trim().parse::<f64>().ok(),
        _ => None,
    }
}

fn parse_confidence_ai_value(v: Option<&Value>) -> Option<f32> {
    let raw = match v {
        Some(Value::Number(n)) => n.as_f64(),
        Some(Value::String(s)) => s.trim().parse::<f64>().ok(),
        _ => None,
    }?;
    Some(raw.clamp(0.0, 1.0) as f32)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ConflictTarget {
    Person,
    Situation,
    Self_,
    Mixed,
    Unknown,
}

fn parse_conflict_target_ai_token(raw: &str) -> Option<ConflictTarget> {
    let t = raw.trim();
    if t.is_empty() {
        return None;
    }
    let lower = t.to_ascii_lowercase();
    match lower.as_str() {
        "person" | "人" | "对人" => Some(ConflictTarget::Person),
        "situation" | "事" | "对事" => Some(ConflictTarget::Situation),
        "self" | "自我" | "对己" | "对自己" => Some(ConflictTarget::Self_),
        "mixed" | "混合" => Some(ConflictTarget::Mixed),
        "unknown" | "不确定" | "未知" => Some(ConflictTarget::Unknown),
        _ => None,
    }
}

fn parse_event_impact_ai_output(
    raw: &str,
) -> Option<(EventType, f64, Option<f32>, Option<ConflictTarget>)> {
    let direct = serde_json::from_str::<Value>(raw.trim());
    let val = direct
        .ok()
        .or_else(|| extract_json_object(raw).and_then(|s| serde_json::from_str::<Value>(s).ok()))?;

    let event_type_raw = val.get("event_type")?.as_str()?;
    let impact_raw = val.get("impact_factor")?;
    let event_type = parse_event_type_ai_token(event_type_raw)?;
    let impact_factor = parse_impact_factor_ai_value(impact_raw)?;
    let confidence = parse_confidence_ai_value(val.get("confidence"));
    let conflict_target = val
        .get("conflict_target")
        .and_then(|v| v.as_str())
        .and_then(parse_conflict_target_ai_token);
    Some((event_type, impact_factor, confidence, conflict_target))
}

fn apply_conservative_conflict_policy(
    event_type: EventType,
    impact_factor: f64,
    conflict_target: Option<ConflictTarget>,
) -> (EventType, f64, bool) {
    if event_type != EventType::Quarrel {
        return (event_type, impact_factor, false);
    }

    match conflict_target {
        Some(ConflictTarget::Person) => (event_type, impact_factor, false),
        Some(ConflictTarget::Situation)
        | Some(ConflictTarget::Self_)
        | Some(ConflictTarget::Mixed)
        | Some(ConflictTarget::Unknown) => (EventType::Complaint, impact_factor.max(-0.45), true),
        None => (event_type, impact_factor, false),
    }
}

fn apply_apology_persona_policy(
    event_type: &EventType,
    impact_factor: f64,
    personality: &PersonalityVector,
    recent_events: &[Event],
) -> f64 {
    if *event_type != EventType::Apology {
        return impact_factor;
    }

    // 主轴：soft_vs_cold，与立绘三轴同源，避免事件与表情策略漂移。
    let (softness, coldness, _) = softness_coldness_volatility(personality);
    let soft_vs_cold = (softness - coldness).clamp(-1.0, 1.0);
    let sensitivity = personality.sensitivity.clamp(0.0, 1.0);
    let recent_quarrel = recent_events
        .iter()
        .take(4)
        .any(|e| e.event_type == EventType::Quarrel);

    let mut adjusted = impact_factor.clamp(-1.0, 1.0);

    // 软人格：更愿意给台阶；低敏感时更容易明显缓和。
    if soft_vs_cold >= 0.12 {
        let floor = if sensitivity < 0.4 { 0.28 } else { 0.16 };
        let ceil = if sensitivity < 0.4 { 0.8 } else { 0.62 };
        adjusted = adjusted.max(floor).min(ceil);
    // 冷人格：道歉后也可能先观察，不立即大幅转暖。
    } else if soft_vs_cold <= -0.12 {
        let mut conservative_ceil: f64 = if sensitivity >= 0.75 {
            0.14
        } else if sensitivity >= 0.55 {
            0.2
        } else {
            0.3
        };
        if recent_quarrel {
            conservative_ceil = conservative_ceil.min(0.22);
        }
        adjusted = adjusted.min(conservative_ceil);
    }

    // 调制：高敏感统一减速“道歉后立刻缓和”。
    if adjusted > 0.0 && sensitivity > 0.6 {
        let slow_down = (1.0 - 0.3 * ((sensitivity - 0.6) / 0.4)).clamp(0.7, 1.0);
        adjusted *= slow_down;
    }

    adjusted.clamp(-1.0, 1.0)
}

fn has_unresolved_quarrel(recent_events: &[Event]) -> bool {
    let mut seen_quarrel = false;
    for e in recent_events.iter().take(8) {
        if e.event_type == EventType::Apology {
            return false;
        }
        if e.event_type == EventType::Quarrel {
            seen_quarrel = true;
        }
    }
    seen_quarrel
}

fn user_message_has_apology_signal(recent_turns: &[(String, String)]) -> bool {
    recent_turns.iter().rev().take(2).any(|(u, _)| {
        let lower = u.to_ascii_lowercase();
        lower.contains("sorry")
            || lower.contains("apolog")
            || u.contains("对不起")
            || u.contains("抱歉")
            || u.contains("道歉")
    })
}

fn apply_recent_context_continuity(
    event_type: EventType,
    impact_factor: f64,
    recent_events: &[Event],
    recent_turns: &[(String, String)],
) -> (EventType, f64) {
    let mut adjusted_type = event_type;
    let mut adjusted_impact = impact_factor.clamp(-1.0, 1.0);
    let unresolved_quarrel = has_unresolved_quarrel(recent_events);
    if unresolved_quarrel {
        match adjusted_type {
            EventType::Apology => {
                adjusted_impact = adjusted_impact.clamp(0.14, 0.55);
            }
            EventType::Praise | EventType::Confession | EventType::Joke => {
                adjusted_impact = adjusted_impact.min(0.18);
            }
            _ => {}
        }
    }
    if adjusted_type != EventType::Apology
        && user_message_has_apology_signal(recent_turns)
        && adjusted_impact.abs() <= 0.25
    {
        adjusted_type = EventType::Apology;
        adjusted_impact = adjusted_impact.max(0.1);
    }
    (adjusted_type, adjusted_impact.clamp(-1.0, 1.0))
}

pub fn soften_impact_factor(ai_impact_factor: f64, personality: &PersonalityVector) -> f64 {
    let clamped = ai_impact_factor.clamp(-1.0, 1.0);
    let stability = PersonalityEngine::calculate_stability_index(personality);
    let soft_index = (personality.warmth + personality.forgiveness + personality.clinginess) / 3.0;
    let cold_index =
        (personality.stubbornness + personality.assertiveness + (1.0 - personality.warmth)) / 3.0;
    let volatility = (personality.sensitivity + personality.talkativeness) / 2.0;

    let directional = if clamped >= 0.0 {
        (1.0 + (soft_index - cold_index) * 0.16).clamp(0.82, 1.08)
    } else {
        (1.0 + (cold_index - soft_index) * 0.16).clamp(0.82, 1.08)
    };
    let volatility_scale = (0.9 + volatility * 0.2).clamp(0.9, 1.1);
    let stability_scale = (0.85 + stability * 0.15).clamp(0.85, 1.0);

    (clamped * directional * volatility_scale * stability_scale).clamp(-1.0, 1.0)
}

fn derive_confidence(
    event_type: &EventType,
    impact_factor: f64,
    ai_confidence: Option<f32>,
) -> f32 {
    if let Some(v) = ai_confidence {
        return v.clamp(0.0, 1.0);
    }
    let rule_base = EventDetector::get_confidence(event_type);
    let impact_hint = (0.55 + impact_factor.abs() * 0.35).clamp(0.0, 1.0) as f32;
    ((rule_base + impact_hint) / 2.0).clamp(0.0, 1.0)
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct EventImpactEstimate {
    pub event_type: EventType,
    pub impact_factor: f64,
    pub confidence: f32,
}

fn build_event_impact_prompt(
    user_message: &str,
    user_emotion: &Emotion,
    personality: &PersonalityVector,
    recent_turns: &[(String, String)],
    recent_events: &[Event],
) -> String {
    let personality_json = personality.to_json_vec();
    let turns = if recent_turns.is_empty() {
        "无".to_string()
    } else {
        recent_turns
            .iter()
            .enumerate()
            .map(|(i, (u, b))| {
                let u_short = u.trim().chars().take(80).collect::<String>();
                let b_short = b.trim().chars().take(80).collect::<String>();
                format!("{}. 用户:{} | 角色:{}", i + 1, u_short, b_short)
            })
            .collect::<Vec<_>>()
            .join("\n")
    };
    let chain = if recent_events.is_empty() {
        "无".to_string()
    } else {
        recent_events
            .iter()
            .take(8)
            .map(|e| format!("{:?}", e.event_type))
            .collect::<Vec<_>>()
            .join(" -> ")
    };
    format!(
        r#"你是「事件影响估计器」。你的任务：根据“用户原话 + 用户情绪 + 角色当前七维性格”，估计本轮事件类型及其对关系的影响强度。

要求输出：只输出严格 JSON（不要解释、不要代码块、不要额外文本）。
JSON 必须是：
{{"event_type":"Quarrel|Apology|Praise|Complaint|Confession|Joke|Ignore","impact_factor":-1.0~1.0,"confidence":0.0~1.0,"conflict_target":"person|situation|self|mixed|unknown(可选)"}}

输入：
- user_message: {user_message}
- user_emotion: {user_emotion}
- bot_emotion (before reply, placeholder): neutral
- personality_vector (7 dims, each 0~1): {personality_json}
- recent_dialogue_turns (old->new, 仅参考): 
{turns}
- recent_event_chain (new->old, 仅参考): {chain}

语义约束：
1) event_type 必须是上述 7 类之一（区分大小写如示例）。
2) impact_factor：范围 [-1, 1]；越接近 +1 越亲近/缓和，越接近 -1 越冲突/对抗；中间值按强弱估计。
3) confidence：你对本次判断的把握度，范围 [0, 1]。
4) 连续性要求：若 recent_event_chain 里出现 Quarrel 且尚未出现 Apology，本轮不要轻率给出强正向 impact_factor。
5) conflict_target 仅在负向冲突语句中填写；无法判断时可省略或填 unknown。
6) 不要输出其他字段。
"#
    )
}

#[allow(clippy::too_many_arguments)]
pub async fn estimate_event_impact(
    llm: &Arc<dyn LlmClient>,
    ollama_model: &str,
    user_message: &str,
    user_emotion: &Emotion,
    personality: &PersonalityVector,
    recent_turns: &[(String, String)],
    recent_events: &[Event],
    knowledge_augment: Option<&KnowledgeEventAugment>,
) -> Result<EventImpactEstimate> {
    let bot_emotion_placeholder = Emotion::Neutral;
    let fallback_event = EventDetector::detect_with_augment(
        user_message,
        user_emotion,
        &bot_emotion_placeholder,
        knowledge_augment.filter(|a| !a.is_empty()),
    )?;
    let fallback_event_type = fallback_event.event_type;
    let fallback_impact = EventDetector::get_impact_factor(&fallback_event_type);
    let fallback_confidence = EventDetector::get_confidence(&fallback_event_type);

    if !event_impact_ai_enabled() {
        return Ok(EventImpactEstimate {
            event_type: fallback_event_type,
            impact_factor: fallback_impact,
            confidence: fallback_confidence,
        });
    }

    let prompt = build_event_impact_prompt(
        user_message,
        user_emotion,
        personality,
        recent_turns,
        recent_events,
    );
    match llm.generate_tag(ollama_model, &prompt).await {
        Ok(raw) => {
            if let Some((event_type, impact_factor, ai_confidence, conflict_target)) =
                parse_event_impact_ai_output(&raw)
            {
                let (ctx_type, ctx_impact) = apply_recent_context_continuity(
                    event_type,
                    impact_factor,
                    recent_events,
                    recent_turns,
                );
                let (final_type, final_impact, downgraded) =
                    apply_conservative_conflict_policy(ctx_type, ctx_impact, conflict_target);
                let apology_adjusted = apply_apology_persona_policy(
                    &final_type,
                    final_impact,
                    personality,
                    recent_events,
                );
                let mut softened = soften_impact_factor(apology_adjusted, personality);
                if downgraded {
                    softened = softened.max(-0.45);
                }
                let confidence = derive_confidence(&final_type, softened, ai_confidence);
                Ok(EventImpactEstimate {
                    event_type: final_type,
                    impact_factor: softened,
                    confidence,
                })
            } else {
                log::warn!(
                    "event_impact LLM output parse/constraint failed, fallback to rules: raw={}",
                    raw.chars().take(300).collect::<String>()
                );
                Ok(EventImpactEstimate {
                    event_type: fallback_event_type,
                    impact_factor: fallback_impact,
                    confidence: fallback_confidence,
                })
            }
        }
        Err(e) => {
            log::warn!("event_impact LLM failed, fallback to rules: {}", e);
            Ok(EventImpactEstimate {
                event_type: fallback_event_type,
                impact_factor: fallback_impact,
                confidence: fallback_confidence,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::llm::MockLlmClient;

    fn p_with_sensitivity(sensitivity: f64) -> PersonalityVector {
        PersonalityVector {
            stubbornness: 0.4,
            clinginess: 0.4,
            sensitivity,
            assertiveness: 0.4,
            forgiveness: 0.4,
            talkativeness: 0.4,
            warmth: 0.4,
        }
    }

    fn soft_low_sensitive_persona() -> PersonalityVector {
        PersonalityVector {
            stubbornness: 0.22,
            clinginess: 0.6,
            sensitivity: 0.22,
            assertiveness: 0.25,
            forgiveness: 0.85,
            talkativeness: 0.5,
            warmth: 0.82,
        }
    }

    fn cold_high_sensitive_persona() -> PersonalityVector {
        PersonalityVector {
            stubbornness: 0.86,
            clinginess: 0.2,
            sensitivity: 0.88,
            assertiveness: 0.82,
            forgiveness: 0.18,
            talkativeness: 0.4,
            warmth: 0.25,
        }
    }

    #[test]
    fn parse_event_impact_output_and_soften() {
        let p = p_with_sensitivity(0.6);
        let (event_type, raw_impact, confidence, conflict_target) =
            parse_event_impact_ai_output(r#"{"event_type":"Praise","impact_factor":1.5}"#).unwrap();
        let impact = soften_impact_factor(raw_impact, &p);
        assert_eq!(event_type, EventType::Praise);
        assert!(impact <= 1.0);
        assert!(impact > 0.7);
        assert!(confidence.is_none());
        assert!(conflict_target.is_none());
    }

    #[tokio::test]
    async fn estimate_event_impact_falls_back_when_parse_fails() {
        let llm: Arc<dyn LlmClient> = Arc::new(MockLlmClient {
            reply: "not a json".to_string(),
        });
        let p = p_with_sensitivity(0.2);
        let estimate = estimate_event_impact(
            &llm,
            "mock-model",
            "我很难受",
            &Emotion::Sad,
            &p,
            &[],
            &[],
            None,
        )
        .await
        .unwrap();
        assert_eq!(estimate.event_type, EventType::Complaint);
        assert_eq!(estimate.impact_factor, -0.5);
        assert!(estimate.confidence > 0.7);
    }

    #[test]
    fn soften_impact_factor_clamps_and_scales() {
        let p = p_with_sensitivity(0.1);
        let softened = soften_impact_factor(-2.0, &p);
        assert!(softened <= 0.0);
        assert!(softened >= -1.0);
        assert!(softened.abs() < 0.98);
    }

    #[test]
    fn quarrel_with_person_target_keeps_quarrel() {
        let (event_type, impact_factor, _) = apply_conservative_conflict_policy(
            EventType::Quarrel,
            -0.8,
            Some(ConflictTarget::Person),
        );
        assert_eq!(event_type, EventType::Quarrel);
        assert_eq!(impact_factor, -0.8);
    }

    #[test]
    fn quarrel_with_situation_or_self_downgrades_to_complaint() {
        let (event_type_a, impact_a, _) = apply_conservative_conflict_policy(
            EventType::Quarrel,
            -0.8,
            Some(ConflictTarget::Situation),
        );
        assert_eq!(event_type_a, EventType::Complaint);
        assert!(impact_a >= -0.45);

        let (event_type_b, impact_b, _) = apply_conservative_conflict_policy(
            EventType::Quarrel,
            -0.9,
            Some(ConflictTarget::Self_),
        );
        assert_eq!(event_type_b, EventType::Complaint);
        assert!(impact_b >= -0.45);
    }

    #[test]
    fn quarrel_without_conflict_target_keeps_backward_compatibility() {
        let (event_type, impact_factor, confidence, conflict_target) =
            parse_event_impact_ai_output(r#"{"event_type":"Quarrel","impact_factor":-0.7}"#)
                .unwrap();
        assert_eq!(event_type, EventType::Quarrel);
        assert_eq!(impact_factor, -0.7);
        assert!(confidence.is_none());
        assert!(conflict_target.is_none());

        let (final_type, final_impact, _) =
            apply_conservative_conflict_policy(event_type, impact_factor, conflict_target);
        assert_eq!(final_type, EventType::Quarrel);
        assert_eq!(final_impact, -0.7);
    }

    #[test]
    fn apology_soft_persona_low_sensitivity_eases_more() {
        let p = soft_low_sensitive_persona();
        let recent_events = vec![Event {
            event_type: EventType::Quarrel,
            user_emotion: "angry".to_string(),
            bot_emotion: "angry".to_string(),
        }];
        let adjusted = apply_apology_persona_policy(&EventType::Apology, 0.1, &p, &recent_events);
        let softened = soften_impact_factor(adjusted, &p);
        assert!(adjusted >= 0.28);
        assert!(softened > 0.2);
    }

    #[test]
    fn apology_cold_persona_high_sensitivity_stays_conservative() {
        let p = cold_high_sensitive_persona();
        let recent_events = vec![Event {
            event_type: EventType::Quarrel,
            user_emotion: "angry".to_string(),
            bot_emotion: "angry".to_string(),
        }];
        let adjusted = apply_apology_persona_policy(&EventType::Apology, 0.75, &p, &recent_events);
        let softened = soften_impact_factor(adjusted, &p);
        assert!(adjusted <= 0.14);
        assert!(softened <= 0.2);
    }

    #[test]
    fn unresolved_quarrel_caps_positive_non_apology_impact() {
        let recent_events = vec![Event {
            event_type: EventType::Quarrel,
            user_emotion: "angry".to_string(),
            bot_emotion: "angry".to_string(),
        }];
        let (t, impact) = apply_recent_context_continuity(
            EventType::Praise,
            0.78,
            &recent_events,
            &[("我知道了".to_string(), "…".to_string())],
        );
        assert_eq!(t, EventType::Praise);
        assert!(impact <= 0.18);
    }

    #[test]
    fn recent_user_apology_signal_can_switch_to_apology() {
        let recent_turns = vec![
            ("上一句".to_string(), "上一轮回复".to_string()),
            ("对不起，我刚刚态度不好".to_string(), "好的".to_string()),
        ];
        let (t, impact) =
            apply_recent_context_continuity(EventType::Complaint, -0.12, &[], &recent_turns);
        assert_eq!(t, EventType::Apology);
        assert!(impact >= 0.1);
    }
}
