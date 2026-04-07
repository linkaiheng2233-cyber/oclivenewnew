//! 事件检测模块
//!
//! 基于文本和情绪的事件类型检测

use crate::error::Result;
use crate::models::knowledge::KnowledgeEventAugment;
use crate::models::{Emotion, Event, EventType};

/// 事件检测器
pub struct EventDetector;

impl EventDetector {
    /// 检测事件类型
    pub fn detect(text: &str, user_emotion: &Emotion, bot_emotion: &Emotion) -> Result<Event> {
        Self::detect_with_augment(text, user_emotion, bot_emotion, None)
    }

    /// 与 [`detect`] 相同，但合并世界观知识块提供的额外关键词（B1）。
    pub fn detect_with_augment(
        text: &str,
        user_emotion: &Emotion,
        bot_emotion: &Emotion,
        augment: Option<&KnowledgeEventAugment>,
    ) -> Result<Event> {
        let event_type = Self::classify_event(text, user_emotion, bot_emotion, augment);

        Ok(Event {
            event_type,
            user_emotion: user_emotion.to_string(),
            bot_emotion: bot_emotion.to_string(),
        })
    }

    fn classify_event(
        text: &str,
        user_emotion: &Emotion,
        bot_emotion: &Emotion,
        augment: Option<&KnowledgeEventAugment>,
    ) -> EventType {
        let text_lower = text.to_lowercase();
        let ex = |et: EventType| {
            augment
                .and_then(|a| a.by_event.get(&et))
                .map(|v| v.as_slice())
        };

        if (user_emotion == &Emotion::Angry || user_emotion == &Emotion::Excited)
            && (bot_emotion == &Emotion::Angry || bot_emotion == &Emotion::Confused)
            && Self::contains_quarrel_keywords(&text_lower, ex(EventType::Quarrel))
        {
            return EventType::Quarrel;
        }

        if (user_emotion == &Emotion::Sad || user_emotion == &Emotion::Shy)
            && (bot_emotion == &Emotion::Happy || bot_emotion == &Emotion::Neutral)
            && Self::contains_apology_keywords(&text_lower, ex(EventType::Apology))
        {
            return EventType::Apology;
        }

        if user_emotion == &Emotion::Happy
            && bot_emotion == &Emotion::Happy
            && Self::contains_praise_keywords(&text_lower, ex(EventType::Praise))
        {
            return EventType::Praise;
        }

        if (user_emotion == &Emotion::Sad || user_emotion == &Emotion::Angry)
            && Self::contains_complaint_keywords(&text_lower, ex(EventType::Complaint))
        {
            return EventType::Complaint;
        }

        if (user_emotion == &Emotion::Excited || user_emotion == &Emotion::Shy)
            && bot_emotion == &Emotion::Happy
            && Self::contains_confession_keywords(&text_lower, ex(EventType::Confession))
        {
            return EventType::Confession;
        }

        if (user_emotion == &Emotion::Happy || user_emotion == &Emotion::Excited)
            && (bot_emotion == &Emotion::Happy || bot_emotion == &Emotion::Excited)
            && Self::contains_joke_keywords(&text_lower, ex(EventType::Joke))
        {
            return EventType::Joke;
        }

        if (user_emotion == &Emotion::Neutral || user_emotion == &Emotion::Confused)
            && bot_emotion == &Emotion::Neutral
            && Self::contains_ignore_keywords(&text_lower, ex(EventType::Ignore))
        {
            return EventType::Ignore;
        }

        EventType::Ignore
    }

    fn contains_quarrel_keywords(text: &str, extra: Option<&[String]>) -> bool {
        let keywords = ["吵", "架", "生气", "讨厌", "烦", "滚", "别理我", "气死"];
        Self::matches_keywords(text, &keywords, extra)
    }

    fn contains_apology_keywords(text: &str, extra: Option<&[String]>) -> bool {
        let keywords = ["对不起", "抱歉", "道歉", "原谅", "错了", "我错了"];
        Self::matches_keywords(text, &keywords, extra)
    }

    fn contains_praise_keywords(text: &str, extra: Option<&[String]>) -> bool {
        let keywords = ["棒", "厉害", "聪明", "漂亮", "优秀", "最好"];
        Self::matches_keywords(text, &keywords, extra)
    }

    fn contains_complaint_keywords(text: &str, extra: Option<&[String]>) -> bool {
        let keywords = ["抱怨", "不满", "不开心", "难受", "委屈", "伤心"];
        Self::matches_keywords(text, &keywords, extra)
    }

    fn contains_confession_keywords(text: &str, extra: Option<&[String]>) -> bool {
        let keywords = ["喜欢", "爱", "表白", "心动", "倾心", "迷恋"];
        Self::matches_keywords(text, &keywords, extra)
    }

    fn contains_joke_keywords(text: &str, extra: Option<&[String]>) -> bool {
        let keywords = ["哈哈", "哈", "笑", "逗", "玩笑", "开玩笑"];
        Self::matches_keywords(text, &keywords, extra)
    }

    fn contains_ignore_keywords(text: &str, extra: Option<&[String]>) -> bool {
        let keywords = ["嗯", "哦", "好吧", "随便", "无所谓"];
        Self::matches_keywords(text, &keywords, extra)
    }

    fn matches_keywords(text: &str, base: &[&str], extra: Option<&[String]>) -> bool {
        if base.iter().any(|k| text.contains(k)) {
            return true;
        }
        extra
            .map(|xs| xs.iter().any(|k| text.contains(k.as_str())))
            .unwrap_or(false)
    }

    /// 获取事件影响因子（用于性格演化）
    pub fn get_impact_factor(event_type: &EventType) -> f64 {
        match event_type {
            EventType::Quarrel => -0.8,
            EventType::Apology => 0.6,
            EventType::Praise => 0.9,
            EventType::Complaint => -0.5,
            EventType::Confession => 0.8,
            EventType::Joke => 0.7,
            EventType::Ignore => 0.0,
        }
    }

    /// 事件置信度（用于前端展示/日志分析）
    pub fn get_confidence(event_type: &EventType) -> f32 {
        match event_type {
            EventType::Quarrel => 0.92,
            EventType::Apology => 0.88,
            EventType::Praise => 0.9,
            EventType::Complaint => 0.82,
            EventType::Confession => 0.86,
            EventType::Joke => 0.84,
            EventType::Ignore => 0.35,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_detect_quarrel() {
        let event =
            EventDetector::detect("你太坏了，我生气了", &Emotion::Angry, &Emotion::Angry).unwrap();
        assert_eq!(event.event_type, EventType::Quarrel);
    }

    #[test]
    fn augment_can_trigger_quarrel_with_custom_keyword() {
        let mut by_event = HashMap::new();
        by_event.insert(EventType::Quarrel, vec!["决裂".to_string()]);
        let aug = KnowledgeEventAugment { by_event };
        let event = EventDetector::detect_with_augment(
            "我们决裂吧",
            &Emotion::Angry,
            &Emotion::Angry,
            Some(&aug),
        )
        .unwrap();
        assert_eq!(event.event_type, EventType::Quarrel);
    }

    #[test]
    fn test_detect_apology() {
        let event = EventDetector::detect("对不起", &Emotion::Sad, &Emotion::Happy).unwrap();
        assert_eq!(event.event_type, EventType::Apology);
    }

    #[test]
    fn test_detect_praise() {
        let event = EventDetector::detect("你真棒", &Emotion::Happy, &Emotion::Happy).unwrap();
        assert_eq!(event.event_type, EventType::Praise);
    }

    #[test]
    fn test_detect_complaint() {
        let event = EventDetector::detect("我很难受", &Emotion::Sad, &Emotion::Neutral).unwrap();
        assert_eq!(event.event_type, EventType::Complaint);
    }

    #[test]
    fn test_detect_confession() {
        let event = EventDetector::detect("我喜欢你", &Emotion::Excited, &Emotion::Happy).unwrap();
        assert_eq!(event.event_type, EventType::Confession);
    }

    #[test]
    fn test_detect_joke() {
        let event = EventDetector::detect("哈哈", &Emotion::Happy, &Emotion::Happy).unwrap();
        assert_eq!(event.event_type, EventType::Joke);
    }

    #[test]
    fn test_get_impact_factor_positive() {
        let factor = EventDetector::get_impact_factor(&EventType::Praise);
        assert_eq!(factor, 0.9);
    }

    #[test]
    fn test_get_impact_factor_negative() {
        let factor = EventDetector::get_impact_factor(&EventType::Quarrel);
        assert_eq!(factor, -0.8);
    }
}
