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
/// ## When to implement
///
/// - **谁**：事件检测后端（内置规则 + LLM、Remote HTTP 插件）。
/// - **何时**：角色需要**事件系统**（影响性格演化、记忆权重等）时。
///
/// ## When not to implement
///
/// - 角色包关闭 event 槽或始终使用内置估计且无需替换时。
/// - 仅做短回复、不持久化事件影响的实验角色可省略自定义实现。
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
