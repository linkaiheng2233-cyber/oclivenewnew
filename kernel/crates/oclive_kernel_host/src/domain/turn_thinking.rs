//! Per-turn Fast/Deep thinking mode for co-present latency (distro `HostProfile` + auto router).

use crate::domain::host_profile::{
    fast_persistence_effective, FastPersistenceMode, HostProfile, TurnThinkingDefault,
};
use crate::models::{Event, EventType};
use oclive_kernel_runtime::domain::emotion_analyzer::EmotionResult;

/// Strong dialogue events that still consolidate favor / long-term memory on Fast + `strong_only`.
pub const STRONG_PERSISTENCE_EVENTS: [EventType; 4] = [
    EventType::Quarrel,
    EventType::Apology,
    EventType::Confession,
    EventType::Praise,
];

#[must_use]
pub fn is_strong_persistence_event(event: EventType) -> bool {
    STRONG_PERSISTENCE_EVENTS.contains(&event)
}

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

    #[must_use]
    pub fn applies_full_persistence(&self, host: &HostProfile, event: &EventType) -> bool {
        if self.mode == TurnThinkingMode::Deep {
            return true;
        }
        match fast_persistence_effective(host) {
            FastPersistenceMode::Legacy => true,
            FastPersistenceMode::StrongOnly => is_strong_persistence_event(*event),
        }
    }

    #[must_use]
    pub fn favor_delta_scale(&self, host: &HostProfile, event: &EventType) -> f64 {
        if self.applies_full_persistence(host, event) {
            1.0
        } else {
            0.0
        }
    }

    #[must_use]
    pub fn memory_importance_after_policy(
        &self,
        host: &HostProfile,
        event: &EventType,
        raw: f64,
    ) -> f64 {
        if self.applies_full_persistence(host, event) {
            raw
        } else {
            0.0
        }
    }

    #[must_use]
    pub fn skip_mutable_profile_evolution(&self, host: &HostProfile, event: &EventType) -> bool {
        !self.applies_full_persistence(host, event)
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
    use crate::domain::host_profile::{
        FastPersistenceMode, HostProfile, TurnThinkingDefault, TurnThinkingProfile,
    };

    fn test_host(default: TurnThinkingDefault) -> HostProfile {
        HostProfile {
            turn_thinking: TurnThinkingProfile {
                default,
                ..TurnThinkingProfile::default()
            },
            ..HostProfile::default()
        }
    }

    fn host_with_fast_persistence(mode: FastPersistenceMode) -> HostProfile {
        HostProfile {
            turn_thinking: TurnThinkingProfile {
                fast_persistence: mode,
                ..TurnThinkingProfile::default()
            },
            ..HostProfile::default()
        }
    }

    fn fast_plan() -> TurnThinkingPlan {
        TurnThinkingPlan {
            mode: TurnThinkingMode::Fast,
            reasons: vec![TurnThinkingReason::AutoCasual],
        }
    }

    fn deep_plan() -> TurnThinkingPlan {
        TurnThinkingPlan {
            mode: TurnThinkingMode::Deep,
            reasons: vec![TurnThinkingReason::AutoLongMessage],
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

    #[test]
    fn strong_only_fast_ignore_skips_persistence() {
        let host = host_with_fast_persistence(FastPersistenceMode::StrongOnly);
        let plan = fast_plan();
        assert!(!plan.applies_full_persistence(&host, &EventType::Ignore));
        assert_eq!(plan.favor_delta_scale(&host, &EventType::Ignore), 0.0);
        assert_eq!(
            plan.memory_importance_after_policy(&host, &EventType::Ignore, 0.8),
            0.0
        );
        assert!(plan.skip_mutable_profile_evolution(&host, &EventType::Ignore));
    }

    #[test]
    fn strong_only_fast_quarrel_persists() {
        let host = host_with_fast_persistence(FastPersistenceMode::StrongOnly);
        let plan = fast_plan();
        assert!(plan.applies_full_persistence(&host, &EventType::Quarrel));
        assert_eq!(plan.favor_delta_scale(&host, &EventType::Quarrel), 1.0);
        assert_eq!(
            plan.memory_importance_after_policy(&host, &EventType::Quarrel, 0.8),
            0.8
        );
        assert!(!plan.skip_mutable_profile_evolution(&host, &EventType::Quarrel));
    }

    #[test]
    fn deep_ignore_still_persists() {
        let host = host_with_fast_persistence(FastPersistenceMode::StrongOnly);
        let plan = deep_plan();
        assert!(plan.applies_full_persistence(&host, &EventType::Ignore));
        assert_eq!(plan.favor_delta_scale(&host, &EventType::Ignore), 1.0);
    }

    #[test]
    fn legacy_fast_ignore_still_persists() {
        let host = host_with_fast_persistence(FastPersistenceMode::Legacy);
        let plan = fast_plan();
        assert!(plan.applies_full_persistence(&host, &EventType::Ignore));
        assert_eq!(
            plan.memory_importance_after_policy(&host, &EventType::Ignore, 0.5),
            0.5
        );
    }

    #[test]
    fn strong_persistence_events_match_rfc_whitelist() {
        assert!(is_strong_persistence_event(EventType::Quarrel));
        assert!(is_strong_persistence_event(EventType::Apology));
        assert!(is_strong_persistence_event(EventType::Confession));
        assert!(is_strong_persistence_event(EventType::Praise));
        assert!(!is_strong_persistence_event(EventType::Ignore));
        assert!(!is_strong_persistence_event(EventType::Joke));
        assert!(!is_strong_persistence_event(EventType::Complaint));
    }
}
