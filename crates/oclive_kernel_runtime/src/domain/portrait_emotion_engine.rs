//! 立绘表情：由 LLM 结合对话、人设向量与近期事件判断，而非简单沿用回复文本情绪。
//!
//! 环境变量 `OCLIVE_PORTRAIT_EMOTION_LLM=0` 时关闭第二次 LLM，仅用启发式（与规则兜底）。

use crate::domain::affect_policy::{
    guarded_drive, hurt_drive, probing_drive, softness_coldness_volatility,
};
use crate::error::Result;
use crate::infrastructure::llm::LlmClient;
use crate::models::{Emotion, Event, EventType, PersonalityVector, Role};
use std::sync::Arc;

const ALLOWED: &[&str] = &[
    "happy", "sad", "angry", "neutral", "excited", "confused", "shy",
];

fn portrait_llm_enabled() -> bool {
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

fn parse_emotion_token(raw: &str) -> Option<String> {
    let t = raw
        .trim()
        .trim_matches(|c| c == '`' || c == '"' || c == '\'');
    let first = t.split_whitespace().next().unwrap_or("");
    let cleaned: String = first
        .chars()
        .take_while(|c| c.is_ascii_alphabetic())
        .collect();
    let lower = cleaned.to_ascii_lowercase();
    if ALLOWED.contains(&lower.as_str()) {
        Some(lower)
    } else {
        None
    }
}

fn heuristic_base(bot_emotion: &Emotion) -> String {
    bot_emotion.to_string()
}

/// 规则兜底（LLM 为主）：用七维综合标量辅助，避免「类型标签」式二分。
fn apply_persona_event_overrides(
    mut tag: String,
    user_emotion_str: &str,
    recent_events: &[Event],
    personality: &PersonalityVector,
) -> String {
    let u = user_emotion_str.to_lowercase();
    let user_sadish = u.contains("sad")
        || u.contains("难过")
        || u.contains("委屈")
        || u.contains("伤心")
        || u.contains("难受");

    let recent_quarrel = recent_events
        .iter()
        .take(4)
        .any(|e| e.event_type == EventType::Quarrel);
    let recent_apology = recent_events
        .iter()
        .take(3)
        .any(|e| e.event_type == EventType::Apology);

    // 轻量三轴：与 `affect_policy` 共用，用于纠偏。
    let (softness, coldness, volatility) = softness_coldness_volatility(personality);

    let axis_gap = (softness - coldness).clamp(-1.0, 1.0);
    // 冲突纠错 1：争吵未道歉 + 用户在难过区间，但 LLM 给出过于积极的 happy
    // 目标：避免“吵架后讨好式反应”，改用更符合角色的冷/困惑/克制态。
    if recent_quarrel && !recent_apology && tag == "happy" {
        if user_sadish {
            // 三轴下用户受伤场景：冷高+波动高更偏对抗，软高时保持克制过渡。
            tag = if axis_gap <= -0.12 && volatility > 0.58 {
                "angry".to_string()
            } else if axis_gap <= -0.05 {
                "confused".to_string()
            } else if axis_gap >= 0.2 && volatility < 0.55 {
                "neutral".to_string()
            } else {
                "confused".to_string()
            };
        } else {
            // 三轴差异优先：在同类输入下保留人设分化感，而不是统一 neutral。
            tag = if coldness > 0.64 && volatility > 0.55 {
                "angry".to_string()
            } else if softness > 0.66 && volatility < 0.46 {
                "shy".to_string()
            } else if axis_gap > 0.08 {
                "confused".to_string()
            } else {
                "neutral".to_string()
            };
        }
    }

    // 冲突纠错 2：道歉后 + LLM 输出过于平淡 neutral
    // 目标：避免“道歉后仍过冷/过平”，但允许冷性角色保留 neutral。
    if recent_apology && tag == "neutral" && softness >= coldness {
        // 高敏感时不强推“立刻开心”，保留更慢缓和。
        if personality.sensitivity >= 0.75 {
            return if user_sadish {
                "confused".to_string()
            } else {
                "neutral".to_string()
            };
        }
        tag = if user_sadish && volatility < 0.7 {
            "shy".to_string()
        } else if volatility >= 0.7 {
            "excited".to_string()
        } else {
            "happy".to_string()
        };
    }

    apply_expressive_mapping(
        tag,
        user_sadish,
        recent_quarrel,
        recent_apology,
        personality,
    )
}

/// 在不新增标签的前提下，增强中间态表达：
/// - 受伤态：偏 sad/shy
/// - 戒备态：偏 angry/confused
/// - 试探态：偏 shy/confused
fn apply_expressive_mapping(
    tag: String,
    user_sadish: bool,
    recent_quarrel: bool,
    recent_apology: bool,
    personality: &PersonalityVector,
) -> String {
    let g = guarded_drive(personality);
    let h = hurt_drive(personality);
    let p = probing_drive(personality);

    let wounded_state =
        (user_sadish || (recent_quarrel && !recent_apology)) && h >= 0.62 && g < 0.74;
    let guarded_state = recent_quarrel && !recent_apology && g >= 0.62;
    let probing_state =
        !guarded_state && !recent_quarrel && p >= 0.57 && (tag == "confused" || tag == "shy");

    // 保持强情绪输出不被过度稀释；只细化中间态和偏平输出。
    if tag == "excited" || tag == "happy" {
        return tag;
    }

    if wounded_state {
        return if h >= 0.76 || tag == "sad" {
            "sad".to_string()
        } else {
            "shy".to_string()
        };
    }

    if guarded_state {
        return if g >= 0.78 || tag == "angry" {
            "angry".to_string()
        } else {
            "confused".to_string()
        };
    }

    if probing_state {
        return if personality.clinginess >= 0.62 && personality.warmth >= 0.52 {
            "shy".to_string()
        } else {
            "confused".to_string()
        };
    }

    tag
}

fn favorability_hint(f: f64) -> &'static str {
    if f < 25.0 {
        "偏低：更易戒备、冷淡、生气或中性，不易立刻亲近。"
    } else if f < 55.0 {
        "中等：态度随事件与性格波动。"
    } else if f < 75.0 {
        "偏高：更容易心软、害羞或正面情绪。"
    } else {
        "很高：更易开心、害羞、柔和，冷战后也更易缓和。"
    }
}

#[allow(clippy::too_many_arguments)]
fn build_portrait_prompt(
    role: &Role,
    core_personality: &PersonalityVector,
    personality: &PersonalityVector,
    favorability: f64,
    user_message: &str,
    reply: &str,
    user_emotion_str: &str,
    bot_emotion: &Emotion,
    recent_events: &[Event],
    recent_turns: &[(String, String)],
) -> String {
    let ev_line = recent_events
        .iter()
        .take(6)
        .map(|e| format!("{:?}", e.event_type))
        .collect::<Vec<_>>()
        .join(", ");
    let ev_line = if ev_line.is_empty() {
        "无".to_string()
    } else {
        ev_line
    };

    let mut turns = String::new();
    if recent_turns.is_empty() {
        turns.push_str("（当前会话尚无更早轮次；以下为第一回合。）\n");
    } else {
        for (i, (u, b)) in recent_turns.iter().enumerate() {
            let u_short = truncate_chars(u, 120);
            let b_short = truncate_chars(b, 120);
            turns.push_str(&format!(
                "{}. 用户：{} … 角色：{}\n",
                i + 1,
                u_short,
                b_short
            ));
        }
    }

    let no_history_note = if recent_turns.is_empty() {
        "【尚无对话历史】立绘宜从平静/中性（neutral）为起点，除非本回合冲突或情绪极强。\n"
    } else {
        ""
    };

    format!(
        r#"你是「立绘表情导演」。只输出一个英文小写单词作为表情标签，不要解释、不要标点、不要换行多余内容。

允许且仅允许以下之一（必须完全一致）：
happy, sad, angry, neutral, excited, confused, shy

原则：
1) 表情要符合角色此刻对用户的真实态度与关系张力，不是复述用户情绪，也不是简单照抄回复语气。
2) **以七维性格向量为决策主依据**（尤其下面的「当前有效性格」）：不要用「依赖型/独立型」等简化标签代替推理；必须综合七个 0~1 数值及其**组合**判断立绘（例如高黏人+高敏感与高倔强+低宽容会呈现完全不同反应）。
3) 各维度对立绘的**倾向参考**（非公式、需与剧情/事件/好感综合）：倔强高→态度更难因对方一句话翻转；黏人高→更易与用户情绪共振、心软或害羞；敏感高→更易受伤、多想或情绪波动大；强势高→更易对抗、冷硬或压着不示弱；宽容高→更易给台阶、缓和；话多高→表情可更外放或「碎念」感；温暖高→更易柔和、亲近感。任意组合都可能成立，勿只套两种人格模板。
4) 「核心性格」与「当前有效性格」：后者 = 核心 + 演化增量；若某维相对核心变化大，代表角色已成长或偏移，立绘应体现这种差异（例如宽容上升则和解时更易软化）。
5) 若近期有争吵/冲突（Quarrel）且尚未出现道歉/和解（Apology），即使用户示弱或难过，也请用**七维综合**推断：例如低宽容、低温暖、高倔强、高强势时更可能继续冷、怒或端着，而非立刻开心安慰脸。
6) 好感度调节关系松紧：好感低更戒备冷淡；好感高更易柔软、害羞、开心，但仍须服从七维向量与事件逻辑。

角色名：{}
人设摘要：{}
核心性格（manifest 七维 0~1）倔强{:.2} 黏人{:.2} 敏感{:.2} 强势{:.2} 宽容{:.2} 话多{:.2} 温暖{:.2}
当前有效性格（核心+演化，AI 可成长）倔强{:.2} 黏人{:.2} 敏感{:.2} 强势{:.2} 宽容{:.2} 话多{:.2} 温暖{:.2}
当前好感度：{:.1}（{}）
近期事件（新→旧）：{}
最近几轮摘要：
{}{}
本回合：
- 用户说：{}
- 角色回复：{}
- 用户情绪标签：{}
- 从回复文本粗读的角色情绪（仅供参考）：{}

只输出一个词。"#,
        role.name,
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
        favorability_hint(favorability),
        ev_line,
        turns,
        no_history_note,
        user_message,
        reply,
        user_emotion_str,
        bot_emotion
    )
}

fn truncate_chars(s: &str, max: usize) -> String {
    let mut t = s.trim().replace('\n', " ");
    if t.chars().count() > max {
        t = t.chars().take(max).collect::<String>() + "…";
    }
    t
}

fn fallback_base(bot_emotion: &Emotion, recent_turns: &[(String, String)]) -> String {
    if recent_turns.is_empty() {
        "neutral".to_string()
    } else {
        heuristic_base(bot_emotion)
    }
}

/// 解析立绘情绪标签；失败时回退：无历史时偏 `neutral`，否则 `bot_emotion`。
#[allow(clippy::too_many_arguments)]
pub async fn resolve_portrait_emotion(
    llm: &Arc<dyn LlmClient>,
    ollama_model: &str,
    role: &Role,
    core_personality: &PersonalityVector,
    personality: &PersonalityVector,
    favorability: f64,
    user_message: &str,
    reply: &str,
    user_emotion_str: &str,
    bot_emotion: &Emotion,
    recent_events: &[Event],
    recent_turns: &[(String, String)],
) -> Result<String> {
    let mut tag = if portrait_llm_enabled() {
        let prompt = build_portrait_prompt(
            role,
            core_personality,
            personality,
            favorability,
            user_message,
            reply,
            user_emotion_str,
            bot_emotion,
            recent_events,
            recent_turns,
        );
        match llm.generate_tag(ollama_model, &prompt).await {
            Ok(raw) => parse_emotion_token(&raw)
                .unwrap_or_else(|| fallback_base(bot_emotion, recent_turns)),
            Err(e) => {
                log::warn!("portrait_emotion LLM failed, fallback: {}", e);
                fallback_base(bot_emotion, recent_turns)
            }
        }
    } else {
        fallback_base(bot_emotion, recent_turns)
    };

    tag = apply_persona_event_overrides(tag, user_emotion_str, recent_events, personality);
    Ok(tag)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_token_trims() {
        assert_eq!(parse_emotion_token("  Happy \n"), Some("happy".to_string()));
        assert_eq!(parse_emotion_token("neutral"), Some("neutral".to_string()));
        assert_eq!(parse_emotion_token("disgust"), None);
    }

    #[test]
    fn override_quarrel_user_sad_confused_for_cold_persona() {
        let recent = vec![Event {
            event_type: EventType::Quarrel,
            user_emotion: "angry".to_string(),
            bot_emotion: "angry".to_string(),
        }];
        let p = PersonalityVector {
            stubbornness: 0.75,
            clinginess: 0.35,
            sensitivity: 0.5,
            assertiveness: 0.7,
            forgiveness: 0.35,
            talkativeness: 0.5,
            warmth: 0.4,
        };
        let out = apply_persona_event_overrides("happy".to_string(), "sad", &recent, &p);
        assert_eq!(out, "confused");
    }

    #[test]
    fn expressive_mapping_prefers_sad_shy_for_hurt_state() {
        let recent = vec![Event {
            event_type: EventType::Quarrel,
            user_emotion: "sad".to_string(),
            bot_emotion: "neutral".to_string(),
        }];
        let p = PersonalityVector {
            stubbornness: 0.35,
            clinginess: 0.73,
            sensitivity: 0.86,
            assertiveness: 0.34,
            forgiveness: 0.58,
            talkativeness: 0.45,
            warmth: 0.72,
        };
        let out = apply_persona_event_overrides("neutral".to_string(), "委屈", &recent, &p);
        assert_eq!(out, "sad");
    }

    #[test]
    fn expressive_mapping_prefers_angry_confused_for_guarded_state() {
        let recent = vec![Event {
            event_type: EventType::Quarrel,
            user_emotion: "sad".to_string(),
            bot_emotion: "neutral".to_string(),
        }];
        let p = PersonalityVector {
            stubbornness: 0.87,
            clinginess: 0.28,
            sensitivity: 0.44,
            assertiveness: 0.82,
            forgiveness: 0.20,
            talkativeness: 0.33,
            warmth: 0.25,
        };
        let out = apply_persona_event_overrides("neutral".to_string(), "ok", &recent, &p);
        assert_eq!(out, "angry");
    }

    #[test]
    fn expressive_mapping_uses_shy_confused_for_probing_state() {
        let recent = vec![Event {
            event_type: EventType::Joke,
            user_emotion: "neutral".to_string(),
            bot_emotion: "neutral".to_string(),
        }];
        let p = PersonalityVector {
            stubbornness: 0.32,
            clinginess: 0.68,
            sensitivity: 0.64,
            assertiveness: 0.36,
            forgiveness: 0.58,
            talkativeness: 0.70,
            warmth: 0.66,
        };
        let out = apply_persona_event_overrides("confused".to_string(), "normal", &recent, &p);
        assert_eq!(out, "shy");
    }
}
