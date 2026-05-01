use crate::models::EventType;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelationState {
    Stranger,
    Acquaintance,
    Friend,
    CloseFriend,
    Partner,
}

impl RelationState {
    pub fn as_str(&self) -> &'static str {
        match self {
            RelationState::Stranger => "Stranger",
            RelationState::Acquaintance => "Acquaintance",
            RelationState::Friend => "Friend",
            RelationState::CloseFriend => "CloseFriend",
            RelationState::Partner => "Partner",
        }
    }

    pub fn parse(s: &str) -> Self {
        match s {
            "Acquaintance" => RelationState::Acquaintance,
            "Friend" => RelationState::Friend,
            "CloseFriend" => RelationState::CloseFriend,
            "Partner" => RelationState::Partner,
            _ => RelationState::Stranger,
        }
    }
}

pub struct RelationEngine;

impl RelationEngine {
    const LOW_CONFIDENCE_THRESHOLD: f64 = 0.60;
    const PROMOTION_TRIGGER_THRESHOLD: f64 = 0.35;
    const MAX_PROMOTION_DAMPING_EXTRA: f64 = 0.12;

    pub fn next_state(
        current: RelationState,
        favorability: f64,
        event: &EventType,
        impact_factor: f64,
        confidence: f32,
    ) -> RelationState {
        Self::next_state_with_damping(current, favorability, event, impact_factor, confidence, 0.0)
    }

    pub fn next_state_with_damping(
        current: RelationState,
        favorability: f64,
        event: &EventType,
        impact_factor: f64,
        confidence: f32,
        avoid_fast_promote: f64,
    ) -> RelationState {
        let target = Self::target_by_score_and_event(
            favorability,
            event,
            impact_factor,
            confidence,
            avoid_fast_promote,
        );
        Self::smooth_step(current, target)
    }

    fn target_by_score_and_event(
        favorability: f64,
        event: &EventType,
        impact_factor: f64,
        confidence: f32,
        avoid_fast_promote: f64,
    ) -> RelationState {
        let score = favorability.clamp(0.0, 100.0);
        let base = if score >= 85.0 {
            RelationState::Partner
        } else if score >= 65.0 {
            RelationState::CloseFriend
        } else if score >= 40.0 {
            RelationState::Friend
        } else if score >= 20.0 {
            RelationState::Acquaintance
        } else {
            RelationState::Stranger
        };

        // 关系跃迁也参考 AI 给出的 impact 强弱，避免“事件类型正确但升降过硬”。
        let confidence_weight = Self::confidence_weight(confidence);
        let impact = impact_factor.clamp(-1.0, 1.0) * confidence_weight;
        let promote_threshold = Self::PROMOTION_TRIGGER_THRESHOLD
            + avoid_fast_promote.clamp(0.0, 1.0) * Self::MAX_PROMOTION_DAMPING_EXTRA;
        match event {
            EventType::Confession | EventType::Praise if impact >= promote_threshold => {
                Self::promote_one(base)
            }
            EventType::Quarrel if impact <= -0.35 => Self::demote_one(base),
            _ => base,
        }
    }

    fn confidence_weight(confidence: f32) -> f64 {
        let c = (confidence as f64).clamp(0.0, 1.0);
        if c >= Self::LOW_CONFIDENCE_THRESHOLD {
            1.0
        } else {
            // 低置信不会完全失效，但会显著减弱关系跃迁触发。
            (0.25 + 0.75 * (c / Self::LOW_CONFIDENCE_THRESHOLD)).clamp(0.25, 1.0)
        }
    }

    fn smooth_step(current: RelationState, target: RelationState) -> RelationState {
        use RelationState::*;
        let rank = |s: RelationState| match s {
            Stranger => 0_i32,
            Acquaintance => 1,
            Friend => 2,
            CloseFriend => 3,
            Partner => 4,
        };
        let cur = rank(current);
        let tar = rank(target);
        if tar > cur + 1 {
            Self::from_rank(cur + 1)
        } else if tar < cur - 1 {
            Self::from_rank(cur - 1)
        } else {
            target
        }
    }

    fn promote_one(state: RelationState) -> RelationState {
        Self::from_rank((Self::rank(state) + 1).min(4))
    }

    fn demote_one(state: RelationState) -> RelationState {
        Self::from_rank((Self::rank(state) - 1).max(0))
    }

    fn rank(state: RelationState) -> i32 {
        match state {
            RelationState::Stranger => 0,
            RelationState::Acquaintance => 1,
            RelationState::Friend => 2,
            RelationState::CloseFriend => 3,
            RelationState::Partner => 4,
        }
    }

    fn from_rank(rank: i32) -> RelationState {
        match rank {
            1 => RelationState::Acquaintance,
            2 => RelationState::Friend,
            3 => RelationState::CloseFriend,
            4 => RelationState::Partner,
            _ => RelationState::Stranger,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relation_promotes_with_good_signal() {
        let next =
            RelationEngine::next_state(RelationState::Stranger, 45.0, &EventType::Praise, 0.7, 0.9);
        assert_eq!(next, RelationState::Acquaintance);
    }

    #[test]
    fn relation_does_not_jump_multiple_levels() {
        let next = RelationEngine::next_state(
            RelationState::Stranger,
            90.0,
            &EventType::Confession,
            0.9,
            0.95,
        );
        assert_eq!(next, RelationState::Acquaintance);
    }

    #[test]
    fn weak_positive_signal_does_not_force_promotion() {
        let next =
            RelationEngine::next_state(RelationState::Friend, 52.0, &EventType::Praise, 0.12, 0.8);
        assert_eq!(next, RelationState::Friend);
    }

    #[test]
    fn weak_negative_signal_does_not_force_demotion() {
        let next =
            RelationEngine::next_state(RelationState::Friend, 52.0, &EventType::Quarrel, -0.1, 0.9);
        assert_eq!(next, RelationState::Friend);
    }

    #[test]
    fn low_confidence_weakens_promotion_trigger() {
        let next =
            RelationEngine::next_state(RelationState::Friend, 52.0, &EventType::Praise, 0.5, 0.2);
        assert_eq!(next, RelationState::Friend);
    }

    #[test]
    fn damping_prevents_fast_promotion_on_positive_streak() {
        let without_damping = RelationEngine::next_state_with_damping(
            RelationState::Friend,
            52.0,
            &EventType::Praise,
            0.42,
            1.0,
            0.0,
        );
        let with_damping = RelationEngine::next_state_with_damping(
            RelationState::Friend,
            52.0,
            &EventType::Praise,
            0.42,
            1.0,
            1.0,
        );
        assert_eq!(without_damping, RelationState::CloseFriend);
        assert_eq!(with_damping, RelationState::Friend);
    }

    #[test]
    fn weak_or_non_consecutive_positive_is_almost_unchanged() {
        let without_damping = RelationEngine::next_state_with_damping(
            RelationState::Friend,
            52.0,
            &EventType::Praise,
            0.6,
            1.0,
            0.0,
        );
        let with_mild_damping = RelationEngine::next_state_with_damping(
            RelationState::Friend,
            52.0,
            &EventType::Praise,
            0.6,
            1.0,
            0.2,
        );
        assert_eq!(without_damping, with_mild_damping);
    }

    #[test]
    fn damping_does_not_affect_negative_path() {
        let without_damping = RelationEngine::next_state_with_damping(
            RelationState::Friend,
            52.0,
            &EventType::Quarrel,
            -0.6,
            1.0,
            0.0,
        );
        let with_damping = RelationEngine::next_state_with_damping(
            RelationState::Friend,
            52.0,
            &EventType::Quarrel,
            -0.6,
            1.0,
            1.0,
        );
        assert_eq!(without_damping, RelationState::Acquaintance);
        assert_eq!(without_damping, with_damping);
    }
}
