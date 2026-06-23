//! JSON-RPC: `event.estimate` — the sidecar returns [`EventImpactEstimate`](crate::domain::event_impact_ai::EventImpactEstimate).
//! `params` includes `personality_source` (`vector`|`profile`), consistent with the pack's `evolution`; the sidecar may ignore it.

use crate::domain::event_estimator::EventEstimator;
use crate::domain::event_impact_ai::EventImpactEstimate;
use crate::domain::ports::LlmClient;
use crate::domain::BuiltinEventEstimator;
use crate::error::Result;
use crate::infrastructure::high_risk_grants::HighRiskGrantStore;
use crate::infrastructure::remote_plugin::adapter::{decode_serde_value, RemotePluginAdapterAsync};
use crate::infrastructure::remote_plugin::config::RemotePluginHttpConfig;
use crate::models::knowledge::KnowledgeEventAugment;
use crate::models::{Emotion, Event, PersonalitySource, PersonalityVector};
use async_trait::async_trait;
use serde_json::json;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

const METHOD_EVENT_ESTIMATE: &str = "event.estimate";

pub struct RemoteEventEstimatorHttp {
    adapter: RemotePluginAdapterAsync,
    fallback: BuiltinEventEstimator,
}

impl RemoteEventEstimatorHttp {
    #[must_use]
    pub fn new(
        http_client: Arc<reqwest::Client>,
        cfg: RemotePluginHttpConfig,
        remote_fallback_allowed: Arc<AtomicBool>,
        high_risk_grants: Arc<HighRiskGrantStore>,
        network_grant_id: Option<String>,
    ) -> Self {
        Self {
            adapter: RemotePluginAdapterAsync::new(
                http_client,
                cfg,
                remote_fallback_allowed,
                high_risk_grants,
                network_grant_id,
            ),
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
        personality_source: PersonalitySource,
        recent_turns: &[(String, String)],
        recent_events: &[Event],
        knowledge_augment: Option<&KnowledgeEventAugment>,
    ) -> Result<EventImpactEstimate> {
        let params = json!({
            "ollama_model": ollama_model,
            "user_message": user_message,
            "user_emotion": user_emotion,
            "personality": personality,
            "personality_source": personality_source,
            "recent_turns": recent_turns,
            "recent_events": recent_events,
            "knowledge_augment": knowledge_augment.map(|a| &a.by_event),
        });
        self.adapter
            .call_with_async_builtin_fallback(
                METHOD_EVENT_ESTIMATE,
                params,
                |v| decode_serde_value(v, "event.estimate decode"),
                || {
                    self.fallback.estimate(
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
                },
            )
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn method_name_matches_remote_protocol() {
        assert_eq!(METHOD_EVENT_ESTIMATE, "event.estimate");
    }
}
