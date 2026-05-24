//! Shared pre-LLM loading, main LLM call, and post-LLM persistence for turn paths.

use crate::domain::chat_llm_fallback::{fallback_reply_for_llm_failure, FallbackReplyContext};
use crate::domain::chat_turn::{relation_favor_for_key, weight_memories_for_scene};
use crate::domain::chat_turn_rules::{soft_append_guard, strip_hallucination_tokens};
use crate::domain::complex_emotion::ComplexEmotionOutput;
use crate::domain::emotion_analyzer::EmotionResult;
use crate::domain::memory_retrieval::MemoryRetrievalInput;
use crate::domain::personality_engine::PersonalityEngine;
use crate::domain::policy::PolicyContext;
use crate::domain::portrait_emotion_engine::resolve_portrait_emotion;
use crate::domain::slot_runner::{CoPresentSlotRunner, SlotRunner};
use crate::domain::user_identity::resolve_effective_user_relation_key;
use crate::models::dto::SendMessageResponse;
use crate::models::knowledge::KnowledgeChunk;
use crate::models::{Emotion, Event, EventType, Memory, PersonalitySource, PersonalityVector, Role};
use oclive_kernel_runtime::domain::relation_engine::RelationState;
use std::time::Instant;

use super::super::context::load_recent_context;
use super::super::emotion_to_dto;
use super::super::favor::{compute_favor_and_relation, FavorRelationInput};
use super::super::scene::{detect_movement_intent, movement_ui_flags};
use super::super::staged::StageRunner;
use crate::domain::chat_engine::chat_stage::ChatStage;
use super::super::turn_context::TurnContext;
use super::super::turn_error::TurnResult;
use super::TurnMode;

pub(crate) const STAGES: StageRunner = StageRunner;

pub(crate) fn skipped_complex_emotion() -> ComplexEmotionOutput {
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

pub(crate) struct PreLlmOutput {
    pub event_runtime: f64,
    pub mutable_for_prompt: String,
    pub personality: PersonalityVector,
    pub recent_turns: Vec<(String, String)>,
    pub recent_turns_for_event: Vec<(String, String)>,
    pub recent_events_for_event: Vec<Event>,
    pub emotion_result: EmotionResult,
    pub user_emotion: Emotion,
    pub user_emotion_str: String,
    pub user_emotion_prompt: String,
    pub ollama_model: String,
    pub prev_stored_narrative_hint: String,
    pub relevant: Vec<Memory>,
    pub user_relation_key: String,
    pub relation_before: String,
    pub favorability_before: f64,
}

pub(crate) struct MiddleOutput {
    pub complex_emotion_out: ComplexEmotionOutput,
    pub knowledge_chunk_count: u32,
    pub ai_event_type: EventType,
    pub ai_impact_factor_final: f64,
    pub ai_event_confidence: f32,
    pub personality: PersonalityVector,
    pub prompt: String,
    pub favor_delta: f64,
    pub relation_after: RelationState,
}

pub(crate) struct MainLlmOutput {
    pub reply: String,
    pub main_llm_fallback: bool,
    pub main_llm_ms: u64,
}

pub(crate) async fn pre_llm(ctx: &TurnContext<'_>) -> TurnResult<PreLlmOutput> {
    let state = ctx.state;
    let req = ctx.req;
    let role = ctx.role;
    let scene_id = ctx.scene_id;
    let srid = ctx.srid;
    let user_message = req.user_message.as_str();
    let pl = &ctx.pl;
    let slot_runner = SlotRunner;

    let (
        event_impact_opt,
        mutable_for_prompt,
        personality,
        (recent_turns, recent_turns_for_event, recent_events_for_event),
    ) = tokio::try_join!(
        async {
            STAGES
                .stage(
                    ChatStage::EventImpactFactor,
                    state.db_manager.get_event_impact_factor(srid),
                )
                .await
        },
        async {
            STAGES
                .stage(
                    ChatStage::MutablePersonality,
                    state.db_manager.get_mutable_personality(srid),
                )
                .await
        },
        async {
            STAGES
                .stage(
                    ChatStage::CurrentPersonality,
                    state.get_current_personality(srid, role),
                )
                .await
        },
        async {
            STAGES
                .stage(ChatStage::LoadRecentContext, load_recent_context(state, srid))
                .await
        },
    )?;
    let event_runtime = event_impact_opt.unwrap_or(role.evolution_config.event_impact_factor);
    let mut personality = personality;

    let emotion_result = STAGES
        .stage(
            ChatStage::UserEmotionAnalyze,
            async { slot_runner.analyze_emotion(pl, user_message) },
        )
        .await?;
    let user_emotion = emotion_result.to_emotion();
    let user_emotion_str = user_emotion.to_string();
    let user_emotion_prompt =
        crate::domain::emotion_analyzer::EmotionAnalyzer::format_for_prompt(&emotion_result);

    let ollama_model = role.resolve_ollama_model(state.ollama_model.as_str());

    let prev_stored_narrative_hint = state.session_cache.stored_complex_emotion_narrative_hint(srid);

    if role.evolution_config.personality_source != PersonalitySource::Profile {
        personality = PersonalityEngine::adjust_by_user_emotion(
            personality,
            &user_emotion_str,
            &role.evolution_bounds,
        );
    }

    let (mut memories, user_relation_key) = tokio::try_join!(
        STAGES.stage(
            ChatStage::LoadMemories,
            state.memory_repo.load_memories(srid, 10),
        ),
        STAGES.stage(
            ChatStage::ResolveUserRelationKey,
            resolve_effective_user_relation_key(state, role, srid, Some(scene_id)),
        ),
    )?;
    let scene_m = role
        .memory_config
        .as_ref()
        .map(|m| m.scene_weight_multiplier)
        .unwrap_or(1.0);
    weight_memories_for_scene(&mut memories, scene_id, scene_m);
    let relevant = STAGES
        .stage(
            ChatStage::MemoryRank,
            async {
                slot_runner.rank_memories(
                    pl,
                    MemoryRetrievalInput {
                        memories: &memories,
                        user_query: user_message,
                        scene_id: Some(scene_id),
                        limit: 8,
                    },
                )
            },
        )
        .await?;
    let seed_favor = role.initial_favorability_for_relation(user_relation_key.as_str());

    STAGES
        .stage(
            ChatStage::EnsureIdentityStatsRow,
            state
                .db_manager
                .ensure_identity_stats_row(srid, user_relation_key.as_str(), seed_favor),
        )
        .await?;

    let (rel_id, rel_global, favorability_before) = tokio::try_join!(
        async {
            STAGES
                .stage(
                    ChatStage::RelationStateForIdentity,
                    state
                        .db_manager
                        .get_relation_state_for_identity(srid, user_relation_key.as_str()),
                )
                .await
        },
        async {
            STAGES
                .stage(
                    ChatStage::RelationStateGlobal,
                    state.db_manager.get_relation_state(srid),
                )
                .await
        },
        async {
            STAGES
                .stage(
                    ChatStage::FavorabilityForIdentity,
                    state
                        .db_manager
                        .favorability_for_identity_with_runtime_fallback(
                            srid,
                            user_relation_key.as_str(),
                        ),
                )
                .await
        },
    )?;
    let relation_before = rel_id
        .or(rel_global)
        .unwrap_or_else(|| "Stranger".to_string());

    Ok(PreLlmOutput {
        event_runtime,
        mutable_for_prompt,
        personality,
        recent_turns,
        recent_turns_for_event,
        recent_events_for_event,
        emotion_result,
        user_emotion,
        user_emotion_str,
        user_emotion_prompt,
        ollama_model,
        prev_stored_narrative_hint,
        relevant,
        user_relation_key,
        relation_before,
        favorability_before,
    })
}

pub(crate) fn compute_turn_favor(
    pre: &PreLlmOutput,
    role: &Role,
    ai_event_type: &EventType,
    ai_impact_factor_final: f64,
    ai_event_confidence: f32,
) -> (f64, RelationState) {
    let rf = relation_favor_for_key(role, pre.user_relation_key.as_str());
    let favor_relation_input = FavorRelationInput {
        relation_before: pre.relation_before.as_str(),
        favorability_before: pre.favorability_before,
        ai_event_type,
        ai_impact_factor_final,
        event_runtime: pre.event_runtime,
        favor_mult: rf.favor_mult,
        event_confidence: ai_event_confidence,
        recent_events_for_event: &pre.recent_events_for_event,
    };
    compute_favor_and_relation(&favor_relation_input)
}

pub(crate) fn worldview_snippet_from_chunks(knowledge_chunks: &[&KnowledgeChunk]) -> String {
    if knowledge_chunks.is_empty() {
        String::new()
    } else {
        crate::models::knowledge::KnowledgeIndex::format_for_prompt(knowledge_chunks, 6000)
    }
}

pub(crate) async fn run_main_llm(
    ctx: &TurnContext<'_>,
    path_label: &str,
    pre: &PreLlmOutput,
    middle: &MiddleOutput,
) -> TurnResult<MainLlmOutput> {
    let role = ctx.role;
    let user_message = ctx.req.user_message.as_str();
    let pl = &ctx.pl;
    let slot_runner = SlotRunner;

    let t_main_llm = Instant::now();
    let mut main_llm_fallback = false;
    let reply_raw = match slot_runner
        .generate_llm(pl, pre.ollama_model.as_str(), &middle.prompt)
        .await
    {
        Ok(s) => s,
        Err(e) => {
            tracing::warn!("{path_label} LLM generate failed, fallback: {e}");
            main_llm_fallback = true;
            fallback_reply_for_llm_failure(
                role,
                &middle.personality,
                user_message,
                &FallbackReplyContext {
                    relation_before: pre.relation_before.as_str(),
                    relation_preview: middle.relation_after.as_str(),
                    favorability_before: pre.favorability_before,
                    event_type: &middle.ai_event_type,
                    impact_factor: middle.ai_impact_factor_final,
                },
            )
        }
    };
    let main_llm_ms = t_main_llm.elapsed().as_millis() as u64;
    let reply = strip_hallucination_tokens(&soft_append_guard(
        &reply_raw,
        &middle.ai_event_type,
        middle.ai_impact_factor_final,
        middle.relation_after.as_str(),
    ));

    Ok(MainLlmOutput {
        reply,
        main_llm_fallback,
        main_llm_ms,
    })
}

pub(crate) async fn post_llm(
    ctx: &TurnContext<'_>,
    mode: TurnMode,
    path_label: &str,
    pre: &PreLlmOutput,
    middle: &MiddleOutput,
    llm: &MainLlmOutput,
    pre_main_llm_ms: u64,
) -> TurnResult<SendMessageResponse> {
    use crate::models::dto::{
        DetectedEventDto, PresenceMode, API_VERSION, SCHEMA_VERSION,
    };

    let state = ctx.state;
    let req = ctx.req;
    let role = ctx.role;
    let scene_id = ctx.scene_id;
    let scenes = std::sync::Arc::clone(&ctx.scenes);
    let mrid = ctx.mrid;
    let srid = ctx.srid;
    let t0 = ctx.t0;
    let preflight_ms = ctx.preflight_ms;
    let immersive = ctx.immersive;
    let pl = &ctx.pl;
    let slot_runner = SlotRunner;
    let user_message = req.user_message.as_str();
    let policies = state.policies_for_scene(Some(scene_id));
    let primary_llm = slot_runner.primary_llm(pl);

    let t_post_llm = Instant::now();
    let reply = llm.reply.clone();
    let previous_emotion_fut = state.db_manager.get_current_emotion(srid);
    let bot_emotion_result = STAGES
        .stage(
            ChatStage::BotReplyEmotionAnalyze,
            async { slot_runner.analyze_emotion(pl, &reply) },
        )
        .await?;
    let previous_emotion = STAGES
        .stage(
            ChatStage::GetCurrentEmotion,
            previous_emotion_fut,
        )
        .await?;
    let bot_emotion = policies
        .emotion
        .resolve_current_emotion(previous_emotion.as_deref(), &bot_emotion_result);
    let bot_emotion_str = bot_emotion.to_string();

    let event = Event {
        event_type: middle.ai_event_type,
        user_emotion: pre.user_emotion_str.clone(),
        bot_emotion: bot_emotion_str.clone(),
    };

    let policy_ctx = PolicyContext {
        role_id: srid,
        user_message,
        reply: &reply,
        event: &event,
        event_confidence: middle.ai_event_confidence,
    };
    let memory_line = policies.memory.build_memory_entry(&policy_ctx);
    let memory_importance = if policies.memory.should_persist(&policy_ctx) {
        policies.memory.importance(&policy_ctx)
    } else {
        0.0
    };
    let recent_events = std::iter::once(event.clone())
        .chain(pre.recent_events_for_event.clone())
        .collect::<Vec<_>>();
    let core_v = PersonalityVector::from(&role.default_personality);
    let portrait_fut = STAGES.stage(
        ChatStage::PortraitEmotionLlm,
        resolve_portrait_emotion(
            &primary_llm,
            pre.ollama_model.as_str(),
            role,
            &core_v,
            &middle.personality,
            pre.favorability_before,
            user_message,
            &reply,
            pre.user_emotion_str.as_str(),
            &bot_emotion,
            &recent_events,
            &pre.recent_turns,
        ),
    );

    let (portrait_emotion_str, profile_evolve) =
        if role.evolution_config.personality_source == PersonalitySource::Profile {
            let impact_scaled =
                (middle.ai_impact_factor_final * pre.event_runtime).clamp(-1.0, 1.0);
            let evolve_fut = async {
                let prev = STAGES
                    .stage(
                        ChatStage::GetMutablePersonality,
                        state.db_manager.get_mutable_personality(srid),
                    )
                    .await?;
                let next = match crate::domain::mutable_profile_llm::evolve_mutable_personality_with_llm(
                    &primary_llm,
                    pre.ollama_model.as_str(),
                    crate::domain::mutable_profile_llm::MutableEvolutionInput {
                        role_name: role.name.as_str(),
                        core_personality: role.core_personality.as_str(),
                        prev_mutable: prev.as_str(),
                        user_message,
                        bot_reply: reply.as_str(),
                        user_emotion: pre.user_emotion_str.as_str(),
                        event_type: &middle.ai_event_type,
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
                Ok((prev, next))
            };
            let (portrait_res, evolve_res) = tokio::join!(portrait_fut, evolve_fut);
            (portrait_res?, Some(evolve_res?))
        } else {
            (portrait_fut.await?, None)
        };

    let favor_current = STAGES
        .stage(
            ChatStage::ApplyChatTurnAtomic,
            state
                .db_manager
                .apply_chat_turn_atomic(crate::infrastructure::db::ChatTurnTxInput {
                    role_id: srid,
                    personality: &middle.personality,
                    current_emotion: bot_emotion_str.as_str(),
                    relation_state: middle.relation_after.as_str(),
                    user_relation_key: pre.user_relation_key.as_str(),
                    favor_delta: middle.favor_delta,
                    memory_content: &memory_line,
                    memory_importance,
                    memory_fifo_limit: policies.memory.fifo_limit(),
                    event: &event,
                    user_message,
                    bot_reply: &reply,
                    scene_id,
                }),
        )
        .await?;

    if matches!(mode, TurnMode::CoPresent) {
        state.session_cache.set_stored_complex_emotion_narrative_hint(
            srid,
            middle.complex_emotion_out.narrative_hint.clone(),
        );
    }

    if let Some((_, next)) = profile_evolve {
        STAGES
            .stage(
                ChatStage::SetMutablePersonality,
                state.db_manager.set_mutable_personality(srid, &next),
            )
            .await?;
        let personality_after =
            crate::domain::profile_personality::effective_vector_from_profile(role, &next);
        let delta_out = PersonalityVector::sub_components(&personality_after, &core_v);
        STAGES
            .stage(
                ChatStage::SetCoreDeltaPersonalityJsonProfile,
                state
                    .db_manager
                    .set_core_delta_personality_json(srid, &core_v.to_json_vec(), &delta_out.to_json_vec()),
            )
            .await?;
        state
            .session_cache
            .personality_cache()
            .set(srid.to_string(), personality_after);
    } else if role.evolution_config.personality_source != PersonalitySource::Profile {
        let delta_out = PersonalityVector::sub_components(&middle.personality, &core_v);
        STAGES
            .stage(
                ChatStage::SetCoreDeltaPersonalityJsonNonProfile,
                state
                    .db_manager
                    .set_core_delta_personality_json(srid, &core_v.to_json_vec(), &delta_out.to_json_vec()),
            )
            .await?;
        state
            .session_cache
            .personality_cache()
            .set(srid.to_string(), middle.personality.clone());
    }

    let movement = detect_movement_intent(
        state,
        &primary_llm,
        role,
        srid,
        scene_id,
        &scenes,
        user_message,
        pre.ollama_model.as_str(),
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
        "send_message {path_label} role_id={mrid} scene_id={scene_id} duration_ms={duration_ms} main_llm_fallback={} offer_destination_picker={offer_destination_picker} offer_together_travel={offer_together_travel}",
        llm.main_llm_fallback,
    );
    tracing::debug!(
        target: "oclive_chat",
        "send_message {path_label} timing preflight_ms={preflight_ms} pre_main_llm_ms={pre_main_llm_ms} main_llm_ms={} post_llm_ms={post_llm_ms} duration_ms={duration_ms}",
        llm.main_llm_ms,
    );

    let (presence_mode, events) = match mode {
        TurnMode::CoPresent => (
            PresenceMode::CoPresent,
            vec![DetectedEventDto {
                event_type: event.event_type.as_ref().to_string(),
                confidence: middle.ai_event_confidence,
            }],
        ),
        TurnMode::RemoteLife => (PresenceMode::RemoteLife, vec![]),
    };

    Ok(SendMessageResponse {
        api_version: API_VERSION,
        schema: SCHEMA_VERSION,
        presence_mode,
        relation_state: middle.relation_after.as_str().to_string(),
        reply,
        emotion: emotion_to_dto(&pre.emotion_result),
        bot_emotion: bot_emotion_str,
        portrait_emotion: portrait_emotion_str,
        favorability_delta: middle.favor_delta as f32,
        favorability_current: favor_current as f32,
        events,
        scene_id: scene_id.to_string(),
        offer_destination_picker,
        offer_together_travel,
        reply_is_fallback: llm.main_llm_fallback,
        knowledge_chunks_in_prompt: middle.knowledge_chunk_count,
        timestamp: chrono::Utc::now().timestamp_millis(),
    })
}
