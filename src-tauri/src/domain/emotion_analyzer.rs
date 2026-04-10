//! 情绪分析模块
//!
//! 基于关键词匹配的7维度情绪分析

use crate::error::Result;
use crate::models::Emotion;

/// 情绪分析结果
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EmotionResult {
    pub joy: f64,
    pub sadness: f64,
    pub anger: f64,
    pub fear: f64,
    pub surprise: f64,
    pub disgust: f64,
    pub neutral: f64,
}

impl EmotionResult {
    /// 转为 `models::Emotion`（与 `EmotionAnalyzer::get_dominant_emotion` 一致）
    pub fn to_emotion(&self) -> Emotion {
        EmotionAnalyzer::get_dominant_emotion(self)
    }
}

/// 情绪分析器
pub struct EmotionAnalyzer;

impl EmotionAnalyzer {
    /// 分析文本情绪
    ///
    /// # Arguments
    /// * `text` - 输入文本
    ///
    /// # Returns
    /// 情绪分析结果
    ///
    /// # Examples
    /// ```
    /// # use oclivenewnew_tauri::domain::emotion_analyzer::EmotionAnalyzer;
    /// let result = EmotionAnalyzer::analyze("我很开心").unwrap();
    /// assert!(result.joy > 0.0);
    /// ```
    pub fn analyze(text: &str) -> Result<EmotionResult> {
        let mut result = EmotionResult {
            joy: 0.0,
            sadness: 0.0,
            anger: 0.0,
            fear: 0.0,
            surprise: 0.0,
            disgust: 0.0,
            neutral: 0.0,
        };

        if text.is_empty() {
            result.neutral = 1.0;
            return Ok(result);
        }

        let text_lower = text.to_lowercase();
        // 英文 token 用首尾空格包裹，避免 glove→love、made→mad 等子串误匹配
        let padded_en = format!(" {text_lower} ");

        // 快乐关键词（中 + 常见英/网语，便于日常聊天识别）
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

        // 悲伤关键词
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

        // 愤怒关键词（「讨厌」以愤怒通道为主，避免与厌恶双计）
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

        // 恐惧 / 焦虑
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

        // 惊讶
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

        // 厌恶（不与「讨厌」重复）
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

        // 归一化
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
    ///
    /// # Arguments
    /// * `result` - 情绪分析结果
    ///
    /// # Returns
    /// 主导情绪类型
    pub fn get_dominant_emotion(result: &EmotionResult) -> Emotion {
        let emotions = [
            (result.joy, Emotion::Happy),
            (result.sadness, Emotion::Sad),
            (result.anger, Emotion::Angry),
            (result.fear, Emotion::Confused),
            (result.surprise, Emotion::Excited),
            (result.disgust, Emotion::Confused),
        ];

        let best = emotions
            .iter()
            .max_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(s, e)| (*s, e.clone()))
            .unwrap_or((0.0, Emotion::Neutral));
        if result.neutral > best.0 {
            Emotion::Neutral
        } else {
            best.1
        }
    }

    /// 非中性情绪维度的最大分量（归一化后，与 `neutral` 互补）
    fn max_affective(result: &EmotionResult) -> f64 {
        result
            .joy
            .max(result.sadness)
            .max(result.anger)
            .max(result.fear)
            .max(result.surprise)
            .max(result.disgust)
    }

    /// 写入主对话 prompt 的一行中文语气线索（含内部标签便于调试与插件对齐）。
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

    /// 计算情绪强度
    ///
    /// # Arguments
    /// * `emotion` - 情绪类型
    ///
    /// # Returns
    /// 情绪强度 [0.0, 1.0]
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
    fn test_normalization() {
        let result = EmotionAnalyzer::analyze("开心高兴").unwrap();
        let sum = result.joy
            + result.sadness
            + result.anger
            + result.fear
            + result.surprise
            + result.disgust
            + result.neutral;
        assert!((sum - 1.0).abs() < 0.01);
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
