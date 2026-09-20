//! Emotion analysis module.
//!
//! Seven-dimension emotion analysis via the embedded lexicon (JSON) with
//! weighted scoring. The lexicon produces multi-label suggestions; complex
//! emotion arbitration is left to the main LLM (B stage).
//!
//! CP-B3-C2: `analyze_material` (a `pub(crate)` entry, so it has no public page of its own) is this
//! crate's **one** lexicon analysis entry. It is the only
//! place that loads the embedded lexicon and runs its matching, and it returns the lexicon's own
//! suggestion unchanged. Two projections exist on top of it: [`EmotionAnalyzer::analyze`] projects
//! the suggestion's scores into the seven-dimension type, while
//! [`crate::domain::base_emotion`]'s Base implementations project the same suggestion's matched
//! entries into a clue report. Sharing that entry shares the **analysis**, not the projections:
//! each call is one analysis, and calling a seven-dimension entry and a Base entry is two analyses.

use crate::domain::lexicon::{Lexicon, LexiconSuggestion};
use crate::error::Result;
use crate::models::Emotion;
pub use oclive_kernel_types::EmotionResult;

/// Runs this crate's one lexicon analysis for `text`.
///
/// This is the shared analysis entry behind both projections this crate offers: the seven-dimension
/// result ([`Lexicon::to_emotion_result`], used by [`EmotionAnalyzer::analyze`]) and the Base clue
/// report built by [`crate::domain::base_emotion`]. It returns the lexicon's suggestion as-is — it
/// chooses no projection, decides nothing about the material, and does not interpret the scores.
/// Entries excluded by the negation heuristic stay in the suggestion's matched list, so a caller can
/// still tell "no entry matched" from "an entry matched and was negated".
///
/// # Errors
///
/// Returns the embedded lexicon's load/validation error unchanged (the lexicon is parsed and
/// validated once inside a `OnceLock`). Nothing here retries, substitutes a fallback analysis or
/// turns a failure into an empty suggestion.
///
/// A call performs one lexicon run. There is no cache, no shared "latest result" and no
/// deduplication between calls, so two calls are two analyses by construction.
pub(crate) fn analyze_material(text: &str) -> Result<LexiconSuggestion> {
    let lexicon = crate::domain::lexicon::lexicon()?;
    Ok(lexicon.analyze(text))
}

/// Emotion analyzer.
pub struct EmotionAnalyzer;

impl EmotionAnalyzer {
    /// # Errors
    ///
    /// Returns [`Err`] with a human-readable message when the operation fails.
    /// Analyzes text emotion.
    ///
    /// # Arguments
    /// * `text` - Input text
    ///
    /// # Returns
    /// Emotion analysis result
    ///
    /// # Examples
    /// ```
    /// # use oclive_kernel_runtime::domain::emotion_analyzer::EmotionAnalyzer;
    /// let result = EmotionAnalyzer::analyze("我很开心").unwrap();
    /// assert!(result.joy > 0.0);
    /// ```
    pub fn analyze(text: &str) -> Result<EmotionResult> {
        // The shared analysis entry; only this projection differs from the Base view's.
        let suggestion = analyze_material(text)?;
        Ok(Lexicon::to_emotion_result(&suggestion))
    }

    /// Returns the dominant emotion.
    ///
    /// # Arguments
    /// * `result` - Emotion analysis result
    ///
    /// # Returns
    /// Dominant emotion type
    #[must_use]
    pub fn get_dominant_emotion(result: &EmotionResult) -> Emotion {
        result.dominant_emotion()
    }

    /// Max non-neutral dimension after normalization (complements `neutral`).
    fn max_affective(result: &EmotionResult) -> f64 {
        result
            .joy
            .max(result.sadness)
            .max(result.anger)
            .max(result.fear)
            .max(result.surprise)
            .max(result.disgust)
    }

    /// One-line Chinese tone hint for the main dialogue prompt (includes internal labels for debugging and plugin alignment).
    #[must_use]
    pub fn format_for_prompt(result: &EmotionResult) -> String {
        let dom = Self::get_dominant_emotion(result);
        let hint_zh = match dom {
            Emotion::Happy => "偏愉快、积极或感激，可先共鸣再展开",
            Emotion::Sad => "偏低落、疲惫或委屈，宜先安抚再聊事",
            Emotion::Angry => "偏冲、不满或烦躁，宜先降温、承认感受",
            Emotion::Excited => "偏兴奋或惊喜，可匹配能量、适度收束",
            Emotion::Confused => "偏不安、困惑或含糊，宜澄清与给安全感",
            Emotion::Shy => "偏拘谨、害羞，宜轻声、给台阶",
            Emotion::Neutral => "整体较平或信息性为主，按常速自然回",
        };
        let intensity = if result.neutral >= 0.55 {
            "弱·偏中性"
        } else {
            let m = Self::max_affective(result);
            if m >= 0.42 {
                "强"
            } else if m >= 0.28 {
                "中"
            } else {
                "弱"
            }
        };
        format!("{}（标签 {}，信号强度：{}）", hint_zh, dom, intensity)
    }

    /// Computes emotion intensity.
    ///
    /// # Arguments
    /// * `emotion` - Emotion type
    ///
    /// # Returns
    /// Emotion intensity [0.0, 1.0]
    #[must_use]
    pub fn calculate_intensity(emotion: &Emotion) -> f64 {
        match emotion {
            Emotion::Happy | Emotion::Angry => 0.8,
            Emotion::Sad | Emotion::Excited => 0.7,
            Emotion::Confused | Emotion::Shy => 0.5,
            Emotion::Neutral => 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyze_happy() {
        let result = EmotionAnalyzer::analyze("我很开心！").unwrap();
        assert!(result.joy > 0.0);
    }

    #[test]
    fn test_analyze_sad() {
        let result = EmotionAnalyzer::analyze("我很难过").unwrap();
        assert!(result.sadness > 0.0);
    }

    #[test]
    fn test_analyze_angry() {
        let result = EmotionAnalyzer::analyze("我很生气").unwrap();
        assert!(result.anger > 0.0);
    }

    #[test]
    fn test_get_dominant_emotion() {
        let result = EmotionAnalyzer::analyze("我很开心！").unwrap();
        let emotion = EmotionAnalyzer::get_dominant_emotion(&result);
        assert_eq!(emotion, Emotion::Happy);
    }

    #[test]
    fn test_calculate_intensity_happy() {
        let intensity = EmotionAnalyzer::calculate_intensity(&Emotion::Happy);
        assert_eq!(intensity, 0.8);
    }

    #[test]
    fn test_calculate_intensity_neutral() {
        let intensity = EmotionAnalyzer::calculate_intensity(&Emotion::Neutral);
        assert_eq!(intensity, 0.0);
    }

    #[test]
    fn test_empty_text() {
        let result = EmotionAnalyzer::analyze("").unwrap();
        assert_eq!(result.neutral, 1.0);
    }

    #[test]
    fn test_normalization_by_max() {
        // ÷max semantics: every matched dimension saturates to 1.0 (top-1 is
        // always 1.0 once any entry matches), so a mixed text keeps both.
        let result = EmotionAnalyzer::analyze("开心难过").unwrap();
        assert_eq!(result.joy, 1.0);
        assert_eq!(result.sadness, 1.0);
        assert_eq!(result.anger, 0.0);
    }

    #[test]
    fn test_analyze_thanks_joy() {
        let result = EmotionAnalyzer::analyze("谢谢你陪我").unwrap();
        assert!(result.joy > result.sadness, "thanks should lift joy");
    }

    #[test]
    fn test_format_for_prompt_includes_tag() {
        let result = EmotionAnalyzer::analyze("我好难过").unwrap();
        let line = EmotionAnalyzer::format_for_prompt(&result);
        assert!(line.contains("sad"), "line={}", line);
        assert!(line.contains("强度"));
    }
}

#[cfg(test)]
mod cp_b3_c2_tests {
    use super::*;
    use crate::domain::user_emotion_analyzer::BuiltinUserEmotionAnalyzer;

    /// The seven components in a fixed order, so a comparison covers every field rather than only
    /// the one that usually differs.
    fn components(result: &EmotionResult) -> [f64; 7] {
        [
            result.joy,
            result.sadness,
            result.anger,
            result.fear,
            result.surprise,
            result.disgust,
            result.neutral,
        ]
    }

    /// CP-B3-C2: the legacy entry, the registered builtin type and the shared analysis entry's own
    /// projection agree **field by field**, and a hand-written sample table pins the actual values —
    /// so "the same helper compared against itself" is not the only compatibility evidence. The
    /// sample values below were read off the embedded lexicon's entries (`开心` joy w3, `难过`
    /// sadness w3, `嗯`/`好的` neutral w1, no `不开心` entry, no entry for `今天星期三`) and the
    /// existing `b2_c4_*`/`test_*` expectations.
    #[test]
    fn cp_b3_c2_legacy_entries_match_the_shared_projection() {
        const SAMPLES: [(&str, [f64; 7]); 8] = [
            ("我很开心", [1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]),
            ("我很难过", [0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0]),
            ("难过开心", [1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0]),
            ("嗯嗯好的", [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0]),
            ("我不开心", [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0]),
            ("今天星期三", [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0]),
            ("中a happy", [1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]),
            ("unhappy", [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0]),
        ];

        for (material, expected) in SAMPLES {
            let legacy = EmotionAnalyzer::analyze(material).expect("analysis must succeed");
            assert_eq!(
                components(&legacy),
                expected,
                "{material}: the legacy entry's sample values"
            );
            assert!(
                legacy.extension.is_none(),
                "{material}: the legacy entry invents no extension"
            );

            let builtin = BuiltinUserEmotionAnalyzer;
            let via_builtin =
                <BuiltinUserEmotionAnalyzer as oclive_kernel_contracts::UserEmotionAnalyzer>::analyze(
                    &builtin, material,
                )
                .expect("analysis must succeed");
            assert_eq!(
                components(&via_builtin),
                expected,
                "{material}: the registered builtin type"
            );
            assert_eq!(
                components(&via_builtin),
                components(&legacy),
                "{material}: legacy and builtin must not drift"
            );

            let suggestion = analyze_material(material).expect("the shared analysis entry");
            let projected = Lexicon::to_emotion_result(&suggestion);
            assert_eq!(
                components(&projected),
                expected,
                "{material}: the shared entry's own projection"
            );
        }
    }

    /// CP-B3-C2: the shared analysis entry returns the lexicon's suggestion unchanged, so the
    /// entries that matched stay visible even when the numeric projection falls back to `neutral`.
    ///
    /// This is a fact about the **suggestion**, not about `EmotionResult`: it is what makes the Base
    /// clue report able to distinguish "no entry matched" from "an entry matched and was negated",
    /// while the seven-dimension fallback cannot.
    #[test]
    fn cp_b3_c2_shared_entry_keeps_hits_when_scores_fall_back() {
        let no_clue = analyze_material("今天星期三").expect("analysis must succeed");
        assert!(no_clue.hits.is_empty(), "no entry matches this material");
        assert_eq!(
            components(&Lexicon::to_emotion_result(&no_clue)),
            [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0],
            "no usable hit still projects onto the compatible neutral fallback"
        );

        let negated = analyze_material("我不开心").expect("analysis must succeed");
        assert_eq!(negated.hits.len(), 1, "the negated entry is still a hit");
        assert!(
            negated.hits[0].negated,
            "the negation heuristic must have fired"
        );
        assert_eq!(negated.hits[0].word, "开心");
        assert_eq!(
            components(&Lexicon::to_emotion_result(&negated)),
            [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0],
            "a negated-only hit projects onto the same fallback"
        );

        let neutral = analyze_material("嗯嗯好的").expect("analysis must succeed");
        let words: Vec<&str> = neutral.hits.iter().map(|hit| hit.word.as_str()).collect();
        assert_eq!(words, vec!["嗯", "好的"], "the neutral entries are hits");
        assert_eq!(
            components(&Lexicon::to_emotion_result(&neutral)),
            [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0],
            "a genuine neutral hit lands on the same value as no hit"
        );
    }

    /// CP-B3-C2: two calls are two analyses — the shared entry caches nothing and carries no state
    /// between calls, so a material analysed twice yields the same suggestion twice and an empty
    /// material is a normal result rather than an error.
    #[test]
    fn cp_b3_c2_shared_entry_is_stateless_and_empty_material_is_ok() {
        let first = analyze_material("我很开心").expect("analysis must succeed");
        let second = analyze_material("我很开心").expect("analysis must succeed");
        assert_eq!(
            components(&Lexicon::to_emotion_result(&first)),
            components(&Lexicon::to_emotion_result(&second))
        );
        assert_eq!(first.hits.len(), second.hits.len());

        let empty = analyze_material("").expect("an empty material is a normal result");
        assert!(empty.hits.is_empty());
        assert_eq!(
            components(&Lexicon::to_emotion_result(&empty)),
            [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0]
        );

        // A second, different material in between does not leak into the next analysis.
        let _ = analyze_material("我很难过").expect("analysis must succeed");
        let third = analyze_material("我很开心").expect("analysis must succeed");
        assert_eq!(
            components(&Lexicon::to_emotion_result(&third)),
            [1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]
        );
    }
}
