//! 关键词七维情绪分析（与历史 `kernel_runtime::domain::emotion_analyzer` 行为一致）。

use oclive_kernel_core::error::Result;
use oclive_kernel_core::models::{Emotion, EmotionResult};

/// 情绪分析器
pub struct EmotionAnalyzer;

impl EmotionAnalyzer {
    /// 分析文本情绪
    pub fn analyze(text: &str) -> Result<EmotionResult> {
        if text.is_empty() {
            return Ok(EmotionResult::strong_neutral());
        }

        let mut result = EmotionResult {
            joy: 0.0,
            sadness: 0.0,
            anger: 0.0,
            fear: 0.0,
            surprise: 0.0,
            disgust: 0.0,
            neutral: 0.0,
        };

        let text_lower = text.to_lowercase();
        let padded_en = format!(" {text_lower} ");

        let joy_keywords = [
            "开心",
            "高兴",
            "太好了",
            "太棒",
            "棒",
            "爱",
            "喜欢",
            "开颜",
            "哈哈",
            "hhh",
            "感谢",
            "谢谢",
            "感激",
            "期待",
            "想见",
            "抱抱",
            "mua",
            "么么",
        ];
        for keyword in &joy_keywords {
            if text_lower.contains(keyword) {
                result.joy += 0.2;
            }
        }
        let joy_en = [
            " happy ",
            " glad ",
            " joy ",
            " thanks ",
            " thank you ",
            " love you ",
            " lol ",
            " haha ",
            " great ",
            " nice ",
            " awesome ",
        ];
        for keyword in &joy_en {
            if padded_en.contains(keyword) {
                result.joy += 0.2;
            }
        }

        let sadness_keywords = [
            "难过",
            "伤心",
            "哭",
            "悲伤",
            "失望",
            "沮丧",
            "委屈",
            "好累",
            "疲惫",
            "心累",
            "崩溃",
            "绝望",
            "孤单",
            "寂寞",
            "想死",
            "没意思",
        ];
        for keyword in &sadness_keywords {
            if text_lower.contains(keyword) {
                result.sadness += 0.2;
            }
        }
        let sadness_en = [
            " sad ",
            " depressed ",
            " tired ",
            " lonely ",
            " upset ",
            " crying ",
        ];
        for keyword in &sadness_en {
            if padded_en.contains(keyword) {
                result.sadness += 0.2;
            }
        }

        let anger_keywords = [
            "生气",
            "愤怒",
            "讨厌",
            "烦死了",
            "烦",
            "气死",
            "恨",
            "滚",
            "闭嘴",
            "无语",
            "服了",
            "凭什么",
            "有病",
        ];
        for keyword in &anger_keywords {
            if text_lower.contains(keyword) {
                result.anger += 0.2;
            }
        }
        let anger_en = [" angry ", " hate ", " annoyed ", " pissed ", " wtf "];
        for keyword in &anger_en {
            if padded_en.contains(keyword) {
                result.anger += 0.2;
            }
        }

        let fear_keywords = ["害怕", "恐惧", "担心", "紧张", "焦虑", "慌", "不安", "吓人"];
        for keyword in &fear_keywords {
            if text_lower.contains(keyword) {
                result.fear += 0.2;
            }
        }
        let fear_en = [
            " afraid ",
            " scared ",
            " fear ",
            " worried ",
            " anxious ",
            " nervous ",
        ];
        for keyword in &fear_en {
            if padded_en.contains(keyword) {
                result.fear += 0.2;
            }
        }

        let surprise_keywords = [
            "惊讶",
            "意外",
            "哇",
            "天哪",
            "没想到",
            "吓一跳",
            "居然",
            "真的假的",
            "诶",
        ];
        for keyword in &surprise_keywords {
            if text_lower.contains(keyword) {
                result.surprise += 0.2;
            }
        }
        let surprise_en = [" wow ", " omg ", " surprised ", " unbelievable "];
        for keyword in &surprise_en {
            if padded_en.contains(keyword) {
                result.surprise += 0.2;
            }
        }

        let disgust_keywords = ["厌恶", "恶心", "反感", "厌烦", "作呕"];
        for keyword in &disgust_keywords {
            if text_lower.contains(keyword) {
                result.disgust += 0.2;
            }
        }
        let disgust_en = [" disgusting ", " gross ", " sick of "];
        for keyword in &disgust_en {
            if padded_en.contains(keyword) {
                result.disgust += 0.2;
            }
        }

        let total: f64 = result.joy
            + result.sadness
            + result.anger
            + result.fear
            + result.surprise
            + result.disgust;

        if total > 0.0 {
            result.joy /= total;
            result.sadness /= total;
            result.anger /= total;
            result.fear /= total;
            result.surprise /= total;
            result.disgust /= total;
        } else {
            result.neutral = 1.0;
        }

        Ok(result)
    }

    /// 获取主导情绪
    #[must_use]
    pub fn get_dominant_emotion(result: &EmotionResult) -> Emotion {
        let mut max_emotion = Emotion::Neutral;
        let mut max_value = result.neutral;

        if result.joy > max_value {
            max_value = result.joy;
            max_emotion = Emotion::Happy;
        }
        if result.sadness > max_value {
            max_value = result.sadness;
            max_emotion = Emotion::Sad;
        }
        if result.disgust > max_value {
            max_value = result.disgust;
            max_emotion = Emotion::Angry;
        }
        if result.anger > max_value {
            max_value = result.anger;
            max_emotion = Emotion::Angry;
        }
        if result.surprise > max_value {
            max_value = result.surprise;
            max_emotion = Emotion::Excited;
        }
        if result.fear > max_value {
            max_emotion = Emotion::Confused;
        }

        max_emotion
    }

    /// 计算情绪强度（0~1）
    #[must_use]
    pub fn calculate_intensity(result: &EmotionResult) -> f64 {
        let max_emotion = Self::get_dominant_emotion(result);
        match max_emotion {
            Emotion::Happy => result.joy,
            Emotion::Sad => result.sadness,
            Emotion::Angry => result.anger.max(result.disgust),
            Emotion::Excited => result.surprise,
            Emotion::Confused => result.fear,
            Emotion::Shy => result.fear,
            Emotion::Neutral => result.neutral,
        }
    }

    /// 格式化情绪结果用于 prompt
    #[must_use]
    pub fn format_for_prompt(result: &EmotionResult) -> String {
        let dominant = Self::get_dominant_emotion(result);
        let intensity = Self::calculate_intensity(result);
        format!(
            "[emotion dominant=\"{}\" intensity=\"{:.2}\"]",
            dominant, intensity
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_text() {
        let result = EmotionAnalyzer::analyze("").unwrap();
        assert_eq!(result.neutral, 1.0);
    }

    #[test]
    fn test_analyze_happy() {
        let result = EmotionAnalyzer::analyze("我很开心").unwrap();
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
    fn test_analyze_thanks_joy() {
        let result = EmotionAnalyzer::analyze("谢谢你").unwrap();
        assert!(result.joy > 0.0);
    }

    #[test]
    fn test_get_dominant_emotion() {
        let result = EmotionResult {
            joy: 0.9,
            sadness: 0.0,
            anger: 0.0,
            fear: 0.0,
            surprise: 0.0,
            disgust: 0.0,
            neutral: 0.1,
        };
        assert_eq!(
            EmotionAnalyzer::get_dominant_emotion(&result),
            Emotion::Happy
        );
    }

    #[test]
    fn test_calculate_intensity_happy() {
        let result = EmotionAnalyzer::analyze("我很开心").unwrap();
        assert!(EmotionAnalyzer::calculate_intensity(&result) > 0.0);
    }

    #[test]
    fn test_calculate_intensity_neutral() {
        let result = EmotionAnalyzer::analyze("....").unwrap();
        assert!(EmotionAnalyzer::calculate_intensity(&result) >= 0.0);
    }

    #[test]
    fn test_format_for_prompt_includes_tag() {
        let result = EmotionAnalyzer::analyze("我很开心").unwrap();
        let s = EmotionAnalyzer::format_for_prompt(&result);
        assert!(s.contains("[emotion"));
    }
}
