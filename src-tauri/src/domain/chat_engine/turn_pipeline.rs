//! Shared turn orchestration for co-present and remote-life paths.

use crate::domain::chat_llm_fallback::{fallback_reply_for_llm_failure, FallbackReplyContext};
use crate::domain::chat_turn::{relation_favor_for_key, weight_memories_for_scene};
use crate::domain::chat_turn_rules::{soft_append_guard, strip_hallucination_tokens};
use crate::domain::complex_emotion::{ComplexEmotionInput, ComplexEmotionOutput};
use crate::domain::life_schedule::{format_life_prompt_line, resolve_life_state};
use crate::domain::memory_retrieval::MemoryRetrievalInput;
use crate::domain::personality_engine::PersonalityEngine;
use crate::domain::policy::PolicyContext;
use crate::domain::portrait_emotion_engine::resolve_portrait_emotion;
use crate::domain::prompt_builder::{effective_reply_quality_anchor, PromptInput};
use crate::domain::remote_life_prompt::build_remote_life_prompt;
use crate::domain::slot_runner::{CoPresentSlotRunner, SlotRunner};
use crate::domain::user_identity::resolve_effective_user_relation_key;
use crate::models::dto::{
    DetectedEventDto, PresenceMode, SendMessageResponse, API_VERSION, SCHEMA_VERSION,
};
use crate::models::knowledge::KnowledgeIndex;
use crate::models::{Event, EventType, Memory, PersonalitySource, PersonalityVector};
use chrono::Utc;
use std::sync::Arc;
use std::time::Instant;

use super::co_present::CoPresentResult;
use super::context::load_recent_context;
use super::emotion_to_dto;
use super::favor::{compute_favor_and_relation, FavorRelationInput};
use super::scene::{detect_movement_intent, movement_ui_flags};
use super::turn_context::TurnContext;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TurnMode {
    CoPresent,
    RemoteLife,
}

fn skipped_complex_emotion() -> ComplexEmotionOutput {
    ComplexEmotionOutput {
        source: "skipped".to_string(),
        narrative_hint: String::new(),
        labels: vec![],
        pattern: None,
        confidence: 0.0,
        intensity: 0.0,
        dissonance_score: 0.0,
        degraded_to_builtin: false,
    }
}

pub async fn execute_turn(ctx: &TurnContext<'_>, mode: TurnMode) -> CoPresentResult<SendMessageResponse> {
    let state = ctx.state;
    let req = ctx.req;
    let role = ctx.role;
    let scene_id = ctx.scene_id;
    let scenes = Arc::clone(&ctx.scenes);
    let virtual_time_ms = ctx.virtual_time_ms;
    let mrid = ctx.mrid;
    let srid = ctx.srid;
    let t0 = ctx.t0;
    let preflight_ms = ctx.preflight_ms;
    let immersive = ctx.immersive;
    let path_label = match mode {
        TurnMode::CoPresent => "co_present",
        TurnMode::RemoteLife => "remote_life",
    };
    let pl = &ctx.pl;
    let t_path0 = Instant::now();
    let user_message = req.user_message.as_str();
    let policies = state.policies_for_scene(Some(scene_id));
    let slot_runner = SlotRunner;

    let (event_impact_opt, mutable_for_prompt, personality) = tokio::try_join!(
        async {
            crate::kernel_stage!(@co_present crate::domain::chat_engine::chat_stage::ChatStage::EventImpactFactor,
                state.db_manager.get_event_impact_factor(srid).await
            )
        },
        async {
            crate::kernel_stage!(@co_present crate::domain::chat_engine::chat_stage::ChatStage::MutablePersonality,
                state.db_manager.get_mutable_personality(srid).await
            )
        },
        async {
            crate::kernel_stage!(@co_present crate::domain::chat_engine::chat_stage::ChatStage::CurrentPersonality,
                state.get_current_personality(srid, role).await
            )
        },
    )?;
    let event_runtime = event_impact_opt.unwrap_or(role.evolution_config.event_impact_factor);
    let mut personality = personality;

    let emotion_result =
        crate::kernel_stage!(@co_present crate::domain::chat_engine::chat_stage::ChatStage::UserEmotionAnalyze, slot_runner.analyze_emotion(&pl, user_message))?;
    let user_emotion = emotion_result.to_emotion();
    let user_emotion_str = user_emotion.to_string();
    let user_emotion_prompt =
        crate::domain::emotion_analyzer::EmotionAnalyzer::format_for_prompt(&emotion_result);

    let ollama_model = role.resolve_ollama_model(state.ollama_model.as_str());
    let (recent_turns, recent_turns_for_event, recent_events_for_event) =
        crate::kernel_stage!(@co_present crate::domain::chat_engine::chat_stage::ChatStage::LoadRecentContext, load_recent_context(state, srid).await)?;

    let prev_stored_narrative_hint = state.session_cache.stored_complex_emotion_narrative_hint(srid);
    let complex_emotion_out = match mode {
        TurnMode::CoPresent => {
            let (prev_user_for_ce, prev_bot_for_ce) = recent_turns
                .last()
                .map(|(u, b)| (Some(u.clone()), b.clone()))
                .unwrap_or((None, String::new()));
            let (uv, ud) = crate::domain::complex_emotion::affect_metrics_from_seven_dim(&emotion_result);
            crate::kernel_stage!(@co_present crate::domain::chat_engine::chat_stage::ChatStage::ComplexEmotionResolveTurn,
                slot_runner.resolve_complex_emotion(
                    &pl,
                    &ComplexEmotionInput {
                        role_id: mrid.to_string(),
                        scene_id: scene_id.to_string(),
                        user_message: user_message.to_string(),
                        bot_reply: prev_bot_for_ce,
                        recent_dialogue_summary: None,
                        previous_narrative_hint: prev_stored_narrative_hint.clone(),
                        user_valence: Some(uv),
                        user_dominance: Some(ud),
                        previous_user_message: prev_user_for_ce,
                    },
                )
            )?
        }
        TurnMode::RemoteLife => skipped_complex_emotion(),
    };

    if role.evolution_config.personality_source != PersonalitySource::Profile {
        personality = PersonalityEngine::adjust_by_user_emotion(
            personality,
            &user_emotion_str,
            &role.evolution_bounds,
        );
    }

    let knowledge_scene = match mode {
        TurnMode::CoPresent => scene_id,
        TurnMode::RemoteLife => ctx
            .character_scene_id
            .as_deref()
            .unwrap_or("default"),
    };
    let knowledge_chunks = role
        .knowledge_index
        .as_ref()
        .map(|idx| idx.retrieve(user_message, Some(knowledge_scene), 8))
        .unwrap_or_default();

    let (ai_event_type, ai_impact_factor_final, ai_event_confidence) = match mode {
        TurnMode::CoPresent => {
            let knowledge_augment_opt = {
                let aug = KnowledgeIndex::merge_event_augment(knowledge_chunks.as_slice());
                if aug.is_empty() {
                    None
                } else {
                    Some(aug)
                }
            };
            let estimate = crate::kernel_stage!(@co_present crate::domain::chat_engine::chat_stage::ChatStage::EventEstimate,
                slot_runner
                    .estimate_event(
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
            (
                estimate.event_type,
                estimate.impact_factor,
                estimate.confidence,
            )
        }
        TurnMode::RemoteLife => (EventType::Ignore, 0.0_f64, 0.0_f32),
    };

    if role.evolution_config.personality_source != PersonalitySource::Profile {
        personality = PersonalityEngine::evolve_by_event(
            personality,
            ai_impact_factor_final * event_runtime,
            &role.evolution_bounds,
        );
    }

    let mut memories =
        crate::kernel_stage!(@co_present crate::domain::chat_engine::chat_stage::ChatStage::LoadMemories, state.memory_repo.load_memories(srid, 10).await)?;
    let scene_m = role
        .memory_config
        .as_ref()
        .map(|m| m.scene_weight_multiplier)
        .unwrap_or(1.0);
    weight_memories_for_scene(&mut memories, scene_id, scene_m);
    let mut relevant = crate::kernel_stage!(@co_present crate::domain::chat_engine::chat_stage::ChatStage::MemoryRank,
        slot_runner.rank_memories(
            &pl,
            MemoryRetrievalInput {
                memories: &memories,
                user_query: user_message,
                scene_id: Some(scene_id),
                limit: 8,
            },
        )
    )?;

    let user_relation_key: String = crate::kernel_stage!(@co_present crate::domain::chat_engine::chat_stage::ChatStage::ResolveUserRelationKey,
        resolve_effective_user_relation_key(state, role, srid, Some(scene_id)).await
    )?;
    let rf = relation_favor_for_key(role, user_relation_key.as_str());
    let seed_favor = role.initial_favorability_for_relation(user_relation_key.as_str());

    let (rel_id, rel_global, _, favorability_before) = tokio::try_join!(
        async {
            crate::kernel_stage!(@co_present crate::domain::chat_engine::chat_stage::ChatStage::RelationStateForIdentity,
                state
                    .db_manager
                    .get_relation_state_for_identity(srid, user_relation_key.as_str())
                    .await
            )
        },
        async {
            crate::kernel_stage!(@co_present crate::domain::chat_engine::chat_stage::ChatStage::RelationStateGlobal,
                state.db_manager.get_relation_state(srid).await
            )
        },
        async {
            crate::kernel_stage!(@co_present crate::domain::chat_engine::chat_stage::ChatStage::EnsureIdentityStatsRow,
                state
                    .db_manager
                    .ensure_identity_stats_row(srid, user_relation_key.as_str(), seed_favor)
                    .await
            )
        },
        async {
            crate::kernel_stage!(@co_present crate::domain::chat_engine::chat_stage::ChatStage::FavorabilityForIdentity,
                state
                    .db_manager
                    .favorability_for_identity_with_runtime_fallback(srid, user_relation_key.as_str())
                    .await
            )
        },
    )?;
    let relation_before = rel_id
        .or(rel_global)
        .unwrap_or_else(|| "Stranger".to_string());
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

    let worldview_snippet: String = if knowledge_chunks.is_empty() {
        String::new()
    } else {
        KnowledgeIndex::format_for_prompt(knowledge_chunks.as_slice(), 6000)
    };

    let prompt = match mode {
        TurnMode::CoPresent => {
            let scene_label = state.storage.scene_display_name_for_role(role, scene_id);
            let scene_detail_buf = state.storage.scene_prompt_enrichment_for_role(role, scene_id);
            let top_topic = slot_runner.top_topic_hint(&pl, role, scene_id);
            let topic_line = top_topic
                .map(|t| format!("在「{}」下，你们可能会多聊「{}」相关的事。", scene_label, t))
                .unwrap_or_default();
            let life_context_line: String = if immersive {
                role.life_schedule
                    .as_ref()
                    .and_then(|s| resolve_life_state(virtual_time_ms, s))
                    .map(|st| format_life_prompt_line(&st, false))
                    .unwrap_or_default()
            } else {
                String::new()
            };
            crate::kernel_stage!(@co_present crate::domain::chat_engine::chat_stage::ChatStage::BuildPrompt,
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
            )?
        }
        TurnMode::RemoteLife => {
            let character_scene_id = ctx
                .character_scene_id
                .as_deref()
                .unwrap_or("default");
            let char_label = state.storage.scene_display_name_for_role(role, character_scene_id);
            let user_label = state.storage.scene_display_name_for_role(role, scene_id);
            let away_material = state
                .storage
                .away_life_material_for_role(role, character_scene_id, scene_id);
            let vt_label = if virtual_time_ms > 0 {
                chrono::DateTime::from_timestamp_millis(virtual_time_ms)
                    .map(|d| d.format("%Y-%m-%d %H:%M").to_string())
                    .unwrap_or_else(|| "未设定".to_string())
            } else {
                "未设定".to_string()
            };
            let life_schedule_line: String = role
                .life_schedule
                .as_ref()
                .and_then(|s| resolve_life_state(virtual_time_ms, s))
                .map(|st| format_life_prompt_line(&st, true))
                .unwrap_or_default();
            let remote_mutable =
                if role.evolution_config.personality_source == PersonalitySource::Profile {
                    mutable_for_prompt.as_str()
                } else {
                    ""
                };
            build_remote_life_prompt(
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
            )
        }
    };

    let pre_main_llm_ms = t_path0.elapsed().as_millis() as u64;
    let t_main_llm = Instant::now();
    let mut main_llm_fallback = false;
    let reply_raw = match slot_runner.generate_llm(&pl, ollama_model.as_str(), &prompt).await {
        Ok(s) => s,
        Err(e) => {
            tracing::warn!("{path_label} LLM generate failed, fallback: {e}");
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
    let previous_emotion_fut = state.db_manager.get_current_emotion(srid);
    let bot_emotion_result =
        crate::kernel_stage!(@co_present crate::domain::chat_engine::chat_stage::ChatStage::BotReplyEmotionAnalyze, slot_runner.analyze_emotion(&pl, &reply))?;
    let previous_emotion = crate::kernel_stage!(
        @co_present crate::domain::chat_engine::chat_stage::ChatStage::GetCurrentEmotion,
        previous_emotion_fut.await
    )?;
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
        event_confidence,
    };
    let memory_line = policies.memory.build_memory_entry(&policy_ctx);
    let memory_importance = if policies.memory.should_persist(&policy_ctx) {
        policies.memory.importance(&policy_ctx)
    } else {
        0.0
    };
    let mut recent_events = recent_events_for_event;
    recent_events.insert(0, event.clone());
    let core_v = PersonalityVector::from(&role.default_personality);
    let portrait_emotion_str = crate::kernel_stage!(@co_present crate::domain::chat_engine::chat_stage::ChatStage::PortraitEmotionLlm,
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

    let favor_current = crate::kernel_stage!(@co_present crate::domain::chat_engine::chat_stage::ChatStage::ApplyChatTurnAtomic,
        state
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
            .await
    )?;

    if matches!(mode, TurnMode::CoPresent) {
        state.session_cache.set_stored_complex_emotion_narrative_hint(
            srid,
            complex_emotion_out.narrative_hint.clone(),
        );
    }

    if role.evolution_config.personality_source == PersonalitySource::Profile {
        let prev = crate::kernel_stage!(@co_present crate::domain::chat_engine::chat_stage::ChatStage::GetMutablePersonality,
            state.db_manager.get_mutable_personality(srid).await
        )?;
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
                    "mutable_profile_llm {path_label} failed role_id={srid} err={e}; keeping previous archive",
                );
                prev.clone()
            }
        };
        crate::kernel_stage!(@co_present crate::domain::chat_engine::chat_stage::ChatStage::SetMutablePersonality,
            state.db_manager.set_mutable_personality(srid, &next).await
        )?;
        let personality_after =
            crate::domain::profile_personality::effective_vector_from_profile(role, &next);
        let delta_out = PersonalityVector::sub_components(&personality_after, &core_v);
        crate::kernel_stage!(@co_present crate::domain::chat_engine::chat_stage::ChatStage::SetCoreDeltaPersonalityJsonProfile,
            state
                .db_manager
                .set_core_delta_personality_json(srid, &core_v.to_json_vec(), &delta_out.to_json_vec())
                .await
        )?;
        state
            .session_cache
            .personality_cache()
            .insert(srid.to_string(), personality_after);
    } else {
        let delta_out = PersonalityVector::sub_components(&personality, &core_v);
        crate::kernel_stage!(@co_present crate::domain::chat_engine::chat_stage::ChatStage::SetCoreDeltaPersonalityJsonNonProfile,
            state
                .db_manager
                .set_core_delta_personality_json(srid, &core_v.to_json_vec(), &delta_out.to_json_vec())
                .await
        )?;
        state
            .session_cache
            .personality_cache()
            .insert(srid.to_string(), personality.clone());
    }

    let movement = detect_movement_intent(
        state,
        &slot_runner.primary_llm(&pl),
        role,
        srid,
        scene_id,
        &scenes,
        user_message,
        ollama_model.as_str(),
    )
    .await;
    let (mut offer_destination_picker, mut offer_together_travel) =
        movement_ui_flags(movement, user_message);
    if matches!(mode, TurnMode::CoPresent) && !immersive {
        offer_destination_picker = false;
        offer_together_travel = false;
    }

    let post_llm_ms = t_post_llm.elapsed().as_millis() as u64;
    let duration_ms = t0.elapsed().as_millis() as u64;
    tracing::info!(
        target: "oclive_chat",
        "send_message {path_label} role_id={mrid} scene_id={scene_id} duration_ms={duration_ms} main_llm_fallback={main_llm_fallback} offer_destination_picker={offer_destination_picker} offer_together_travel={offer_together_travel}",
    );
    tracing::debug!(
        target: "oclive_chat",
        "send_message {path_label} timing preflight_ms={preflight_ms} pre_main_llm_ms={pre_main_llm_ms} main_llm_ms={main_llm_ms} post_llm_ms={post_llm_ms} duration_ms={duration_ms}",
    );

    let (presence_mode, events) = match mode {
        TurnMode::CoPresent => (
            PresenceMode::CoPresent,
            vec![DetectedEventDto {
                event_type: format!("{:?}", event.event_type),
                confidence: event_confidence,
            }],
        ),
        TurnMode::RemoteLife => (PresenceMode::RemoteLife, vec![]),
    };

    Ok(SendMessageResponse {
        api_version: API_VERSION,
        schema: SCHEMA_VERSION,
        presence_mode,
        relation_state: relation_after.as_str().to_string(),
        reply,
        emotion: emotion_to_dto(&emotion_result),
        bot_emotion: bot_emotion_str,
        portrait_emotion: portrait_emotion_str,
        favorability_delta: favor_delta as f32,
        favorability_current: favor_current as f32,
        events,
        scene_id: scene_id.to_string(),
        offer_destination_picker,
        offer_together_travel,
        reply_is_fallback: main_llm_fallback,
        knowledge_chunks_in_prompt: knowledge_chunks.len() as u32,
        timestamp: chrono::Utc::now().timestamp_millis(),
    })
}
