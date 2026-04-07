//! 事件影响估计可替换门面；默认委托 [`estimate_event_impact`](super::event_impact_ai::estimate_event_impact)。
#![allow(clippy::too_many_arguments)] // `EventEstimator::estimate` 与编排层参数一致，不宜为 clippy 拆结构体

use crate::domain::event_impact_ai::EventImpactEstimate;
use crate::error::Result;
use crate::infrastructure::llm::LlmClient;
use crate::models::knowledge::KnowledgeEventAugment;
use crate::models::{Emotion, Event, PersonalityVector};
use async_trait::async_trait;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

#[async_trait]
pub trait EventEstimator: Send + Sync {
    async fn estimate(
        &self,
        llm: &Arc<dyn LlmClient>,
        ollama_model: &str,
        user_message: &str,
        user_emotion: &Emotion,
        personality: &PersonalityVector,
        recent_turns: &[(String, String)],
        recent_events: &[Event],
        knowledge_augment: Option<&KnowledgeEventAugment>,
    ) -> Result<EventImpactEstimate>;
}

pub struct BuiltinEventEstimator;

#[async_trait]
impl EventEstimator for BuiltinEventEstimator {
    async fn estimate(
        &self,
        llm: &Arc<dyn LlmClient>,
        ollama_model: &str,
        user_message: &str,
        user_emotion: &Emotion,
        personality: &PersonalityVector,
        recent_turns: &[(String, String)],
        recent_events: &[Event],
        knowledge_augment: Option<&KnowledgeEventAugment>,
    ) -> Result<EventImpactEstimate> {
        super::event_impact_ai::estimate_event_impact(
            llm,
            ollama_model,
            user_message,
            user_emotion,
            personality,
            recent_turns,
            recent_events,
            knowledge_augment,
        )
        .await
    }
}

/// 第二套内置：在 [`BuiltinEventEstimator`] 的结果上将 `impact_factor` 乘以 **0.5**（更保守，用于验证 `event` 枚举可切换）。
pub struct BuiltinEventEstimatorV2;

#[async_trait]
impl EventEstimator for BuiltinEventEstimatorV2 {
    async fn estimate(
        &self,
        llm: &Arc<dyn LlmClient>,
        ollama_model: &str,
        user_message: &str,
        user_emotion: &Emotion,
        personality: &PersonalityVector,
        recent_turns: &[(String, String)],
        recent_events: &[Event],
        knowledge_augment: Option<&KnowledgeEventAugment>,
    ) -> Result<EventImpactEstimate> {
        let mut est = BuiltinEventEstimator
            .estimate(
                llm,
                ollama_model,
                user_message,
                user_emotion,
                personality,
                recent_turns,
                recent_events,
                knowledge_augment,
            )
            .await?;
        est.impact_factor *= 0.5;
        Ok(est)
    }
}

pub struct RemoteEventEstimatorPlaceholder {
    inner: BuiltinEventEstimator,
    warned: AtomicBool,
}

impl RemoteEventEstimatorPlaceholder {
    pub fn new() -> Self {
        Self {
            inner: BuiltinEventEstimator,
            warned: AtomicBool::new(false),
        }
    }

    fn warn_once(&self) {
        if self
            .warned
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
        {
            log::warn!(
                target: "oclive_plugin",
                "event backend Remote is not connected; using builtin event impact"
            );
        }
    }
}

#[async_trait]
impl EventEstimator for RemoteEventEstimatorPlaceholder {
    async fn estimate(
        &self,
        llm: &Arc<dyn LlmClient>,
        ollama_model: &str,
        user_message: &str,
        user_emotion: &Emotion,
        personality: &PersonalityVector,
        recent_turns: &[(String, String)],
        recent_events: &[Event],
        knowledge_augment: Option<&KnowledgeEventAugment>,
    ) -> Result<EventImpactEstimate> {
        self.warn_once();
        self.inner
            .estimate(
                llm,
                ollama_model,
                user_message,
                user_emotion,
                personality,
                recent_turns,
                recent_events,
                knowledge_augment,
            )
            .await
    }
}

impl Default for RemoteEventEstimatorPlaceholder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::llm::MockLlmClient;
    use crate::models::{Emotion, PersonalityVector};
    use std::sync::Arc;

    struct EnvUnsetGuard {
        key: &'static str,
        prev: Option<String>,
    }

    impl EnvUnsetGuard {
        fn set(key: &'static str, value: &str) -> Self {
            let prev = std::env::var(key).ok();
            std::env::set_var(key, value);
            Self { key, prev }
        }
    }

    impl Drop for EnvUnsetGuard {
        fn drop(&mut self) {
            match &self.prev {
                Some(v) => std::env::set_var(self.key, v),
                None => std::env::remove_var(self.key),
            }
        }
    }

    #[tokio::test]
    async fn builtin_v2_halves_rule_based_impact() {
        let _g = EnvUnsetGuard::set("OCLIVE_EVENT_IMPACT_LLM", "0");
        let llm: Arc<dyn LlmClient> = Arc::new(MockLlmClient {
            reply: String::new(),
        });
        let p = PersonalityVector::zero();
        let msg = "我很抱怨这个";
        let user_emotion = Emotion::Sad;
        let b = BuiltinEventEstimator
            .estimate(&llm, "m", msg, &user_emotion, &p, &[], &[], None)
            .await
            .unwrap();
        let v2 = BuiltinEventEstimatorV2
            .estimate(&llm, "m", msg, &user_emotion, &p, &[], &[], None)
            .await
            .unwrap();
        assert_eq!(b.event_type, v2.event_type);
        assert!((b.impact_factor - 2.0 * v2.impact_factor).abs() < 1e-9);
        assert!(b.impact_factor.abs() > 1e-6);
    }
}
