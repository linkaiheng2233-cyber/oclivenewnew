use crate::domain::relation_engine::RelationState;
use crate::models::{Event, EventType};

#[must_use]
pub fn confidence_decay_weight(confidence: f32) -> f64 {
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

#[must_use]
pub fn avoid_fast_promote_score(
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

#[must_use]
pub fn smooth_favor_delta_for_short_streak(raw_delta: f64, recent_events: &[Event]) -> f64 {
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

#[must_use]
pub fn soft_append_guard(
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

/// Strips meaningless English fragments occasionally emitted by the model (e.g. `uppyuppy`) to avoid polluting dialogue.
#[must_use]
pub fn strip_hallucination_tokens(reply: &str) -> String {
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

const CARE_PACKAGE_KEYWORDS: &[&str] = &[
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

fn split_sentences(text: &str) -> Vec<String> {
    let mut sentences = Vec::new();
    let mut current = String::new();
    for ch in text.chars() {
        current.push(ch);
        if matches!(ch, '。' | '！' | '？' | '!' | '?') {
            let trimmed = current.trim().to_string();
            if !trimmed.is_empty() {
                sentences.push(trimmed);
            }
            current.clear();
        }
    }
    let tail = current.trim();
    if !tail.is_empty() {
        sentences.push(tail.to_string());
    }
    sentences
}

fn sentence_repeats_care_template(sentence: &str, previous: &str) -> bool {
    count_matches(sentence, CARE_PACKAGE_KEYWORDS) >= 2
        && CARE_PACKAGE_KEYWORDS
            .iter()
            .filter(|w| previous.contains(**w) && sentence.contains(**w))
            .count()
            >= 2
}

/// Trims care-package template sentences when the model repeats the previous turn's concern list.
#[must_use]
pub fn trim_template_repeat_reply(previous: &str, reply: &str) -> String {
    let previous = previous.trim();
    let reply = reply.trim();
    if previous.is_empty() || reply.is_empty() {
        return reply.to_string();
    }
    if count_matches(previous, CARE_PACKAGE_KEYWORDS) < 2
        || count_matches(reply, CARE_PACKAGE_KEYWORDS) < 2
    {
        return reply.to_string();
    }
    let shared = CARE_PACKAGE_KEYWORDS
        .iter()
        .filter(|w| previous.contains(**w) && reply.contains(**w))
        .count();
    if shared < 2 {
        return reply.to_string();
    }

    let sentences = split_sentences(reply);
    if sentences.is_empty() {
        return reply.to_string();
    }
    if sentences.len() == 1 {
        if sentence_repeats_care_template(&sentences[0], previous) {
            return String::new();
        }
        return sentences[0].clone();
    }

    let mut kept = Vec::new();
    for (idx, sentence) in sentences.iter().enumerate() {
        if idx == 0 || !sentence_repeats_care_template(sentence, previous) {
            kept.push(sentence.as_str());
        }
    }
    let trimmed = kept.join("");
    if trimmed.trim().is_empty() {
        sentences
            .into_iter()
            .take(2)
            .filter(|s| count_matches(s, CARE_PACKAGE_KEYWORDS) < 2)
            .collect::<Vec<_>>()
            .join("")
    } else {
        trimmed
    }
}

#[cfg(test)]
mod hallucination_tests {
    use super::{strip_hallucination_tokens, trim_template_repeat_reply};

    #[test]
    fn strip_removes_uppyuppy_variants() {
        let s = strip_hallucination_tokens("早安 uppyuppy 想吃蛋糕 Uppyuppy 吗");
        assert!(!s.to_lowercase().contains("uppyuppy"));
        assert!(s.contains("早安"));
        assert!(s.contains("蛋糕"));
    }

    #[test]
    fn trim_removes_care_package_repeat_sentences() {
        let prev = "记得出门晒晒太阳呀，作业写完没？早点睡别熬夜，多喝热水暖暖手。";
        let reply = "在呢。记得出门晒晒太阳，作业写完没？早点睡，多喝热水哦。";
        let out = trim_template_repeat_reply(prev, reply);
        assert!(out.contains("在呢"));
        assert!(!out.contains("作业写完没"));
        assert!(!out.contains("多喝热水"));
    }

    #[test]
    fn trim_skips_when_no_template_overlap() {
        let prev = "今天天气不错。";
        let reply = "嗯，我也觉得。";
        assert_eq!(trim_template_repeat_reply(prev, reply), "嗯，我也觉得。");
    }
}
