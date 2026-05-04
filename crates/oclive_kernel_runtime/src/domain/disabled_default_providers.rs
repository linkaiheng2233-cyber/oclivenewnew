//! 关闭各 `default-*-providers` 特性时的桩实现，供 builtin 槽与 Remote 占位 / HTTP 回退使用。
#![allow(dead_code)]
// 各 `Disabled*` 仅在对应 `default-*-providers` 关闭时实例化；默认 `full` 下会未被引用。

use crate::domain::emotion_analyzer::EmotionResult;
use crate::error::Result;
use crate::infrastructure::llm::LlmClient;
use crate::models::knowledge::KnowledgeEventAugment;
use crate::models::{
    Emotion, Event, EventType, Memory, MemoryContext, PersonalitySource, PersonalityVector,
};
use async_trait::async_trait;
use oclive_kernel_core::complex_emotion::{
    ComplexEmotionInput, ComplexEmotionOutput, ComplexEmotionProvider,
};
use oclive_kernel_core::event_estimator::EventEstimator;
use oclive_kernel_core::memory_retrieval::{MemoryRetrieval, MemoryRetrievalInput};
use oclive_kernel_core::prompt::{PromptAssembler, PromptInput, TopicHintContext};
use oclive_kernel_core::user_emotion_analyzer::UserEmotionAnalyzer;
use oclive_kernel_models::EventImpactEstimate;
use oclive_memory_builtin::classic;
use std::sync::Arc;

pub struct DisabledMemoryRetrieval;

impl MemoryRetrieval for DisabledMemoryRetrieval {
    fn rank_memories(&self, input: MemoryRetrievalInput<'_>) -> Vec<Memory> {
        let limit = input.limit.max(1);
        input.memories.iter().take(limit).cloned().collect()
    }

    fn build_context(&self, memories: &[Memory], max_tokens: usize) -> MemoryContext {
        classic::build_context(memories, max_tokens)
    }

    fn search_memories(&self, keyword: &str, memories: &[Memory]) -> Vec<Memory> {
        classic::search_memories(keyword, memories)
    }
}

/// `MemoryBackend::None`：无检索结果、空上下文（`MODULE_NONE_SEMANTICS.md` §1）。
pub struct NoneMemoryRetrieval;

impl MemoryRetrieval for NoneMemoryRetrieval {
    fn rank_memories(&self, _input: MemoryRetrievalInput<'_>) -> Vec<Memory> {
        Vec::new()
    }

    fn build_context(&self, _memories: &[Memory], _max_tokens: usize) -> MemoryContext {
        MemoryContext {
            memories: Vec::new(),
            total_tokens: 0,
        }
    }

    fn search_memories(&self, _keyword: &str, _memories: &[Memory]) -> Vec<Memory> {
        Vec::new()
    }
}

pub struct DisabledUserEmotionAnalyzer;

impl UserEmotionAnalyzer for DisabledUserEmotionAnalyzer {
    fn analyze(&self, _text: &str) -> Result<EmotionResult> {
        Ok(EmotionResult {
            joy: 0.0,
            sadness: 0.0,
            anger: 0.0,
            fear: 0.0,
            surprise: 0.0,
            disgust: 0.0,
            neutral: 1.0,
        })
    }
}

pub struct DisabledEventEstimator;

#[async_trait]
impl EventEstimator for DisabledEventEstimator {
    async fn estimate(
        &self,
        _llm: &Arc<dyn LlmClient>,
        _ollama_model: &str,
        _user_message: &str,
        _user_emotion: &Emotion,
        _personality: &PersonalityVector,
        _personality_source: PersonalitySource,
        _recent_turns: &[(String, String)],
        _recent_events: &[Event],
        _knowledge_augment: Option<&KnowledgeEventAugment>,
    ) -> Result<EventImpactEstimate> {
        Ok(EventImpactEstimate {
            event_type: EventType::Ignore,
            impact_factor: 0.0,
            confidence: 0.0,
        })
    }
}

pub struct DisabledPromptAssembler;

impl PromptAssembler for DisabledPromptAssembler {
    fn build_prompt(&self, _input: &PromptInput<'_>) -> String {
        String::new()
    }

    fn top_topic_hint(&self, _ctx: &TopicHintContext<'_>, _scene_id: &str) -> Option<String> {
        None
    }
}

/// `PromptBackend::None`：最小非空 prompt（`MODULE_NONE_SEMANTICS.md` §4）。
pub struct NonePromptAssembler;

impl PromptAssembler for NonePromptAssembler {
    fn build_prompt(&self, input: &PromptInput<'_>) -> String {
        format!(
            "[oclive] Prompt 模块未启用（backend=none）。以下为最小占位上下文。\n\n用户：\n{}",
            input.user_input
        )
    }

    fn top_topic_hint(&self, _ctx: &TopicHintContext<'_>, _scene_id: &str) -> Option<String> {
        None
    }
}

pub struct DisabledComplexEmotionProvider;

impl ComplexEmotionProvider for DisabledComplexEmotionProvider {
    fn resolve_turn(&self, _input: &ComplexEmotionInput) -> Result<ComplexEmotionOutput> {
        Ok(ComplexEmotionOutput {
            source: "disabled".to_string(),
            narrative_hint: None,
            labels: vec![],
            pattern: None,
            confidence: 0.0,
            intensity: 0.0,
            dissonance_score: 0.0,
            degraded_to_builtin: false,
        })
    }
}

#[cfg(test)]
mod disabled_semantics_tests {
    use super::*;
    use crate::infrastructure::llm::NoneLlmClient;
    use crate::models::Emotion;

    /// 与 `MODULE_NONE_SEMANTICS.md` §3（`event_type = Ignore`、零影响/零置信度）及
    /// `default-event-providers` 关闭时的降级链一致。
    #[tokio::test]
    async fn disabled_event_estimator_returns_ignore_zero_impact() {
        let est = DisabledEventEstimator;
        let llm: Arc<dyn LlmClient> = Arc::new(NoneLlmClient);
        let personality = PersonalityVector::zero();
        let out = est
            .estimate(
                &llm,
                "",
                "hi",
                &Emotion::Neutral,
                &personality,
                PersonalitySource::default(),
                &[],
                &[],
                None,
            )
            .await
            .expect("disabled estimator must not error");
        assert_eq!(out.event_type, EventType::Ignore);
        assert_eq!(out.impact_factor, 0.0);
        assert_eq!(out.confidence, 0.0);
    }
}
