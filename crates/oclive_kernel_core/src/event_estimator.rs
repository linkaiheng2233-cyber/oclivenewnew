//! 事件影响估计可替换门面（**Builtin 算法主体**在 **`oclive_event_builtin`**；Remote / 目录插件可替换；**`EventImpactEstimate`** DTO 在 **`oclive_kernel_models`**）。

#![allow(clippy::too_many_arguments)] // 与编排层 `process_message` 参数一致

use crate::error::Result;
use crate::llm::LlmClient;
use crate::models::Emotion;
use async_trait::async_trait;
use oclive_kernel_models::{Event, EventImpactEstimate, KnowledgeEventAugment, PersonalityVector};
use oclive_validation::PersonalitySource;
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
        personality_source: PersonalitySource,
        recent_turns: &[(String, String)],
        recent_events: &[Event],
        knowledge_augment: Option<&KnowledgeEventAugment>,
    ) -> Result<EventImpactEstimate>;
}

/// 进程内事件影响 **算法**：`estimate_event_impact` 由 **`oclive_event_builtin::event_impact_ai`** 提供；
/// `BuiltinEventEstimator` 薄壳在同 crate，经 runtime **`KernelEventImpactEngine`** 注册到 **`PluginHost`**。
#[async_trait]
pub trait EventImpactEngine: Send + Sync {
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
    ) -> Result<EventImpactEstimate>;
}
