//! 关闭各 `default-*-providers` 特性时的桩实现，供 builtin 槽与 Remote 占位 / HTTP 回退使用。
#![allow(dead_code)]
// 各 `Disabled*` 仅在对应 `default-*-providers` 关闭时实例化；默认 `full` 下会未被引用。

use crate::domain::emotion_analyzer::EmotionResult;
use crate::domain::event_estimator::EventEstimator;
use crate::domain::event_impact_ai::EventImpactEstimate;
use crate::domain::memory_engine::MemoryEngine;
use crate::domain::prompt_assembler::PromptAssembler;
use crate::domain::prompt_builder::PromptInput;
use crate::error::Result;
use crate::infrastructure::llm::LlmClient;
use crate::models::knowledge::KnowledgeEventAugment;
use crate::models::{
    Emotion, Event, EventType, Memory, MemoryContext, PersonalitySource, PersonalityVector, Role,
};
use async_trait::async_trait;
use oclive_kernel_core::complex_emotion::{
    ComplexEmotionInput, ComplexEmotionOutput, ComplexEmotionProvider,
};
use oclive_kernel_core::memory_retrieval::{MemoryRetrieval, MemoryRetrievalInput};
use oclive_kernel_core::user_emotion_analyzer::UserEmotionAnalyzer;
use std::sync::Arc;

pub struct DisabledMemoryRetrieval;

impl MemoryRetrieval for DisabledMemoryRetrieval {
    fn rank_memories(&self, input: MemoryRetrievalInput<'_>) -> Vec<Memory> {
        let limit = input.limit.max(1);
        input.memories.iter().take(limit).cloned().collect()
    }

    fn build_context(&self, memories: &[Memory], max_tokens: usize) -> MemoryContext {
        MemoryEngine::build_context(memories, max_tokens)
    }

    fn search_memories(&self, keyword: &str, memories: &[Memory]) -> Vec<Memory> {
        MemoryEngine::search_memories(keyword, memories)
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

    fn top_topic_hint(&self, _role: &Role, _scene_id: &str) -> Option<String> {
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
