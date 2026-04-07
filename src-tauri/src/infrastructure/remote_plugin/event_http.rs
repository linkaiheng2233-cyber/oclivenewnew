//! JSON-RPC：`event.estimate` — 侧车返回 [`EventImpactEstimate`](crate::domain::event_impact_ai::EventImpactEstimate)。

use crate::domain::event_estimator::EventEstimator;
use crate::domain::event_impact_ai::EventImpactEstimate;
use crate::domain::BuiltinEventEstimator;
use crate::error::Result;
use crate::infrastructure::llm::LlmClient;
use crate::infrastructure::remote_plugin::config::RemotePluginHttpConfig;
use crate::infrastructure::remote_plugin::jsonrpc;
use crate::models::knowledge::KnowledgeEventAugment;
use crate::models::{Emotion, Event, PersonalityVector};
use async_trait::async_trait;
use serde_json::json;
use std::sync::Arc;

pub struct RemoteEventEstimatorHttp {
    client: reqwest::Client,
    cfg: RemotePluginHttpConfig,
    fallback: BuiltinEventEstimator,
}

impl RemoteEventEstimatorHttp {
    pub fn new(cfg: RemotePluginHttpConfig) -> Self {
        let client = reqwest::Client::builder()
            .timeout(cfg.timeout)
            .build()
            .expect("reqwest async client");
        Self {
            client,
            cfg,
            fallback: BuiltinEventEstimator,
        }
    }
}

#[async_trait]
impl EventEstimator for RemoteEventEstimatorHttp {
    async fn estimate(
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
        let params = json!({
            "ollama_model": ollama_model,
            "user_message": user_message,
            "user_emotion": user_emotion,
            "personality": personality,
            "recent_turns": recent_turns,
            "recent_events": recent_events,
            "knowledge_augment": knowledge_augment.map(|a| &a.by_event),
        });
        match jsonrpc::call_async(
            &self.client,
            &self.cfg.endpoint,
            "event.estimate",
            params,
            self.cfg.bearer_token.as_deref(),
        )
        .await
        {
            Ok(v) => {
                let est: EventImpactEstimate = serde_json::from_value(v).map_err(|e| {
                    crate::error::AppError::OllamaError(format!("event.estimate decode: {}", e))
                })?;
                Ok(est)
            }
            Err(e) => {
                log::warn!(
                    target: "oclive_plugin",
                    "event.estimate remote failed: {}; builtin fallback",
                    e
                );
                self.fallback
                    .estimate(
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
    }
}
