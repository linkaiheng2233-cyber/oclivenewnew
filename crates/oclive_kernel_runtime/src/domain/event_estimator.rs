//! 事件影响估计：`EventEstimator` trait 定义于 [`oclive_kernel_core::event_estimator`]；
//! 开启 **`default-event-providers`** 时 Builtin 薄壳在 **`oclive_event_builtin`**，经 [`KernelEventImpactEngine`](super::event_impact_bridge::KernelEventImpactEngine) 委托 [`estimate_event_impact`](super::event_impact_ai::estimate_event_impact)。
#![allow(clippy::too_many_arguments)] // `EventEstimator::estimate` 与编排层参数一致，不宜为 clippy 拆结构体

pub use oclive_kernel_core::event_estimator::EventEstimator;

#[cfg(not(feature = "default-event-providers"))]
use crate::domain::disabled_default_providers::DisabledEventEstimator;
#[cfg(feature = "default-event-providers")]
use crate::domain::event_impact_bridge::KernelEventImpactEngine;
use crate::error::Result;
use crate::infrastructure::llm::LlmClient;
use crate::models::knowledge::KnowledgeEventAugment;
use crate::models::{Emotion, Event, PersonalitySource, PersonalityVector};
use async_trait::async_trait;
#[cfg(feature = "default-event-providers")]
pub use oclive_event_builtin::{BuiltinEventEstimator, BuiltinEventEstimatorV2};
use oclive_kernel_models::EventImpactEstimate;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

#[must_use]
pub fn default_event_slot_v1() -> Arc<dyn EventEstimator> {
    #[cfg(feature = "default-event-providers")]
    {
        Arc::new(BuiltinEventEstimator::new(Arc::new(
            KernelEventImpactEngine,
        )))
    }
    #[cfg(not(feature = "default-event-providers"))]
    {
        Arc::new(DisabledEventEstimator)
    }
}

#[must_use]
pub fn default_event_slot_v2() -> Arc<dyn EventEstimator> {
    #[cfg(feature = "default-event-providers")]
    {
        Arc::new(BuiltinEventEstimatorV2::new(Arc::new(
            KernelEventImpactEngine,
        )))
    }
    #[cfg(not(feature = "default-event-providers"))]
    {
        Arc::new(DisabledEventEstimator)
    }
}

pub struct RemoteEventEstimatorPlaceholder {
    inner: Arc<dyn EventEstimator>,
    warned: AtomicBool,
}

impl RemoteEventEstimatorPlaceholder {
    pub fn new() -> Self {
        Self {
            inner: default_event_slot_v1(),
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
        personality_source: PersonalitySource,
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
                personality_source,
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

#[cfg(all(test, feature = "default-event-providers"))]
mod tests {
    use super::*;
    use crate::infrastructure::llm::MockLlmClient;
    use crate::models::{Emotion, PersonalitySource, PersonalityVector};
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
        use crate::domain::event_impact_bridge::KernelEventImpactEngine;
        let b = BuiltinEventEstimator::new(Arc::new(KernelEventImpactEngine))
            .estimate(
                &llm,
                "m",
                msg,
                &user_emotion,
                &p,
                PersonalitySource::Vector,
                &[],
                &[],
                None,
            )
            .await
            .unwrap();
        let v2 = BuiltinEventEstimatorV2::new(Arc::new(KernelEventImpactEngine))
            .estimate(
                &llm,
                "m",
                msg,
                &user_emotion,
                &p,
                PersonalitySource::Vector,
                &[],
                &[],
                None,
            )
            .await
            .unwrap();
        assert_eq!(b.event_type, v2.event_type);
        assert!((b.impact_factor - 2.0 * v2.impact_factor).abs() < 1e-9);
        assert!(b.impact_factor.abs() > 1e-6);
    }
}
