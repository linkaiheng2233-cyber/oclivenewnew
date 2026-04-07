//! 好感与关系阶段计算（本回合）

use crate::domain::chat_turn_rules::{
    avoid_fast_promote_score, confidence_decay_weight, smooth_favor_delta_for_short_streak,
};
use crate::domain::{RelationEngine, RelationState};
use crate::models::{Event, EventType};

pub(super) struct FavorRelationInput<'a> {
    pub relation_before: &'a str,
    pub favorability_before: f64,
    pub ai_event_type: &'a EventType,
    pub ai_impact_factor_final: f64,
    pub event_runtime: f64,
    pub favor_mult: f64,
    pub event_confidence: f32,
    pub recent_events_for_event: &'a [Event],
}

pub(super) fn compute_favor_and_relation(input: &FavorRelationInput<'_>) -> (f64, RelationState) {
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
