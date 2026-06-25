//! Per-turn Fast/Deep thinking mode for co-present latency (distro `HostProfile` + auto router).

use crate::domain::host_profile::{HostProfile, TurnThinkingDefault};
use oclive_kernel_runtime::domain::emotion_analyzer::EmotionResult;
use crate::models::{Event, EventType};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TurnThinkingMode {
    Fast,
    Deep,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TurnThinkingReason {
    ForcedDefault,
    AutoCasual,
    AutoEmotion,
    AutoEventChain,
    AutoLongMessage,
    AutoDeepKeyword,
}

#[derive(Debug, Clone)]
pub struct TurnThinkingPlan {
    pub mode: TurnThinkingMode,
    pub reasons: Vec<TurnThinkingReason>,
}

impl TurnThinkingPlan {
    #[must_use]
    pub fn use_event_impact_llm(&self, host: &HostProfile) -> bool {
        self.mode == TurnThinkingMode::Deep && host.event_impact_llm
    }

    #[must_use]
    pub fn skip_complex_emotion(&self, host: &HostProfile) -> bool {
        host.skip_complex_emotion
            || (self.mode == TurnThinkingMode::Fast && host.turn_thinking.fast_skip_complex_emotion)
    }

    #[must_use]
    pub fn use_concise_prompt(&self, host: &HostProfile) -> bool {
        self.mode == TurnThinkingMode::Fast || host.prompt_profile.is_concise()
    }

    #[must_use]
    pub fn knowledge_retrieve_limit(&self, host: &HostProfile) -> usize {
        if self.mode == TurnThinkingMode::Fast {
            host.turn_thinking.fast_knowledge_limit
        } else {
            8
        }
    }

    #[must_use]
    pub fn memory_cap(&self, host: &HostProfile) -> usize {
        if self.mode == TurnThinkingMode::Fast {
            host.turn_thinking
                .fast_memory_cap
                .min(host.memory_retrieval.retrieval_limit())
        } else {
            host.memory_retrieval.retrieval_limit()
        }
    }
}

#[must_use]
pub fn resolve_turn_thinking(
    host: &HostProfile,
    user_message: &str,
    emotion: &EmotionResult,
    recent_events: &[Event],
) -> TurnThinkingPlan {
    match host.turn_thinking.default {
        TurnThinkingDefault::Fast => TurnThinkingPlan {
            mode: TurnThinkingMode::Fast,
            reasons: vec![TurnThinkingReason::ForcedDefault],
        },
        TurnThinkingDefault::Deep => TurnThinkingPlan {
            mode: TurnThinkingMode::Deep,
            reasons: vec![TurnThinkingReason::ForcedDefault],
        },
        TurnThinkingDefault::Auto => resolve_auto(host, user_message, emotion, recent_events),
    }
}

fn resolve_auto(
    host: &HostProfile,
    user_message: &str,
    emotion: &EmotionResult,
    recent_events: &[Event],
) -> TurnThinkingPlan {
    let mut reasons = Vec::new();
    if user_message.chars().count() >= host.turn_thinking.auto_deep_min_chars {
        reasons.push(TurnThinkingReason::AutoLongMessage);
    }
    if emotion_high_arousal(emotion) {
        reasons.push(TurnThinkingReason::AutoEmotion);
    }
    if recent_events
        .iter()
        .any(|e| matches!(e.event_type, EventType::Quarrel))
    {
        reasons.push(TurnThinkingReason::AutoEventChain);
    }
    if host
        .turn_thinking
        .auto_deep_keywords
        .iter()
        .any(|kw| user_message.contains(kw.as_str()))
    {
        reasons.push(TurnThinkingReason::AutoDeepKeyword);
    }
    if reasons.is_empty() {
        return TurnThinkingPlan {
            mode: TurnThinkingMode::Fast,
            reasons: vec![TurnThinkingReason::AutoCasual],
        };
    }
    TurnThinkingPlan {
        mode: TurnThinkingMode::Deep,
        reasons,
    }
}

fn emotion_high_arousal(er: &EmotionResult) -> bool {
    er.anger >= 0.45 || er.sadness >= 0.45 || er.fear >= 0.4
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::host_profile::{HostProfile, TurnThinkingDefault, TurnThinkingProfile};

    fn test_host(default: TurnThinkingDefault) -> HostProfile {
        HostProfile {
            turn_thinking: TurnThinkingProfile {
                default,
                ..TurnThinkingProfile::default()
            },
            ..HostProfile::default()
        }
    }

    fn neutral_emotion() -> EmotionResult {
        EmotionResult {
            joy: 0.1,
            sadness: 0.1,
            anger: 0.05,
            fear: 0.05,
            surprise: 0.1,
            disgust: 0.05,
            neutral: 0.6,
            extension: None,
        }
    }

    #[test]
    fn auto_casual_short_message_is_fast() {
        let plan = resolve_turn_thinking(
            &test_host(TurnThinkingDefault::Auto),
            "你好",
            &neutral_emotion(),
            &[],
        );
        assert_eq!(plan.mode, TurnThinkingMode::Fast);
    }

    #[test]
    fn auto_long_message_is_deep() {
        let msg = "a".repeat(100);
        let plan = resolve_turn_thinking(
            &test_host(TurnThinkingDefault::Auto),
            msg.as_str(),
            &neutral_emotion(),
            &[],
        );
        assert_eq!(plan.mode, TurnThinkingMode::Deep);
    }

    #[test]
    fn fast_plan_skips_event_llm_even_when_host_allows() {
        let mut host = test_host(TurnThinkingDefault::Fast);
        host.event_impact_llm = true;
        let plan = resolve_turn_thinking(&host, "hi", &neutral_emotion(), &[]);
        assert!(!plan.use_event_impact_llm(&host));
    }
}
