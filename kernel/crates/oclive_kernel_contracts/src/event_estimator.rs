//! Replaceable facade trait for event impact estimation.

#![allow(clippy::too_many_arguments)]

use crate::LlmClient;
use async_trait::async_trait;
use oclive_kernel_types::{
    Emotion, Event, EventImpactEstimate, KnowledgeEventAugment, PersonalitySource,
    PersonalityVector, Result,
};
use std::sync::Arc;

/// Estimates narrative/event impact from dialogue, personality, and recent context.
///
/// ## When to implement
///
/// - **Who**: event detection backends (builtin rules + LLM, Remote HTTP plugin).
/// - **When**: when a role needs the **event system** (affecting personality evolution, memory weighting, etc.).
///
/// ## When not to implement
///
/// - When the role pack disables the event slot, or always uses the builtin estimate and needs no replacement.
/// - Experimental roles that only produce short replies and do not persist event impact may omit a custom implementation.
///
/// # Examples
///
/// ```no_run
/// use oclive_kernel_contracts::{EventEstimator, LlmClient};
/// use oclive_kernel_types::{Emotion, PersonalitySource, PersonalityVector};
/// use std::sync::Arc;
///
/// async fn estimate(
///     est: &dyn EventEstimator,
///     llm: Arc<dyn LlmClient>,
/// ) -> oclive_kernel_types::Result<()> {
///     let impact = est
///         .estimate(
///             &llm,
///             "qwen2.5:7b",
///             "今天天气不错",
///             &Emotion::Neutral,
///             &PersonalityVector::zero(),
///             PersonalitySource::default(),
///             &[],
///             &[],
///             None,
///         )
///         .await?;
///     let _ = impact;
///     Ok(())
/// }
/// ```
#[async_trait]
pub trait EventEstimator: Send + Sync {
    /// Estimates this turn's event impact (may call an LLM or a rules engine).
    ///
    /// # Errors
    ///
    /// Returns `Err` when the LLM call fails, the input context is invalid, or the estimate result cannot be deserialized.
    ///
    /// # Panics
    ///
    /// Does not panic.
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
