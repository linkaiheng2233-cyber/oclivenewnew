//! Shared pre-LLM loading, main LLM call, and post-LLM persistence for turn paths.

use crate::domain::chat_llm_fallback::{fallback_reply_for_llm_failure, FallbackReplyContext};
use crate::domain::chat_turn::{relation_favor_for_key, weight_memories_for_scene};
use crate::domain::chat_turn_rules::{soft_append_guard, strip_hallucination_tokens};
use crate::domain::complex_emotion::{
    affect_metrics_from_seven_dim, ComplexEmotionInput, ComplexEmotionOutput,
};
use crate::domain::emotion_analyzer::EmotionResult;
use crate::domain::memory_retrieval::MemoryRetrievalInput;
use crate::domain::memory_engine::MemoryEngine;
use crate::domain::personality_engine::PersonalityEngine;
use crate::domain::policy::PolicyContext;
use crate::domain::portrait_emotion_engine::resolve_portrait_emotion;
use crate::domain::slot_runner::SlotRunner;
use crate::domain::user_identity::resolve_effective_user_relation_key;
use crate::models::dto::SendMessageResponse;
use crate::models::knowledge::KnowledgeChunk;
use crate::models::{Emotion, Event, EventType, Memory, PersonalitySource, PersonalityVector, Role};
use crate::state::SessionCache;
use oclive_kernel_runtime::domain::relation_engine::RelationState;
use std::sync::Arc;
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

/// Profile mutable-personality LLM + DB writes run off the critical path; next turn reads from DB.
#[allow(clippy::too_many_arguments)]
fn spawn_mutable_profile_evolution(
    db: Arc<crate::infrastructure::db::DbManager>,
    session_cache: Arc<SessionCache>,
    primary_llm: Arc<dyn crate::domain::ports::LlmClient>,
    role: Role,
    srid: String,
    path_label: String,
    ollama_model: String,
    user_message: String,
    reply: String,
    user_emotion: String,
    event_type: EventType,
    impact_scaled: f64,
) {
    tokio::spawn(async move {
        let prev = match db.get_mutable_personality(&srid).await {
            Ok(p) => p,
            Err(e) => {
                tracing::warn!(
                    target: "oclive_chat",
                    role_id = %srid,
                    error = %e,
                    "background mutable_profile: get_mutable_personality failed"
                );
                return;
            }
        };
        let next = match crate::domain::mutable_profile_llm::evolve_mutable_personality_with_llm(
            &primary_llm,
            ollama_model.as_str(),
            crate::domain::mutable_profile_llm::MutableEvolutionInput {
                role_name: role.name.as_str(),
                core_personality: role.core_personality.as_str(),
                prev_mutable: prev.as_str(),
                user_message: user_message.as_str(),
                bot_reply: reply.as_str(),
                user_emotion: user_emotion.as_str(),
                event_type: &event_type,
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
                    path_label = %path_label,
                    role_id = %srid,
                    error = %e,
                    "background mutable_profile_llm failed; keeping previous archive"
                );
                return;
            }
        };
        if let Err(e) = db.set_mutable_personality(&srid, &next).await {
            tracing::warn!(
                target: "oclive_chat",
                role_id = %srid,
                error = %e,
                "background mutable_profile: set_mutable_personality failed"
            );
            return;
        }
        let core_v = PersonalityVector::from(&role.default_personality);
        let personality_after =
            crate::domain::profile_personality::effective_vector_from_profile(&role, &next);
        let delta_out = PersonalityVector::sub_components(&personality_after, &core_v);
        if let Err(e) = db
            .set_core_delta_personality_json(
                &srid,
                &core_v.to_json_vec(),
                &delta_out.to_json_vec(),
            )
            .await
        {
            tracing::warn!(
                target: "oclive_chat",
                role_id = %srid,
                error = %e,
                "background mutable_profile: set_core_delta_personality_json failed"
            );
            return;
        }
        session_cache
            .personality_cache()
            .set(srid, personality_after);
    });
}

pub(crate) fn latest_recent_turn_pair(
    recent_turns: &[(String, String)],
) -> (Option<String>, String) {
    recent_turns
        .last()
        .map(|(u, b)| (Some(u.clone()), b.clone()))
        .unwrap_or((None, String::new()))
}

pub(crate) fn build_complex_emotion_turn_input(
    role_id: &str,
    scene_id: &str,
    user_message: &str,
    emotion_result: &EmotionResult,
    previous_narrative_hint: String,
    recent_turns: &[(String, String)],
) -> ComplexEmotionInput {
    let (previous_user_message, bot_reply) = latest_recent_turn_pair(recent_turns);
    let (user_valence, user_dominance) = affect_metrics_from_seven_dim(emotion_result);
    ComplexEmotionInput {
        role_id: role_id.to_string(),
        scene_id: scene_id.to_string(),
        user_message: user_message.to_string(),
        bot_reply,
        recent_dialogue_summary: None,
        previous_narrative_hint,
        user_valence: Some(user_valence),
        user_dominance: Some(user_dominance),
        previous_user_message,
    }
}

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
    pub llm_fallback_reason: Option<String>,
    pub main_llm_ms: u64,
}

async fn prefetch_context(
    state: &crate::state::AppState,
    role: &Role,
    srid: &str,
) -> TurnResult<(
    f64,
    String,
    PersonalityVector,
    (Vec<(String, String)>, Vec<(String, String)>, Vec<Event>),
)> {
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
    Ok((
        event_runtime,
        mutable_for_prompt,
        personality,
        (recent_turns, recent_turns_for_event, recent_events_for_event),
    ))
}

async fn apply_time_evolution(
    state: &crate::state::AppState,
    role: &Role,
    srid: &str,
    immersive: bool,
    virtual_time_ms: i64,
    mut personality: PersonalityVector,
    mut mutable_for_prompt: String,
) -> TurnResult<(PersonalityVector, String)> {
    if immersive && virtual_time_ms > 0 {
        let time_evo = crate::domain::time_driven_evolution::check_and_evolve_by_time(
            state,
            role,
            srid,
            virtual_time_ms,
            immersive,
        )
        .await
        .map_err(|e| super::super::turn_error::TurnError::wrap("time_driven_evolution", e))?;
        if let Some(p) = time_evo.personality {
            personality = p;
        }
        if let Some(m) = time_evo.mutable_for_prompt {
            mutable_for_prompt = m;
        }
    }
    Ok((personality, mutable_for_prompt))
}

async fn resolve_user_emotion_for_turn(
    pl: &crate::domain::plugin_host::ResolvedRolePlugins,
    user_message: &str,
) -> TurnResult<(EmotionResult, Emotion, String, String)> {
    let emotion_result = STAGES
        .stage(
            ChatStage::UserEmotionAnalyze,
            async { SlotRunner::analyze_emotion(pl, user_message) },
        )
        .await?;
    let user_emotion = emotion_result.to_emotion();
    let user_emotion_str = user_emotion.to_string();
    let user_emotion_prompt =
        crate::domain::emotion_analyzer::EmotionAnalyzer::format_for_prompt(&emotion_result);
    Ok((
        emotion_result,
        user_emotion,
        user_emotion_str,
        user_emotion_prompt,
    ))
}

async fn load_prev_narrative_hint(state: &crate::state::AppState, srid: &str) -> String {
    match crate::domain::complex_emotion_store::load_stored_narrative_hint(state, srid).await {
        Ok(h) => h,
        Err(e) => {
            tracing::warn!(
                target: "oclive_complex_emotion",
                role_id = %srid,
                error = %e,
                "load_stored_narrative_hint failed; using empty hint"
            );
            String::new()
        }
    }
}

async fn load_memories_and_relation_key(
    state: &crate::state::AppState,
    role: &Role,
    srid: &str,
    scene_id: &str,
    _pl: &crate::domain::plugin_host::ResolvedRolePlugins,
    _user_message: &str,
    immersive: bool,
    virtual_time_ms: i64,
) -> TurnResult<(Vec<Memory>, String)> {
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
    if immersive && virtual_time_ms > 0 {
        MemoryEngine::apply_time_decay_batch(
            &mut memories,
            virtual_time_ms,
            &role.pack_memory_config,
        );
    }
    Ok((memories, user_relation_key))
}

async fn apply_memory_reinforcement(
    state: &crate::state::AppState,
    role: &Role,
    srid: &str,
    memories: &[Memory],
    user_emotion_str: &str,
    event_runtime: f64,
    mut personality: PersonalityVector,
    mut mutable_for_prompt: String,
) -> TurnResult<(PersonalityVector, String)> {
    let mem_cfg = &role.pack_memory_config;
    if role.evolution_config.personality_source != PersonalitySource::Profile {
        for m in memories {
            if m.mention_count >= mem_cfg.reinforced_mention_threshold {
                personality = PersonalityEngine::evolve_by_reinforced_memory(
                    personality,
                    &m.content,
                    &user_emotion_str,
                    event_runtime,
                    &role.evolution_bounds,
                );
            }
        }
    } else {
        let mut important_memory_archive_dirty = false;
        for m in memories {
            if m.mention_count >= mem_cfg.reinforced_mention_threshold {
                let snippet =
                    crate::domain::profile_personality::memory_snippet_for_profile(&m.content);
                let first_date = m.created_at.format("%Y-%m-%d").to_string();
                let next_archive = crate::domain::profile_personality::upsert_important_memory_section(
                    &mutable_for_prompt,
                    &snippet,
                    &first_date,
                    m.mention_count,
                );
                if next_archive != mutable_for_prompt {
                    mutable_for_prompt = next_archive;
                    important_memory_archive_dirty = true;
                }

                let line = format!("因反复提及「{snippet}」，相关性格倾向略有沉淀。");
                mutable_for_prompt = crate::domain::relation_estrangement::append_mutable_profile_section(
                    &mutable_for_prompt,
                    "记忆塑造",
                    &line,
                );
            }
        }
        if important_memory_archive_dirty {
            let trimmed =
                crate::domain::profile_personality::trim_mutable_storage(&mutable_for_prompt);
            STAGES
                .stage(
                    ChatStage::SetMutablePersonality,
                    state.db_manager.set_mutable_personality(srid, &trimmed),
                )
                .await?;
            mutable_for_prompt = trimmed;
            personality = crate::domain::profile_personality::effective_vector_from_profile(
                role,
                &mutable_for_prompt,
            );
            state
                .session_cache
                .personality_cache()
                .set(srid.to_string(), personality.clone());
        }
    }
    Ok((personality, mutable_for_prompt))
}

async fn rank_relevant_memories(
    pl: &crate::domain::plugin_host::ResolvedRolePlugins,
    memories: &[Memory],
    user_message: &str,
    scene_id: &str,
) -> TurnResult<Vec<Memory>> {
    STAGES
        .stage(
            ChatStage::MemoryRank,
            async {
                SlotRunner::rank_memories(
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
        .await
}

async fn resolve_relation_before_turn(
    state: &crate::state::AppState,
    role: &Role,
    srid: &str,
    user_relation_key: &str,
    immersive: bool,
) -> TurnResult<(String, f64)> {
    let seed_favor = role.initial_favorability_for_relation(user_relation_key);
    STAGES
        .stage(
            ChatStage::EnsureIdentityStatsRow,
            state
                .db_manager
                .ensure_identity_stats_row(srid, user_relation_key, seed_favor),
        )
        .await?;

    crate::domain::relation_estrangement::apply_estrangement_at_turn_start(
        state.db_manager.as_ref(),
        role,
        srid,
        user_relation_key,
        immersive,
    )
    .await
    .map_err(|e| super::super::turn_error::TurnError::wrap("estrangement", e))?;

    let (rel_id, rel_global, favorability_before) = tokio::try_join!(
        async {
            STAGES
                .stage(
                    ChatStage::RelationStateForIdentity,
                    state
                        .db_manager
                        .get_relation_state_for_identity(srid, user_relation_key),
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
                            user_relation_key,
                        ),
                )
                .await
        },
    )?;
    let relation_before = rel_id
        .or(rel_global)
        .unwrap_or_else(|| "Stranger".to_string());
    Ok((relation_before, favorability_before))
}

pub(crate) async fn pre_llm(ctx: &TurnContext<'_>) -> TurnResult<PreLlmOutput> {
    let state = ctx.state;
    let req = ctx.req;
    let role = ctx.role;
    let scene_id = ctx.scene_id;
    let srid = ctx.srid;
    let user_message = req.user_message.as_str();
    let pl = &ctx.pl;

    let (
        event_runtime,
        mut mutable_for_prompt,
        mut personality,
        (recent_turns, recent_turns_for_event, recent_events_for_event),
    ) = prefetch_context(state, role, srid).await?;
    (personality, mutable_for_prompt) = apply_time_evolution(
        state,
        role,
        srid,
        ctx.immersive,
        ctx.virtual_time_ms,
        personality,
        mutable_for_prompt,
    )
    .await?;
    let (emotion_result, user_emotion, user_emotion_str, user_emotion_prompt) =
        resolve_user_emotion_for_turn(pl, user_message).await?;
    let ollama_model = crate::domain::effective_llm_model::resolve_effective_ollama_model(
        state,
        role,
        srid,
    )
    .await
    .map_err(|e| super::super::turn_error::TurnError::wrap("resolve_llm_model", e))?;
    let prev_stored_narrative_hint = load_prev_narrative_hint(state, srid).await;
    if role.evolution_config.personality_source != PersonalitySource::Profile {
        personality = PersonalityEngine::adjust_by_user_emotion(
            personality,
            &user_emotion_str,
            &role.evolution_bounds,
        );
    }
    let (memories, user_relation_key) =
        load_memories_and_relation_key(
            state,
            role,
            srid,
            scene_id,
            pl,
            user_message,
            ctx.immersive,
            ctx.virtual_time_ms,
        )
        .await?;
    (personality, mutable_for_prompt) = apply_memory_reinforcement(
        state,
        role,
        srid,
        &memories,
        &user_emotion_str,
        event_runtime,
        personality,
        mutable_for_prompt,
    )
    .await?;
    let relevant = rank_relevant_memories(pl, &memories, user_message, scene_id).await?;
    let (relation_before, favorability_before) = resolve_relation_before_turn(
        state,
        role,
        srid,
        user_relation_key.as_str(),
        ctx.immersive,
    )
    .await?;

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
    let t_main_llm = Instant::now();
    let mut main_llm_fallback = false;
    let mut llm_fallback_reason = None;
    let reply_raw = match SlotRunner::generate_llm(pl, pre.ollama_model.as_str(), &middle.prompt)
        .await
    {
        Ok(s) => s,
        Err(e) => {
            let reason = e.to_frontend_error();
            tracing::warn!("{path_label} LLM generate failed, fallback: {reason}");
            main_llm_fallback = true;
            llm_fallback_reason = Some(reason);
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
        llm_fallback_reason,
        main_llm_ms,
    })
}

struct PostTurnPolicy {
    bot_emotion: Emotion,
    bot_emotion_str: String,
    event: Event,
    memory_line: String,
    memory_importance: f64,
    recent_events: Vec<Event>,
}

struct PostPersistOutcome {
    favor_current: f64,
    movement: bool,
    portrait_emotion_str: String,
}

struct ChatAppendIds {
    user_message_id: Option<String>,
    assistant_message_id: Option<String>,
    user_message_timestamp: Option<String>,
    assistant_message_timestamp: Option<String>,
}

async fn analyze_bot_emotion_and_policy(
    state: &crate::state::AppState,
    pl: &crate::domain::plugin_host::ResolvedRolePlugins,
    policies: std::sync::Arc<crate::infrastructure::policy_registry::PolicySet>,
    srid: &str,
    user_message: &str,
    reply: &str,
    pre: &PreLlmOutput,
    middle: &MiddleOutput,
) -> TurnResult<PostTurnPolicy> {
    let previous_emotion_fut = state.db_manager.get_current_emotion(srid);
    let bot_emotion_result = STAGES
        .stage(
            ChatStage::BotReplyEmotionAnalyze,
            async { SlotRunner::analyze_emotion(pl, reply) },
        )
        .await?;
    let previous_emotion = STAGES
        .stage(ChatStage::GetCurrentEmotion, previous_emotion_fut)
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
        reply,
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
        .collect();
    Ok(PostTurnPolicy {
        bot_emotion,
        bot_emotion_str,
        event,
        memory_line,
        memory_importance,
        recent_events,
    })
}

#[allow(clippy::too_many_arguments)]
fn spawn_profile_evolution_after_llm(
    state: &crate::state::AppState,
    primary_llm: Arc<dyn crate::domain::ports::LlmClient>,
    role: &Role,
    srid: &str,
    path_label: &str,
    pre: &PreLlmOutput,
    middle: &MiddleOutput,
    user_message: &str,
    reply: &str,
) {
    if role.evolution_config.personality_source != PersonalitySource::Profile {
        return;
    }
    let impact_scaled = (middle.ai_impact_factor_final * pre.event_runtime).clamp(-1.0, 1.0);
    spawn_mutable_profile_evolution(
        Arc::clone(&state.db_manager),
        Arc::clone(&state.session_cache),
        primary_llm,
        role.clone(),
        srid.to_string(),
        path_label.to_string(),
        pre.ollama_model.clone(),
        user_message.to_string(),
        reply.to_string(),
        pre.user_emotion_str.clone(),
        middle.ai_event_type,
        impact_scaled,
    );
}

#[allow(clippy::too_many_arguments)]
async fn persist_atomic_movement_portrait(
    state: &crate::state::AppState,
    mode: TurnMode,
    policies: std::sync::Arc<crate::infrastructure::policy_registry::PolicySet>,
    primary_llm: Arc<dyn crate::domain::ports::LlmClient>,
    role: &Role,
    scene_id: &str,
    scenes: Arc<[String]>,
    srid: &str,
    user_message: &str,
    pre: &PreLlmOutput,
    middle: &MiddleOutput,
    policy: &PostTurnPolicy,
    reply: &str,
) -> TurnResult<PostPersistOutcome> {
    let core_v = PersonalityVector::from(&role.default_personality);
    let reply_for_portrait = reply.to_string();
    let movement_fut = detect_movement_intent(
        state,
        &primary_llm,
        role,
        srid,
        scene_id,
        &scenes,
        user_message,
        pre.ollama_model.as_str(),
    );
    let portrait_fut = if matches!(mode, TurnMode::CoPresent) {
        None
    } else {
        Some(STAGES.stage(
            ChatStage::PortraitEmotionLlm,
            resolve_portrait_emotion(
                &primary_llm,
                pre.ollama_model.as_str(),
                role,
                &core_v,
                &middle.personality,
                pre.favorability_before,
                user_message,
                &reply_for_portrait,
                pre.user_emotion_str.as_str(),
                &policy.bot_emotion,
                &policy.recent_events,
                &pre.recent_turns,
            ),
        ))
    };
    let atomic_fut = STAGES.stage(
        ChatStage::ApplyChatTurnAtomic,
        state.db_manager.apply_chat_turn_atomic(crate::infrastructure::db::ChatTurnTxInput {
            role_id: srid,
            personality: &middle.personality,
            current_emotion: policy.bot_emotion_str.as_str(),
            relation_state: middle.relation_after.as_str(),
            user_relation_key: pre.user_relation_key.as_str(),
            favor_delta: middle.favor_delta,
            memory_content: &policy.memory_line,
            memory_importance: policy.memory_importance,
            memory_fifo_limit: policies.memory.fifo_limit(),
            memory_similarity_threshold: role.pack_memory_config.similarity_threshold,
            event: &policy.event,
            user_message,
            bot_reply: reply,
            scene_id,
        }),
    );
    if let Some(portrait_fut) = portrait_fut {
        let (favor_current, movement, portrait_res) =
            tokio::join!(atomic_fut, movement_fut, portrait_fut);
        Ok(PostPersistOutcome {
            favor_current: favor_current?,
            movement,
            portrait_emotion_str: portrait_res?,
        })
    } else {
        let (favor_current, movement) = tokio::join!(atomic_fut, movement_fut);
        Ok(PostPersistOutcome {
            favor_current: favor_current?,
            movement,
            portrait_emotion_str: policy.bot_emotion_str.clone(),
        })
    }
}

async fn append_turn_to_chat_storage(
    state: &crate::state::AppState,
    mode: TurnMode,
    mrid: &str,
    srid: &str,
    scene_id: &str,
    role: &Role,
    pre: &PreLlmOutput,
    llm: &MainLlmOutput,
    policy: &PostTurnPolicy,
    user_message: &str,
    reply: &str,
) -> ChatAppendIds {
    let mut ids = ChatAppendIds {
        user_message_id: None,
        assistant_message_id: None,
        user_message_timestamp: None,
        assistant_message_timestamp: None,
    };
    if !matches!(mode, TurnMode::CoPresent) || reply.trim().is_empty() {
        return ids;
    }
    let persist = crate::infrastructure::chat_storage::TurnPersistInput {
        session_id: srid.to_string(),
        role_id: mrid.to_string(),
        scene_id: scene_id.to_string(),
        user_message: user_message.to_string(),
        assistant_reply: reply.to_string(),
        reply_is_fallback: llm.main_llm_fallback,
        model_name: Some(pre.ollama_model.clone()),
        response_ms: llm.main_llm_ms,
        user_emotion: Some(pre.user_emotion_str.clone()),
        bot_emotion: Some(policy.bot_emotion_str.clone()),
        max_messages_per_session: role.pack_chat_storage_config.max_messages_per_session,
        auto_cleanup_config: crate::infrastructure::chat_storage::AutoCleanupConfig::from_role_config(
            &role.pack_chat_storage_config,
        ),
        chat_storage_location: role.pack_chat_storage_config.location.clone(),
    };
    match state.conversation_store.append_turn(persist).await {
        Ok(stored) => {
            ids.user_message_id = Some(stored.user_message_id);
            ids.assistant_message_id = Some(stored.assistant_message_id);
            ids.user_message_timestamp = Some(stored.user_message_timestamp);
            ids.assistant_message_timestamp = Some(stored.assistant_message_timestamp);
        }
        Err(e) => {
            tracing::warn!(
                target: "oclive_chat_storage",
                session_id = %srid,
                error = %e,
                "append_turn failed"
            );
        }
    }
    ids
}

async fn persist_non_profile_personality_delta(
    state: &crate::state::AppState,
    role: &Role,
    srid: &str,
    middle: &MiddleOutput,
) -> TurnResult<()> {
    if role.evolution_config.personality_source == PersonalitySource::Profile {
        return Ok(());
    }
    let core_v = PersonalityVector::from(&role.default_personality);
    let delta_out = PersonalityVector::sub_components(&middle.personality, &core_v);
    STAGES
        .stage(
            ChatStage::SetCoreDeltaPersonalityJsonNonProfile,
            state.db_manager.set_core_delta_personality_json(
                srid,
                &core_v.to_json_vec(),
                &delta_out.to_json_vec(),
            ),
        )
        .await?;
    state
        .session_cache
        .personality_cache()
        .set(srid.to_string(), middle.personality.clone());
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn assemble_send_message_response(
    mode: TurnMode,
    immersive: bool,
    scene_id: &str,
    middle: &MiddleOutput,
    pre: &PreLlmOutput,
    llm: &MainLlmOutput,
    policy: &PostTurnPolicy,
    persist: &PostPersistOutcome,
    chat_ids: &ChatAppendIds,
    movement: bool,
    user_message: &str,
    reply: String,
) -> SendMessageResponse {
    use crate::models::dto::{
        DetectedEventDto, PresenceMode, API_VERSION, SCHEMA_VERSION,
    };

    let (mut offer_destination_picker, mut offer_together_travel) =
        movement_ui_flags(movement, user_message);
    if matches!(mode, TurnMode::CoPresent) && !immersive {
        offer_destination_picker = false;
        offer_together_travel = false;
    }
    let (presence_mode, events) = match mode {
        TurnMode::CoPresent => (
            PresenceMode::CoPresent,
            vec![DetectedEventDto {
                event_type: policy.event.event_type.as_ref().to_string(),
                confidence: middle.ai_event_confidence,
            }],
        ),
        TurnMode::RemoteLife => (PresenceMode::RemoteLife, vec![]),
    };
    SendMessageResponse {
        api_version: API_VERSION,
        schema: SCHEMA_VERSION,
        presence_mode,
        relation_state: middle.relation_after.as_str().to_string(),
        reply,
        emotion: emotion_to_dto(&pre.emotion_result),
        bot_emotion: policy.bot_emotion_str.clone(),
        portrait_emotion: persist.portrait_emotion_str.clone(),
        favorability_delta: middle.favor_delta as f32,
        favorability_current: persist.favor_current as f32,
        events,
        scene_id: scene_id.to_string(),
        offer_destination_picker,
        offer_together_travel,
        reply_is_fallback: llm.main_llm_fallback,
        llm_fallback_reason: llm.llm_fallback_reason.clone(),
        knowledge_chunks_in_prompt: middle.knowledge_chunk_count,
        timestamp: chrono::Utc::now().timestamp_millis(),
        user_message_id: chat_ids.user_message_id.clone(),
        assistant_message_id: chat_ids.assistant_message_id.clone(),
        user_message_timestamp: chat_ids.user_message_timestamp.clone(),
        assistant_message_timestamp: chat_ids.assistant_message_timestamp.clone(),
    }
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
    let state = ctx.state;
    let role = ctx.role;
    let scene_id = ctx.scene_id;
    let scenes = std::sync::Arc::clone(&ctx.scenes);
    let mrid = ctx.mrid;
    let srid = ctx.srid;
    let t0 = ctx.t0;
    let preflight_ms = ctx.preflight_ms;
    let immersive = ctx.immersive;
    let pl = &ctx.pl;
    let user_message = ctx.req.user_message.as_str();
    let policies = state.policies_for_scene(Some(scene_id));
    let primary_llm = SlotRunner::primary_llm(pl);

    let t_post_llm = Instant::now();
    let reply = llm.reply.clone();

    let policy = analyze_bot_emotion_and_policy(
        state,
        pl,
        std::sync::Arc::clone(&policies),
        srid,
        user_message,
        &reply,
        pre,
        middle,
    )
    .await?;

    spawn_profile_evolution_after_llm(
        state,
        Arc::clone(&primary_llm),
        role,
        srid,
        path_label,
        pre,
        middle,
        user_message,
        &reply,
    );

    let persist_out = persist_atomic_movement_portrait(
        state,
        mode,
        policies,
        primary_llm,
        role,
        scene_id,
        scenes,
        srid,
        user_message,
        pre,
        middle,
        &policy,
        &reply,
    )
    .await?;

    if matches!(mode, TurnMode::CoPresent) {
        crate::domain::complex_emotion_store::persist_stored_narrative_hint(
            state,
            srid,
            middle.complex_emotion_out.narrative_hint.clone(),
        )
        .await;
    }

    let chat_ids = append_turn_to_chat_storage(
        state, mode, mrid, srid, scene_id, role, pre, llm, &policy, user_message, &reply,
    )
    .await;

    persist_non_profile_personality_delta(state, role, srid, middle).await?;

    let (offer_destination_picker, offer_together_travel) =
        movement_ui_flags(persist_out.movement, user_message);
    let (offer_destination_picker, offer_together_travel) = if matches!(mode, TurnMode::CoPresent)
        && !immersive
    {
        (false, false)
    } else {
        (offer_destination_picker, offer_together_travel)
    };

    let post_llm_ms = t_post_llm.elapsed().as_millis() as u64;
    let duration_ms = t0.elapsed().as_millis() as u64;
    tracing::info!(
        target: "oclive_chat",
        path_label = path_label,
        role_id = %mrid,
        scene_id = %scene_id,
        duration_ms = duration_ms,
        main_llm_fallback = llm.main_llm_fallback,
        offer_destination_picker = offer_destination_picker,
        offer_together_travel = offer_together_travel,
        "send_message end",
    );
    tracing::debug!(
        target: "oclive_chat",
        path_label = path_label,
        preflight_ms = preflight_ms,
        pre_main_llm_ms = pre_main_llm_ms,
        main_llm_ms = llm.main_llm_ms,
        post_llm_ms = post_llm_ms,
        duration_ms = duration_ms,
        "send_message stage timing",
    );

    Ok(assemble_send_message_response(
        mode,
        immersive,
        scene_id,
        middle,
        pre,
        llm,
        &policy,
        &persist_out,
        &chat_ids,
        persist_out.movement,
        user_message,
        reply,
    ))
}
