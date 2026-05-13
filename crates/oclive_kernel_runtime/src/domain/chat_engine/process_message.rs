//! 用户消息主编排：`process_message`（内核权威；与仓库 `creator-docs/kernel/KERNEL_ENTRY_CHECKLIST.md` 中 `send_message` 对齐）。
//!
//! **性能备忘**：本路径不经过 `serde_json::Value` 热克隆；`scene_id` / `user_message` 等 `String` 克隆用于跨 `await` 与多消费者（DB、策略、Tracing），拆成 `&str` 会牵动大量签名，暂维持分配。
//!
//! **插件**：`ResolvedRolePlugins` 在入口附近由 `KernelAppState::resolved_plugins_for_session` 从 `plugin_host::BackendRegistry` 解析；本函数不直接操作 `local_plugins` 锁或目录插件运行时，避免与异步插件 I/O 的锁序交织。

use super::co_present;
use super::context::validate_scene_id;
use super::emotion_to_dto;
use super::presence::user_is_remote_from_character;
use crate::domain::agent::AgentInput;
use crate::domain::chat_llm_fallback::{fallback_reply_for_llm_failure, FallbackReplyContext};
use crate::domain::chat_turn::{relation_favor_for_key, weight_memories_for_scene};
use crate::domain::chat_turn_rules::{soft_append_guard, strip_hallucination_tokens};
use crate::domain::complex_emotion::{affect_metrics_from_seven_dim, ComplexEmotionInput};
use crate::domain::emotion_analyzer::EmotionResultExt;
use crate::domain::life_schedule::{format_life_prompt_line, resolve_life_state};
use crate::domain::memory_retrieval::MemoryRetrievalInput;
use crate::domain::personality_engine::PersonalityEngine;
use crate::domain::policy::PolicyContext;
use crate::domain::portrait_emotion_engine::resolve_portrait_emotion;
use crate::domain::remote_life_prompt::{build_remote_life_prompt, compose_remote_stub_reply};
use crate::domain::user_identity::resolve_effective_user_relation_key;
use crate::error::{AppError, Result};
use crate::models::dto::{
    PresenceMode, SendMessageRequest, SendMessageResponse, API_VERSION, SCHEMA_VERSION,
};
use crate::models::{
    Event, EventType, KnowledgeIndex, Memory, PersonalitySource, PersonalityVector, Role,
};
use crate::state::KernelAppState;
use chrono::Utc;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Instant;

/// `process_message` 退出时写入 **INFO** 总耗时（与 `tracing` span 对齐）。
struct ProcessMessageTraceFinish {
    span: tracing::Span,
    start: Instant,
    role_id: String,
    scene_id: String,
    session_ns: String,
    user_len: usize,
}

impl Drop for ProcessMessageTraceFinish {
    fn drop(&mut self) {
        let elapsed_ms = self.start.elapsed().as_millis() as u64;
        tracing::info!(
            target: "oclive_process_message",
            parent: &self.span,
            elapsed_ms,
            role_id = %self.role_id,
            scene_id = %self.scene_id,
            session_ns = %self.session_ns,
            user_len = self.user_len,
            "process_message finished"
        );
    }
}

/// 处理一条用户消息：分析情绪 → 检测事件 → 演化性格 → 构建 Prompt → 调用 LLM → 持久化
pub async fn process_message(
    state: &KernelAppState,
    req: &SendMessageRequest,
) -> Result<SendMessageResponse> {
    let mrid = req.role_id.as_str();
    let state_rid = super::conversation_state_role_id(mrid, req.session_id.as_deref());
    let srid = state_rid.as_str();
    let requested_scene_id = req
        .scene_id
        .clone()
        .unwrap_or_else(|| "default".to_string());
    let (scene_id, scenes) = validate_scene_id(state, mrid, requested_scene_id)?;
    let trace_span = tracing::info_span!(
        "process_message",
        role_id = mrid,
        scene_id = %scene_id,
        session_ns = srid,
        user_len = req.user_message.len(),
    );
    let _trace_enter = trace_span.enter();
    let t0 = Instant::now();
    let _trace_finish = ProcessMessageTraceFinish {
        span: trace_span.clone(),
        start: t0,
        role_id: mrid.to_string(),
        scene_id: scene_id.clone(),
        session_ns: srid.to_string(),
        user_len: req.user_message.len(),
    };
    log::debug!(
        target: "oclive_chat",
        "send_message start role_id={} scene_id={} session_ns={}",
        mrid,
        scene_id,
        srid
    );

    state.chat_generation_cancel.store(false, Ordering::Release);

    let io = Instant::now();
    state.db_manager.ensure_role_runtime(srid).await?;
    tracing::debug!(
        target: "oclive_chat_io",
        role_id = %mrid,
        session_ns = %srid,
        op = "ensure_role_runtime",
        elapsed_ms = io.elapsed().as_millis() as u64
    );

    let io = Instant::now();
    let role = ensure_role_loaded(state, mrid).await?;
    tracing::debug!(
        target: "oclive_chat_io",
        role_id = %mrid,
        op = "ensure_role_loaded",
        elapsed_ms = io.elapsed().as_millis() as u64
    );
    state
        .db_manager
        .ensure_interaction_mode_seeded(srid, role.interaction_mode.as_deref())
        .await?;

    let effective_backends = state.effective_plugin_backends_for_session(role.as_ref(), srid);
    let effective_sources = state.effective_plugin_backend_sources_for_session(srid);
    log::debug!(
        target: "oclive_chat",
        "send_message backends role_id={} scene_id={} session_ns={} {}",
        mrid,
        scene_id,
        srid,
        super::backend_resolution_summary(&effective_backends, &effective_sources)
    );

    let pl = state.resolved_plugins_for_session(role.as_ref(), Some(srid));
    let agent_llm_model =
        super::resolve_main_llm_model_for_generate(state, role.as_ref(), srid).await?;
    let agent_out = pl
        .agent
        .process(AgentInput {
            role_id: mrid.to_string(),
            session_namespace: srid.to_string(),
            message: req.user_message.clone(),
            model: agent_llm_model,
        })
        .await?;
    if agent_out.handled {
        state
            .db_manager
            .set_user_presence_scene(srid, scene_id.as_str())
            .await?;
        let analyzed = pl.emotion.analyze(req.user_message.as_str())?;
        let emotion_result: crate::domain::emotion_analyzer::EmotionResult = analyzed;
        let user_relation_key = resolve_effective_user_relation_key(
            state,
            role.as_ref(),
            srid,
            Some(scene_id.as_str()),
        )
        .await?;
        let relation_state = state
            .db_manager
            .get_relation_state_for_identity(srid, user_relation_key.as_str())
            .await?
            .or(state.db_manager.get_relation_state(srid).await?)
            .unwrap_or_else(|| "Stranger".to_string());
        let favor_current = state
            .db_manager
            .favorability_for_identity_with_runtime_fallback(srid, user_relation_key.as_str())
            .await?;
        let portrait_emotion = state
            .db_manager
            .get_current_emotion(srid)
            .await?
            .unwrap_or_else(|| "neutral".to_string());

        let prev_complex_hint = state.db_manager.get_complex_emotion_hint(srid).await?;
        {
            let (valence, dominance) = affect_metrics_from_seven_dim(&emotion_result);
            let ce_in = ComplexEmotionInput {
                role_id: mrid.to_string(),
                scene_id: scene_id.clone(),
                user_message: req.user_message.clone(),
                bot_reply: agent_out.reply.clone(),
                recent_dialogue_summary: None,
                previous_narrative_hint: prev_complex_hint.clone().unwrap_or_default(),
                user_valence: Some(valence),
                user_dominance: Some(dominance),
                previous_user_message: None,
            };
            if let Ok(out) = pl.complex_emotion.resolve_turn(&ce_in) {
                let _ = state
                    .db_manager
                    .set_complex_emotion_hint(srid, out.narrative_hint.as_deref())
                    .await;
            }
        }

        return Ok(SendMessageResponse {
            api_version: API_VERSION,
            schema: SCHEMA_VERSION,
            presence_mode: PresenceMode::CoPresent,
            relation_state,
            reply: agent_out.reply,
            emotion: emotion_to_dto(&emotion_result),
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
        });
    }

    state
        .db_manager
        .set_user_presence_scene(srid, scene_id.as_str())
        .await?;

    let current_scene = state.db_manager.get_current_scene(srid).await?;
    let immersive = state
        .db_manager
        .get_interaction_mode(srid)
        .await?
        .is_immersive();
    let remote_life_enabled = state.db_manager.get_remote_life_enabled(srid).await?;
    let is_remote =
        immersive && user_is_remote_from_character(scene_id.as_str(), current_scene.as_deref());
    let preflight_ms = t0.elapsed().as_millis() as u64;
    if is_remote {
        if !remote_life_enabled {
            return process_remote_stub(
                state,
                req,
                role.as_ref(),
                scene_id.as_str(),
                t0,
                srid,
                preflight_ms,
            )
            .await;
        }
        let char_scene = current_scene.as_deref().unwrap_or("default");
        return process_remote_life(
            state,
            req,
            role.as_ref(),
            scene_id.as_str(),
            char_scene,
            t0,
            mrid,
            srid,
            preflight_ms,
        )
        .await;
    }

    co_present::process_co_present(
        state,
        req,
        role.as_ref(),
        scene_id,
        scenes,
        immersive,
        t0,
        mrid,
        srid,
        preflight_ms,
    )
    .await
}

/// 加载角色 `Arc`（与 `pipeline_actions::load_role` 语义一致，供入口直编路径复用）。
async fn ensure_role_loaded(state: &KernelAppState, role_id: &str) -> Result<Arc<Role>> {
    state.load_role_cached(role_id)
}

async fn process_remote_stub(
    state: &KernelAppState,
    req: &SendMessageRequest,
    role: &Role,
    scene_id: &str,
    t0: Instant,
    srid: &str,
    preflight_ms: u64,
) -> Result<SendMessageResponse> {
    let role_id = req.role_id.as_str();
    let user_message = req.user_message.as_str();
    let pl = state.resolved_plugins_for_session(role, Some(srid));
    let analyzed = pl.emotion.analyze(user_message)?;
    let emotion_result: crate::domain::emotion_analyzer::EmotionResult = analyzed;
    let user_relation_key: String =
        resolve_effective_user_relation_key(state, role, srid, Some(scene_id)).await?;
    let relation_before = state
        .db_manager
        .get_relation_state_for_identity(srid, user_relation_key.as_str())
        .await?
        .or(state.db_manager.get_relation_state(srid).await?)
        .unwrap_or_else(|| "Stranger".to_string());
    let favorability_before = state
        .db_manager
        .favorability_for_identity_with_runtime_fallback(srid, user_relation_key.as_str())
        .await?;
    let portrait_emotion_str = state
        .db_manager
        .get_current_emotion(srid)
        .await?
        .unwrap_or_else(|| "neutral".to_string());
    let reply = compose_remote_stub_reply(role);
    let duration_ms = t0.elapsed().as_millis() as u64;
    log::info!(
        target: "oclive_chat",
        "send_message remote_stub role_id={} scene_id={} duration_ms={}",
        role_id,
        scene_id,
        duration_ms
    );
    log::debug!(
        target: "oclive_chat",
        "send_message remote_stub timing preflight_ms={} duration_ms={}",
        preflight_ms,
        duration_ms
    );
    Ok(SendMessageResponse {
        api_version: API_VERSION,
        schema: SCHEMA_VERSION,
        presence_mode: PresenceMode::RemoteStub,
        relation_state: relation_before,
        reply,
        emotion: emotion_to_dto(&emotion_result),
        bot_emotion: "neutral".to_string(),
        portrait_emotion: portrait_emotion_str,
        favorability_delta: 0.0,
        favorability_current: favorability_before as f32,
        events: vec![],
        scene_id: scene_id.to_string(),
        offer_destination_picker: false,
        offer_together_travel: false,
        reply_is_fallback: false,
        knowledge_chunks_in_prompt: 0,
        timestamp: chrono::Utc::now().timestamp_millis(),
    })
}

#[allow(clippy::too_many_arguments)]
async fn process_remote_life(
    state: &KernelAppState,
    req: &SendMessageRequest,
    role: &Role,
    scene_id: &str,
    char_scene_id: &str,
    t0: Instant,
    mrid: &str,
    srid: &str,
    preflight_ms: u64,
) -> Result<SendMessageResponse> {
    let t_path = Instant::now();
    let role_id = req.role_id.as_str();
    let user_message = req.user_message.as_str();
    let event_runtime = state
        .db_manager
        .get_event_impact_factor(srid)
        .await?
        .unwrap_or(role.evolution_config.event_impact_factor);

    let mutable_for_prompt = state.db_manager.get_mutable_personality(srid).await?;

    let mut personality = state.get_current_personality(srid, role).await?;

    let pl = state.resolved_plugins_for_session(role, Some(srid));
    let analyzed = pl.emotion.analyze(user_message)?;
    let emotion_result: crate::domain::emotion_analyzer::EmotionResult = analyzed;
    let user_emotion = emotion_result.to_emotion();
    let user_emotion_str = user_emotion.to_string();

    let main_llm_model = super::resolve_main_llm_model_for_generate(state, role, srid).await?;
    let (recent_turns, _recent_turns_for_event, recent_events_for_event) =
        super::context::load_recent_context(state, srid).await?;

    if role.evolution_config.personality_source != PersonalitySource::Profile {
        personality = PersonalityEngine::adjust_by_user_emotion(
            personality,
            &user_emotion_str,
            &role.evolution_bounds,
        );
    }

    let ai_event_type = EventType::Ignore;
    let ai_impact_factor_final = 0.0_f64;
    let ai_event_confidence = 0.0_f32;

    if role.evolution_config.personality_source != PersonalitySource::Profile {
        personality = PersonalityEngine::evolve_by_event(
            personality,
            ai_impact_factor_final * event_runtime,
            &role.evolution_bounds,
        );
    }

    let mut memories = state.memory_repo.load_memories(srid, 10).await?;
    let scene_m = role
        .memory_config
        .as_ref()
        .map(|m| m.scene_weight_multiplier)
        .unwrap_or(1.0);
    weight_memories_for_scene(&mut memories, scene_id, scene_m);
    let mut relevant = pl.memory.rank_memories(MemoryRetrievalInput {
        memories: &memories,
        user_query: user_message,
        scene_id: Some(scene_id),
        limit: 8,
    });

    let user_relation_key: String =
        resolve_effective_user_relation_key(state, role, srid, Some(scene_id)).await?;
    let rf = relation_favor_for_key(role, user_relation_key.as_str());

    let relation_before = state
        .db_manager
        .get_relation_state_for_identity(srid, user_relation_key.as_str())
        .await?
        .or(state.db_manager.get_relation_state(srid).await?)
        .unwrap_or_else(|| "Stranger".to_string());
    let seed_favor = role.initial_favorability_for_relation(user_relation_key.as_str());
    state
        .db_manager
        .ensure_identity_stats_row(srid, user_relation_key.as_str(), seed_favor)
        .await?;
    let favorability_before = state
        .db_manager
        .favorability_for_identity_with_runtime_fallback(srid, user_relation_key.as_str())
        .await?;
    let favor_relation_input = super::favor::FavorRelationInput {
        relation_before: relation_before.as_str(),
        favorability_before,
        ai_event_type: &ai_event_type,
        ai_impact_factor_final,
        event_runtime,
        favor_mult: rf.favor_mult,
        event_confidence: ai_event_confidence,
        recent_events_for_event: &recent_events_for_event,
    };
    let (favor_delta, relation_after) =
        super::favor::compute_favor_and_relation(&favor_relation_input);

    let char_label = state.storage.scene_display_name(mrid, char_scene_id);
    let user_label = state.storage.scene_display_name(mrid, scene_id);
    let away_material = state
        .storage
        .away_life_material(mrid, char_scene_id, scene_id);
    let vt_ms = state
        .db_manager
        .get_virtual_time_ms(srid)
        .await?
        .unwrap_or(0);
    let vt_label = if vt_ms > 0 {
        chrono::DateTime::from_timestamp_millis(vt_ms)
            .map(|d| d.format("%Y-%m-%d %H:%M").to_string())
            .unwrap_or_else(|| "未设定".to_string())
    } else {
        "未设定".to_string()
    };
    let life_schedule_line: String = role
        .life_schedule
        .as_ref()
        .and_then(|s| resolve_life_state(vt_ms, s))
        .map(|st| format_life_prompt_line(&st, true))
        .unwrap_or_default();

    let knowledge_chunks = role
        .knowledge_index
        .as_ref()
        .map(|idx| idx.retrieve(user_message, Some(char_scene_id), 8))
        .unwrap_or_default();
    let worldview_snippet: String = if knowledge_chunks.is_empty() {
        String::new()
    } else {
        KnowledgeIndex::format_for_prompt(knowledge_chunks.as_slice(), 6000)
    };

    let remote_mutable = if role.evolution_config.personality_source == PersonalitySource::Profile {
        mutable_for_prompt.as_str()
    } else {
        ""
    };
    let prev_complex_hint = state.db_manager.get_complex_emotion_hint(srid).await?;
    let prev_complex_hint_trimmed = prev_complex_hint
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty());
    let prompt = build_remote_life_prompt(
        role,
        away_material.as_str(),
        char_label.as_str(),
        user_label.as_str(),
        user_message,
        favorability_before,
        relation_before.as_str(),
        vt_label.as_str(),
        life_schedule_line.as_str(),
        worldview_snippet.as_str(),
        remote_mutable,
        prev_complex_hint_trimmed,
    );

    let pre_main_llm_ms = t_path.elapsed().as_millis() as u64;
    let t_main_llm = Instant::now();
    let mut main_llm_fallback = false;
    let reply_raw = match super::llm_cancelable::run_llm_generate_cancelable(
        state,
        pl.llm.clone(),
        main_llm_model.as_str(),
        &prompt,
    )
    .await
    {
        Ok(s) => s,
        Err(AppError::ChatGenerationCancelled) => {
            return Err(AppError::ChatGenerationCancelled);
        }
        Err(e) => {
            log::warn!("remote_life LLM generate failed, fallback: {}", e);
            main_llm_fallback = true;
            fallback_reply_for_llm_failure(
                role,
                &personality,
                user_message,
                &FallbackReplyContext {
                    relation_before: relation_before.as_str(),
                    relation_preview: relation_after.as_str(),
                    favorability_before,
                    event_type: &ai_event_type,
                    impact_factor: ai_impact_factor_final,
                },
            )
        }
    };
    let main_llm_ms = t_main_llm.elapsed().as_millis() as u64;
    let t_post_llm = Instant::now();
    let reply = strip_hallucination_tokens(&soft_append_guard(
        &reply_raw,
        &ai_event_type,
        ai_impact_factor_final,
        relation_after.as_str(),
    ));
    {
        let (valence, dominance) = affect_metrics_from_seven_dim(&emotion_result);
        let ce_in = ComplexEmotionInput {
            role_id: role.id.to_string(),
            scene_id: scene_id.to_string(),
            user_message: user_message.to_string(),
            bot_reply: reply.to_string(),
            recent_dialogue_summary: None,
            previous_narrative_hint: prev_complex_hint.unwrap_or_default(),
            user_valence: Some(valence),
            user_dominance: Some(dominance),
            previous_user_message: None,
        };
        if let Ok(out) = pl.complex_emotion.resolve_turn(&ce_in) {
            let _ = state
                .db_manager
                .set_complex_emotion_hint(srid, out.narrative_hint.as_deref())
                .await;
        }
    }
    let bot_analyzed = pl.emotion.analyze(&reply)?;
    let bot_emotion_result: crate::domain::emotion_analyzer::EmotionResult = bot_analyzed;
    let previous_emotion = state.db_manager.get_current_emotion(srid).await?;
    let policies = state.policies_for_scene(Some(scene_id));
    let bot_emotion = policies
        .emotion
        .resolve_current_emotion(previous_emotion.as_deref(), &bot_emotion_result);
    let bot_emotion_str = bot_emotion.to_string();

    let event = Event {
        event_type: ai_event_type,
        user_emotion: user_emotion_str.clone(),
        bot_emotion: bot_emotion_str.clone(),
    };

    relevant.insert(
        0,
        Memory {
            id: "__relation_state__".to_string(),
            role_id: srid.to_string(),
            content: format!(
                "当前关系阶段: {} -> {}",
                relation_before,
                relation_after.as_str()
            ),
            importance: 0.95,
            weight: 1.0,
            created_at: Utc::now(),
            scene_id: Some(scene_id.to_string()),
        },
    );
    let policy_ctx = PolicyContext {
        role_id: srid,
        user_message,
        reply: &reply,
        event: &event,
        event_confidence: ai_event_confidence,
    };
    let memory_line = policies.memory.build_memory_entry(&policy_ctx);
    let should_persist_memory = policies.memory.should_persist(&policy_ctx);
    let memory_importance = if should_persist_memory {
        policies.memory.importance(&policy_ctx)
    } else {
        0.0
    };
    let mut recent_events = recent_events_for_event;
    recent_events.insert(0, event.clone());
    let core_v = PersonalityVector::from(&role.default_personality);
    let portrait_emotion_str = resolve_portrait_emotion(
        &pl.llm,
        main_llm_model.as_str(),
        role,
        &core_v,
        &personality,
        favorability_before,
        user_message,
        &reply,
        user_emotion_str.as_str(),
        &bot_emotion,
        &recent_events,
        &recent_turns,
    )
    .await?;

    let io = Instant::now();
    let favor_current = state
        .db_manager
        .apply_chat_turn_atomic(crate::infrastructure::db::ChatTurnTxInput {
            role_id: srid,
            personality: &personality,
            current_emotion: bot_emotion_str.as_str(),
            relation_state: relation_after.as_str(),
            user_relation_key: user_relation_key.as_str(),
            favor_delta,
            memory_content: &memory_line,
            memory_importance,
            memory_fifo_limit: policies.memory.fifo_limit(),
            event: &event,
            user_message,
            bot_reply: &reply,
            scene_id,
        })
        .await?;
    tracing::debug!(
        target: "oclive_chat_io",
        role_id = %mrid,
        session_ns = %srid,
        op = "apply_chat_turn_atomic",
        elapsed_ms = io.elapsed().as_millis() as u64
    );

    if let Err(e) =
        crate::domain::expert_graph_events::apply_expert_graph_event_triggers_after_turn(
            state,
            role_id,
            srid,
            user_message,
            reply.as_str(),
        )
        .await
    {
        log::warn!(
            target: "oclive_chat",
            "expert_graph event triggers failed role_id={} session_ns={} err={}",
            role_id,
            srid,
            e
        );
    }

    if role.evolution_config.personality_source == PersonalitySource::Profile {
        let prev = state.db_manager.get_mutable_personality(srid).await?;
        let impact_scaled = (ai_impact_factor_final * event_runtime).clamp(-1.0, 1.0);
        let next = match crate::domain::mutable_profile_llm::evolve_mutable_personality_with_llm(
            &pl.llm,
            main_llm_model.as_str(),
            crate::domain::mutable_profile_llm::MutableEvolutionInput {
                role_name: role.name.as_str(),
                core_personality: role.core_personality.as_str(),
                prev_mutable: prev.as_str(),
                user_message,
                bot_reply: reply.as_str(),
                user_emotion: user_emotion_str.as_str(),
                event_type: &ai_event_type,
                impact_scaled,
                evolution: &role.evolution_config,
            },
        )
        .await
        {
            Ok(s) => s,
            Err(e) => {
                log::warn!(
                    target: "oclive_chat",
                    "mutable_profile_llm remote_life failed role_id={} err={}; keeping previous archive",
                    srid,
                    e
                );
                prev.clone()
            }
        };
        state
            .db_manager
            .set_mutable_personality(srid, &next)
            .await?;
        let personality_after =
            crate::domain::profile_personality::effective_vector_from_profile(role, &next);
        let delta_out = PersonalityVector::sub_components(&personality_after, &core_v);
        state
            .db_manager
            .set_core_delta_personality_json(srid, &core_v.to_json_vec(), &delta_out.to_json_vec())
            .await?;
        state
            .personality_cache
            .write()
            .insert(srid.to_string(), personality_after);
    } else {
        let delta_out = PersonalityVector::sub_components(&personality, &core_v);
        state
            .db_manager
            .set_core_delta_personality_json(srid, &core_v.to_json_vec(), &delta_out.to_json_vec())
            .await?;
        state
            .personality_cache
            .write()
            .insert(srid.to_string(), personality.clone());
    }

    let scenes = state.storage.list_scene_ids(mrid)?;
    let movement = super::scene::detect_movement_intent(
        state,
        &pl.llm,
        mrid,
        srid,
        scene_id,
        &scenes,
        user_message,
        main_llm_model.as_str(),
    )
    .await;
    let (offer_destination_picker, offer_together_travel) =
        super::scene::movement_ui_flags(movement, user_message);

    let post_llm_ms = t_post_llm.elapsed().as_millis() as u64;
    let duration_ms = t0.elapsed().as_millis() as u64;
    log::info!(
        target: "oclive_chat",
        "send_message remote_life role_id={} scene_id={} duration_ms={} main_llm_fallback={} offer_destination_picker={} offer_together_travel={}",
        role_id,
        scene_id,
        duration_ms,
        main_llm_fallback,
        offer_destination_picker,
        offer_together_travel
    );
    log::debug!(
        target: "oclive_chat",
        "send_message remote_life timing preflight_ms={} pre_main_llm_ms={} main_llm_ms={} post_llm_ms={} duration_ms={}",
        preflight_ms,
        pre_main_llm_ms,
        main_llm_ms,
        post_llm_ms,
        duration_ms
    );

    Ok(SendMessageResponse {
        api_version: API_VERSION,
        schema: SCHEMA_VERSION,
        presence_mode: PresenceMode::RemoteLife,
        relation_state: relation_after.as_str().to_string(),
        reply,
        emotion: emotion_to_dto(&emotion_result),
        bot_emotion: bot_emotion_str,
        portrait_emotion: portrait_emotion_str,
        favorability_delta: favor_delta as f32,
        favorability_current: favor_current as f32,
        events: vec![],
        scene_id: scene_id.to_string(),
        offer_destination_picker,
        offer_together_travel,
        reply_is_fallback: main_llm_fallback,
        knowledge_chunks_in_prompt: knowledge_chunks.len() as u32,
        timestamp: chrono::Utc::now().timestamp_millis(),
    })
}
