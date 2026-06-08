//! # Experimental core single-step execution (aligned with `co_present` sub-stages)
#![cfg(feature = "dual_core")]
//!
//! **Role**: maps `slot.<key>.<method>` in `pipeline.experimental` to co-present sub-stages
//! (emotion analysis, memory ranking, prompt assembly, etc.); does not reimplement business rules—reuses [`SlotRunner`](super::slot_runner::SlotRunner) and existing engines.
//!
//! **Upstream**: [`DualPipelineRunner`](super::dual_pipeline::DualPipelineRunner) calls [`ExperimentalStepCtx::run_method`] step by step after topological sort.
//! **Downstream**: co-present stage logic (via [`SlotRunner`](super::slot_runner::SlotRunner)).
//!
//! Allowed methods: see [`dual_pipeline_registry`](super::dual_pipeline_registry).

use crate::domain::agent_context::build_agent_input;
use crate::domain::chat_engine::context::load_recent_context;
use crate::domain::chat_engine::favor::{compute_favor_and_relation, FavorRelationInput};
use crate::domain::chat_engine::message_error::ProcessMessageError;
use crate::domain::chat_engine::plugin_resolve::resolve_plugins_for_session;
use crate::domain::chat_engine::turn_pipeline::build_complex_emotion_turn_input;
use crate::domain::chat_turn::relation_favor_for_key;
use crate::domain::emotion_analyzer::{EmotionAnalyzer, EmotionResult};
use crate::domain::memory_retrieval::MemoryRetrievalInput;
use crate::domain::plugin_host::ResolvedRolePlugins;
use crate::domain::prompt_builder::{effective_reply_quality_anchor, PromptInput};
use crate::domain::slot_runner::SlotRunner;
use crate::domain::user_identity_loader::resolve_active_user_identity;
use crate::error::AppError;
use crate::models::dto::{
    PresenceMode, SendMessageRequest, SendMessageResponse, API_VERSION, SCHEMA_VERSION,
};
use crate::models::{EventType, KnowledgeIndex, PersonalityVector, Role};
use crate::state::AppState;

use crate::domain::dual_pipeline_registry::required_slot_type_for_method;
use crate::domain::expert_routing::{execute_expert_route, ExpertTriggerMiss};
use oclive_validation::load_expert_routing_from_role_dir;

pub struct ExperimentalStepCtx<'a> {
    pub state: &'a AppState,
    pub role: &'a Role,
    pub req: &'a SendMessageRequest,
    pub scene_id: String,
    pub mrid: &'a str,
    pub srid: &'a str,
    pub pl: ResolvedRolePlugins,
    pub slot_runner: SlotRunner,
    pub user_message: &'a str,
    emotion_result: Option<EmotionResult>,
    personality: Option<PersonalityVector>,
    ranked_memories: Option<Vec<crate::models::Memory>>,
    assembled_prompt: Option<String>,
}

impl<'a> ExperimentalStepCtx<'a> {
    /// Resolve session plugins and slots; construct experimental step context.
    ///
    /// # Errors
    ///
    /// Returns [`ProcessMessageError`] when plugin resolution or session backend config is invalid.
    pub async fn new(
        state: &'a AppState,
        role: &'a Role,
        req: &'a SendMessageRequest,
        scene_id: String,
        mrid: &'a str,
        srid: &'a str,
    ) -> Result<Self, ProcessMessageError> {
        let pl = resolve_plugins_for_session(
            state.plugin_host_port(),
            role,
            Some(srid),
            &state.effective_plugin_backends_for_session(role, srid),
            state
                .effective_slot_registry_for_session(role, srid)
                .as_ref(),
        );
        Ok(Self {
            state,
            role,
            req,
            scene_id,
            mrid,
            srid,
            pl,
            slot_runner: SlotRunner,
            user_message: req.user_message.as_str(),
            emotion_result: None,
            personality: None,
            ranked_memories: None,
            assembled_prompt: None,
        })
    }

    /// Execute one step `slot.<registry_key>.<method>`.
    ///
    /// # Errors
    ///
    /// Returns [`ProcessMessageError`] on slot type mismatch, co-present sub-stage failure, or DB/plugin error;
    /// unimplemented method returns [`StepOutcome::Failed`] (caller degrades).
    pub async fn run_method(
        &mut self,
        registry_key: &str,
        method: &str,
    ) -> Result<StepOutcome, ProcessMessageError> {
        let Some(expected_type) = required_slot_type_for_method(method) else {
            return Ok(StepOutcome::Failed(format!(
                "slot.{registry_key}.{method} 未实现"
            )));
        };
        let entry = self
            .role
            .slot_registry
            .as_ref()
            .and_then(|r| r.get(registry_key))
            .ok_or_else(|| stage_err(format!("unknown registry key「{registry_key}」")))?;
        let slot_type = entry.slot_type.trim();
        if slot_type != expected_type {
            return Ok(StepOutcome::Failed(format!(
                "slot.{registry_key}.{method} 要求 type={expected_type}（当前 {slot_type}）"
            )));
        }

        match method {
            "analyze" => {
                self.ensure_emotion().await?;
                Ok(StepOutcome::Continue)
            }
            "retrieve" => {
                self.run_retrieve().await?;
                Ok(StepOutcome::Continue)
            }
            "detect" => {
                self.run_detect().await?;
                Ok(StepOutcome::Continue)
            }
            "assemble" => {
                self.run_assemble().await?;
                Ok(StepOutcome::Continue)
            }
            "generate" => Ok(StepOutcome::NeedsStableCompletion),
            "process" => self.run_agent_process().await,
            "resolve_turn" => {
                self.run_resolve_turn().await?;
                Ok(StepOutcome::Continue)
            }
            _ => Ok(StepOutcome::Failed(format!(
                "slot.{registry_key}.{method} 未实现"
            ))),
        }
    }

    async fn ensure_emotion(&mut self) -> Result<&EmotionResult, ProcessMessageError> {
        if self.emotion_result.is_none() {
            let er =
                SlotRunner::analyze_emotion(&self.pl, self.user_message).map_err(map_slot_err)?;
            self.emotion_result = Some(er);
        }
        self.emotion_result.as_ref().ok_or_else(|| {
            ProcessMessageError::dual_core_invalid("emotion_result unset after analyze")
        })
    }

    async fn ensure_personality(&mut self) -> Result<PersonalityVector, ProcessMessageError> {
        if let Some(ref p) = self.personality {
            return Ok(p.clone());
        }
        let p = self
            .state
            .get_current_personality(self.srid, self.role)
            .await
            .map_err(map_db_err)?;
        self.personality = Some(p.clone());
        Ok(p)
    }

    async fn run_retrieve(&mut self) -> Result<(), ProcessMessageError> {
        let mut memories = self
            .state
            .memory_repo
            .load_memories(self.srid, 10)
            .await
            .map_err(map_db_err)?;
        let scene_m = self
            .role
            .memory_config
            .as_ref()
            .map(|m| m.scene_weight_multiplier)
            .unwrap_or(1.0);
        crate::domain::chat_turn::weight_memories_for_scene(
            &mut memories,
            self.scene_id.as_str(),
            scene_m,
        );
        let ranked = SlotRunner::rank_memories(
            &self.pl,
            MemoryRetrievalInput {
                memories: &memories,
                user_query: self.user_message,
                scene_id: Some(self.scene_id.as_str()),
                limit: 8,
            },
        )
        .map_err(map_slot_err)?;
        self.ranked_memories = Some(ranked);
        Ok(())
    }

    async fn run_detect(&mut self) -> Result<(), ProcessMessageError> {
        let er = self.ensure_emotion().await?;
        let user_emotion = er.to_emotion();
        let personality = self.ensure_personality().await?;
        let ollama_model = self
            .role
            .resolve_ollama_model(self.state.ollama_model.as_str());
        let (_turns, recent_turns_for_event, recent_events_for_event) =
            load_recent_context(self.state, self.srid)
                .await
                .map_err(map_db_err)?;
        let knowledge_augment_opt = self
            .role
            .knowledge_index
            .as_ref()
            .map(|idx| idx.retrieve(self.user_message, Some(self.scene_id.as_str()), 8))
            .map(|chunks| KnowledgeIndex::merge_event_augment(chunks.as_slice()))
            .filter(|aug| !aug.is_empty());

        let _estimate = SlotRunner::estimate_event(
            &self.pl,
            ollama_model.as_str(),
            self.user_message,
            &user_emotion,
            &personality,
            self.role.evolution_config.personality_source,
            &recent_turns_for_event,
            &recent_events_for_event,
            knowledge_augment_opt.as_ref(),
        )
        .await
        .map_err(map_slot_err)?;
        Ok(())
    }

    async fn run_assemble(&mut self) -> Result<(), ProcessMessageError> {
        let er = self.ensure_emotion().await?;
        let user_emotion_prompt = EmotionAnalyzer::format_for_prompt(er);
        let personality = self.ensure_personality().await?;
        let memories = if let Some(ref m) = self.ranked_memories {
            m.clone()
        } else {
            let mut mem = self
                .state
                .memory_repo
                .load_memories(self.srid, 10)
                .await
                .map_err(map_db_err)?;
            let scene_m = self
                .role
                .memory_config
                .as_ref()
                .map(|m| m.scene_weight_multiplier)
                .unwrap_or(1.0);
            crate::domain::chat_turn::weight_memories_for_scene(
                &mut mem,
                self.scene_id.as_str(),
                scene_m,
            );
            SlotRunner::rank_memories(
                &self.pl,
                MemoryRetrievalInput {
                    memories: &mem,
                    user_query: self.user_message,
                    scene_id: Some(self.scene_id.as_str()),
                    limit: self.state.host_profile.memory_retrieval.retrieval_limit(),
                },
            )
            .map_err(map_slot_err)?
        };
        let resolved_identity = resolve_active_user_identity(
            self.state,
            self.role,
            self.srid,
            Some(self.scene_id.as_str()),
        )
        .await
        .map_err(map_db_err)?;
        let user_relation_key = resolved_identity.relation_key.clone();
        let rf = relation_favor_for_key(self.role, user_relation_key.as_str());
        let rel_id = self
            .state
            .db_manager
            .get_relation_state_for_identity(self.srid, user_relation_key.as_str())
            .await
            .map_err(map_db_err)?;
        let rel_global = self
            .state
            .db_manager
            .get_relation_state(self.srid)
            .await
            .map_err(map_db_err)?;
        let relation_before = rel_id
            .or(rel_global)
            .unwrap_or_else(|| "Stranger".to_string());
        let favorability_before = self
            .state
            .db_manager
            .favorability_for_identity_with_runtime_fallback(self.srid, user_relation_key.as_str())
            .await
            .map_err(map_db_err)?;
        let neutral_event = EventType::Ignore;
        let event_runtime = self
            .state
            .db_manager
            .get_event_impact_factor(self.srid)
            .await
            .ok()
            .flatten()
            .unwrap_or(self.role.evolution_config.event_impact_factor);
        let (_favor_delta, relation_after) = compute_favor_and_relation(&FavorRelationInput {
            relation_before: relation_before.as_str(),
            favorability_before,
            ai_event_type: &neutral_event,
            ai_impact_factor_final: 0.0,
            event_runtime,
            favor_mult: rf.favor_mult,
            event_confidence: 0.0,
            recent_events_for_event: &[],
        });
        let scene_label = self
            .state
            .storage
            .scene_display_name_for_role(self.role, self.scene_id.as_str());
        let scene_detail_buf = self
            .state
            .storage
            .scene_prompt_enrichment_for_role(self.role, self.scene_id.as_str());
        let top_topic = SlotRunner::top_topic_hint(&self.pl, self.role, self.scene_id.as_str());
        let mut topic_line = top_topic
            .map(|t| format!("在「{scene_label}」下，你们可能会多聊「{t}」相关的事。"))
            .unwrap_or_default();
        let expert_frag = self.state.session_cache.expert_prompt_enhance(self.srid);
        if !expert_frag.trim().is_empty() {
            if !topic_line.is_empty() {
                topic_line.push('\n');
            }
            topic_line.push_str(expert_frag.trim());
        }
        let mutable_for_prompt = self
            .state
            .db_manager
            .get_mutable_personality(self.srid)
            .await
            .map_err(map_db_err)?;
        let prev_hint = self
            .state
            .session_cache
            .stored_complex_emotion_narrative_hint(self.srid);
        let transition = crate::domain::relation_transition::consume_relation_transition_at_turn_start(
            &self.state.session_cache,
            self.state.db_manager.as_ref(),
            self.role,
            self.srid,
        )
        .await
        .map_err(map_db_err)?;
        let host_state_hint = self
            .state
            .host_profile
            .state_expression_hint(favorability_before);
        let prompt = SlotRunner::build_prompt(
            &self.pl,
            &PromptInput {
                role: self.role,
                personality: &personality,
                memories: &memories,
                user_input: self.user_message,
                user_emotion: user_emotion_prompt.as_str(),
                user_relation_id: user_relation_key.as_str(),
                relation_hint: resolved_identity.relation_hint.as_str(),
                user_identity_template: resolved_identity.template_body.as_str(),
                user_identity_id: resolved_identity.identity_id.as_str(),
                relation_before: relation_before.as_str(),
                favorability_before,
                relation_preview: relation_after.as_str(),
                favorability_preview: favorability_before,
                event_type: &neutral_event,
                impact_factor: 0.0,
                scene_label: scene_label.as_str(),
                scene_detail: scene_detail_buf.as_str(),
                topic_hint_line: topic_line.as_str(),
                life_context_line: "",
                worldview_snippet: "",
                mutable_personality: mutable_for_prompt.as_str(),
                reply_quality_anchor: effective_reply_quality_anchor(self.role),
                previous_complex_emotion_narrative_hint: prev_hint.as_str(),
                host_prompt_overlay: "",
                host_state_expression_hint: host_state_hint,
                relation_transition_hint: transition.hint.as_str(),
            },
        )
        .map_err(map_slot_err)?;
        self.assembled_prompt = Some(prompt);
        Ok(())
    }

    async fn run_resolve_turn(&mut self) -> Result<(), ProcessMessageError> {
        let er = self.ensure_emotion().await?.clone();
        let prev_hint = self
            .state
            .session_cache
            .stored_complex_emotion_narrative_hint(self.srid);
        let (recent_turns, _a, _b) = load_recent_context(self.state, self.srid)
            .await
            .map_err(map_db_err)?;
        let ce_input = build_complex_emotion_turn_input(
            self.mrid,
            self.scene_id.as_str(),
            self.user_message,
            &er,
            prev_hint,
            &recent_turns,
        );
        let _out =
            SlotRunner::resolve_complex_emotion(&self.pl, &ce_input).map_err(map_slot_err)?;
        Ok(())
    }

    async fn run_agent_process(&mut self) -> Result<StepOutcome, ProcessMessageError> {
        if self.state.host_profile.skip_agent {
            return Ok(StepOutcome::Continue);
        }
        let model = self
            .role
            .resolve_ollama_model(self.state.ollama_model.as_str());
        let agent_input = build_agent_input(
            self.state,
            self.role,
            self.srid,
            self.scene_id.as_str(),
            self.user_message,
            model.as_str(),
            self.state.plugins.agent_mcp_bridge().as_ref(),
            None,
        )
        .await
        .map_err(map_db_err)?;
        let agent_out = self
            .pl
            .agent
            .process(agent_input)
            .await
            .map_err(map_slot_err)?;
        if !agent_out.handled {
            return Ok(StepOutcome::Continue);
        }
        self.state
            .db_manager
            .set_user_presence_scene(self.srid, self.scene_id.as_str())
            .await
            .map_err(map_db_err)?;
        let emotion_result = self
            .pl
            .emotion
            .analyze(self.user_message)
            .map_err(map_slot_err)?;
        let resolved_identity = resolve_active_user_identity(
            self.state,
            self.role,
            self.srid,
            Some(self.scene_id.as_str()),
        )
        .await
        .map_err(map_db_err)?;
        let user_relation_key = resolved_identity.relation_key.clone();
        let rel_id = self
            .state
            .db_manager
            .get_relation_state_for_identity(self.srid, user_relation_key.as_str())
            .await
            .map_err(map_db_err)?;
        let runtime = self
            .state
            .db_manager
            .get_role_runtime_snapshot(self.srid)
            .await
            .map_err(map_db_err)?;
        let rel_global = runtime.as_ref().and_then(|r| r.relation_state.clone());
        let relation_state = rel_id
            .or(rel_global)
            .unwrap_or_else(|| "Stranger".to_string());
        let favor_current = self
            .state
            .db_manager
            .favorability_for_identity_with_runtime_fallback(self.srid, user_relation_key.as_str())
            .await
            .map_err(map_db_err)?;
        let portrait_emotion = runtime
            .as_ref()
            .and_then(|r| r.emotion.clone())
            .unwrap_or_else(|| "neutral".to_string());
        let scene_id = self.scene_id.clone();
        Ok(StepOutcome::AgentComplete(Box::new(SendMessageResponse {
            api_version: API_VERSION,
            schema: SCHEMA_VERSION,
            presence_mode: PresenceMode::CoPresent,
            relation_state,
            reply: agent_out.reply,
            emotion: crate::domain::chat_engine::emotion_to_dto(&emotion_result),
            bot_emotion: portrait_emotion.clone(),
            portrait_emotion,
            favorability_delta: 0.0,
            favorability_current: favor_current as f32,
            events: vec![],
            scene_id,
            offer_destination_picker: false,
            offer_together_travel: false,
            reply_is_fallback: false,
            llm_fallback_reason: None,
            knowledge_chunks_in_prompt: 0,
            timestamp: chrono::Utc::now().timestamp_millis(),
            user_message_id: None,
            assistant_message_id: None,
            user_message_timestamp: None,
            assistant_message_timestamp: None,
            chat_persist_failed: None,
            chat_persist_error: None,
            dual_core_degraded: None,
            raw_reply: None,
        })))
    }

    /// Expert sub-route via `slot.expert.invoke`; skip when no route matches.
    ///
    /// # Errors
    ///
    /// May return error when a sub-step fails and `fallback` is not `skip`; on `skip`, logs warn and continues main pipeline.
    pub async fn run_expert_invoke(&mut self) -> Result<StepOutcome, ProcessMessageError> {
        let role_dir = self.state.storage.roles_dir().join(self.role.id.as_str());
        let Some(doc) = load_expert_routing_from_role_dir(&role_dir) else {
            return Ok(StepOutcome::Continue);
        };
        match execute_expert_route(self, &doc).await? {
            Ok(outcome) => Ok(outcome),
            Err(ExpertTriggerMiss) => Ok(StepOutcome::Continue),
        }
    }
}

pub enum StepOutcome {
    Continue,
    NeedsStableCompletion,
    AgentComplete(Box<SendMessageResponse>),
    Failed(String),
}

fn stage_err(msg: impl Into<String>) -> ProcessMessageError {
    ProcessMessageError::dual_core_invalid(msg)
}

fn map_slot_err(e: AppError) -> ProcessMessageError {
    ProcessMessageError::dual_core(e)
}

fn map_db_err(e: AppError) -> ProcessMessageError {
    ProcessMessageError::dual_core(e)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::dual_pipeline_registry::EXPERIMENTAL_METHOD_SPECS;

    #[test]
    fn registry_covers_seven_slot_types() {
        let types: std::collections::HashSet<_> = EXPERIMENTAL_METHOD_SPECS
            .iter()
            .map(|s| s.slot_type)
            .collect();
        for t in [
            "memory",
            "emotion",
            "event",
            "prompt",
            "llm",
            "agent",
            "complex_emotion",
        ] {
            assert!(types.contains(t), "missing type {t}");
        }
    }

    #[test]
    fn required_slot_type_matches_registry() {
        assert_eq!(required_slot_type_for_method("detect"), Some("event"));
        assert_eq!(required_slot_type_for_method("nope"), None);
    }

    #[test]
    fn registry_maps_each_method_to_co_present_stage() {
        for spec in EXPERIMENTAL_METHOD_SPECS {
            assert!(!spec.co_present_stage.is_empty());
            assert!(!spec.method.is_empty());
        }
    }
}
