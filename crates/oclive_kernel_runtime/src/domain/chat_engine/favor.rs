//! 好感与关系阶段计算（本回合）——纯逻辑迁移。

use crate::models::{Event, EventType};
use crate::domain::relation_engine::{RelationEngine, RelationState};

fn confidence_decay_weight(confidence: f32) -> f64 {
    let c = (confidence as f64).clamp(0.0, 1.0);
    let threshold = 0.60_f64;
    if c >= threshold {
        1.0
    } else {
        (0.25 + 0.75 * (c / threshold)).clamp(0.25, 1.0)
    }
}

fn avoid_fast_promote_score(
    current_event: &EventType,
    current_impact_factor: f64,
    recent_events: &[Event],
) -> f64 {
    let is_current_strong_positive =
        matches!(current_event, EventType::Praise | EventType::Confession)
            && current_impact_factor >= 0.55;
    if !is_current_strong_positive {
        return 0.0;
    }

    const WINDOW: usize = 4;
    let mut prev_positive_streak = 0usize;
    for event in recent_events.iter().take(WINDOW) {
        if matches!(event.event_type, EventType::Praise | EventType::Confession) {
            prev_positive_streak += 1;
        } else {
            break;
        }
    }
    let streak = prev_positive_streak + 1;
    match streak {
        0..=1 => 0.0,
        2 => 0.35,
        3 => 0.7,
        _ => 1.0,
    }
}

fn event_direction(event_type: &EventType) -> i8 {
    match event_type {
        EventType::Praise | EventType::Confession => 1,
        EventType::Quarrel | EventType::Complaint | EventType::Ignore => -1,
        EventType::Apology | EventType::Joke => 0,
    }
}

fn smooth_favor_delta_for_short_streak(raw_delta: f64, recent_events: &[Event]) -> f64 {
    const WINDOW: usize = 4;
    const MIN_ACTIVE_DELTA: f64 = 0.03;
    if raw_delta.abs() < MIN_ACTIVE_DELTA {
        return raw_delta;
    }

    let current_dir = if raw_delta > 0.0 { 1 } else { -1 };
    let mut streak = 1usize;
    for event in recent_events.iter().take(WINDOW) {
        let dir = event_direction(&event.event_type);
        if dir == 0 {
            break;
        }
        if dir == current_dir {
            streak += 1;
        } else {
            break;
        }
    }

    let scale = match streak {
        0..=1 => 1.0,
        2 => 0.94,
        3 => 0.88,
        _ => 0.82,
    };
    raw_delta * scale
}

pub struct FavorRelationInput<'a> {
    pub relation_before: &'a str,
    pub favorability_before: f64,
    pub ai_event_type: &'a EventType,
    pub ai_impact_factor_final: f64,
    pub event_runtime: f64,
    pub favor_mult: f64,
    pub event_confidence: f32,
    pub recent_events_for_event: &'a [Event],
}

pub fn compute_favor_and_relation(input: &FavorRelationInput<'_>) -> (f64, RelationState) {
    let confidence_weight = confidence_decay_weight(input.event_confidence);
    let favor_delta_raw = (input.ai_impact_factor_final
        * 0.05
        * input.event_runtime
        * input.favor_mult
        * confidence_weight)
        .clamp(-0.2_f64, 0.2_f64);
    let favor_delta =
        smooth_favor_delta_for_short_streak(favor_delta_raw, input.recent_events_for_event)
            .clamp(-0.2_f64, 0.2_f64);
    let avoid_fast_promote = avoid_fast_promote_score(
        input.ai_event_type,
        input.ai_impact_factor_final,
        input.recent_events_for_event,
    );
    let relation_after = RelationEngine::next_state_with_damping(
        RelationState::parse(input.relation_before),
        (input.favorability_before + favor_delta).clamp(0.0, 100.0),
        input.ai_event_type,
        input.ai_impact_factor_final,
        input.event_confidence,
        avoid_fast_promote,
    );
    (favor_delta, relation_after)
}
