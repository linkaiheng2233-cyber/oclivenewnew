//! Emotion analysis module.
//!
//! Seven-dimension emotion analysis via the embedded lexicon (JSON) with
//! weighted scoring. The lexicon produces multi-label suggestions; complex
//! emotion arbitration is left to the main LLM (B stage).

use crate::domain::lexicon::Lexicon;
use crate::error::Result;
use crate::models::Emotion;
pub use oclive_kernel_types::EmotionResult;

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
        let lexicon = crate::domain::lexicon::lexicon()?;
        let suggestion = lexicon.analyze(text);
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
