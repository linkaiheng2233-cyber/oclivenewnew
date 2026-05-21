//! JSON-RPC：`event.estimate` — 侧车返回 [`EventImpactEstimate`](crate::domain::event_impact_ai::EventImpactEstimate)。
//! `params` 含 `personality_source`（`vector`|`profile`），与包内 `evolution` 一致；侧车可忽略。

use crate::domain::event_estimator::EventEstimator;
use crate::domain::event_impact_ai::EventImpactEstimate;
use crate::domain::BuiltinEventEstimator;
use crate::error::{AppError, Result};
use crate::infrastructure::high_risk_grants::HighRiskGrantStore;
use crate::infrastructure::llm::LlmClient;
use crate::infrastructure::remote_fallback_policy::remote_fallback_load;
use crate::infrastructure::remote_plugin::config::RemotePluginHttpConfig;
use crate::infrastructure::remote_plugin::jsonrpc::{self, RemoteRpcChannel};
use crate::models::knowledge::KnowledgeEventAugment;
use crate::models::{Emotion, Event, PersonalitySource, PersonalityVector};
use async_trait::async_trait;
use serde_json::json;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

const METHOD_EVENT_ESTIMATE: &str = "event.estimate";

pub struct RemoteEventEstimatorHttp {
    client: reqwest::Client,
    cfg: RemotePluginHttpConfig,
    fallback: BuiltinEventEstimator,
    remote_fallback_allowed: Arc<AtomicBool>,
    high_risk_grants: Arc<HighRiskGrantStore>,
    network_grant_id: Option<String>,
}

impl RemoteEventEstimatorHttp {
    /// # Errors
    ///
    /// Returns [`Err`] with a human-readable message when the operation fails.
    pub fn new(
        cfg: RemotePluginHttpConfig,
        remote_fallback_allowed: Arc<AtomicBool>,
        high_risk_grants: Arc<HighRiskGrantStore>,
        network_grant_id: Option<String>,
    ) -> std::result::Result<Self, reqwest::Error> {
        let client = reqwest::Client::builder()
            .connect_timeout(cfg.connect_timeout())
            .timeout(cfg.timeout)
            .build()?;
        Ok(Self {
            client,
            cfg,
            fallback: BuiltinEventEstimator,
            remote_fallback_allowed,
            high_risk_grants,
            network_grant_id,
        })
    }

    fn network_grant(&self) -> Option<(&HighRiskGrantStore, &str)> {
        self.network_grant_id
            .as_deref()
            .map(|id| (self.high_risk_grants.as_ref(), id))
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
        match jsonrpc::call_async(
            RemoteRpcChannel::Plugin,
            &self.client,
            &self.cfg.endpoint,
            METHOD_EVENT_ESTIMATE,
            params,
            self.cfg.bearer_token.as_deref(),
            self.network_grant(),
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
                if matches!(e, AppError::HighRiskCapabilityNotGranted { .. }) {
                    return Err(e);
                }
                if remote_fallback_load(&self.remote_fallback_allowed) {
                    tracing::warn!(
                        target: "oclive_plugin",
                        "event.estimate remote failed endpoint={} err={}; fallback=builtin",
                        self.cfg.endpoint,
                        e
                    );
                    self.fallback
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
                } else {
                    Err(AppError::RemoteServiceUnavailable(format!(
                        "event.estimate remote failed endpoint={} err={}",
                        self.cfg.endpoint, e
                    )))
                }
            }
        }
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
