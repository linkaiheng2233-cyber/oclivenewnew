//! Lexicon-based emotion suggestion provider.
//!
//! Loads the keyword lexicon (Chinese + English in one JSON file) at compile
//! time via `include_str!` and produces **multi-label suggestions** (not
//! verdicts): weighted accumulation -> ÷max normalization -> tie-break ->
//! top-3 labels plus per-hit details. The main LLM remains the sole arbiter
//! of complex emotions (B stage); this module only feeds priors.

use std::sync::OnceLock;

use serde::Deserialize;

use oclive_kernel_types::{AppError, EmotionResult};

use crate::error::Result;

const LEXICON_JSON: &str = include_str!("lexicon_zh_cn.json");

/// Negation markers checked inside a fixed window before a matched word.
const NEGATION_MARKERS: [char; 3] = ['不', '没', '别'];
/// Characters scanned before a matched word to detect negation.
const NEGATION_WINDOW_CHARS: usize = 5;

/// Canonical dimension order; array index == score slot index.
///
/// This order is also the tie-break priority: earlier dimensions win ties
/// (joy > sadness > anger > fear > surprise > disgust > neutral).
const ALL_DIMENSIONS: [EmotionDimension; 7] = [
    EmotionDimension::Joy,
    EmotionDimension::Sadness,
    EmotionDimension::Anger,
    EmotionDimension::Fear,
    EmotionDimension::Surprise,
    EmotionDimension::Disgust,
    EmotionDimension::Neutral,
];

/// The seven `EmotionResult` dimensions used as lexicon labels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum EmotionDimension {
    Joy,
    Sadness,
    Anger,
    Fear,
    Surprise,
    Disgust,
    Neutral,
}

impl EmotionDimension {
    /// Slot index inside [`ALL_DIMENSIONS`] / `LexiconSuggestion::scores`.
    fn index(self) -> usize {
        ALL_DIMENSIONS
            .iter()
            .position(|dimension| *dimension == self)
            .expect("dimension is part of ALL_DIMENSIONS")
    }
}

/// How an entry is matched against the input text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
enum MatchMode {
    /// Plain substring match (default; used by Chinese entries).
    #[default]
    Substring,
    /// English word/phrase match with ASCII identifier boundaries. This
    /// accepts punctuation and Unicode whitespace while avoiding false hits
    /// like `glove -> love`.
    SpaceBoundary,
}

/// A single lexicon entry (mirrors the JSON schema).
/// `source_batch`/`anchor_sentence` are audit/reserved fields consumed by B stage.
#[derive(Deserialize)]
#[allow(dead_code)]
struct LexiconEntry {
    /// Bare word without padding spaces (JSON stores the raw form).
    word: String,
    /// One or more emotion labels.
    labels: Vec<EmotionDimension>,
    /// 1..=5; the more direct the emotion meaning, the higher the weight.
    weight: u8,
    #[serde(default)]
    match_mode: MatchMode,
    /// Audit / rollback batch identifier (e.g. `seed` for the initial migration).
    source_batch: String,
    /// Reserved for a future embedding-based phase; always `null` for now.
    #[serde(default)]
    anchor_sentence: Option<String>,
}

/// Parsed lexicon file (top-level JSON shape).
#[derive(Deserialize)]
struct LexiconFile {
    schema_version: u32,
    lang: String,
    labels: Vec<String>,
    entries: Vec<LexiconEntry>,
}

/// Compile-time embedded, validated lexicon.
pub(crate) struct Lexicon {
    entries: Vec<LexiconEntry>,
}

/// A single matched word with its contribution details.
/// Consumed by tests now and by B-stage consumers (memory/portrait/events).
#[cfg_attr(test, derive(Debug))]
#[allow(dead_code)]
pub(crate) struct LexiconHit {
    pub(crate) word: String,
    pub(crate) labels: Vec<EmotionDimension>,
    pub(crate) weight: u8,
    pub(crate) negated: bool,
}

/// Multi-label suggestion produced from the lexicon.
///
/// This is a prior/auxiliary signal only: it never overrides an existing
/// emotion state on degradation (the caller decides what to keep).
/// `top_labels`/`hits` are consumed by tests now and by B-stage consumers.
#[cfg_attr(test, derive(Debug))]
#[allow(dead_code)]
pub(crate) struct LexiconSuggestion {
    /// Normalized per-dimension scores, indexed by [`ALL_DIMENSIONS`].
    pub(crate) scores: [f64; 7],
    /// Top-3 labels after ÷max normalization and tie-break (score > 0 only;
    /// exactly `[(Neutral, 1.0)]` when nothing matched).
    pub(crate) top_labels: Vec<(EmotionDimension, f64)>,
    /// Matched words, including negation-filtered ones (for debugging/audit).
    pub(crate) hits: Vec<LexiconHit>,
}

/// Returns the embedded lexicon, parsing and validating it once.
pub(crate) fn lexicon() -> Result<&'static Lexicon> {
    static LEXICON: OnceLock<std::result::Result<Lexicon, String>> = OnceLock::new();
    match LEXICON.get_or_init(|| Lexicon::load(LEXICON_JSON)) {
        Ok(lexicon) => Ok(lexicon),
        Err(reason) => Err(AppError::InvalidParameter(format!(
            "emotion lexicon invalid: {reason}"
        ))),
    }
}

impl Lexicon {
    /// Parses and validates the lexicon JSON.
    fn load(json: &str) -> std::result::Result<Self, String> {
        let file: LexiconFile = serde_json::from_str(json)
            .map_err(|err| format!("lexicon JSON parse failed: {err}"))?;
        if file.schema_version != 1 {
            return Err(format!(
                "unsupported schema_version: {}",
                file.schema_version
            ));
        }
        if file.lang != "zh-CN" {
            return Err(format!("unexpected lang: {}", file.lang));
        }
        let expected_labels = [
            "joy", "sadness", "anger", "fear", "surprise", "disgust", "neutral",
        ];
        if file.labels != expected_labels {
            return Err(format!("labels mismatch: {:?}", file.labels));
        }
        if file.entries.is_empty() {
            return Err("lexicon has no entries".to_owned());
        }
        let mut seen = std::collections::HashSet::new();
        for entry in &file.entries {
            if entry.word.trim().is_empty() {
                return Err("entry with empty word".to_owned());
            }
            if entry.labels.is_empty() {
                return Err(format!("word {:?} has no labels", entry.word));
            }
            if !(1..=5).contains(&entry.weight) {
                return Err(format!(
                    "word {:?} weight {} out of 1..=5",
                    entry.word, entry.weight
                ));
            }
            if !seen.insert(entry.word.as_str()) {
                return Err(format!("duplicate word: {:?}", entry.word));
            }
        }
        Ok(Self {
            entries: file.entries,
        })
    }

    /// Runs the weighted multi-label analysis over `text`.
    pub(crate) fn analyze(&self, text: &str) -> LexiconSuggestion {
        let text_lower = text.to_lowercase();
        let mut acc = [0.0_f64; 7];
        let mut hits = Vec::with_capacity(self.entries.len().min(32));
        for entry in &self.entries {
            let Some(match_start) = self.match_start(&text_lower, entry) else {
                continue;
            };
            let negated = is_negated(&text_lower, match_start);
            if !negated {
                for label in &entry.labels {
                    acc[label.index()] += f64::from(entry.weight);
                }
            }
            hits.push(LexiconHit {
                word: entry.word.clone(),
                labels: entry.labels.clone(),
                weight: entry.weight,
                negated,
            });
        }
        let max = acc.iter().copied().fold(0.0_f64, f64::max);
        let scores = if max > 0.0 {
            let mut normalized = [0.0_f64; 7];
            for (index, value) in acc.iter().enumerate() {
                normalized[index] = value / max;
            }
            normalized
        } else {
            let mut neutral = [0.0_f64; 7];
            neutral[EmotionDimension::Neutral.index()] = 1.0;
            neutral
        };
        let mut ranked: Vec<(usize, f64)> = scores
            .iter()
            .enumerate()
            .filter(|(_, value)| **value > 0.0)
            .map(|(index, value)| (index, *value))
            .collect();
        // Higher score first; ties resolved by dimension priority (lower
        // `ALL_DIMENSIONS` index wins), keeping `dominant_emotion` semantics.
        ranked.sort_by(|a, b| match b.1.partial_cmp(&a.1) {
            Some(std::cmp::Ordering::Equal) | None => a.0.cmp(&b.0),
            Some(ordering) => ordering,
        });
        let top_labels = ranked
            .into_iter()
            .take(3)
            .map(|(index, score)| (ALL_DIMENSIONS[index], score))
            .collect();
        LexiconSuggestion {
            scores,
            top_labels,
            hits,
        }
    }

    /// Byte offset of the first match in the lowercased input.
    fn match_start(&self, text_lower: &str, entry: &LexiconEntry) -> Option<usize> {
        match entry.match_mode {
            MatchMode::Substring => text_lower.find(&entry.word),
            MatchMode::SpaceBoundary => {
                match_ascii_word_boundary(text_lower, &entry.word.to_lowercase())
            }
        }
    }

    /// Convenience: maps the suggestion back to the kernel `EmotionResult`.
    pub(crate) fn to_emotion_result(suggestion: &LexiconSuggestion) -> EmotionResult {
        let mut result = EmotionResult {
            joy: 0.0,
            sadness: 0.0,
            anger: 0.0,
            fear: 0.0,
            surprise: 0.0,
            disgust: 0.0,
            neutral: 0.0,
            extension: None,
        };
        for (index, dimension) in ALL_DIMENSIONS.iter().enumerate() {
            match dimension {
                EmotionDimension::Joy => result.joy = suggestion.scores[index],
                EmotionDimension::Sadness => result.sadness = suggestion.scores[index],
                EmotionDimension::Anger => result.anger = suggestion.scores[index],
                EmotionDimension::Fear => result.fear = suggestion.scores[index],
                EmotionDimension::Surprise => result.surprise = suggestion.scores[index],
                EmotionDimension::Disgust => result.disgust = suggestion.scores[index],
                EmotionDimension::Neutral => result.neutral = suggestion.scores[index],
            }
        }
        result
    }
}

/// True when one of the negation markers appears in the 5-char window right
/// before `match_start` (simple version; no segmentation involved).
fn is_negated(haystack: &str, match_start: usize) -> bool {
    let prefix = &haystack[..match_start];
    let window: String = prefix.chars().rev().take(NEGATION_WINDOW_CHARS).collect();
    window.chars().any(|ch| NEGATION_MARKERS.contains(&ch)) || english_negation_before(prefix)
}

fn match_ascii_word_boundary(haystack: &str, needle: &str) -> Option<usize> {
    haystack.match_indices(needle).find_map(|(start, _)| {
        let before = haystack[..start].chars().next_back();
        let end = start + needle.len();
        let after = haystack[end..].chars().next();
        let is_identifier = |ch: char| ch.is_ascii_alphanumeric() || ch == '_';
        if before.is_none_or(|ch| !is_identifier(ch)) && after.is_none_or(|ch| !is_identifier(ch)) {
            Some(start)
        } else {
            None
        }
    })
}

fn english_negation_before(prefix: &str) -> bool {
    let trimmed =
        prefix.trim_end_matches(|ch: char| !ch.is_ascii_alphanumeric() && ch != '\'' && ch != '’');
    // The same separator predicate as before, but the separator's *width* is kept: the token starts
    // after the whole character, so `trimmed[index + ch.len_utf8()..]` is always a char boundary.
    let token_start = trimmed
        .char_indices()
        .rev()
        .find(|(_, ch)| !ch.is_ascii_alphanumeric() && *ch != '\'' && *ch != '’')
        .map_or(0, |(index, ch)| index + ch.len_utf8());
    let token = &trimmed[token_start..];
    matches!(token, "no" | "not" | "never") || token.ends_with("n't") || token.ends_with("n’t")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn analyze(text: &str) -> LexiconSuggestion {
        Lexicon::load(LEXICON_JSON)
            .expect("embedded lexicon must parse")
            .analyze(text)
    }

    fn score(suggestion: &LexiconSuggestion, dimension: EmotionDimension) -> f64 {
        suggestion.scores[dimension.index()]
    }

    fn top1(suggestion: &LexiconSuggestion) -> EmotionDimension {
        suggestion.top_labels[0].0
    }

    #[test]
    fn test_load_rejects_invalid_json() {
        assert!(Lexicon::load("not json").is_err());
    }

    #[test]
    fn test_load_rejects_unknown_label() {
        let json = r#"
        {
          "schema_version": 1,
          "lang": "zh-CN",
          "labels": ["joy", "sadness", "anger", "fear", "surprise", "disgust", "neutral"],
          "entries": [
            { "word": "开心", "labels": ["ecstasy"], "weight": 3, "match_mode": "substring", "source_batch": "seed" }
          ]
        }
        "#;
        assert!(Lexicon::load(json).is_err());
    }

    #[test]
    fn test_load_rejects_weight_out_of_range() {
        let json = r#"
        {
          "schema_version": 1,
          "lang": "zh-CN",
          "labels": ["joy", "sadness", "anger", "fear", "surprise", "disgust", "neutral"],
          "entries": [
            { "word": "开心", "labels": ["joy"], "weight": 9, "match_mode": "substring", "source_batch": "seed" }
          ]
        }
        "#;
        assert!(Lexicon::load(json).is_err());
    }

    #[test]
    fn test_weighted_accumulation() {
        // Two joy hits accumulate before normalization.
        let suggestion = analyze("开心哈哈");
        assert_eq!(score(&suggestion, EmotionDimension::Joy), 1.0);
        let suggestion = analyze("开心难过");
        assert_eq!(score(&suggestion, EmotionDimension::Joy), 1.0);
        assert_eq!(score(&suggestion, EmotionDimension::Sadness), 1.0);
    }

    #[test]
    fn test_tie_break_prefers_joy() {
        let suggestion = analyze("开心难过");
        assert_eq!(top1(&suggestion), EmotionDimension::Joy);
    }

    #[test]
    fn test_negation_window() {
        let suggestion = analyze("我没生气");
        assert_eq!(score(&suggestion, EmotionDimension::Anger), 0.0);
        assert_eq!(top1(&suggestion), EmotionDimension::Neutral);
        assert!(suggestion.hits.iter().any(|hit| hit.negated));
    }

    #[test]
    fn test_space_boundary_no_substring_false_positive() {
        let suggestion = analyze("i am unhappy");
        assert_eq!(score(&suggestion, EmotionDimension::Sadness), 0.0);
        let suggestion = analyze("i am happy");
        assert_eq!(score(&suggestion, EmotionDimension::Joy), 1.0);
    }

    #[test]
    fn test_space_boundary_accepts_punctuation_and_unicode_whitespace() {
        let suggestion = analyze("I'm HAPPY!\nso sad.\tangry?");
        assert_eq!(score(&suggestion, EmotionDimension::Joy), 1.0);
        assert_eq!(score(&suggestion, EmotionDimension::Sadness), 1.0);
        assert_eq!(score(&suggestion, EmotionDimension::Anger), 1.0);
    }

    #[test]
    fn test_english_negation_suppresses_direct_emotion_word() {
        for text in ["not happy", "never sad", "isn't angry", "isn’t afraid"] {
            let suggestion = analyze(text);
            assert_eq!(
                top1(&suggestion),
                EmotionDimension::Neutral,
                "negated input must stay neutral: {text:?}"
            );
            assert!(
                suggestion.hits.iter().any(|hit| hit.negated),
                "negated keyword should remain visible in audit hits: {text:?}"
            );
        }
    }

    #[test]
    fn test_empty_text_falls_back_to_neutral() {
        let suggestion = analyze("");
        assert_eq!(score(&suggestion, EmotionDimension::Neutral), 1.0);
        assert_eq!(top1(&suggestion), EmotionDimension::Neutral);
    }

    /// Counter-examples: the stated dimension must NOT be hit at all
    /// (strict assertion; keeps the LLM prior unpolluted).
    #[test]
    fn test_counter_examples_strict() {
        let cases = [
            ("我没生气", EmotionDimension::Anger),
            ("才没有讨厌你", EmotionDimension::Anger),
            ("我服了行了吧", EmotionDimension::Anger),
            ("大写的服", EmotionDimension::Anger),
            ("滚动", EmotionDimension::Anger),
            ("我想死你了", EmotionDimension::Sadness),
            ("可爱", EmotionDimension::Anger),
            ("打哈欠", EmotionDimension::Surprise),
            ("书架", EmotionDimension::Anger),
            ("我一点都不难过", EmotionDimension::Sadness),
            ("不害怕", EmotionDimension::Fear),
            ("没什么大不了", EmotionDimension::Sadness),
            ("笑哭", EmotionDimension::Sadness),
            ("哭笑不得", EmotionDimension::Sadness),
        ];
        for (text, forbidden) in cases {
            let suggestion = analyze(text);
            assert_eq!(
                score(&suggestion, forbidden),
                0.0,
                "counter-example {text:?} must not hit {forbidden:?}: {:?}",
                suggestion.hits
            );
        }
    }

    /// Structural invariants mirroring the collection rules (v1.3 four-layer
    /// 收录原则): single-char words may only stay at the lowest weight, so a
    /// future expansion must consciously justify any single-char entry.
    #[test]
    fn test_embedded_lexicon_four_layer_rules() {
        let lexicon = Lexicon::load(LEXICON_JSON).expect("embedded lexicon must parse");
        assert!(!lexicon.entries.is_empty(), "lexicon must not be empty");
        for entry in &lexicon.entries {
            assert!(!entry.word.trim().is_empty(), "empty word in lexicon");
            assert!(
                !entry.labels.is_empty(),
                "entry without labels: {:?}",
                entry.word
            );
            if entry.word.chars().count() == 1 {
                assert_eq!(
                    entry.weight, 1,
                    "single-char {:?} must stay at weight 1 (layer 4)",
                    entry.word
                );
            }
        }
    }

    /// Positive examples: the stated dimension must be top-1.
    #[test]
    fn test_positive_examples_top1() {
        let cases = [
            ("我好开心", EmotionDimension::Joy),
            ("气死我了", EmotionDimension::Anger),
            ("好难过", EmotionDimension::Sadness),
            ("好害怕", EmotionDimension::Fear),
            ("哇真的假的", EmotionDimension::Surprise),
            ("好恶心", EmotionDimension::Disgust),
            ("嗯嗯好的", EmotionDimension::Neutral),
        ];
        for (text, expected) in cases {
            let suggestion = analyze(text);
            assert_eq!(
                top1(&suggestion),
                expected,
                "positive example {text:?} must be top-1 (hits: {:?})",
                suggestion.hits
            );
        }
    }

    #[test]
    fn test_counter_example_neutral_hit_only() {
        // "行吧" is a neutral entry; the assertion is anger-not-hit, not zero hits.
        let suggestion = analyze("我服了行了吧");
        assert_eq!(score(&suggestion, EmotionDimension::Anger), 0.0);
        assert_eq!(score(&suggestion, EmotionDimension::Neutral), 1.0);
        assert_eq!(top1(&suggestion), EmotionDimension::Neutral);
    }

    // ===== B2-C4-R1: UTF-8 boundary regression on the shared negation helper =====
    //
    // `is_negated` calls `english_negation_before` for every hit, regardless of the entry's
    // `match_mode`, so a multi-byte character before an English token used to reach a byte slice
    // that started inside a character. These tests use the real `Lexicon::analyze` and the real
    // helper; they do not reference `KeywordEmotionBase`, so the shared fix stays independently
    // committable.

    /// The rule marker of the hit for `word`, or `None` when that entry did not match.
    fn hit_negated(suggestion: &LexiconSuggestion, word: &str) -> Option<bool> {
        suggestion
            .hits
            .iter()
            .find(|hit| hit.word == word)
            .map(|hit| hit.negated)
    }

    /// The labels of the hit for `word`, or `None` when that entry did not match.
    fn hit_labels(suggestion: &LexiconSuggestion, word: &str) -> Option<Vec<EmotionDimension>> {
        suggestion
            .hits
            .iter()
            .find(|hit| hit.word == word)
            .map(|hit| hit.labels.clone())
    }

    /// The words of all hits, in the order `Lexicon::analyze` produced them.
    fn hit_words(suggestion: &LexiconSuggestion) -> Vec<String> {
        suggestion.hits.iter().map(|hit| hit.word.clone()).collect()
    }

    #[test]
    fn b2_c4_utf8_mixed_material_is_analysed_without_panicking() {
        // The original C4 counter-example: two subjects, one multi-byte comma, two Chinese entries.
        let suggestion = analyze("Alice 很难过，Bob 很开心");
        assert_eq!(
            hit_negated(&suggestion, "开心"),
            Some(false),
            "the joy entry must match and stay unnegated: {:?}",
            suggestion.hits
        );
        assert_eq!(hit_negated(&suggestion, "难过"), Some(false));
        assert!(
            !suggestion.hits.iter().any(|hit| hit.negated),
            "no rule marker may fire in this material: {:?}",
            suggestion.hits
        );

        // A *Chinese* substring entry after a multi-byte character reaches the same helper.
        let suggestion = analyze("中a开心");
        assert_eq!(
            hit_negated(&suggestion, "开心"),
            Some(false),
            "the Chinese substring entry must match: {:?}",
            suggestion.hits
        );
        assert_eq!(
            hit_labels(&suggestion, "开心"),
            Some(vec![EmotionDimension::Joy])
        );

        // Mixed script before an English space-boundary entry.
        let suggestion = analyze("中a happy");
        assert_eq!(hit_negated(&suggestion, "happy"), Some(false));
        assert_eq!(
            hit_labels(&suggestion, "happy"),
            Some(vec![EmotionDimension::Joy])
        );

        // Multi-byte text followed by an English entry, with nothing else matching.
        let suggestion = analyze("中文，happy");
        assert_eq!(
            hit_words(&suggestion),
            vec!["happy".to_string()],
            "only the English entry matches here: {:?}",
            suggestion.hits
        );
        assert_eq!(hit_negated(&suggestion, "happy"), Some(false));
        assert_eq!(score(&suggestion, EmotionDimension::Joy), 1.0);
    }

    #[test]
    fn b2_c4_utf8_english_negation_tokens_still_fire_after_multibyte_text() {
        let cases = [
            ("中文，not happy", "happy", true),
            ("中文，not 开心", "开心", true),
            ("中a not happy", "happy", true),
            ("中文，happy", "happy", false),
        ];
        for (text, word, expected) in cases {
            let suggestion = analyze(text);
            assert_eq!(
                hit_negated(&suggestion, word),
                Some(expected),
                "{text:?} must report is_negated={expected} for {word:?}: {:?}",
                suggestion.hits
            );
        }
    }

    #[test]
    fn b2_c4_utf8_helper_boundary_table() {
        // The token must start after the *whole* separator character, whatever its UTF-8 width,
        // and the existing separator predicate / token set must be unchanged.
        let cases = [
            ("énot", true),    // 2-byte separator
            ("，never", true), // 3-byte separator
            ("🙂not", true),   // 4-byte separator
            (" not", true),    // 1-byte ASCII separator
            ("not", true),     // no separator at all
            ("", false),       // empty prefix
            ("，", false),     // separator only, empty token
            ("， ", false),    // separator plus trailing separator
            ("中文，not", true),
            ("；never", true),
            ("isn't", true),
            ("isn’t", true),
            ("'not", false), // straight apostrophe is not a separator; token becomes "'not"
            ("’not", false), // curly apostrophe is not a separator either
            ("，happy", false),
            ("，nothing", false),
        ];
        for (prefix, expected) in cases {
            assert_eq!(
                english_negation_before(prefix),
                expected,
                "english_negation_before({prefix:?})"
            );
        }
    }

    #[test]
    fn b2_c4_utf8_hits_order_is_lexicon_order_in_mixed_material() {
        // Fact recorded here because it shapes the C4 assertions: in this material the entries that
        // actually match are the Chinese ones (`开心` inside `很开心`, `难过` inside `很难过`), not
        // the English words, and the order is the lexicon's traversal order — not text order.
        let suggestion = analyze("Alice 很难过，Bob 很开心");
        assert_eq!(
            hit_words(&suggestion),
            vec!["开心".to_string(), "难过".to_string()],
            "hits must keep the lexicon's traversal order (text order is the reverse)"
        );
    }
}
