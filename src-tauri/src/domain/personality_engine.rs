//! 性格引擎：旧七维向量演化（倔强、黏人、敏感、强势、宽容、话多、温暖）

use crate::models::{EvolutionBounds, PersonalityVector};

pub struct PersonalityEngine;

impl PersonalityEngine {
    pub fn evolve_by_event(
        mut personality: PersonalityVector,
        impact_factor: f64,
        bounds: &EvolutionBounds,
    ) -> PersonalityVector {
        let clamped_factor = impact_factor.clamp(-1.0, 1.0);

        if clamped_factor > 0.0 {
            personality.warmth += clamped_factor * 0.1;
            personality.forgiveness += clamped_factor * 0.08;
            personality.talkativeness += clamped_factor * 0.06;
        } else if clamped_factor < 0.0 {
            personality.sensitivity += clamped_factor.abs() * 0.1;
            personality.stubbornness += clamped_factor.abs() * 0.05;
            personality.warmth += clamped_factor * 0.05;
        }

        personality.clamp(bounds);
        personality
    }

    pub fn adjust_by_user_emotion(
        mut personality: PersonalityVector,
        user_emotion_str: &str,
        bounds: &EvolutionBounds,
    ) -> PersonalityVector {
        match user_emotion_str {
            "happy" => {
                personality.talkativeness += 0.05;
                personality.warmth += 0.05;
            }
            "sad" => {
                personality.sensitivity += 0.08;
                personality.warmth += 0.1;
                personality.clinginess += 0.04;
            }
            "angry" => {
                personality.assertiveness += 0.08;
                personality.sensitivity -= 0.05;
            }
            "excited" => {
                personality.talkativeness += 0.1;
                personality.clinginess += 0.06;
            }
            "confused" => {
                personality.assertiveness -= 0.05;
                personality.sensitivity += 0.05;
            }
            "shy" => {
                personality.sensitivity += 0.1;
                personality.assertiveness -= 0.08;
            }
            _ => {}
        }

        personality.clamp(bounds);
        personality
    }

    pub fn calculate_similarity(pv1: &PersonalityVector, pv2: &PersonalityVector) -> f64 {
        let diff: f64 = ((pv1.stubbornness - pv2.stubbornness).abs()
            + (pv1.clinginess - pv2.clinginess).abs()
            + (pv1.sensitivity - pv2.sensitivity).abs()
            + (pv1.assertiveness - pv2.assertiveness).abs()
            + (pv1.forgiveness - pv2.forgiveness).abs()
            + (pv1.talkativeness - pv2.talkativeness).abs()
            + (pv1.warmth - pv2.warmth).abs())
            / 7.0;

        (1.0 - diff).max(0.0)
    }

    pub fn calculate_stability_index(personality: &PersonalityVector) -> f64 {
        let base = 1.0 - personality.sensitivity * 0.35;
        base.clamp(0.0, 1.0)
    }

    pub fn calculate_extroversion_index(personality: &PersonalityVector) -> f64 {
        (personality.warmth + personality.talkativeness + personality.assertiveness) / 3.0
    }

    pub fn calculate_rationality_index(personality: &PersonalityVector) -> f64 {
        let base = (personality.assertiveness + personality.forgiveness) / 2.0;
        (base - personality.sensitivity * 0.2).clamp(0.0, 1.0)
    }

    pub fn get_dominant_traits(personality: &PersonalityVector) -> Vec<String> {
        let mut traits = Vec::new();
        let threshold = 0.6;

        if personality.stubbornness > threshold {
            traits.push("倔强".to_string());
        }
        if personality.clinginess > threshold {
            traits.push("黏人".to_string());
        }
        if personality.sensitivity > threshold {
            traits.push("敏感".to_string());
        }
        if personality.assertiveness > threshold {
            traits.push("强势".to_string());
        }
        if personality.forgiveness > threshold {
            traits.push("宽容".to_string());
        }
        if personality.talkativeness > threshold {
            traits.push("话多".to_string());
        }
        if personality.warmth > threshold {
            traits.push("温暖".to_string());
        }

        if traits.is_empty() {
            traits.push("平衡".to_string());
        }

        traits
    }

    pub fn smooth_evolution(
        current: &PersonalityVector,
        target: &PersonalityVector,
        smoothing_factor: f64,
    ) -> PersonalityVector {
        let factor = smoothing_factor.clamp(0.0, 1.0);

        PersonalityVector {
            stubbornness: current.stubbornness
                + (target.stubbornness - current.stubbornness) * factor,
            clinginess: current.clinginess + (target.clinginess - current.clinginess) * factor,
            sensitivity: current.sensitivity + (target.sensitivity - current.sensitivity) * factor,
            assertiveness: current.assertiveness
                + (target.assertiveness - current.assertiveness) * factor,
            forgiveness: current.forgiveness + (target.forgiveness - current.forgiveness) * factor,
            talkativeness: current.talkativeness
                + (target.talkativeness - current.talkativeness) * factor,
            warmth: current.warmth + (target.warmth - current.warmth) * factor,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_personality() -> PersonalityVector {
        PersonalityVector {
            stubbornness: 0.5,
            clinginess: 0.5,
            sensitivity: 0.5,
            assertiveness: 0.5,
            forgiveness: 0.5,
            talkativeness: 0.5,
            warmth: 0.5,
        }
    }

    fn create_test_bounds() -> EvolutionBounds {
        EvolutionBounds::full_01()
    }

    #[test]
    fn test_evolve_by_positive_event() {
        let personality = create_test_personality();
        let bounds = create_test_bounds();
        let evolved = PersonalityEngine::evolve_by_event(personality, 0.8, &bounds);

        assert!(evolved.warmth > 0.5);
        assert!(evolved.forgiveness > 0.5);
    }

    #[test]
    fn test_evolve_by_negative_event() {
        let personality = create_test_personality();
        let bounds = create_test_bounds();
        let evolved = PersonalityEngine::evolve_by_event(personality, -0.8, &bounds);

        assert!(evolved.sensitivity > 0.5);
    }

    #[test]
    fn test_adjust_by_user_emotion_happy() {
        let personality = create_test_personality();
        let bounds = create_test_bounds();
        let adjusted = PersonalityEngine::adjust_by_user_emotion(personality, "happy", &bounds);

        assert!(adjusted.talkativeness > 0.5);
        assert!(adjusted.warmth > 0.5);
    }

    #[test]
    fn test_calculate_similarity() {
        let pv1 = create_test_personality();
        let pv2 = create_test_personality();
        let similarity = PersonalityEngine::calculate_similarity(&pv1, &pv2);

        assert_eq!(similarity, 1.0);
    }

    #[test]
    fn test_calculate_stability_index() {
        let personality = create_test_personality();
        let stability = PersonalityEngine::calculate_stability_index(&personality);

        assert!((0.0..=1.0).contains(&stability));
    }

    #[test]
    fn test_calculate_extroversion_index() {
        let personality = create_test_personality();
        let extroversion = PersonalityEngine::calculate_extroversion_index(&personality);

        assert_eq!(extroversion, 0.5);
    }

    #[test]
    fn test_get_dominant_traits() {
        let mut personality = create_test_personality();
        personality.warmth = 0.8;
        personality.talkativeness = 0.8;

        let traits = PersonalityEngine::get_dominant_traits(&personality);
        assert!(traits.contains(&"温暖".to_string()));
        assert!(traits.contains(&"话多".to_string()));
    }

    #[test]
    fn test_smooth_evolution() {
        let current = create_test_personality();
        let mut target = create_test_personality();
        target.warmth = 0.8;

        let smoothed = PersonalityEngine::smooth_evolution(&current, &target, 0.5);
        assert!(smoothed.warmth > current.warmth && smoothed.warmth < target.warmth);
    }
}
