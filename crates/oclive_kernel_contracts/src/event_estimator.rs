//! 事件影响估计可替换门面 trait。

#![allow(clippy::too_many_arguments)]

use crate::LlmClient;
use async_trait::async_trait;
use oclive_kernel_types::{
    Emotion, Event, EventImpactEstimate, KnowledgeEventAugment, PersonalitySource,
    PersonalityVector, Result,
};
use std::sync::Arc;

/// Estimates narrative/event impact from dialogue, personality, and recent context.
#[async_trait]
pub trait EventEstimator: Send + Sync {
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
    ) -> Result<EventImpactEstimate>;
}
