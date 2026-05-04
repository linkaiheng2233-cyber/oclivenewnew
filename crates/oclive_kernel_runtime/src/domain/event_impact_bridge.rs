//! `EventImpactEngine` 桥接：将设施薄壳委托到 [`crate::domain::event_impact_ai::estimate_event_impact`]。

use async_trait::async_trait;
use oclive_kernel_core::error::Result;
use oclive_kernel_core::event_estimator::EventImpactEngine;
use oclive_kernel_core::llm::LlmClient;
use oclive_kernel_core::models::Emotion;
use oclive_kernel_models::{Event, EventImpactEstimate, KnowledgeEventAugment, PersonalityVector};
use std::sync::Arc;

/// 运行时默认的进程内事件影响引擎（规则 + 可选 LLM）。
pub struct KernelEventImpactEngine;

#[async_trait]
impl EventImpactEngine for KernelEventImpactEngine {
    async fn estimate_event_impact(
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
        crate::domain::event_impact_ai::estimate_event_impact(
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
