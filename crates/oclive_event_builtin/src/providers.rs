//! 进程内 Builtin / BuiltinV2：`EventEstimator` → [`EventImpactEngine`](oclive_kernel_core::EventImpactEngine)。

use async_trait::async_trait;
use oclive_kernel_core::error::Result;
use oclive_kernel_core::event_estimator::EventImpactEngine;
use oclive_kernel_core::llm::LlmClient;
use oclive_kernel_core::models::Emotion;
use oclive_kernel_core::EventEstimator;
use oclive_kernel_models::{Event, EventImpactEstimate, KnowledgeEventAugment, PersonalityVector};
use oclive_validation::PersonalitySource;
use std::sync::Arc;

pub struct BuiltinEventEstimator {
    engine: Arc<dyn EventImpactEngine>,
}

impl BuiltinEventEstimator {
    #[must_use]
    pub fn new(engine: Arc<dyn EventImpactEngine>) -> Self {
        Self { engine }
    }
}

#[async_trait]
impl EventEstimator for BuiltinEventEstimator {
    async fn estimate(
        &self,
        llm: &Arc<dyn LlmClient>,
        ollama_model: &str,
        user_message: &str,
        user_emotion: &Emotion,
        personality: &PersonalityVector,
        _personality_source: PersonalitySource,
        recent_turns: &[(String, String)],
        recent_events: &[Event],
        knowledge_augment: Option<&KnowledgeEventAugment>,
    ) -> Result<EventImpactEstimate> {
        self.engine
            .estimate_event_impact(
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

/// 在 [`BuiltinEventEstimator`] 的结果上将 `impact_factor` 乘以 **0.5**（更保守，用于验证 `event` 枚举可切换）。
pub struct BuiltinEventEstimatorV2 {
    inner: BuiltinEventEstimator,
}

impl BuiltinEventEstimatorV2 {
    #[must_use]
    pub fn new(engine: Arc<dyn EventImpactEngine>) -> Self {
        Self {
            inner: BuiltinEventEstimator::new(engine),
        }
    }
}

#[async_trait]
impl EventEstimator for BuiltinEventEstimatorV2 {
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
        let mut est = self
            .inner
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
            .await?;
        est.impact_factor *= 0.5;
        Ok(est)
    }
}
