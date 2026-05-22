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
///             &PersonalityVector::default(),
///             PersonalitySource::Manifest,
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
    /// 估计本回合事件影响（可调用 LLM 或规则引擎）。
    ///
    /// # Errors
    ///
    /// 当 LLM 调用失败、输入上下文不合法或估计结果无法反序列化时返回 `Err`。
    ///
    /// # Panics
    ///
    /// 不 panic。
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
