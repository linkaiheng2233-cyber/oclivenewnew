use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Favorability {
    pub role_id: String,
    pub value: f64,
}

impl Favorability {
    pub fn new(role_id: String) -> Self {
        Self {
            role_id,
            value: 0.0,
        }
    }

    pub fn add(&mut self, delta: f64) {
        self.value = (self.value + delta).clamp(-100.0, 100.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_favorability_add() {
        let mut fav = Favorability::new("test".to_string());
        fav.add(50.0);
        assert_eq!(fav.value, 50.0);
        fav.add(100.0);
        assert_eq!(fav.value, 100.0); // 应该被 clamp 到 100
    }
}
