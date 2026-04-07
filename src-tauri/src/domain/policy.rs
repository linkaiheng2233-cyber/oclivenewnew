use crate::domain::emotion_analyzer::EmotionResult;
use crate::domain::event_detector::EventDetector;
use crate::error::Result;
use crate::models::{Emotion, Event, EventType};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct EmotionPolicyConfig {
    pub neutral_hold_enabled: bool,
    pub low_confidence_hold_threshold: f64,
}

impl Default for EmotionPolicyConfig {
    fn default() -> Self {
        Self {
            neutral_hold_enabled: true,
            low_confidence_hold_threshold: 0.6,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct MemoryPolicyConfig {
    pub ignore_single_char_filter: bool,
    pub default_importance: f64,
    pub fifo_limit: i32,
}

impl Default for MemoryPolicyConfig {
    fn default() -> Self {
        Self {
            ignore_single_char_filter: true,
            default_importance: 0.5,
            fifo_limit: 500,
        }
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct PolicyConfig {
    pub emotion: EmotionPolicyConfig,
    pub memory: MemoryPolicyConfig,
}

pub struct PolicyContext<'a> {
    pub role_id: &'a str,
    pub user_message: &'a str,
    pub reply: &'a str,
    pub event: &'a Event,
    pub event_confidence: f32,
}

pub trait EmotionPolicy: Send + Sync {
    fn resolve_current_emotion(&self, previous: Option<&str>, analyzed: &EmotionResult) -> Emotion;
}

pub trait EventPolicy: Send + Sync {
    fn detect(&self, text: &str, user_emotion: &Emotion, bot_emotion: &Emotion) -> Result<Event>;
    fn impact(&self, event_type: &EventType) -> f64;
    fn confidence(&self, event_type: &EventType) -> f32;
}

pub trait MemoryPolicy: Send + Sync {
    fn build_memory_entry(&self, ctx: &PolicyContext<'_>) -> String;
    fn should_persist(&self, ctx: &PolicyContext<'_>) -> bool;
    fn importance(&self, ctx: &PolicyContext<'_>) -> f64;
    fn fifo_limit(&self) -> i32;
}

pub struct DefaultEmotionPolicy {
    config: EmotionPolicyConfig,
}

impl DefaultEmotionPolicy {
    pub fn new(config: EmotionPolicyConfig) -> Self {
        Self { config }
    }
}

fn parse_emotion(s: &str) -> Option<Emotion> {
    match s {
        "happy" => Some(Emotion::Happy),
        "sad" => Some(Emotion::Sad),
        "angry" => Some(Emotion::Angry),
        "neutral" => Some(Emotion::Neutral),
        "excited" => Some(Emotion::Excited),
        "confused" => Some(Emotion::Confused),
        "shy" => Some(Emotion::Shy),
        _ => None,
    }
}

fn dominant_score(result: &EmotionResult, emotion: &Emotion) -> f64 {
    match emotion {
        Emotion::Happy => result.joy,
        Emotion::Sad => result.sadness,
        Emotion::Angry => result.anger,
        Emotion::Excited => result.surprise,
        Emotion::Confused => result.fear.max(result.disgust),
        Emotion::Shy => result.sadness.min(0.6),
        Emotion::Neutral => result.neutral,
    }
}

impl EmotionPolicy for DefaultEmotionPolicy {
    fn resolve_current_emotion(&self, previous: Option<&str>, analyzed: &EmotionResult) -> Emotion {
        let current = analyzed.to_emotion();
        let Some(prev) = previous.and_then(parse_emotion) else {
            return current;
        };
        if current == prev {
            return current;
        }
        if self.config.neutral_hold_enabled && current == Emotion::Neutral {
            return prev;
        }
        let score = dominant_score(analyzed, &current);
        if score < self.config.low_confidence_hold_threshold {
            prev
        } else {
            current
        }
    }
}

pub struct DefaultEventPolicy;

impl EventPolicy for DefaultEventPolicy {
    fn detect(&self, text: &str, user_emotion: &Emotion, bot_emotion: &Emotion) -> Result<Event> {
        EventDetector::detect(text, user_emotion, bot_emotion)
    }

    fn impact(&self, event_type: &EventType) -> f64 {
        EventDetector::get_impact_factor(event_type)
    }

    fn confidence(&self, event_type: &EventType) -> f32 {
        EventDetector::get_confidence(event_type)
    }
}

pub struct DefaultMemoryPolicy {
    config: MemoryPolicyConfig,
}

impl DefaultMemoryPolicy {
    pub fn new(config: MemoryPolicyConfig) -> Self {
        Self { config }
    }
}

impl MemoryPolicy for DefaultMemoryPolicy {
    fn build_memory_entry(&self, ctx: &PolicyContext<'_>) -> String {
        format!("用户: {}\n助手: {}", ctx.user_message, ctx.reply)
    }

    fn should_persist(&self, ctx: &PolicyContext<'_>) -> bool {
        if !self.config.ignore_single_char_filter {
            return true;
        }
        !(matches!(ctx.event.event_type, EventType::Ignore)
            && ctx.user_message.trim().chars().count() <= 1)
    }

    fn importance(&self, ctx: &PolicyContext<'_>) -> f64 {
        let confidence_boost = (ctx.event_confidence as f64 - 0.5).max(0.0) * 0.2;
        let value = self.config.default_importance + confidence_boost;
        value.clamp(0.0, 1.0)
    }

    fn fifo_limit(&self) -> i32 {
        self.config.fifo_limit
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Event;

    fn neutral_result() -> EmotionResult {
        EmotionResult {
            joy: 0.0,
            sadness: 0.0,
            anger: 0.0,
            fear: 0.0,
            surprise: 0.0,
            disgust: 0.0,
            neutral: 1.0,
        }
    }

    #[test]
    fn default_emotion_policy_keeps_previous_on_neutral() {
        let policy = DefaultEmotionPolicy::new(EmotionPolicyConfig::default());
        let resolved = policy.resolve_current_emotion(Some("happy"), &neutral_result());
        assert_eq!(resolved, Emotion::Happy);
    }

    #[test]
    fn memory_policy_matrix_should_persist() {
        let cases = vec![
            (
                MemoryPolicyConfig {
                    ignore_single_char_filter: true,
                    ..MemoryPolicyConfig::default()
                },
                "?",
                EventType::Ignore,
                0.35_f32,
                false,
            ),
            (
                MemoryPolicyConfig {
                    ignore_single_char_filter: false,
                    ..MemoryPolicyConfig::default()
                },
                "?",
                EventType::Ignore,
                0.35_f32,
                true,
            ),
            (
                MemoryPolicyConfig::default(),
                "你好呀",
                EventType::Ignore,
                0.35_f32,
                true,
            ),
        ];

        for (cfg, user_message, event_type, confidence, expected) in cases {
            let policy = DefaultMemoryPolicy::new(cfg);
            let event = Event {
                event_type,
                user_emotion: "neutral".to_string(),
                bot_emotion: "neutral".to_string(),
            };
            let ctx = PolicyContext {
                role_id: "mumu",
                user_message,
                reply: "ok",
                event: &event,
                event_confidence: confidence,
            };
            assert_eq!(policy.should_persist(&ctx), expected);
        }
    }

    #[test]
    fn memory_policy_matrix_importance_scales_with_confidence() {
        let cfg = MemoryPolicyConfig {
            default_importance: 0.5,
            ..MemoryPolicyConfig::default()
        };
        let policy = DefaultMemoryPolicy::new(cfg);
        let low_event = Event {
            event_type: EventType::Joke,
            user_emotion: "neutral".to_string(),
            bot_emotion: "neutral".to_string(),
        };
        let high_event = Event {
            event_type: EventType::Joke,
            user_emotion: "neutral".to_string(),
            bot_emotion: "neutral".to_string(),
        };
        let low = PolicyContext {
            role_id: "mumu",
            user_message: "hello",
            reply: "ok",
            event: &low_event,
            event_confidence: 0.35,
        };
        let high = PolicyContext {
            role_id: "mumu",
            user_message: "hello",
            reply: "ok",
            event: &high_event,
            event_confidence: 0.92,
        };
        let low_importance = policy.importance(&low);
        let high_importance = policy.importance(&high);
        assert!(high_importance > low_importance);
        assert!((0.0..=1.0).contains(&high_importance));
    }
}
