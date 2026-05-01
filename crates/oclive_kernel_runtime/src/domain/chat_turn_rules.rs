use crate::domain::relation_engine::RelationState;
use crate::models::{Event, EventType};

pub(crate) fn confidence_decay_weight(confidence: f32) -> f64 {
    let c = (confidence as f64).clamp(0.0, 1.0);
    let threshold = 0.60_f64;
    if c >= threshold {
        1.0
    } else {
        (0.25 + 0.75 * (c / threshold)).clamp(0.25, 1.0)
    }
}

fn contains_any(text: &str, needles: &[&str]) -> bool {
    needles.iter().any(|w| text.contains(w))
}

fn count_matches(text: &str, needles: &[&str]) -> usize {
    needles.iter().filter(|w| text.contains(**w)).count()
}

fn is_low_relation_stage(relation_preview: &str) -> bool {
    matches!(
        RelationState::parse(relation_preview),
        RelationState::Stranger | RelationState::Acquaintance
    )
}

pub(crate) fn avoid_fast_promote_score(
    current_event: &EventType,
    current_impact_factor: f64,
    recent_events: &[Event],
) -> f64 {
    let is_current_strong_positive =
        matches!(current_event, EventType::Praise | EventType::Confession)
            && current_impact_factor >= 0.55;
    if !is_current_strong_positive {
        return 0.0;
    }

    const WINDOW: usize = 4;
    let mut prev_positive_streak = 0usize;
    for event in recent_events.iter().take(WINDOW) {
        if matches!(event.event_type, EventType::Praise | EventType::Confession) {
            prev_positive_streak += 1;
        } else {
            break;
        }
    }
    let streak = prev_positive_streak + 1;
    match streak {
        0..=1 => 0.0,
        2 => 0.35,
        3 => 0.7,
        _ => 1.0,
    }
}

fn event_direction(event_type: &EventType) -> i8 {
    match event_type {
        EventType::Praise | EventType::Confession => 1,
        EventType::Quarrel | EventType::Complaint | EventType::Ignore => -1,
        EventType::Apology | EventType::Joke => 0,
    }
}

pub(crate) fn smooth_favor_delta_for_short_streak(raw_delta: f64, recent_events: &[Event]) -> f64 {
    const WINDOW: usize = 4;
    const MIN_ACTIVE_DELTA: f64 = 0.03;
    if raw_delta.abs() < MIN_ACTIVE_DELTA {
        return raw_delta;
    }

    let current_dir = if raw_delta > 0.0 { 1 } else { -1 };
    let mut streak = 1usize;
    for event in recent_events.iter().take(WINDOW) {
        let dir = event_direction(&event.event_type);
        if dir == 0 {
            break;
        }
        if dir == current_dir {
            streak += 1;
        } else {
            break;
        }
    }

    let scale = match streak {
        0..=1 => 1.0,
        2 => 0.94,
        3 => 0.88,
        _ => 0.82,
    };
    raw_delta * scale
}

pub(crate) fn soft_append_guard(
    reply: &str,
    event_type: &EventType,
    impact_factor: f64,
    relation_preview: &str,
) -> String {
    let soft_lines = [
        "不过我们先把语气放慢一点，慢慢聊清楚就好。",
        "先别急着把话说满，我们一步一步把感觉对齐。",
        "这会儿先稳一点，等彼此都舒服了再往前走。",
    ];
    if soft_lines.iter().any(|line| reply.contains(line)) {
        return reply.to_string();
    }

    let lower = reply.to_lowercase();
    let sweet_words = [
        "宝贝",
        "亲爱的",
        "想你",
        "抱抱",
        "么么哒",
        "老婆",
        "老公",
        "honey",
        "baby",
        "kiss",
    ];
    let strong_intimacy_words = [
        "永远在一起",
        "一辈子",
        "结婚",
        "不离不弃",
        "只属于你",
        "做你男朋友",
        "做你女朋友",
        "爱你一生",
    ];

    let sweet_hits = count_matches(reply, &sweet_words) + count_matches(&lower, &sweet_words);
    let has_strong_intimacy = contains_any(reply, &strong_intimacy_words);
    let conflict_negative = (matches!(event_type, EventType::Quarrel) || impact_factor < 0.0)
        && (sweet_hits >= 2 || has_strong_intimacy);
    let conflict_low_stage = is_low_relation_stage(relation_preview) && has_strong_intimacy;
    if !(conflict_negative || conflict_low_stage) {
        return reply.to_string();
    }

    let mut out = reply.trim_end().to_string();
    if !out.ends_with('。')
        && !out.ends_with('！')
        && !out.ends_with('？')
        && !out.ends_with('.')
        && !out.ends_with('!')
        && !out.ends_with('?')
    {
        out.push('。');
    }
    out.push_str(soft_lines[out.len() % soft_lines.len()]);
    out
}

/// 去掉模型偶发输出的无意义英文碎片（如 `uppyuppy`），避免污染对话。
pub(crate) fn strip_hallucination_tokens(reply: &str) -> String {
    const JUNK: &str = "uppyuppy";
    let junk_len = JUNK.chars().count();
    let chars: Vec<char> = reply.chars().collect();
    let mut s = String::with_capacity(reply.len());
    let mut i = 0;
    while i < chars.len() {
        if i + junk_len <= chars.len() {
            let chunk: String = chars[i..i + junk_len].iter().collect();
            if chunk.eq_ignore_ascii_case(JUNK) {
                i += junk_len;
                continue;
            }
        }
        s.push(chars[i]);
        i += 1;
    }
    let lines: Vec<String> = s
        .lines()
        .map(|line| {
            let mut t = line.to_string();
            while t.contains("  ") {
                t = t.replace("  ", " ");
            }
            t.trim_end().to_string()
        })
        .collect();
    lines.join("\n").trim().to_string()
}

#[cfg(test)]
mod hallucination_tests {
    use super::strip_hallucination_tokens;

    #[test]
    fn strip_removes_uppyuppy_variants() {
        let s = strip_hallucination_tokens("早安 uppyuppy 想吃蛋糕 Uppyuppy 吗");
        assert!(!s.to_lowercase().contains("uppyuppy"));
        assert!(s.contains("早安"));
        assert!(s.contains("蛋糕"));
    }
}
