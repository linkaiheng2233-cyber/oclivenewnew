use serde::{Deserialize, Serialize};

use super::role::PersonalityDefaults;

/// 运行时性格向量（旧七维，与 `PersonalityDefaults` 同序）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PersonalityVector {
    pub stubbornness: f64,
    pub clinginess: f64,
    pub sensitivity: f64,
    pub assertiveness: f64,
    pub forgiveness: f64,
    pub talkativeness: f64,
    pub warmth: f64,
}

impl From<&PersonalityDefaults> for PersonalityVector {
    fn from(d: &PersonalityDefaults) -> Self {
        Self {
            stubbornness: f64::from(d.stubbornness),
            clinginess: f64::from(d.clinginess),
            sensitivity: f64::from(d.sensitivity),
            assertiveness: f64::from(d.assertiveness),
            forgiveness: f64::from(d.forgiveness),
            talkativeness: f64::from(d.talkativeness),
            warmth: f64::from(d.warmth),
        }
    }
}

impl PersonalityVector {
    pub fn zero() -> Self {
        Self {
            stubbornness: 0.0,
            clinginess: 0.0,
            sensitivity: 0.0,
            assertiveness: 0.0,
            forgiveness: 0.0,
            talkativeness: 0.0,
            warmth: 0.0,
        }
    }

    /// 七维数组顺序：倔强…温暖
    pub fn to_vec7(&self) -> Vec<f64> {
        vec![
            self.stubbornness,
            self.clinginess,
            self.sensitivity,
            self.assertiveness,
            self.forgiveness,
            self.talkativeness,
            self.warmth,
        ]
    }

    pub fn from_vec7(v: &[f64]) -> Self {
        let g = |i: usize| v.get(i).copied().unwrap_or(0.0);
        Self {
            stubbornness: g(0),
            clinginess: g(1),
            sensitivity: g(2),
            assertiveness: g(3),
            forgiveness: g(4),
            talkativeness: g(5),
            warmth: g(6),
        }
    }

    /// effective = clamp(core + delta) 各维到 evolution_bounds
    pub fn effective_from_core_delta(
        core: &PersonalityDefaults,
        delta: &PersonalityVector,
        bounds: &crate::models::EvolutionBounds,
    ) -> Self {
        let mut e = PersonalityVector::from(core);
        e.stubbornness += delta.stubbornness;
        e.clinginess += delta.clinginess;
        e.sensitivity += delta.sensitivity;
        e.assertiveness += delta.assertiveness;
        e.forgiveness += delta.forgiveness;
        e.talkativeness += delta.talkativeness;
        e.warmth += delta.warmth;
        e.clamp(bounds);
        e
    }

    pub fn to_json_vec(&self) -> String {
        serde_json::to_string(&self.to_vec7()).unwrap_or_else(|_| "[]".to_string())
    }

    pub fn from_json_vec(s: &str) -> Result<Self, serde_json::Error> {
        let v: Vec<f64> = serde_json::from_str(s)?;
        Ok(Self::from_vec7(&v))
    }

    /// 分量差（用于从有效向量反推 delta）
    pub fn sub_components(a: &Self, b: &Self) -> Self {
        Self {
            stubbornness: a.stubbornness - b.stubbornness,
            clinginess: a.clinginess - b.clinginess,
            sensitivity: a.sensitivity - b.sensitivity,
            assertiveness: a.assertiveness - b.assertiveness,
            forgiveness: a.forgiveness - b.forgiveness,
            talkativeness: a.talkativeness - b.talkativeness,
            warmth: a.warmth - b.warmth,
        }
    }

    pub fn clamp(&mut self, bounds: &crate::models::EvolutionBounds) {
        self.stubbornness = self
            .stubbornness
            .clamp(bounds.stubbornness.0, bounds.stubbornness.1);
        self.clinginess = self
            .clinginess
            .clamp(bounds.clinginess.0, bounds.clinginess.1);
        self.sensitivity = self
            .sensitivity
            .clamp(bounds.sensitivity.0, bounds.sensitivity.1);
        self.assertiveness = self
            .assertiveness
            .clamp(bounds.assertiveness.0, bounds.assertiveness.1);
        self.forgiveness = self
            .forgiveness
            .clamp(bounds.forgiveness.0, bounds.forgiveness.1);
        self.talkativeness = self
            .talkativeness
            .clamp(bounds.talkativeness.0, bounds.talkativeness.1);
        self.warmth = self.warmth.clamp(bounds.warmth.0, bounds.warmth.1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::EvolutionBounds;

    #[test]
    fn test_clamp() {
        let mut pv = PersonalityVector {
            stubbornness: 1.5,
            clinginess: -0.5,
            sensitivity: 0.5,
            assertiveness: 0.5,
            forgiveness: 0.5,
            talkativeness: 0.5,
            warmth: 0.5,
        };
        let bounds = EvolutionBounds::full_01();
        pv.clamp(&bounds);
        assert_eq!(pv.stubbornness, 1.0);
        assert_eq!(pv.clinginess, 0.0);
    }
}
