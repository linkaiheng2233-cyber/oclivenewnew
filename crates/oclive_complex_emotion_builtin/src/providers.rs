//! `feature = "providers"`：关键词 `ComplexEmotionProvider`（`builtin_keyword_v1`）。

use oclive_kernel_core::complex_emotion::{
    ComplexEmotionInput, ComplexEmotionOutput, ComplexEmotionProvider,
};
use oclive_kernel_core::error::Result;

fn user_text_len_chars(s: &str) -> usize {
    s.trim().chars().count()
}

fn contains_any(hay: &str, needles: &[&str]) -> bool {
    needles.iter().any(|n| hay.contains(n))
}

pub struct BuiltinKeywordComplexEmotionProvider;

impl BuiltinKeywordComplexEmotionProvider {
    const CONF: f64 = 0.7;
    const INT: f64 = 0.5;
    const SOURCE: &'static str = "builtin_keyword_v1";

    fn base_output(
        pattern: Option<String>,
        narrative_hint: String,
        labels: Vec<String>,
    ) -> ComplexEmotionOutput {
        ComplexEmotionOutput {
            source: Self::SOURCE.to_string(),
            narrative_hint: Some(narrative_hint),
            labels,
            pattern,
            confidence: Self::CONF,
            intensity: Self::INT,
            dissonance_score: 0.0,
            degraded_to_builtin: false,
        }
    }

    fn default_fallback() -> ComplexEmotionOutput {
        ComplexEmotionOutput {
            source: Self::SOURCE.to_string(),
            narrative_hint: Some("未命中特定模式；保持自然对话节奏即可。".to_string()),
            labels: vec![],
            pattern: None,
            confidence: 0.35,
            intensity: 0.25,
            dissonance_score: 0.0,
            degraded_to_builtin: false,
        }
    }

    #[must_use]
    pub fn resolve_turn_inner(&self, input: &ComplexEmotionInput) -> ComplexEmotionOutput {
        let u = input.user_message.as_str();
        let v = input.user_valence.unwrap_or(0.0);
        let d = input.user_dominance.unwrap_or(0.0);

        if let Some(prev) = input.previous_user_message.as_deref() {
            if user_text_len_chars(prev) <= 2 && user_text_len_chars(u) <= 2 {
                return Self::base_output(
                    Some("waning_engagement".to_string()),
                    "对话热度下降，角色可尝试提出新话题或幽默打破沉闷。".to_string(),
                    vec!["low_energy".to_string()],
                );
            }
        }

        if contains_any(u, &["没事", "我没事", "不用管我"]) && v < 0.0 {
            return Self::base_output(
                Some("suppressed_distress".to_string()),
                "用户可能在掩饰情绪，角色宜保持温柔关注，不必追问。".to_string(),
                vec!["masking".to_string(), "support".to_string()],
            );
        }

        if contains_any(u, &["随便", "都行", "你定"]) {
            return Self::base_output(
                Some("disengagement".to_string()),
                "用户可能缺乏兴致，角色可主动提供简单选项或转换话题。".to_string(),
                vec!["low_drive".to_string()],
            );
        }

        if contains_any(u, &["真好", "真羡慕你"]) && d < 0.0 {
            return Self::base_output(
                Some("wistful_envy".to_string()),
                "用户流露向往与轻微落差感，角色可适度分享脆弱面拉近距离。".to_string(),
                vec!["social_compare".to_string()],
            );
        }

        Self::default_fallback()
    }
}

impl ComplexEmotionProvider for BuiltinKeywordComplexEmotionProvider {
    fn resolve_turn(&self, input: &ComplexEmotionInput) -> Result<ComplexEmotionOutput> {
        Ok(self.resolve_turn_inner(input))
    }
}

#[cfg(test)]
mod keyword_tests {
    use super::*;

    fn inp(
        user: &str,
        prev: Option<&str>,
        valence: Option<f64>,
        dominance: Option<f64>,
    ) -> ComplexEmotionInput {
        ComplexEmotionInput {
            role_id: "r".into(),
            scene_id: "s".into(),
            user_message: user.into(),
            bot_reply: "ok".into(),
            recent_dialogue_summary: None,
            previous_narrative_hint: String::new(),
            user_valence: valence,
            user_dominance: dominance,
            previous_user_message: prev.map(String::from),
        }
    }

    #[test]
    fn suppressed_distress_valence_negative() {
        let p = BuiltinKeywordComplexEmotionProvider;
        let o = p.resolve_turn_inner(&inp("我没事啦", None, Some(-0.3), Some(0.0)));
        assert_eq!(o.pattern.as_deref(), Some("suppressed_distress"));
        assert_eq!(o.source, "builtin_keyword_v1");
        assert!((o.confidence - 0.7).abs() < 1e-9);
        assert!((o.intensity - 0.5).abs() < 1e-9);
    }

    #[test]
    fn disengagement_triggers() {
        let p = BuiltinKeywordComplexEmotionProvider;
        let o = p.resolve_turn_inner(&inp("随便吧", None, Some(0.5), Some(0.5)));
        assert_eq!(o.pattern.as_deref(), Some("disengagement"));
    }

    #[test]
    fn wistful_envy_dominance_negative() {
        let p = BuiltinKeywordComplexEmotionProvider;
        let o = p.resolve_turn_inner(&inp("真好呀真羡慕你", None, Some(0.1), Some(-0.4)));
        assert_eq!(o.pattern.as_deref(), Some("wistful_envy"));
    }

    #[test]
    fn waning_engagement_two_short_user_lines() {
        let p = BuiltinKeywordComplexEmotionProvider;
        let o = p.resolve_turn_inner(&inp("嗯", Some("好"), Some(0.0), Some(0.0)));
        assert_eq!(o.pattern.as_deref(), Some("waning_engagement"));
    }
}
