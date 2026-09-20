//! Pluggable facade for event impact estimation; defaults to the shared core of
//! [`event_impact_ai`](super::event_impact_ai).
#![allow(clippy::too_many_arguments)]

use crate::domain::event_impact_ai::EventImpactEstimate;
use crate::domain::ports::LlmClient;
use crate::error::Result;
use crate::models::knowledge::KnowledgeEventAugment;
use crate::models::{Emotion, Event, PersonalitySource, PersonalityVector};
use async_trait::async_trait;
pub use oclive_kernel_contracts::EventEstimator;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub struct BuiltinEventEstimator;

#[async_trait]
impl EventEstimator for BuiltinEventEstimator {
    /// The product estimate entry: it runs the one shared execution
    /// `super::event_impact_ai::estimate_event_impact_core` exactly
    /// once and returns its **numeric** projection.
    ///
    /// It passes no Base context, because the product call has none; the same core's narrative
    /// projection is read only by the crate-private Base view, and this entry never inspects it.
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
        let core = super::event_impact_ai::estimate_event_impact_core(
            llm,
            ollama_model,
            user_message,
            user_emotion,
            personality,
            recent_turns,
            recent_events,
            knowledge_augment,
            true,
            None,
        )
        .await?;
        Ok(core.estimate)
    }
}

pub struct RemoteEventEstimatorPlaceholder {
    inner: BuiltinEventEstimator,
    warned: AtomicBool,
}

impl RemoteEventEstimatorPlaceholder {
    #[must_use]
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
            tracing::warn!(
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
