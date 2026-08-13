//! `[EMO]` structured marker parsing for the main-LLM reply (B M1 slice 1).
//!
//! The main LLM is the single arbiter of complex emotion: its reply may end
//! with one `[EMO]{...}[/EMO]` block. The host parses the **last** block,
//! validates it against the seven-dimension label enum, strips it from the
//! display reply, and derives the remaining `ComplexEmotionOutput` fields
//! deterministically (no second LLM call).

use crate::models::Emotion;
use oclive_kernel_runtime::domain::complex_emotion::ComplexEmotionOutput;
use serde::Deserialize;

/// Opening marker. Keep in sync with the prompt instruction in
/// `oclive_kernel_runtime::domain::prompt_builder`.
pub const EMO_MARKER_OPEN: &str = "[EMO]";
/// Closing marker.
pub const EMO_MARKER_CLOSE: &str = "[/EMO]";
/// `ComplexEmotionOutput.source` for marker-derived output.
pub const EMO_MARKER_SOURCE: &str = "llm_emo_marker";
/// `ComplexEmotionOutput.source` for the degraded keep branch (no marker, non-plugin backend).
pub const DEGRADED_KEEP_SOURCE: &str = "degraded_keep";
/// Hard upper bound for model/plugin-generated narrative hints.
pub const MAX_NARRATIVE_HINT_CHARS: usize = 200;

/// Applies the contract limit without splitting UTF-8 code points.
#[must_use]
pub(crate) fn truncate_narrative_hint(raw: &str) -> String {
    raw.chars().take(MAX_NARRATIVE_HINT_CHARS).collect()
}

/// Seven-dimension emotion labels accepted by the `[EMO]` contract (v1.5 §11.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmoLabel {
    Joy,
    Sadness,
    Anger,
    Fear,
    Surprise,
    Disgust,
    Neutral,
}

impl EmoLabel {
    /// Parses a lowercase English label; unknown labels return `None`.
    #[must_use]
    pub fn parse(raw: &str) -> Option<Self> {
        match raw.trim() {
            "joy" => Some(Self::Joy),
            "sadness" => Some(Self::Sadness),
            "anger" => Some(Self::Anger),
            "fear" => Some(Self::Fear),
            "surprise" => Some(Self::Surprise),
            "disgust" => Some(Self::Disgust),
            "neutral" => Some(Self::Neutral),
            _ => None,
        }
    }

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Joy => "joy",
            Self::Sadness => "sadness",
            Self::Anger => "anger",
            Self::Fear => "fear",
            Self::Surprise => "surprise",
            Self::Disgust => "disgust",
            Self::Neutral => "neutral",
        }
    }

    /// Maps a seven-dimension label to the persisted six-slot emotion graph.
    #[must_use]
    const fn as_emotion(self) -> Emotion {
        match self {
            Self::Joy => Emotion::Happy,
            Self::Sadness => Emotion::Sad,
            Self::Anger => Emotion::Angry,
            Self::Surprise => Emotion::Excited,
            Self::Fear | Self::Disgust => Emotion::Confused,
            Self::Neutral => Emotion::Neutral,
        }
    }
}

/// Maps the first recognized complex-emotion label to the six-slot graph.
///
/// Marker and plugin outputs share this consumer so archived labels, events,
/// current emotion, and portrait selection cannot diverge by producer.
#[must_use]
pub(crate) fn dominant_emotion_from_labels(labels: &[String]) -> Option<Emotion> {
    labels
        .first()
        .and_then(|label| EmoLabel::parse(label))
        .map(EmoLabel::as_emotion)
}

/// Parsed marker payload after validation.
#[derive(Debug, Clone)]
pub struct EmoMarker {
    /// 1..=3 validated labels in LLM priority order; `labels[0]` is dominant.
    pub labels: Vec<EmoLabel>,
    /// Clamped to `0.0..=1.0` (defaults to 0.5 when the field is absent).
    pub intensity: f64,
    /// `None` when the field is absent; `Some` when provided (may be empty).
    pub narrative_hint: Option<String>,
}

impl EmoMarker {
    /// Maps `labels[0]` to the six-slot emotion graph.
    ///
    /// Mirrors `EmotionResult::dominant_emotion()` (fear/disgust -> Confused,
    /// surprise -> Excited); Shy intentionally does not participate.
    #[must_use]
    pub fn dominant_emotion(&self) -> Emotion {
        self.labels[0].as_emotion()
    }

    /// Deterministic pattern for common two-label combos (order-insensitive);
    /// other combos return `None`.
    #[must_use]
    pub fn pattern(&self) -> Option<String> {
        use EmoLabel::{Anger, Disgust, Fear, Joy, Sadness, Surprise};
        if self.labels.len() < 2 {
            return None;
        }
        let mut pair = [self.labels[0], self.labels[1]];
        pair.sort_by_key(|l| l.as_str());
        match pair {
            [Anger, Sadness] => Some("resentful_sad"),
            [Joy, Sadness] => Some("bittersweet"),
            [Joy, Surprise] => Some("delighted_surprise"),
            [Fear, Sadness] => Some("anxious_grief"),
            [Anger, Fear] => Some("threatened_anger"),
            [Anger, Disgust] => Some("contempt"),
            [Fear, Surprise] => Some("startled_fear"),
            _ => None,
        }
        .map(str::to_string)
    }

    /// Label dispersion: single label -> 0.0; conflicting pair -> 0.8;
    /// same-valence pair -> 0.25; three labels -> 0.6.
    #[must_use]
    pub fn dissonance_score(&self) -> f64 {
        match self.labels.len() {
            1 => 0.0,
            3 => 0.6,
            _ => {
                if is_conflicting_pair(self.labels[0], self.labels[1]) {
                    0.8
                } else {
                    0.25
                }
            }
        }
    }

    /// Assembles the full deterministic `ComplexEmotionOutput` (confidence = intensity).
    #[must_use]
    pub fn to_complex_emotion_output(&self) -> ComplexEmotionOutput {
        ComplexEmotionOutput {
            source: EMO_MARKER_SOURCE.to_string(),
            narrative_hint: self.narrative_hint.clone().unwrap_or_default(),
            labels: self.labels.iter().map(|l| l.as_str().to_string()).collect(),
            pattern: self.pattern(),
            confidence: self.intensity,
            intensity: self.intensity,
            dissonance_score: self.dissonance_score(),
            degraded_to_builtin: false,
            extension: None,
        }
    }
}

/// Parses the **last** `[EMO]{...}[/EMO]` block and returns the marker plus the
/// reply with **all** complete blocks removed (display hygiene).
///
/// Degradation contract: any malformed block (bad JSON, invalid enum, wrong
/// label count, missing closer) returns `None` so the caller can fall back;
/// the attempted markers are still stripped for display hygiene.
#[must_use]
pub fn parse_and_strip(reply: &str) -> (String, Option<EmoMarker>) {
    let mut blocks = Vec::new();
    let mut unclosed_start = None;
    let mut scan = 0;
    while let Some(rel_start) = reply[scan..].find(EMO_MARKER_OPEN) {
        let start = scan + rel_start;
        let body_start = start + EMO_MARKER_OPEN.len();
        let Some(rel_close) = reply[body_start..].find(EMO_MARKER_CLOSE) else {
            unclosed_start = Some(start);
            break;
        };
        let body_end = body_start + rel_close;
        let close_end = body_end + EMO_MARKER_CLOSE.len();
        blocks.push((start, body_start, body_end, close_end));
        scan = close_end;
    }

    let marker = if unclosed_start.is_some() {
        None
    } else {
        blocks
            .last()
            .copied()
            .and_then(|(_, body_start, body_end, _)| parse_marker(&reply[body_start..body_end]))
    };

    if blocks.is_empty() && unclosed_start.is_none() {
        return (reply.to_string(), None);
    }

    let mut cleaned = reply.to_string();
    if let Some(start) = unclosed_start {
        cleaned.replace_range(start.., "");
    }
    for (start, _, _, close_end) in blocks.into_iter().rev() {
        cleaned.replace_range(start..close_end, "");
    }
    let cleaned = cleaned.trim_end().to_string();
    (cleaned, marker)
}

/// Parses and validates the JSON payload; returns `None` on any violation.
fn parse_marker(raw: &str) -> Option<EmoMarker> {
    let parsed: EmoMarkerRaw = serde_json::from_str(raw).ok()?;
    if parsed.labels.is_empty() || parsed.labels.len() > 3 {
        return None;
    }
    let mut labels = Vec::with_capacity(parsed.labels.len());
    for label in &parsed.labels {
        let emo_label = EmoLabel::parse(label)?;
        if !labels.contains(&emo_label) {
            labels.push(emo_label);
        }
    }
    if labels.is_empty() {
        return None;
    }
    let intensity = parsed.intensity.unwrap_or(0.5).clamp(0.0, 1.0);
    Some(EmoMarker {
        labels,
        intensity,
        narrative_hint: parsed
            .narrative_hint
            .as_deref()
            .map(truncate_narrative_hint),
    })
}

#[derive(Debug, Deserialize)]
struct EmoMarkerRaw {
    labels: Vec<String>,
    #[serde(default)]
    intensity: Option<f64>,
    #[serde(default)]
    narrative_hint: Option<String>,
}

fn is_conflicting_pair(a: EmoLabel, b: EmoLabel) -> bool {
    fn valence(label: EmoLabel) -> i8 {
        match label {
            EmoLabel::Joy | EmoLabel::Surprise => 1,
            EmoLabel::Neutral => 0,
            EmoLabel::Sadness | EmoLabel::Anger | EmoLabel::Fear | EmoLabel::Disgust => -1,
        }
    }
    valence(a) * valence(b) < 0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn marker(labels: &[&str], intensity: f64, hint: Option<&str>) -> EmoMarker {
        let labels = labels
            .iter()
            .map(|l| EmoLabel::parse(l).expect("valid label"))
            .collect();
        EmoMarker {
            labels,
            intensity,
            narrative_hint: hint.map(str::to_string),
        }
    }

    #[test]
    fn parses_good_marker_and_strips_block() {
        let reply = "今天也很开心呢~\n\n[EMO]{\"labels\":[\"joy\",\"surprise\"],\"intensity\":0.8,\"narrative_hint\":\"用户夸我，心情雀跃\"}[/EMO]";
        let (cleaned, parsed) = parse_and_strip(reply);
        assert_eq!(cleaned, "今天也很开心呢~");
        let parsed = parsed.expect("marker parsed");
        assert_eq!(parsed.labels, vec![EmoLabel::Joy, EmoLabel::Surprise]);
        assert_eq!(parsed.intensity, 0.8);
        assert_eq!(parsed.narrative_hint.as_deref(), Some("用户夸我，心情雀跃"));
        assert_eq!(parsed.dominant_emotion(), Emotion::Happy);
    }

    #[test]
    fn missing_marker_returns_none_and_keeps_reply() {
        let reply = "普通的回复，没有任何标记";
        let (cleaned, parsed) = parse_and_strip(reply);
        assert_eq!(cleaned, reply);
        assert!(parsed.is_none());
    }

    #[test]
    fn bad_json_degrades_but_still_strips() {
        let reply = "台词\n\n[EMO]{not json}[/EMO]";
        let (cleaned, parsed) = parse_and_strip(reply);
        assert_eq!(cleaned, "台词");
        assert!(parsed.is_none());
    }

    #[test]
    fn invalid_enum_degrades() {
        let reply = "[EMO]{\"labels\":[\"angry\",\"hype\"],\"intensity\":0.5}[/EMO]";
        let (_, parsed) = parse_and_strip(reply);
        assert!(parsed.is_none());
    }

    #[test]
    fn wrong_label_count_degrades() {
        let empty = "[EMO]{\"labels\":[],\"intensity\":0.5}[/EMO]";
        assert!(parse_and_strip(empty).1.is_none());
        let too_many =
            "[EMO]{\"labels\":[\"joy\",\"sadness\",\"anger\",\"fear\"],\"intensity\":0.5}[/EMO]";
        assert!(parse_and_strip(too_many).1.is_none());
    }

    #[test]
    fn takes_last_block_when_multiple() {
        let reply = "[EMO]{\"labels\":[\"joy\"],\"intensity\":0.2}[/EMO]中间[EMO]{\"labels\":[\"anger\",\"sadness\"],\"intensity\":0.9}[/EMO]";
        let (cleaned, parsed) = parse_and_strip(reply);
        assert_eq!(cleaned, "中间");
        let parsed = parsed.expect("last marker parsed");
        assert_eq!(parsed.labels, vec![EmoLabel::Anger, EmoLabel::Sadness]);
        assert_eq!(parsed.intensity, 0.9);
    }

    #[test]
    fn unclosed_marker_degrades_and_strips_internal_tail() {
        let reply = "台词[EMO]{\"labels\":[\"joy\"]}";
        let (cleaned, parsed) = parse_and_strip(reply);
        assert_eq!(cleaned, "台词");
        assert!(parsed.is_none());
    }

    #[test]
    fn trailing_unclosed_marker_invalidates_last_attempt_and_strips_all_markers() {
        let reply = "台词[EMO]{\"labels\":[\"joy\"]}[/EMO]中间[EMO]{\"labels\":[\"anger\"]}";
        let (cleaned, parsed) = parse_and_strip(reply);
        assert_eq!(cleaned, "台词中间");
        assert!(parsed.is_none());
    }

    #[test]
    fn intensity_is_clamped_and_defaulted() {
        let high = "[EMO]{\"labels\":[\"joy\"],\"intensity\":2.5}[/EMO]";
        assert_eq!(parse_and_strip(high).1.unwrap().intensity, 1.0);
        let low = "[EMO]{\"labels\":[\"joy\"],\"intensity\":-3}[/EMO]";
        assert_eq!(parse_and_strip(low).1.unwrap().intensity, 0.0);
        let missing = "[EMO]{\"labels\":[\"joy\"]}[/EMO]";
        assert_eq!(parse_and_strip(missing).1.unwrap().intensity, 0.5);
    }

    #[test]
    fn hint_missing_and_empty_are_distinct() {
        let missing = "[EMO]{\"labels\":[\"joy\"],\"intensity\":0.5}[/EMO]";
        assert!(parse_and_strip(missing).1.unwrap().narrative_hint.is_none());
        let empty = "[EMO]{\"labels\":[\"joy\"],\"intensity\":0.5,\"narrative_hint\":\"\"}[/EMO]";
        assert_eq!(
            parse_and_strip(empty).1.unwrap().narrative_hint,
            Some(String::new())
        );
    }

    #[test]
    fn narrative_hint_is_hard_capped_by_unicode_characters() {
        let hint = format!(
            "{}{}",
            "情".repeat(MAX_NARRATIVE_HINT_CHARS),
            "🙂".repeat(5)
        );
        let reply = format!(
            "台词[EMO]{{\"labels\":[\"joy\"],\"intensity\":0.5,\"narrative_hint\":\"{hint}\"}}[/EMO]"
        );
        let parsed = parse_and_strip(&reply).1.expect("marker parsed");
        let stored = parsed.narrative_hint.expect("hint");
        assert_eq!(stored.chars().count(), MAX_NARRATIVE_HINT_CHARS);
        assert_eq!(stored, "情".repeat(MAX_NARRATIVE_HINT_CHARS));
    }

    #[test]
    fn dominant_mapping_matches_six_slot_graph() {
        let cases = [
            (EmoLabel::Joy, Emotion::Happy),
            (EmoLabel::Sadness, Emotion::Sad),
            (EmoLabel::Anger, Emotion::Angry),
            (EmoLabel::Surprise, Emotion::Excited),
            (EmoLabel::Fear, Emotion::Confused),
            (EmoLabel::Disgust, Emotion::Confused),
            (EmoLabel::Neutral, Emotion::Neutral),
        ];
        for (label, expected) in cases {
            let m = marker(&[label.as_str()], 0.5, None);
            assert_eq!(m.dominant_emotion(), expected);
        }
    }

    #[test]
    fn pattern_map_covers_common_pairs() {
        let m = marker(&["anger", "sadness"], 0.7, None);
        assert_eq!(m.pattern().as_deref(), Some("resentful_sad"));
        let reversed = marker(&["sadness", "anger"], 0.7, None);
        assert_eq!(reversed.pattern().as_deref(), Some("resentful_sad"));
        let unknown = marker(&["neutral", "surprise"], 0.7, None);
        assert_eq!(unknown.pattern(), None);
        let single = marker(&["joy"], 0.5, None);
        assert_eq!(single.pattern(), None);
    }

    #[test]
    fn dissonance_is_dispersion_based() {
        assert_eq!(marker(&["joy"], 0.5, None).dissonance_score(), 0.0);
        assert_eq!(
            marker(&["joy", "sadness"], 0.5, None).dissonance_score(),
            0.8
        );
        assert_eq!(
            marker(&["anger", "sadness"], 0.5, None).dissonance_score(),
            0.25
        );
        assert_eq!(
            marker(&["joy", "surprise", "neutral"], 0.5, None).dissonance_score(),
            0.6
        );
    }

    #[test]
    fn derived_output_aligns_fields() {
        let m = marker(&["anger", "sadness"], 0.9, Some("测试提示"));
        let out = m.to_complex_emotion_output();
        assert_eq!(out.source, EMO_MARKER_SOURCE);
        assert_eq!(out.narrative_hint, "测试提示");
        assert_eq!(out.labels, vec!["anger", "sadness"]);
        assert_eq!(out.pattern.as_deref(), Some("resentful_sad"));
        assert_eq!(out.confidence, 0.9);
        assert_eq!(out.intensity, 0.9);
        assert_eq!(out.dissonance_score, 0.25);
        assert!(!out.degraded_to_builtin);
    }

    #[test]
    fn duplicate_labels_are_deduped() {
        let reply = "[EMO]{\"labels\":[\"joy\",\"joy\"],\"intensity\":0.4}[/EMO]";
        let parsed = parse_and_strip(reply).1.expect("marker parsed");
        assert_eq!(parsed.labels, vec![EmoLabel::Joy]);
    }
}
