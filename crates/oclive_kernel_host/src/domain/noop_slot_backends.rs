//! Zero-cost backends for `plugin_backends.* = none` (see MODULE_NONE_SEMANTICS.md).

use crate::domain::agent::{AgentInput, AgentOutput, AgentProvider};
use crate::domain::ports::LlmClient;
use crate::domain::{EventEstimator, MemoryRetrieval, PromptAssembler, UserEmotionAnalyzer};
use crate::error::Result;
use async_trait::async_trait;
use oclive_kernel_types::{
    Emotion, EmotionResult, Event, EventImpactEstimate, EventType, KnowledgeEventAugment, Memory,
    MemoryContext, MemoryRetrievalInput, PersonalitySource, PersonalityVector, PromptInput, Role,
};
use std::sync::Arc;

pub struct NoopMemoryRetrieval;

impl MemoryRetrieval for NoopMemoryRetrieval {
    fn rank_memories(&self, _input: MemoryRetrievalInput<'_>) -> Result<Vec<Memory>> {
        Ok(Vec::new())
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

pub struct NoopUserEmotionAnalyzer;

impl UserEmotionAnalyzer for NoopUserEmotionAnalyzer {
    fn analyze(&self, _text: &str) -> Result<EmotionResult> {
        Ok(EmotionResult {
            joy: 0.0,
            sadness: 0.0,
            anger: 0.0,
            fear: 0.0,
            surprise: 0.0,
            disgust: 0.0,
            neutral: 1.0,
            extension: None,
        })
    }
}

pub struct NoopEventEstimator;

#[async_trait]
impl EventEstimator for NoopEventEstimator {
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
            confidence: 1.0,
        })
    }
}

pub struct NoopPromptAssembler;

impl PromptAssembler for NoopPromptAssembler {
    fn build_prompt(&self, _input: &PromptInput<'_>) -> Result<String> {
        Err(crate::error::AppError::InvalidParameter(
            "plugin_backends.prompt=none is not allowed on the co-present dialogue path".into(),
        ))
    }

    fn top_topic_hint(&self, _role: &Role, _scene_id: &str) -> Option<String> {
        None
    }
}

pub struct NoopLlmClient;

#[async_trait]
impl LlmClient for NoopLlmClient {
    async fn generate(&self, _model: &str, _prompt: &str) -> Result<String> {
        Err(crate::error::AppError::InvalidParameter(
            "plugin_backends.llm=none is not allowed on the co-present dialogue path".into(),
        ))
    }

    async fn generate_tag(&self, _model: &str, _prompt: &str) -> Result<String> {
        Err(crate::error::AppError::InvalidParameter(
            "plugin_backends.llm=none is not allowed on the co-present dialogue path".into(),
        ))
    }

    async fn startup_probe(&self) -> Result<()> {
        Ok(())
    }
}

pub struct NoopAgentProvider;

#[async_trait]
impl AgentProvider for NoopAgentProvider {
    async fn process(&self, _input: AgentInput) -> Result<AgentOutput> {
        Ok(AgentOutput {
            handled: false,
            reply: String::new(),
        })
    }
}
