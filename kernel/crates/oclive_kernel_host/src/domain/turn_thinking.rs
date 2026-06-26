//! Per-turn Fast/Deep thinking mode for co-present latency (distro `HostProfile` + auto router).

use crate::domain::host_profile::{
    fast_persistence_effective, FastPersistenceMode, HostProfile, TurnThinkingDefault,
};
use crate::error::Result;
use crate::infrastructure::db::DbManager;
use crate::models::{Event, EventType, Role};
use oclive_kernel_runtime::domain::emotion_analyzer::EmotionResult;
use oclive_kernel_types::{
    TurnThinkingAndGroup, TurnThinkingEphemeralArchiveConfig, TurnThinkingLatchConfig,
    TurnThinkingSignalRule,
};

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
    AutoThisTurnEvent,
    AutoDeepLatch,
    AutoPackAndRule,
    AutoHighSadness,
    AutoHighAnger,
    AutoHighFear,
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

/// Effective routing policy: host default OR rules ++ pack OR; pack AND groups appended.
#[derive(Debug, Clone)]
pub struct TurnThinkingPolicy {
    pub or_rules: Vec<TurnThinkingSignalRule>,
    pub and_groups: Vec<TurnThinkingAndGroup>,
    pub latch: TurnThinkingLatchConfig,
    pub ephemeral: Option<TurnThinkingEphemeralArchiveConfig>,
}

#[derive(Debug)]
pub struct TurnThinkingSignals<'a> {
    pub user_message: &'a str,
    pub emotion: &'a EmotionResult,
    pub recent_events: &'a [Event],
    pub this_turn_event: EventType,
    pub deep_latch_active: bool,
}

#[must_use]
pub fn host_default_or_rules(host: &HostProfile) -> Vec<TurnThinkingSignalRule> {
    vec![
        TurnThinkingSignalRule::LongMessage {
            min_chars: Some(host.turn_thinking.auto_deep_min_chars as u32),
        },
        TurnThinkingSignalRule::HighArousal,
        TurnThinkingSignalRule::RecentEvent {
            events: vec!["Quarrel".into()],
        },
        TurnThinkingSignalRule::Keyword {
            keywords: host.turn_thinking.auto_deep_keywords.clone(),
        },
        TurnThinkingSignalRule::ThisTurnEvent {
            events: vec!["Quarrel".into(), "Confession".into()],
        },
    ]
}

#[must_use]
pub fn effective_turn_thinking_policy(host: &HostProfile, role: &Role) -> TurnThinkingPolicy {
    let mut or_rules = host_default_or_rules(host);
    let mut and_groups = Vec::new();
    let mut latch = TurnThinkingLatchConfig::default();
    let mut ephemeral = None;

    if let Some(cfg) = role.pack_turn_thinking_config.as_ref() {
        or_rules.extend(cfg.deep_when.or.clone());
        and_groups.extend(cfg.deep_when.and.clone());
        latch = cfg.latch.clone();
        ephemeral = cfg.ephemeral_archive.clone();
    }

    TurnThinkingPolicy {
        or_rules,
        and_groups,
        latch,
        ephemeral,
    }
}

fn parse_event_names(names: &[String]) -> Vec<EventType> {
    names.iter().filter_map(|n| parse_event_name(n)).collect()
}

fn parse_event_name(name: &str) -> Option<EventType> {
    match name.trim() {
        "Quarrel" => Some(EventType::Quarrel),
        "Apology" => Some(EventType::Apology),
        "Praise" => Some(EventType::Praise),
        "Complaint" => Some(EventType::Complaint),
        "Confession" => Some(EventType::Confession),
        "Joke" => Some(EventType::Joke),
        "Ignore" => Some(EventType::Ignore),
        _ => None,
    }
}

fn event_list_matches(events: &[String], event: EventType) -> bool {
    events.iter().any(|n| parse_event_name(n) == Some(event))
}

fn reason_for_rule(rule: &TurnThinkingSignalRule) -> TurnThinkingReason {
    match rule {
        TurnThinkingSignalRule::LongMessage { .. } => TurnThinkingReason::AutoLongMessage,
        TurnThinkingSignalRule::HighArousal => TurnThinkingReason::AutoEmotion,
        TurnThinkingSignalRule::HighSadness => TurnThinkingReason::AutoHighSadness,
        TurnThinkingSignalRule::HighAnger => TurnThinkingReason::AutoHighAnger,
        TurnThinkingSignalRule::HighFear => TurnThinkingReason::AutoHighFear,
        TurnThinkingSignalRule::ThisTurnEvent { .. } => TurnThinkingReason::AutoThisTurnEvent,
        TurnThinkingSignalRule::RecentEvent { .. } => TurnThinkingReason::AutoEventChain,
        TurnThinkingSignalRule::Keyword { .. } => TurnThinkingReason::AutoDeepKeyword,
        TurnThinkingSignalRule::DeepLatchActive => TurnThinkingReason::AutoDeepLatch,
    }
}

fn evaluate_or_rule(
    rule: &TurnThinkingSignalRule,
    signals: &TurnThinkingSignals<'_>,
    host: &HostProfile,
) -> bool {
    match rule {
        TurnThinkingSignalRule::LongMessage { min_chars } => {
            let min = min_chars
                .map(|v| v as usize)
                .unwrap_or(host.turn_thinking.auto_deep_min_chars);
            signals.user_message.chars().count() >= min
        }
        TurnThinkingSignalRule::HighArousal => emotion_high_arousal(signals.emotion),
        TurnThinkingSignalRule::HighSadness => signals.emotion.sadness >= 0.45,
        TurnThinkingSignalRule::HighAnger => signals.emotion.anger >= 0.45,
        TurnThinkingSignalRule::HighFear => signals.emotion.fear >= 0.4,
        TurnThinkingSignalRule::ThisTurnEvent { events } => {
            event_list_matches(events, signals.this_turn_event)
        }
        TurnThinkingSignalRule::RecentEvent { events } => signals
            .recent_events
            .iter()
            .any(|e| event_list_matches(events, e.event_type)),
        TurnThinkingSignalRule::Keyword { keywords } => keywords
            .iter()
            .any(|kw| signals.user_message.contains(kw.as_str())),
        TurnThinkingSignalRule::DeepLatchActive => signals.deep_latch_active,
    }
}

#[must_use]
pub fn evaluate_policy(
    signals: &TurnThinkingSignals<'_>,
    policy: &TurnThinkingPolicy,
    host: &HostProfile,
) -> TurnThinkingPlan {
    let mut reasons = Vec::new();
    for rule in &policy.or_rules {
        if evaluate_or_rule(rule, signals, host) {
            reasons.push(reason_for_rule(rule));
        }
    }
    for group in &policy.and_groups {
        if group.all.is_empty() {
            continue;
        }
        if group
            .all
            .iter()
            .all(|rule| evaluate_or_rule(rule, signals, host))
        {
            reasons.push(TurnThinkingReason::AutoPackAndRule);
        }
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

#[must_use]
pub fn resolve_turn_thinking(
    host: &HostProfile,
    role: &Role,
    user_message: &str,
    emotion: &EmotionResult,
    recent_events: &[Event],
    this_turn_event: EventType,
    deep_latch_active: bool,
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
        TurnThinkingDefault::Auto => {
            if deep_latch_active {
                return TurnThinkingPlan {
                    mode: TurnThinkingMode::Deep,
                    reasons: vec![TurnThinkingReason::AutoDeepLatch],
                };
            }
            let policy = effective_turn_thinking_policy(host, role);
            let signals = TurnThinkingSignals {
                user_message,
                emotion,
                recent_events,
                this_turn_event,
                deep_latch_active,
            };
            evaluate_policy(&signals, &policy, host)
        }
    }
}

fn emotion_high_arousal(er: &EmotionResult) -> bool {
    er.anger >= 0.45 || er.sadness >= 0.45 || er.fear >= 0.4
}

fn truncate_chars(s: &str, max: usize) -> String {
    s.chars().take(max).collect()
}

fn event_label(event: EventType) -> &'static str {
    match event {
        EventType::Quarrel => "争吵",
        EventType::Apology => "道歉",
        EventType::Praise => "表扬",
        EventType::Complaint => "抱怨",
        EventType::Confession => "告白",
        EventType::Joke => "玩笑",
        EventType::Ignore => "忽视",
    }
}

fn build_ephemeral_event_summary(event: EventType, user_message: &str, max_chars: usize) -> String {
    let snippet = truncate_chars(user_message.trim(), max_chars.saturating_sub(24));
    format!("【{}】用户：{}", event_label(event), snippet)
}

/// Post-turn: latch enter/exit + ephemeral TTL and situation summary (no main-chain LLM).
///
/// # Errors
///
/// Returns [`anyhow::Error`] when `role_runtime` latch or ephemeral columns cannot be updated.
pub async fn update_turn_thinking_runtime_state(
    db: &DbManager,
    role_id: &str,
    policy: &TurnThinkingPolicy,
    this_turn_event: EventType,
    user_message: &str,
) -> Result<()> {
    let enter_events = parse_event_names(&policy.latch.enter_on);
    let exit_events = parse_event_names(&policy.latch.exit_on);

    if enter_events.contains(&this_turn_event) {
        db.set_deep_latch_active(role_id, true).await?;
    }
    if exit_events.contains(&this_turn_event) {
        db.set_deep_latch_active(role_id, false).await?;
    }

    let Some(ref ep_cfg) = policy.ephemeral else {
        return Ok(());
    };
    if !ep_cfg.enabled {
        return Ok(());
    }

    let mut ttl = db.get_ephemeral_ttl_turns(role_id).await?;
    let mut text = db.get_ephemeral_personality(role_id).await?;

    if ttl > 0 {
        ttl = ttl.saturating_sub(1);
    }
    if ttl == 0 {
        text.clear();
    }

    let update_events = parse_event_names(&ep_cfg.update_on_events);
    if update_events.contains(&this_turn_event) {
        text = build_ephemeral_event_summary(this_turn_event, user_message, ep_cfg.max_chars);
        ttl = ep_cfg.ttl_turns;
    } else if enter_events.contains(&this_turn_event) {
        text = format!(
            "【局面】{}后需认真回应，语气勿敷衍。",
            event_label(this_turn_event)
        );
        ttl = ep_cfg.ttl_turns;
    } else if exit_events.contains(&this_turn_event) {
        text = format!(
            "【局面】{}后气氛缓和，可逐步恢复正常语气。",
            event_label(this_turn_event)
        );
        ttl = ep_cfg.ttl_turns;
    }

    db.set_ephemeral_personality(role_id, &text).await?;
    db.set_ephemeral_ttl_turns(role_id, ttl).await?;
    Ok(())
}

#[must_use]
pub fn ephemeral_for_prompt(snapshot_ttl: Option<u32>, snapshot_text: Option<&str>) -> String {
    match snapshot_ttl {
        Some(t) if t > 0 => snapshot_text.unwrap_or("").trim().to_string(),
        _ => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::host_profile::{
        FastPersistenceMode, HostProfile, TurnThinkingDefault, TurnThinkingProfile,
    };
    use oclive_kernel_types::{
        RolePackTurnThinkingConfig, TurnThinkingDeepWhen, TurnThinkingLatchConfig,
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

    fn test_role() -> Role {
        Role {
            id: "test".into(),
            name: "Test".into(),
            ..Default::default()
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
            &test_role(),
            "你好",
            &neutral_emotion(),
            &[],
            EventType::Ignore,
            false,
        );
        assert_eq!(plan.mode, TurnThinkingMode::Fast);
    }

    #[test]
    fn auto_long_message_is_deep() {
        let msg = "a".repeat(100);
        let plan = resolve_turn_thinking(
            &test_host(TurnThinkingDefault::Auto),
            &test_role(),
            msg.as_str(),
            &neutral_emotion(),
            &[],
            EventType::Ignore,
            false,
        );
        assert_eq!(plan.mode, TurnThinkingMode::Deep);
    }

    #[test]
    fn this_turn_quarrel_short_message_is_deep() {
        let plan = resolve_turn_thinking(
            &test_host(TurnThinkingDefault::Auto),
            &test_role(),
            "你烦死了",
            &neutral_emotion(),
            &[],
            EventType::Quarrel,
            false,
        );
        assert_eq!(plan.mode, TurnThinkingMode::Deep);
        assert!(plan
            .reasons
            .contains(&TurnThinkingReason::AutoThisTurnEvent));
    }

    #[test]
    fn deep_latch_active_forces_deep() {
        let plan = resolve_turn_thinking(
            &test_host(TurnThinkingDefault::Auto),
            &test_role(),
            "嗯",
            &neutral_emotion(),
            &[],
            EventType::Ignore,
            true,
        );
        assert_eq!(plan.mode, TurnThinkingMode::Deep);
    }

    #[test]
    fn pack_and_group_requires_all_signals() {
        let mut role = test_role();
        role.pack_turn_thinking_config = Some(RolePackTurnThinkingConfig {
            deep_when: TurnThinkingDeepWhen {
                or: vec![],
                and: vec![TurnThinkingAndGroup {
                    all: vec![
                        TurnThinkingSignalRule::LongMessage {
                            min_chars: Some(10),
                        },
                        TurnThinkingSignalRule::HighSadness,
                    ],
                }],
            },
            ..Default::default()
        });
        let plan = resolve_turn_thinking(
            &test_host(TurnThinkingDefault::Auto),
            &role,
            "短句",
            &neutral_emotion(),
            &[],
            EventType::Ignore,
            false,
        );
        assert_eq!(plan.mode, TurnThinkingMode::Fast);

        let mut er = neutral_emotion();
        er.sadness = 0.5;
        let long = "a".repeat(20);
        let plan = resolve_turn_thinking(
            &test_host(TurnThinkingDefault::Auto),
            &role,
            long.as_str(),
            &er,
            &[],
            EventType::Ignore,
            false,
        );
        assert_eq!(plan.mode, TurnThinkingMode::Deep);
        assert!(plan.reasons.contains(&TurnThinkingReason::AutoPackAndRule));
    }

    #[test]
    fn host_pack_or_merge_extends_or_rules() {
        let mut role = test_role();
        role.pack_turn_thinking_config = Some(RolePackTurnThinkingConfig {
            deep_when: TurnThinkingDeepWhen {
                or: vec![TurnThinkingSignalRule::DeepLatchActive],
                and: vec![],
            },
            ..Default::default()
        });
        let policy = effective_turn_thinking_policy(&test_host(TurnThinkingDefault::Auto), &role);
        assert!(
            policy.or_rules.len()
                > host_default_or_rules(&test_host(TurnThinkingDefault::Auto)).len()
        );
    }

    #[test]
    fn fast_plan_skips_event_llm_even_when_host_allows() {
        let mut host = test_host(TurnThinkingDefault::Fast);
        host.event_impact_llm = true;
        let plan = resolve_turn_thinking(
            &host,
            &test_role(),
            "hi",
            &neutral_emotion(),
            &[],
            EventType::Ignore,
            false,
        );
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

    #[test]
    fn latch_config_parsed_from_pack() {
        let mut role = test_role();
        role.pack_turn_thinking_config = Some(RolePackTurnThinkingConfig {
            latch: TurnThinkingLatchConfig {
                enter_on: vec!["Quarrel".into()],
                exit_on: vec!["Apology".into()],
            },
            ..Default::default()
        });
        let policy = effective_turn_thinking_policy(&test_host(TurnThinkingDefault::Auto), &role);
        assert_eq!(policy.latch.enter_on, vec!["Quarrel".to_string()]);
        assert_eq!(policy.latch.exit_on, vec!["Apology".to_string()]);
    }

    #[test]
    fn ephemeral_for_prompt_respects_ttl() {
        assert_eq!(ephemeral_for_prompt(Some(3), Some("局面紧张")), "局面紧张");
        assert_eq!(ephemeral_for_prompt(Some(0), Some("x")), "");
        assert_eq!(ephemeral_for_prompt(None, Some("x")), "");
    }
}
