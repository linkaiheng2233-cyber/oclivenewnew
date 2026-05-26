//! # 实验核单步执行（与 `co_present` 子阶段对齐）
#![cfg(feature = "dual_core")]
//!
//! **角色**：将 `pipeline.experimental` 中的 `slot.<key>.<method>` 映射到共景子阶段
//!（情绪分析、记忆排序、Prompt 组装等）；不重复实现业务规则，只复用 [`SlotRunner`](super::slot_runner::SlotRunner) 与现有引擎。
//!
//! **上游**：[`DualPipelineRunner`](super::dual_pipeline::DualPipelineRunner) 拓扑排序后逐步调用 [`ExperimentalStepCtx::run_method`].
//! **下游**：`co_present` 各阶段逻辑（通过 `CoPresentSlotRunner` 等）。
//!
//! method 合法集合见 [`dual_pipeline_registry`](super::dual_pipeline_registry)。

use crate::domain::agent::AgentInput;
use crate::domain::chat_engine::context::load_recent_context;
use crate::domain::chat_engine::favor::{compute_favor_and_relation, FavorRelationInput};
use crate::domain::chat_engine::message_error::ProcessMessageError;
use crate::domain::chat_engine::plugin_resolve::resolve_plugins_for_session;
use crate::domain::chat_turn::relation_favor_for_key;
use crate::domain::complex_emotion::affect_metrics_from_seven_dim;
use crate::domain::emotion_analyzer::{EmotionAnalyzer, EmotionResult};
use crate::domain::memory_retrieval::MemoryRetrievalInput;
use crate::domain::plugin_host::ResolvedRolePlugins;
use crate::domain::prompt_builder::{effective_reply_quality_anchor, PromptInput};
use crate::domain::slot_runner::{CoPresentSlotRunner, SlotRunner};
use crate::domain::user_identity::resolve_effective_user_relation_key;
use crate::error::AppError;
use crate::models::dto::{
    PresenceMode, SendMessageRequest, SendMessageResponse, API_VERSION, SCHEMA_VERSION,
};
use crate::models::{EventType, KnowledgeIndex, PersonalityVector, Role};
use crate::state::AppState;

use crate::domain::dual_pipeline::topological_sort_pipeline_steps;
use crate::domain::dual_pipeline_registry::required_slot_type_for_method;
use oclive_validation::{
    load_expert_routing_from_role_dir, match_expert_route, parse_pipeline_action_kind,
    ExpertFallback, ExpertRouteStep, PipelineActionKind, PipelineStep,
};

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
    /// 解析会话插件与槽位，构造实验步上下文。
    ///
    /// # Errors
    ///
    /// 插件解析或会话后端配置无效时返回 [`ProcessMessageError`]。
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
            state.effective_slot_registry_for_session(role, srid).as_ref(),
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

    /// 执行单步 `slot.<registry_key>.<method>`。
    ///
    /// # Errors
    ///
    /// 槽位类型不匹配、共景子阶段失败或 DB/插件错误时返回 [`ProcessMessageError`]；
    /// 未实现 method 时返回 [`StepOutcome::Failed`]（由调用方降级）。
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
            let er = self
                .slot_runner
                .analyze_emotion(&self.pl, self.user_message)
                .map_err(map_slot_err)?;
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
        let ranked = self
            .slot_runner
            .rank_memories(
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
        let ollama_model = self.role.resolve_ollama_model(self.state.ollama_model.as_str());
        let (_turns, recent_turns_for_event, recent_events_for_event) =
            load_recent_context(self.state, self.srid)
                .await
                .map_err(map_db_err)?;
        let knowledge_augment_opt = self
            .role
            .knowledge_index
            .as_ref()
            .map(|idx| {
                idx.retrieve(self.user_message, Some(self.scene_id.as_str()), 8)
            })
            .map(|chunks| KnowledgeIndex::merge_event_augment(chunks.as_slice()))
            .filter(|aug| !aug.is_empty());

        let _estimate = self
            .slot_runner
            .estimate_event(
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
            self.slot_runner
                .rank_memories(
                    &self.pl,
                    MemoryRetrievalInput {
                        memories: &mem,
                        user_query: self.user_message,
                        scene_id: Some(self.scene_id.as_str()),
                        limit: 8,
                    },
                )
                .map_err(map_slot_err)?
        };
        let user_relation_key = resolve_effective_user_relation_key(
            self.state,
            self.role,
            self.srid,
            Some(self.scene_id.as_str()),
        )
        .await
        .map_err(map_db_err)?;
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
        let top_topic = self
            .slot_runner
            .top_topic_hint(&self.pl, self.role, self.scene_id.as_str());
        let topic_line = top_topic
            .map(|t| format!("在「{scene_label}」下，你们可能会多聊「{t}」相关的事。"))
            .unwrap_or_default();
        let mutable_for_prompt = self
            .state
            .db_manager
            .get_mutable_personality(self.srid)
            .await
            .map_err(map_db_err)?;
        let prev_hint = self.state.session_cache.stored_complex_emotion_narrative_hint(self.srid);
        let prompt = self
            .slot_runner
            .build_prompt(
                &self.pl,
                &PromptInput {
                    role: self.role,
                    personality: &personality,
                    memories: &memories,
                    user_input: self.user_message,
                    user_emotion: user_emotion_prompt.as_str(),
                    user_relation_id: user_relation_key.as_str(),
                    relation_hint: rf.relation_hint,
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
                },
            )
            .map_err(map_slot_err)?;
        self.assembled_prompt = Some(prompt);
        Ok(())
    }

    async fn run_resolve_turn(&mut self) -> Result<(), ProcessMessageError> {
        let er = self.ensure_emotion().await?.clone();
        let prev_hint = self.state.session_cache.stored_complex_emotion_narrative_hint(self.srid);
        let (recent_turns, _a, _b) = load_recent_context(self.state, self.srid)
            .await
            .map_err(map_db_err)?;
        let (prev_user, prev_bot) = recent_turns
            .last()
            .map(|(u, b)| (Some(u.clone()), b.clone()))
            .unwrap_or((None, String::new()));
        let (uv, ud) = affect_metrics_from_seven_dim(&er);
        let _out = self
            .slot_runner
            .resolve_complex_emotion(
                &self.pl,
                &crate::domain::complex_emotion::ComplexEmotionInput {
                    role_id: self.mrid.to_string(),
                    scene_id: self.scene_id.clone(),
                    user_message: self.user_message.to_string(),
                    bot_reply: prev_bot,
                    recent_dialogue_summary: None,
                    previous_narrative_hint: prev_hint,
                    user_valence: Some(uv),
                    user_dominance: Some(ud),
                    previous_user_message: prev_user,
                },
            )
            .map_err(map_slot_err)?;
        Ok(())
    }

    async fn run_agent_process(&mut self) -> Result<StepOutcome, ProcessMessageError> {
        let agent_out = self
            .pl
            .agent
            .process(AgentInput {
                role_id: self.mrid.to_string(),
                session_namespace: self.srid.to_string(),
                message: self.req.user_message.clone(),
                model: self.role.resolve_ollama_model(self.state.ollama_model.as_str()),
            })
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
        let user_relation_key = resolve_effective_user_relation_key(
            self.state,
            self.role,
            self.srid,
            Some(self.scene_id.as_str()),
        )
        .await
        .map_err(map_db_err)?;
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
        let relation_state = rel_id
            .or(rel_global)
            .unwrap_or_else(|| "Stranger".to_string());
        let favor_current = self
            .state
            .db_manager
            .favorability_for_identity_with_runtime_fallback(self.srid, user_relation_key.as_str())
            .await
            .map_err(map_db_err)?;
        let portrait_emotion = self
            .state
            .db_manager
            .get_current_emotion(self.srid)
            .await
            .map_err(map_db_err)?
            .unwrap_or_else(|| "neutral".to_string());
        let scene_id = self.scene_id.clone();
        Ok(StepOutcome::AgentComplete(SendMessageResponse {
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
            knowledge_chunks_in_prompt: 0,
            timestamp: chrono::Utc::now().timestamp_millis(),
        }))
    }

    /// 执行专家路由子流程（`slot.expert.invoke`）；无匹配路由时跳过。
    ///
    /// # Errors
    ///
    /// 子步骤失败且 `fallback` 非 `skip` 时可能返回错误；`skip` 时记录 warn 并继续主 pipeline。
    pub async fn run_expert_invoke(&mut self) -> Result<StepOutcome, ProcessMessageError> {
        let role_dir = self.state.storage.roles_dir().join(self.role.id.as_str());
        let Some(doc) = load_expert_routing_from_role_dir(&role_dir) else {
            return Ok(StepOutcome::Continue);
        };
        let Some(route) = match_expert_route(&doc, self.scene_id.as_str(), self.user_message) else {
            return Ok(StepOutcome::Continue);
        };
        if route.steps.is_empty() {
            return Ok(StepOutcome::Continue);
        }

        let pipeline_steps: Vec<PipelineStep> = route
            .steps
            .iter()
            .map(|s: &ExpertRouteStep| PipelineStep {
                action: s.action.clone(),
                depends_on: s.depends_on.clone(),
            })
            .collect();

        let ordered = match topological_sort_pipeline_steps(&pipeline_steps) {
            Ok(o) => o,
            Err(e) => {
                tracing::warn!(
                    target: "oclive_expert",
                    session_ns = %self.srid,
                    error = %e,
                    "专家路由 steps 拓扑排序失败，跳过专家流程"
                );
                return Ok(StepOutcome::Continue);
            }
        };

        let fallback = doc.fallback_mode();
        let mut wants_stable_completion = false;

        for step in ordered {
            let outcome = match parse_pipeline_action_kind(step.action.as_str()) {
                Ok(PipelineActionKind::ExpertInvoke) => Ok(StepOutcome::Continue),
                Ok(PipelineActionKind::Slot {
                    registry_key,
                    method,
                }) => self.run_method(&registry_key, method.as_str()).await,
                Err(e) => Ok(StepOutcome::Failed(e)),
            };
            match outcome {
                Ok(StepOutcome::Continue) => {}
                Ok(StepOutcome::NeedsStableCompletion) => wants_stable_completion = true,
                Ok(StepOutcome::AgentComplete(resp)) => return Ok(StepOutcome::AgentComplete(resp)),
                Ok(StepOutcome::Failed(msg)) => {
                    tracing::warn!(
                        target: "oclive_expert",
                        session_ns = %self.srid,
                        action = %step.action,
                        error = %msg,
                        "专家子步骤失败"
                    );
                    return Self::apply_expert_fallback(self, fallback, &msg).await;
                }
                Err(e) => {
                    tracing::warn!(
                        target: "oclive_expert",
                        session_ns = %self.srid,
                        action = %step.action,
                        error = %e,
                        "专家子步骤失败"
                    );
                    return Self::apply_expert_fallback(self, fallback, &e.to_string()).await;
                }
            }
        }

        if wants_stable_completion {
            Ok(StepOutcome::NeedsStableCompletion)
        } else {
            Ok(StepOutcome::Continue)
        }
    }

    async fn apply_expert_fallback(
        ctx: &mut Self,
        fallback: ExpertFallback,
        reason: &str,
    ) -> Result<StepOutcome, ProcessMessageError> {
        match fallback {
            ExpertFallback::Skip => Ok(StepOutcome::Continue),
            ExpertFallback::RetryWithDefault => {
                let key = ctx
                    .role
                    .slot_registry
                    .as_ref()
                    .and_then(|r| {
                        r.iter()
                            .find(|(_, e)| e.slot_type.trim() == "llm")
                            .map(|(k, _)| k.clone())
                    })
                    .unwrap_or_else(|| "llm".into());
                tracing::warn!(
                    target: "oclive_expert",
                    session_ns = %ctx.srid,
                    llm_key = %key,
                    reason = %reason,
                    "专家流程降级：使用默认 LLM 重试 generate"
                );
                match ctx.run_method(key.as_str(), "generate").await? {
                    StepOutcome::NeedsStableCompletion | StepOutcome::Continue => {
                        Ok(StepOutcome::NeedsStableCompletion)
                    }
                    other => Ok(other),
                }
            }
        }
    }
}

pub enum StepOutcome {
    Continue,
    NeedsStableCompletion,
    AgentComplete(SendMessageResponse),
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
        let types: std::collections::HashSet<_> =
            EXPERIMENTAL_METHOD_SPECS.iter().map(|s| s.slot_type).collect();
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
        assert_eq!(
            required_slot_type_for_method("detect"),
            Some("event")
        );
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
