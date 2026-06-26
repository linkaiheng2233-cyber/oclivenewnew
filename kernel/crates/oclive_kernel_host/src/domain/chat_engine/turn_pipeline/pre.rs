//! Pre-LLM context loading and turn favor helpers.

use crate::domain::chat_turn::{relation_favor_for_key, weight_memories_for_scene};
use crate::domain::complex_emotion::{affect_metrics_from_seven_dim, ComplexEmotionInput};
use crate::domain::emotion_analyzer::EmotionResult;
use crate::domain::memory_retrieval::MemoryRetrievalInput;
use crate::domain::personality_engine::PersonalityEngine;
use crate::domain::slot_runner::SlotRunner;
use crate::models::knowledge::KnowledgeChunk;
use crate::models::{Emotion, Event, Memory, PersonalitySource, PersonalityVector, Role};
use oclive_kernel_runtime::domain::memory_engine::MemoryEngine;
use oclive_kernel_runtime::domain::relation_engine::RelationState;

use super::super::favor::{compute_favor_and_relation, FavorRelationInput};
use super::super::relation_snapshot::load_relation_snapshot;
use super::super::staged::StageRunner;
use super::super::turn_context::TurnContext;
use super::super::turn_error::TurnResult;
use crate::domain::chat_engine::chat_stage::ChatStage;

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
        extension: None,
    }
}

pub(crate) struct PreLlmMemory {
    pub event_runtime: f64,
    pub mutable_for_prompt: String,
    pub personality: PersonalityVector,
    pub recent_turns: Vec<(String, String)>,
    pub recent_turns_for_event: Vec<(String, String)>,
    pub recent_events_for_event: Vec<Event>,
    pub ollama_model: String,
    pub relevant: Vec<Memory>,
}

pub(crate) struct PreLlmRelation {
    pub user_relation_key: String,
    pub user_identity_id: String,
    pub user_identity_template: String,
    pub relation_hint: String,
    pub relation_before: String,
    pub favorability_before: f64,
    pub relation_transition_hint: String,
}

pub(crate) struct PreLlmHints {
    pub emotion_result: EmotionResult,
    pub user_emotion: Emotion,
    pub user_emotion_str: String,
    pub user_emotion_prompt: String,
    pub prev_stored_narrative_hint: String,
}

pub(crate) struct PreLlmOutput {
    pub memory: PreLlmMemory,
    pub relation: PreLlmRelation,
    pub hints: PreLlmHints,
}

pub(crate) struct MiddleOutput {
    pub turn_thinking: crate::domain::turn_thinking::TurnThinkingPlan,
    pub complex_emotion_out: crate::domain::complex_emotion::ComplexEmotionOutput,
    pub knowledge_chunk_count: u32,
    pub ai_event_type: crate::models::EventType,
    pub ai_impact_factor_final: f64,
    pub ai_event_confidence: f32,
    pub personality: PersonalityVector,
    pub prompt: String,
    pub favor_delta: f64,
    pub relation_after: RelationState,
    pub prompt_stable_hash: Option<u64>,
    pub prompt_stable_len: Option<usize>,
    pub prefix_cache_expected_hit: Option<bool>,
    pub use_ollama_prefix_opts: bool,
}

pub(crate) struct MainLlmOutput {
    pub reply: String,
    pub main_llm_fallback: bool,
    pub llm_fallback_reason: Option<String>,
    pub main_llm_ms: u64,
    pub llm_prompt_eval_ms: Option<u64>,
}

async fn prefetch_context(
    ctx: &TurnContext<'_>,
) -> TurnResult<(
    f64,
    String,
    PersonalityVector,
    (Vec<(String, String)>, Vec<(String, String)>, Vec<Event>),
)> {
    let state = ctx.state;
    let role = ctx.role;
    let srid = ctx.srid;
    let snapshot = &ctx.runtime_snapshot;
    let event_runtime = snapshot
        .event_impact_factor
        .unwrap_or(role.evolution_config.event_impact_factor);
    let mutable_for_prompt = snapshot.mutable_personality.clone().unwrap_or_default();
    let personality = if role.evolution_config.personality_source == PersonalitySource::Profile {
        STAGES
            .stage(ChatStage::CurrentPersonality, async {
                Ok(
                    crate::domain::profile_personality::effective_vector_from_profile(
                        role,
                        snapshot.mutable_personality.as_deref().unwrap_or(""),
                    ),
                )
            })
            .await?
    } else {
        STAGES
            .stage(
                ChatStage::CurrentPersonality,
                state.get_current_personality(srid, role),
            )
            .await?
    };
    let pf = &ctx.prefetch;
    Ok((
        event_runtime,
        mutable_for_prompt,
        personality,
        (
            pf.recent_turns.to_vec(),
            pf.recent_turns_for_event.to_vec(),
            pf.recent_events.to_vec(),
        ),
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

/// Six-slot emotion analyzer policy for this user message (orchestration, not path lookup).
async fn resolve_user_emotion_for_turn(
    pl: &crate::domain::plugin_host::ResolvedRolePlugins,
    user_message: &str,
) -> TurnResult<(EmotionResult, Emotion, String, String)> {
    let emotion_result = STAGES
        .stage(ChatStage::UserEmotionAnalyze, async {
            SlotRunner::analyze_emotion(pl, user_message)
        })
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

#[allow(clippy::too_many_arguments)]
async fn load_memories_and_relation_key(
    ctx: &TurnContext<'_>,
) -> TurnResult<(
    Vec<Memory>,
    crate::domain::user_identity_loader::ResolvedUserIdentity,
)> {
    let state = ctx.state;
    let role = ctx.role;
    let srid = ctx.srid;
    let scene_id = ctx.scene_id;
    let immersive = ctx.immersive;
    let virtual_time_ms = ctx.virtual_time_ms;
    let resolved_identity = ctx.prefetch.resolved_identity.clone();
    let mut memories = STAGES
        .stage(
            ChatStage::LoadMemories,
            state.memory_repo.load_memories(srid, 10),
        )
        .await?;
    let scene_m = role
        .memory_config
        .as_ref()
        .map(|m| m.scene_weight_multiplier)
        .unwrap_or(1.0);
    weight_memories_for_scene(&mut memories, scene_id, scene_m);
    let cfg = &role.pack_memory_config;
    if immersive && virtual_time_ms > 0 {
        MemoryEngine::decay_memories_in_place(
            &mut memories,
            |m| {
                oclive_kernel_runtime::domain::virtual_time::virtual_days_between_ms(
                    m.created_at.timestamp_millis(),
                    virtual_time_ms,
                )
            },
            cfg,
        );
    } else {
        let now_ms = chrono::Utc::now().timestamp_millis();
        MemoryEngine::decay_memories_in_place(
            &mut memories,
            |m| {
                let ref_ms = m.accessed_at.unwrap_or(m.created_at).timestamp_millis();
                oclive_kernel_runtime::domain::virtual_time::virtual_days_between_ms(ref_ms, now_ms)
            },
            cfg,
        );
    }
    memories = MemoryEngine::filter_for_prompt_threshold(memories, cfg);
    Ok((memories, resolved_identity))
}

#[allow(clippy::too_many_arguments)]
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
                    user_emotion_str,
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
                let next_archive =
                    crate::domain::profile_personality::upsert_important_memory_section(
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
                mutable_for_prompt =
                    crate::domain::relation_estrangement::append_mutable_profile_section(
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
    state: &crate::state::AppState,
    srid: &str,
    pl: &crate::domain::plugin_host::ResolvedRolePlugins,
    memories: &[Memory],
    user_message: &str,
    scene_id: &str,
    decay_source: &[Memory],
) -> TurnResult<Vec<Memory>> {
    let limit = state.host_profile.memory_retrieval.retrieval_limit();
    let mut relevant = STAGES
        .stage(ChatStage::MemoryRank, async {
            SlotRunner::rank_memories(
                pl,
                MemoryRetrievalInput {
                    memories,
                    user_query: user_message,
                    scene_id: Some(scene_id),
                    limit,
                },
            )
        })
        .await?;
    let now = chrono::Utc::now();
    for m in &mut relevant {
        m.accessed_at = Some(now);
    }
    if let Err(e) = state
        .db_manager
        .persist_memory_decay_batch(srid, decay_source, &relevant)
        .await
    {
        tracing::warn!(
            target: "oclive_memory",
            role_id = %srid,
            error = %e,
            "persist_memory_decay_batch failed"
        );
    }
    Ok(relevant)
}

/// Turn-start relation seed + estrangement policy before favor/event stages run.
async fn resolve_relation_before_turn(
    state: &crate::state::AppState,
    role: &Role,
    srid: &str,
    user_relation_key: &str,
    immersive: bool,
    runtime_snapshot: &crate::domain::role_runtime_snapshot::RoleRuntimeSnapshot,
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
            load_relation_snapshot(state, srid, user_relation_key, Some(runtime_snapshot)),
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

    let wave1_start = std::time::Instant::now();
    let (
        (
            event_runtime,
            mut mutable_for_prompt,
            mut personality,
            (recent_turns, recent_turns_for_event, recent_events_for_event),
        ),
        (emotion_result, user_emotion, user_emotion_str, user_emotion_prompt),
        ollama_model,
        prev_stored_narrative_hint,
        (memories, resolved_identity),
    ) = tokio::try_join!(
        prefetch_context(ctx),
        resolve_user_emotion_for_turn(pl, user_message),
        async {
            crate::domain::effective_llm_model::resolve_effective_ollama_model(state, role, srid)
                .await
                .map_err(|e| super::super::turn_error::TurnError::wrap("resolve_llm_model", e))
        },
        async {
            Ok::<String, super::super::turn_error::TurnError>(
                load_prev_narrative_hint(state, srid).await,
            )
        },
        load_memories_and_relation_key(ctx),
    )?;
    tracing::debug!(
        target: "oclive_turn",
        stage = "pre_llm_wave1",
        elapsed_ms = wave1_start.elapsed().as_millis() as u64,
        "pre_llm wave1 parallel prefetch complete"
    );

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
    if role.evolution_config.personality_source != PersonalitySource::Profile {
        personality = PersonalityEngine::adjust_by_user_emotion(
            personality,
            &user_emotion_str,
            &role.evolution_bounds,
        );
    }
    let user_relation_key = resolved_identity.relation_key.clone();
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
    let relevant = rank_relevant_memories(
        state,
        srid,
        pl,
        &memories,
        user_message,
        scene_id,
        &memories,
    )
    .await?;
    let (relation_before, favorability_before) = resolve_relation_before_turn(
        state,
        role,
        srid,
        user_relation_key.as_str(),
        ctx.immersive,
        &ctx.runtime_snapshot,
    )
    .await?;
    let transition = crate::domain::relation_transition::consume_relation_transition_at_turn_start(
        &state.session_cache,
        state.db_manager.as_ref(),
        role,
        srid,
    )
    .await
    .map_err(|e| super::super::turn_error::TurnError::wrap("relation_transition", e))?;
    if transition.profile_strip_needed
        && role.evolution_config.personality_source == PersonalitySource::Profile
    {
        mutable_for_prompt = STAGES
            .stage(
                ChatStage::MutablePersonality,
                state.db_manager.get_mutable_personality(srid),
            )
            .await?;
    }

    Ok(PreLlmOutput {
        memory: PreLlmMemory {
            event_runtime,
            mutable_for_prompt,
            personality,
            recent_turns,
            recent_turns_for_event,
            recent_events_for_event,
            ollama_model,
            relevant,
        },
        relation: PreLlmRelation {
            user_relation_key,
            user_identity_id: resolved_identity.identity_id,
            user_identity_template: resolved_identity.template_body,
            relation_hint: resolved_identity.relation_hint,
            relation_before,
            favorability_before,
            relation_transition_hint: transition.hint,
        },
        hints: PreLlmHints {
            emotion_result,
            user_emotion,
            user_emotion_str,
            user_emotion_prompt,
            prev_stored_narrative_hint,
        },
    })
}

pub(crate) fn compute_turn_favor(
    pre: &PreLlmOutput,
    role: &Role,
    ai_event_type: &crate::models::EventType,
    ai_impact_factor_final: f64,
    ai_event_confidence: f32,
) -> (f64, RelationState) {
    let rf = relation_favor_for_key(role, pre.relation.user_relation_key.as_str());
    let favor_relation_input = FavorRelationInput {
        relation_before: pre.relation.relation_before.as_str(),
        favorability_before: pre.relation.favorability_before,
        ai_event_type,
        ai_impact_factor_final,
        event_runtime: pre.memory.event_runtime,
        favor_mult: rf.favor_mult,
        event_confidence: ai_event_confidence,
        recent_events_for_event: &pre.memory.recent_events_for_event,
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
