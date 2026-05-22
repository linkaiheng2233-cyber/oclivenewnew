//! # 共景模式编排（同屏对话主路径）
//!
//! **角色**：用户与角色**同屏**时的回合编排——加载近期上下文、复杂情感 `narrative_hint`、按阶段调用 [`SlotRunner`](../slot_runner.rs) / [`CoPresentSlotRunner`] 执行多实例槽位，再组 Prompt、调 LLM、写库与返回 DTO。
//!
//! **上游**：[`process_message`](super::process_message) 在排除 Agent 短路与异地分支后调用本模块。
//! **下游**：`PromptBuilder`、`DbManager`、各 `dyn` 端口（经 `ResolvedRolePlugins`）；多实例合并策略见 [`slot_runner`](../slot_runner.rs)。
//!
//! **关键决策**：共景路径**显式分阶段**（`CoPresentError::stage`），便于日志与 OOCP 断言；槽位调用统一走 `SlotRunner`，避免在共景里散落 `pl.emotion` 直连。

use crate::domain::chat_llm_fallback::{fallback_reply_for_llm_failure, FallbackReplyContext};
use crate::domain::chat_turn::{relation_favor_for_key, weight_memories_for_scene};
use crate::domain::chat_turn_rules::{soft_append_guard, strip_hallucination_tokens};
use crate::domain::life_schedule::{format_life_prompt_line, resolve_life_state};
use crate::domain::memory_retrieval::MemoryRetrievalInput;
use crate::domain::personality_engine::PersonalityEngine;
use crate::domain::policy::PolicyContext;
use crate::domain::portrait_emotion_engine::resolve_portrait_emotion;
use crate::domain::prompt_builder::{effective_reply_quality_anchor, PromptInput};
use crate::domain::slot_runner::{CoPresentSlotRunner, SlotRunner};
use crate::domain::user_identity::resolve_effective_user_relation_key;
use crate::error::AppError;
use crate::models::dto::{
    DetectedEventDto, PresenceMode, SendMessageRequest, SendMessageResponse, API_VERSION,
    SCHEMA_VERSION,
};
use crate::models::knowledge::KnowledgeIndex;
use crate::models::{Event, Memory, PersonalitySource, PersonalityVector, Role};
use crate::state::AppState;
use chrono::Utc;
use std::time::Instant;
use thiserror::Error;

use super::context::load_recent_context;
use super::emotion_to_dto;
use super::favor::{compute_favor_and_relation, FavorRelationInput};
use super::scene::{detect_movement_intent, movement_ui_flags};

/// 共景路径中的失败，带 `stage` 便于与日志对齐。
#[derive(Debug, Error)]
#[error("共景({stage}): {source}")]
pub struct CoPresentError {
    pub(crate) stage: &'static str,
    #[source]
    pub(crate) source: AppError,
}

impl CoPresentError {
    pub fn wrap(stage: &'static str, source: AppError) -> Self {
        Self { stage, source }
    }
}

impl From<CoPresentError> for AppError {
    fn from(e: CoPresentError) -> Self {
        e.source
    }
}

pub(crate) type CoPresentResult<T> = std::result::Result<T, CoPresentError>;

#[allow(clippy::too_many_arguments)] // 编排入口：场景 / 计时 / 多 id 与 `Role` 并列传入，不宜为 clippy 强塞单结构体
pub(crate) async fn process_co_present(
    state: &AppState,
    req: &SendMessageRequest,
    role: &Role,
    scene_id: String,
    scenes: Vec<String>,
    immersive: bool,
    t0: Instant,
    mrid: &str,
    srid: &str,
    preflight_ms: u64,
) -> CoPresentResult<SendMessageResponse> {
    let t_cp0 = Instant::now();
    let user_message = req.user_message.as_str();
    let policies = state.policies_for_scene(Some(scene_id.as_str()));
    let pl = super::resolve_plugins_for_session(
        state.plugin_host_port(),
        role,
        Some(srid),
        &state.effective_plugin_backends_for_session(role, srid),
        state.effective_slot_registry_for_session(role, srid).as_ref(),
    );
    let slot_runner = SlotRunner;

    let event_runtime = crate::map_copresent_err!("event_impact_factor", state.db_manager.get_event_impact_factor(srid).await)?
    .unwrap_or(role.evolution_config.event_impact_factor);

    let mutable_for_prompt = crate::map_copresent_err!("mutable_personality", state.db_manager.get_mutable_personality(srid).await)?;

    let mut personality = crate::map_copresent_err!("current_personality", state.get_current_personality(srid, role).await)?;

    let emotion_result = crate::map_copresent_err!("user_emotion_analyze", slot_runner.analyze_emotion(&pl, user_message))?;
    crate::domain::debug_trace::emit_step(
        "user_emotion_analyze",
        serde_json::json!({ "text_len": user_message.len() }),
        serde_json::json!({ "emotion": format!("{:?}", emotion_result.to_emotion()) }),
    );
    let user_emotion = emotion_result.to_emotion();
    let user_emotion_str = user_emotion.to_string();
    let user_emotion_prompt =
        crate::domain::emotion_analyzer::EmotionAnalyzer::format_for_prompt(&emotion_result);

    let ollama_model = role.resolve_ollama_model(state.ollama_model.as_str());
    let (recent_turns, recent_turns_for_event, recent_events_for_event) = crate::map_copresent_err!("load_recent_context", load_recent_context(state, srid).await)?;
    crate::domain::debug_trace::emit_step(
        "load_recent_context",
        serde_json::json!({ "srid": srid, "user_message_len": user_message.len() }),
        serde_json::json!({ "turns": recent_turns.len() }),
    );

    let prev_stored_narrative_hint = state.stored_complex_emotion_narrative_hint(srid);
    let (prev_user_for_ce, prev_bot_for_ce) = recent_turns
        .last()
        .map(|(u, b)| (Some(u.clone()), b.clone()))
        .unwrap_or((None, String::new()));
    let (uv, ud) = crate::domain::complex_emotion::affect_metrics_from_seven_dim(&emotion_result);
    let complex_emotion_out = crate::map_copresent_err!("complex_emotion_resolve_turn",
        slot_runner.resolve_complex_emotion(
                    &pl,
                    &crate::domain::complex_emotion::ComplexEmotionInput {
                        role_id: mrid.to_string(),
                        scene_id: scene_id.clone(),
                        user_message: user_message.to_string(),
                        bot_reply: prev_bot_for_ce,
                        recent_dialogue_summary: None,
                        previous_narrative_hint: prev_stored_narrative_hint.clone(),
                        user_valence: Some(uv),
                        user_dominance: Some(ud),
                        previous_user_message: prev_user_for_ce,
                    },
                )
    )?;

    if role.evolution_config.personality_source != PersonalitySource::Profile {
        personality = PersonalityEngine::adjust_by_user_emotion(
            personality,
            &user_emotion_str,
            &role.evolution_bounds,
        );
    }

    let knowledge_chunks = role
        .knowledge_index
        .as_ref()
        .map(|idx| idx.retrieve(user_message, Some(scene_id.as_str()), 8))
        .unwrap_or_default();
    let knowledge_augment_opt = {
        let aug = KnowledgeIndex::merge_event_augment(knowledge_chunks.as_slice());
        if aug.is_empty() {
            None
        } else {
            Some(aug)
        }
    };

    let estimate = crate::map_copresent_err!("event_estimate",
        slot_runner.estimate_event(
                    &pl,
                    ollama_model.as_str(),
                    user_message,
                    &user_emotion,
                    &personality,
                    role.evolution_config.personality_source,
                    &recent_turns_for_event,
                    &recent_events_for_event,
                    knowledge_augment_opt.as_ref(),
                )
                .await
    )?;
    crate::domain::debug_trace::emit_step(
        "event_estimate",
        serde_json::json!({ "scene_id": scene_id }),
        serde_json::json!({
            "event_type": estimate.event_type,
            "impact_factor": estimate.impact_factor,
            "confidence": estimate.confidence
        }),
    );
    let ai_event_type = estimate.event_type;
    let ai_impact_factor_final = estimate.impact_factor;
    let ai_event_confidence = estimate.confidence;

    if role.evolution_config.personality_source != PersonalitySource::Profile {
        personality = PersonalityEngine::evolve_by_event(
            personality,
            ai_impact_factor_final * event_runtime,
            &role.evolution_bounds,
        );
    }

    let mut memories = crate::map_copresent_err!("load_memories", state.memory_repo.load_memories(srid, 10).await)?;
    let scene_m = role
        .memory_config
        .as_ref()
        .map(|m| m.scene_weight_multiplier)
        .unwrap_or(1.0);
    weight_memories_for_scene(&mut memories, scene_id.as_str(), scene_m);
    let mut relevant = crate::map_copresent_err!("memory_rank",
        slot_runner.rank_memories(
                    &pl,
                    MemoryRetrievalInput {
                        memories: &memories,
                        user_query: user_message,
                        scene_id: Some(scene_id.as_str()),
                        limit: 8,
                    },
                )
    )?;
    crate::domain::debug_trace::emit_step(
        "memory_rank",
        serde_json::json!({ "candidates": memories.len() }),
        serde_json::json!({ "ranked": relevant.len() }),
    );

    let user_relation_key: String = crate::map_copresent_err!("resolve_user_relation_key", resolve_effective_user_relation_key(state, role, srid, Some(scene_id.as_str())).await)?;
    let rf = relation_favor_for_key(role, user_relation_key.as_str());

    let rel_id = crate::map_copresent_err!("relation_state_for_identity",
        state
                    .db_manager
                    .get_relation_state_for_identity(srid, user_relation_key.as_str())
                    .await
    )?;
    let rel_global = crate::map_copresent_err!("relation_state_global", state.db_manager.get_relation_state(srid).await)?;
    let relation_before = rel_id
        .or(rel_global)
        .unwrap_or_else(|| "Stranger".to_string());
    let seed_favor = role.initial_favorability_for_relation(user_relation_key.as_str());
    crate::map_copresent_err!("ensure_identity_stats_row",
        state
                    .db_manager
                    .ensure_identity_stats_row(srid, user_relation_key.as_str(), seed_favor)
                    .await
    )?;
    let favorability_before = crate::map_copresent_err!("favorability_for_identity",
        state
                    .db_manager
                    .favorability_for_identity_with_runtime_fallback(srid, user_relation_key.as_str())
                    .await
    )?;
    let event_confidence = ai_event_confidence;
    let favor_relation_input = FavorRelationInput {
        relation_before: relation_before.as_str(),
        favorability_before,
        ai_event_type: &ai_event_type,
        ai_impact_factor_final,
        event_runtime,
        favor_mult: rf.favor_mult,
        event_confidence,
        recent_events_for_event: &recent_events_for_event,
    };
    let (favor_delta, relation_after) = compute_favor_and_relation(&favor_relation_input);

    let scene_label = state.storage.scene_display_name(mrid, scene_id.as_str());
    let scene_detail_buf = state
        .storage
        .scene_prompt_enrichment(mrid, scene_id.as_str());
    let top_topic = slot_runner.top_topic_hint(&pl, role, scene_id.as_str());
    let topic_line = top_topic
        .map(|t| format!("在「{}」下，你们可能会多聊「{}」相关的事。", scene_label, t))
        .unwrap_or_default();

    let virtual_time_ms = crate::map_copresent_err!("virtual_time_ms", state.db_manager.get_virtual_time_ms(srid).await)?
    .unwrap_or(0);
    let life_context_line: String = if immersive {
        role.life_schedule
            .as_ref()
            .and_then(|s| resolve_life_state(virtual_time_ms, s))
            .map(|st| format_life_prompt_line(&st, false))
            .unwrap_or_default()
    } else {
        String::new()
    };

    let worldview_snippet: String = if knowledge_chunks.is_empty() {
        String::new()
    } else {
        KnowledgeIndex::format_for_prompt(knowledge_chunks.as_slice(), 6000)
    };

    let prompt = crate::map_copresent_err!("build_prompt",
        slot_runner.build_prompt(
                    &pl,
                    &PromptInput {
                        role,
                        personality: &personality,
                        memories: &relevant,
                        user_input: user_message,
                        user_emotion: user_emotion_prompt.as_str(),
                        user_relation_id: user_relation_key.as_str(),
                        relation_hint: rf.relation_hint,
                        relation_before: relation_before.as_str(),
                        favorability_before,
                        relation_preview: relation_after.as_str(),
                        favorability_preview: (favorability_before + favor_delta).clamp(0.0, 100.0),
                        event_type: &ai_event_type,
                        impact_factor: ai_impact_factor_final,
                        scene_label: &scene_label,
                        scene_detail: scene_detail_buf.as_str(),
                        topic_hint_line: &topic_line,
                        life_context_line: life_context_line.as_str(),
                        worldview_snippet: worldview_snippet.as_str(),
                        mutable_personality: mutable_for_prompt.as_str(),
                        reply_quality_anchor: effective_reply_quality_anchor(role),
                        previous_complex_emotion_narrative_hint: prev_stored_narrative_hint.as_str(),
                    },
                )
    )?;
    crate::domain::debug_trace::emit_step(
        "build_prompt",
        serde_json::json!({ "memories": relevant.len(), "scene_id": scene_id }),
        serde_json::json!({ "prompt_len": prompt.len() }),
    );

    let pre_main_llm_ms = t_cp0.elapsed().as_millis() as u64;
    let t_main_llm = Instant::now();
    let mut main_llm_fallback = false;
    let reply_raw = match slot_runner.generate_llm(&pl, ollama_model.as_str(), &prompt).await {
        Ok(s) => s,
        Err(e) => {
            tracing::warn!("main LLM generate failed, talkativeness fallback: {}", e);
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
    crate::domain::debug_trace::emit_step(
        "llm_generate",
        serde_json::json!({ "model": ollama_model, "prompt_len": prompt.len() }),
        serde_json::json!({
            "reply_len": reply_raw.len(),
            "fallback": main_llm_fallback
        }),
    );
    let t_post_llm = Instant::now();
    let reply = strip_hallucination_tokens(&soft_append_guard(
        &reply_raw,
        &ai_event_type,
        ai_impact_factor_final,
        relation_after.as_str(),
    ));
    let bot_emotion_result = crate::map_copresent_err!("bot_reply_emotion_analyze", slot_runner.analyze_emotion(&pl, &reply))?;
    crate::domain::debug_trace::emit_step(
        "postprocess",
        serde_json::json!({ "reply_len": reply.len() }),
        serde_json::json!({ "bot_emotion": format!("{:?}", bot_emotion_result.to_emotion()) }),
    );
    let previous_emotion = crate::map_copresent_err!("get_current_emotion", state.db_manager.get_current_emotion(srid).await)?;
    let bot_emotion = policies
        .emotion
        .resolve_current_emotion(previous_emotion.as_deref(), &bot_emotion_result);
    let bot_emotion_str = bot_emotion.to_string();

    // 复用同一次 AI 结果：event_type 与 impact_factor 不再基于 bot_emotion 重新探测。
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
            scene_id: Some(scene_id.clone()),
        },
    );
    let policy_ctx = PolicyContext {
        role_id: srid,
        user_message,
        reply: &reply,
        event: &event,
        event_confidence,
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
    let portrait_emotion_str = crate::map_copresent_err!("portrait_emotion_llm",
        resolve_portrait_emotion(
                    &slot_runner.primary_llm(&pl),
                    ollama_model.as_str(),
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
                .await
    )?;

    let favor_current = crate::map_copresent_err!("apply_chat_turn_atomic",
        state
                    .db_manager
                    .apply_chat_turn_atomic(crate::infrastructure::db::ChatTurnTxInput {
                        role_id: srid,
                        personality: &personality,
                        // 与用户可见语气一致：用语义情绪驱动立绘/状态；立绘 LLM 细调仍通过返回值 portrait_emotion 下发前端
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
                        scene_id: scene_id.as_str(),
                    })
                    .await
    )?;

    state.set_stored_complex_emotion_narrative_hint(
        srid,
        complex_emotion_out.narrative_hint.clone(),
    );

    if role.evolution_config.personality_source == PersonalitySource::Profile {
        let prev = crate::map_copresent_err!("get_mutable_personality", state.db_manager.get_mutable_personality(srid).await)?;
        let impact_scaled = (ai_impact_factor_final * event_runtime).clamp(-1.0, 1.0);
        let next = match crate::domain::mutable_profile_llm::evolve_mutable_personality_with_llm(
            &slot_runner.primary_llm(&pl),
            ollama_model.as_str(),
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
                tracing::warn!(
                    target: "oclive_chat",
                    "mutable_profile_llm failed role_id={} err={}; keeping previous archive",
                    srid,
                    e
                );
                prev.clone()
            }
        };
        crate::map_copresent_err!("set_mutable_personality", state.db_manager.set_mutable_personality(srid, &next).await)?;
        let personality_after =
            crate::domain::profile_personality::effective_vector_from_profile(role, &next);
        let delta_out = PersonalityVector::sub_components(&personality_after, &core_v);
        crate::map_copresent_err!("set_core_delta_personality_json_profile",
            state
                            .db_manager
                            .set_core_delta_personality_json(
                                srid,
                                &core_v.to_json_vec(),
                                &delta_out.to_json_vec()
                            )
                            .await
        )?;
        state
            .personality_cache
            .write()
            .insert(srid.to_string(), personality_after);
    } else {
        let delta_out = PersonalityVector::sub_components(&personality, &core_v);
        crate::map_copresent_err!("set_core_delta_personality_json_non_profile",
            state
                            .db_manager
                            .set_core_delta_personality_json(
                                srid,
                                &core_v.to_json_vec(),
                                &delta_out.to_json_vec()
                            )
                            .await
        )?;
        state
            .personality_cache
            .write()
            .insert(srid.to_string(), personality.clone());
    }

    let events = vec![DetectedEventDto {
        event_type: format!("{:?}", event.event_type),
        confidence: event_confidence,
    }];

    let movement = detect_movement_intent(
        state,
        &slot_runner.primary_llm(&pl),
        mrid,
        srid,
        scene_id.as_str(),
        &scenes,
        user_message,
        ollama_model.as_str(),
    )
    .await;
    let (mut offer_destination_picker, mut offer_together_travel) =
        movement_ui_flags(movement, user_message);
    if !immersive {
        offer_destination_picker = false;
        offer_together_travel = false;
    }

    let post_llm_ms = t_post_llm.elapsed().as_millis() as u64;
    let duration_ms = t0.elapsed().as_millis() as u64;
    tracing::info!(
        target: "oclive_chat",
        "send_message co_present role_id={} scene_id={} duration_ms={} main_llm_fallback={} offer_destination_picker={} offer_together_travel={}",
        mrid,
        scene_id,
        duration_ms,
        main_llm_fallback,
        offer_destination_picker,
        offer_together_travel
    );
    tracing::debug!(
        target: "oclive_chat",
        "send_message co_present timing preflight_ms={} pre_main_llm_ms={} main_llm_ms={} post_llm_ms={} duration_ms={}",
        preflight_ms,
        pre_main_llm_ms,
        main_llm_ms,
        post_llm_ms,
        duration_ms
    );

    Ok(SendMessageResponse {
        api_version: API_VERSION,
        schema: SCHEMA_VERSION,
        presence_mode: PresenceMode::CoPresent,
        relation_state: relation_after.as_str().to_string(),
        reply,
        emotion: emotion_to_dto(&emotion_result),
        bot_emotion: bot_emotion_str,
        portrait_emotion: portrait_emotion_str,
        favorability_delta: favor_delta as f32,
        favorability_current: favor_current as f32,
        events,
        scene_id,
        offer_destination_picker,
        offer_together_travel,
        reply_is_fallback: main_llm_fallback,
        knowledge_chunks_in_prompt: knowledge_chunks.len() as u32,
        timestamp: chrono::Utc::now().timestamp_millis(),
    })
}
