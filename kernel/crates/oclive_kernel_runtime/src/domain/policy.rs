use crate::domain::emotion_analyzer::EmotionResult;
use crate::domain::event_detector::EventDetector;
use crate::error::Result;
use crate::models::{Emotion, Event, EventType};
pub use oclive_kernel_types::{
    EmotionPolicyConfig, MemoryPolicyConfig, PolicyConfig, PolicyContext,
};

pub use oclive_kernel_contracts::{EmotionPolicy, EventPolicy, MemoryPolicy};

pub struct DefaultEmotionPolicy;

impl DefaultEmotionPolicy {
    #[must_use]
    pub fn new(_config: EmotionPolicyConfig) -> Self {
        Self
    }
}

impl EmotionPolicy for DefaultEmotionPolicy {
    fn resolve_current_emotion(
        &self,
        _previous: Option<&str>,
        analyzed: &EmotionResult,
    ) -> Emotion {
        // B M1 slice 2: the main LLM is the sole arbiter of complex emotion;
        // hold / low-confidence logic removed (v1.5 §11.2). Degraded turns
        // keep the previous emotion at the call site (post_llm), not here.
        analyzed.to_emotion()
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
    #[must_use]
    pub fn new(config: MemoryPolicyConfig) -> Self {
        Self { config }
    }
}

impl MemoryPolicy for DefaultMemoryPolicy {
    fn build_memory_entry(&self, ctx: &PolicyContext<'_>) -> String {
        // Chat storage already owns the verbatim transcript. Long-term memory
        // must contain user-side facts/preferences, not an assistant answer
        // that can be retrieved later and replayed as if it were a new reply.
        format!("用户曾表达：{}", ctx.user_message.trim())
    }

    fn should_persist(&self, ctx: &PolicyContext<'_>) -> bool {
        if !self.config.ignore_single_char_filter {
            return true;
        }
        let message = ctx.user_message.trim();
        if message.chars().count() <= 1 {
            return false;
        }

        // Relationship-changing events are worth consolidating even when the
        // wording is short. Ordinary questions and task instructions remain in
        // chat history instead of polluting cross-turn memory.
        if matches!(
            ctx.event.event_type,
            EventType::Quarrel
                | EventType::Apology
                | EventType::Praise
                | EventType::Complaint
                | EventType::Confession
        ) {
            return true;
        }

        if message.contains('？') || message.contains('?') {
            return false;
        }
        const USER_FACT_SIGNALS: &[&str] = &[
            "我叫",
            "叫我",
            "我的名字",
            "我是",
            "我住在",
            "我来自",
            "我的工作",
            "我喜欢",
            "我不喜欢",
            "我讨厌",
            "我偏好",
            "我希望",
            "我想要",
            "我不想",
            "我打算",
            "我计划",
            "我决定",
            "请记住",
            "记住我",
            "别忘",
            "以后不要",
            "以后请",
            "请叫我",
        ];
        USER_FACT_SIGNALS
            .iter()
            .any(|signal| message.contains(signal))
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
            extension: None,
        }
    }

    #[test]
    fn default_emotion_policy_returns_analyzed_directly() {
        let policy = DefaultEmotionPolicy::new(EmotionPolicyConfig::default());
        // Previous value no longer participates: neutral_result() wins even
        // though the previous displayed emotion was happy (B M1 slice 2).
        let resolved = policy.resolve_current_emotion(Some("happy"), &neutral_result());
        assert_eq!(resolved, Emotion::Neutral);
    }

    #[test]
    fn default_emotion_policy_ignores_previous_when_analyzed_differs() {
        let policy = DefaultEmotionPolicy::new(EmotionPolicyConfig::default());
        let mut result = neutral_result();
        result.joy = 0.9;
        result.neutral = 0.1;
        assert_eq!(
            policy.resolve_current_emotion(Some("sad"), &result),
            Emotion::Happy
        );
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
                false,
            ),
            (
                MemoryPolicyConfig::default(),
                "我不喜欢每轮都被追问",
                EventType::Ignore,
                0.35_f32,
                true,
            ),
            (
                MemoryPolicyConfig::default(),
                "我叫什么？",
                EventType::Ignore,
                0.35_f32,
                false,
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
    fn memory_entry_keeps_user_fact_but_not_assistant_transcript() {
        let policy = DefaultMemoryPolicy::new(MemoryPolicyConfig::default());
        let event = Event {
            event_type: EventType::Ignore,
            user_emotion: "neutral".to_string(),
            bot_emotion: "neutral".to_string(),
        };
        let ctx = PolicyContext {
            role_id: "mumu",
            user_message: "我喜欢简洁回答",
            reply: "好的，我会记住。",
            event: &event,
            event_confidence: 0.35,
        };
        let entry = policy.build_memory_entry(&ctx);
        assert_eq!(entry, "用户曾表达：我喜欢简洁回答");
        assert!(!entry.contains("好的"));
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
