//! Pre-LLM context loading and turn favor helpers.

use crate::domain::chat_turn::{relation_favor_for_key, weight_memories_for_scene};
use crate::domain::complex_emotion::{affect_metrics_from_seven_dim, ComplexEmotionInput};
use crate::domain::emotion_analyzer::EmotionResult;
use crate::domain::memory_retrieval::MemoryRetrievalInput;
use crate::domain::memory_engine::MemoryEngine;
use crate::domain::personality_engine::PersonalityEngine;
use crate::domain::slot_runner::SlotRunner;
use crate::domain::user_identity::resolve_effective_user_relation_key;
use crate::models::knowledge::KnowledgeChunk;
use crate::models::{Emotion, Event, Memory, PersonalitySource, PersonalityVector, Role};
use oclive_kernel_runtime::domain::relation_engine::RelationState;

use super::super::context::load_recent_context;
use super::super::favor::{compute_favor_and_relation, FavorRelationInput};
use super::super::relation_snapshot::load_relation_snapshot;
use super::super::staged::StageRunner;
use crate::domain::chat_engine::chat_stage::ChatStage;
use super::super::turn_context::TurnContext;
use super::super::turn_error::TurnResult;

pub(crate) const STAGES: StageRunner = StageRunner;

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

pub(crate) fn skipped_complex_emotion() -> crate::domain::complex_emotion::ComplexEmotionOutput {
    crate::domain::complex_emotion::ComplexEmotionOutput {
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
    pub complex_emotion_out: crate::domain::complex_emotion::ComplexEmotionOutput,
    pub knowledge_chunk_count: u32,
    pub ai_event_type: crate::models::EventType,
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

    let snapshot = STAGES
        .stage(
            ChatStage::RelationStateForIdentity,
            load_relation_snapshot(state, srid, user_relation_key),
        )
        .await?;
    Ok((snapshot.relation_state, snapshot.favorability))
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
    ai_event_type: &crate::models::EventType,
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
