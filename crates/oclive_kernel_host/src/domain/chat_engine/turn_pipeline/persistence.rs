//! Post-LLM persistence: atomic DB writes, chat storage, profile evolution.

use crate::domain::portrait_emotion_engine::resolve_portrait_emotion;
use crate::models::{Event, PersonalitySource, PersonalityVector, Role};
use std::sync::{Arc, LazyLock};
use tokio::sync::Semaphore;

use super::super::scene::detect_movement_intent;
use super::super::turn_context::TurnIds;
use super::super::turn_error::TurnResult;
use super::pre::{MainLlmOutput, MiddleOutput, PreLlmOutput, STAGES};
use super::TurnMode;
use crate::domain::chat_engine::chat_stage::ChatStage;

static MUTABLE_PROFILE_EVOLUTION_SEM: LazyLock<Semaphore> = LazyLock::new(|| Semaphore::new(2));

/// Profile mutable-personality LLM + DB writes run off the critical path; next turn reads from DB.
#[allow(clippy::too_many_arguments)]
pub(crate) fn spawn_mutable_profile_evolution(
    db: Arc<crate::infrastructure::db::DbManager>,
    session_cache: Arc<crate::state::SessionCache>,
    primary_llm: Arc<dyn crate::domain::ports::LlmClient>,
    role: Role,
    srid: String,
    path_label: String,
    ollama_model: String,
    user_message: String,
    reply: String,
    user_emotion: String,
    event_type: crate::models::EventType,
    impact_scaled: f64,
) {
    tokio::spawn(async move {
        let Ok(_permit) = MUTABLE_PROFILE_EVOLUTION_SEM.acquire().await else {
            return;
        };
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
            .set_core_delta_personality_json(&srid, &core_v.to_json_vec(), &delta_out.to_json_vec())
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

pub(crate) struct PostTurnPolicy {
    pub bot_emotion: crate::models::Emotion,
    pub bot_emotion_str: String,
    pub event: Event,
    pub memory_line: String,
    pub memory_importance: f64,
    pub recent_events: Vec<Event>,
}

pub(crate) struct PostPersistOutcome {
    pub favor_current: f64,
    pub movement: bool,
    pub portrait_emotion_str: String,
}

#[derive(Debug, Default)]
pub(crate) struct ChatAppendIds {
    pub user_message_id: Option<String>,
    pub assistant_message_id: Option<String>,
    pub user_message_timestamp: Option<String>,
    pub assistant_message_timestamp: Option<String>,
    pub chat_persist_failed: Option<bool>,
    pub chat_persist_error: Option<String>,
}

#[allow(clippy::too_many_arguments)]
pub(crate) async fn persist_atomic_movement_portrait(
    state: &crate::state::AppState,
    mode: TurnMode,
    policies: std::sync::Arc<crate::infrastructure::policy_registry::PolicySet>,
    primary_llm: Arc<dyn crate::domain::ports::LlmClient>,
    role: &Role,
    ids: TurnIds<'_>,
    scenes: Arc<[String]>,
    user_message: &str,
    pre: &PreLlmOutput,
    middle: &MiddleOutput,
    policy: &PostTurnPolicy,
    reply: &str,
) -> TurnResult<PostPersistOutcome> {
    let TurnIds { srid, scene_id, .. } = ids;
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
        state
            .db_manager
            .apply_chat_turn_atomic(crate::infrastructure::db::ChatTurnTxInput {
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

async fn append_turn_inner(
    state: &crate::state::AppState,
    srid: &str,
    persist: crate::infrastructure::chat_storage::TurnPersistInput,
    log_label: &str,
) -> ChatAppendIds {
    let mut ids = ChatAppendIds::default();
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
                "{log_label}"
            );
            ids.chat_persist_failed = Some(true);
            ids.chat_persist_error = Some(e.to_string());
        }
    }
    ids
}

#[allow(clippy::too_many_arguments)]
pub(crate) async fn append_turn_to_chat_storage(
    state: &crate::state::AppState,
    mode: TurnMode,
    ids: TurnIds<'_>,
    role: &Role,
    pre: &PreLlmOutput,
    llm: &MainLlmOutput,
    policy: &PostTurnPolicy,
    user_message: &str,
    reply: &str,
) -> ChatAppendIds {
    let TurnIds {
        mrid,
        srid,
        scene_id,
    } = ids;
    if !matches!(mode, TurnMode::CoPresent) || reply.trim().is_empty() {
        return ChatAppendIds::default();
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
        auto_cleanup_config:
            crate::infrastructure::chat_storage::AutoCleanupConfig::from_role_config(
                &role.pack_chat_storage_config,
            ),
        chat_storage_location: role.pack_chat_storage_config.location.clone(),
    };
    append_turn_inner(state, srid, persist, "append_turn failed").await
}

pub(crate) async fn append_agent_turn_to_chat_storage(
    state: &crate::state::AppState,
    ids: TurnIds<'_>,
    role: &Role,
    user_message: &str,
    reply: &str,
    user_emotion: &str,
    bot_emotion: &str,
) -> ChatAppendIds {
    let TurnIds {
        mrid,
        srid,
        scene_id,
    } = ids;
    if reply.trim().is_empty() {
        return ChatAppendIds::default();
    }
    let persist = crate::infrastructure::chat_storage::TurnPersistInput {
        session_id: srid.to_string(),
        role_id: mrid.to_string(),
        scene_id: scene_id.to_string(),
        user_message: user_message.to_string(),
        assistant_reply: reply.to_string(),
        reply_is_fallback: false,
        model_name: None,
        response_ms: 0,
        user_emotion: Some(user_emotion.to_string()),
        bot_emotion: Some(bot_emotion.to_string()),
        max_messages_per_session: role.pack_chat_storage_config.max_messages_per_session,
        auto_cleanup_config:
            crate::infrastructure::chat_storage::AutoCleanupConfig::from_role_config(
                &role.pack_chat_storage_config,
            ),
        chat_storage_location: role.pack_chat_storage_config.location.clone(),
    };
    append_turn_inner(state, srid, persist, "append_turn (agent) failed").await
}

pub(crate) async fn persist_non_profile_personality_delta(
    state: &crate::state::AppState,
    role: &Role,
    srid: &str,
    middle: &MiddleOutput,
) {
    if role.evolution_config.personality_source == PersonalitySource::Profile {
        return;
    }
    let core_v = PersonalityVector::from(&role.default_personality);
    let delta_out = PersonalityVector::sub_components(&middle.personality, &core_v);
    let db_result = STAGES
        .stage(
            ChatStage::SetCoreDeltaPersonalityJsonNonProfile,
            state.db_manager.set_core_delta_personality_json(
                srid,
                &core_v.to_json_vec(),
                &delta_out.to_json_vec(),
            ),
        )
        .await;
    if let Err(e) = db_result {
        tracing::warn!(
            target: "oclive_chat",
            role_id = %srid,
            error = %e,
            "set_core_delta_personality_json_non_profile failed; chat turn already committed"
        );
        return;
    }
    state
        .session_cache
        .personality_cache()
        .set(srid.to_string(), middle.personality.clone());
}

#[cfg(test)]
mod persist_non_profile_tests {
    use super::*;
    use crate::domain::complex_emotion::ComplexEmotionOutput;
    use crate::models::{EventType, EvolutionBounds, EvolutionConfig, PersonalityDefaults};
    use oclive_kernel_runtime::domain::relation_engine::RelationState;
    use std::sync::Arc;

    fn vector_role() -> Role {
        Role {
            id: "test".to_string(),
            name: "Test".to_string(),
            description: String::new(),
            version: "1".to_string(),
            author: String::new(),
            core_personality: String::new(),
            default_personality: PersonalityDefaults {
                stubbornness: 0.5,
                clinginess: 0.5,
                sensitivity: 0.5,
                assertiveness: 0.5,
                forgiveness: 0.5,
                talkativeness: 0.5,
                warmth: 0.5,
            },
            evolution_bounds: EvolutionBounds::full_01(),
            user_relations: vec![],
            evolution_config: EvolutionConfig {
                personality_source: PersonalitySource::Vector,
                ..EvolutionConfig::default()
            },
            memory_config: None,
            default_relation: "friend".to_string(),
            ollama_model: None,
            identity_binding: crate::models::role::IdentityBinding::default(),
            life_trajectory: None,
            life_schedule: None,
            remote_presence: None,
            autonomous_scene: None,
            interaction_mode: None,
            min_runtime_version: None,
            dev_only: false,
            plugin_backends: Arc::new(crate::models::PluginBackends::default()),
            slot_registry: None,
            slot_groups: None,
            ui_config: crate::models::UiConfig::default(),
            knowledge_index: None,
            author_pack: None,
            reply_quality_anchor: None,
            time_config: crate::models::RoleTimeConfig::default(),
            pack_memory_config: crate::models::RolePackMemoryConfig::default(),
            pack_relation_config: crate::models::RolePackRelationConfig::default(),
            pack_evolution_config: crate::models::RolePackEvolutionConfig::default(),
            pack_chat_storage_config: crate::models::RolePackChatStorageConfig::default(),
            runtime_config: None,
            pipeline_experimental: None,
            scene_ids: Arc::from(Vec::<String>::new()),
            scene_config_cache: Arc::new(
                parking_lot::RwLock::new(std::collections::HashMap::new()),
            ),
            scene_text_cache: Arc::new(parking_lot::RwLock::new(std::collections::HashMap::new())),
            user_identity_catalog: None,
            pack_reply_post_processor_config: Default::default(),
        }
    }

    fn middle_with_personality(personality: PersonalityVector) -> MiddleOutput {
        MiddleOutput {
            complex_emotion_out: ComplexEmotionOutput {
                source: "builtin".to_string(),
                narrative_hint: String::new(),
                labels: vec![],
                pattern: None,
                confidence: 0.0,
                intensity: 0.0,
                dissonance_score: 0.0,
                degraded_to_builtin: false,
            },
            knowledge_chunk_count: 0,
            ai_event_type: EventType::Praise,
            ai_impact_factor_final: 0.0,
            ai_event_confidence: 0.0,
            personality,
            prompt: String::new(),
            favor_delta: 0.0,
            relation_after: RelationState::Stranger,
        }
    }

    #[tokio::test]
    async fn persist_non_profile_personality_delta_db_failure_is_non_fatal() {
        let state = crate::state::AppState::new_in_memory_with_llm(
            Arc::new(crate::infrastructure::llm::MockLlmClient {
                reply: "ok".to_string(),
            }),
            "./roles",
        )
        .await
        .expect("state");
        let srid = "role_delta_fail";
        state
            .db_manager
            .ensure_role_runtime(srid)
            .await
            .expect("ensure runtime");

        let old = PersonalityVector::from(&vector_role().default_personality);
        let new_warmth = 0.9;
        let new_personality = PersonalityVector {
            warmth: new_warmth,
            ..old
        };
        state
            .session_cache
            .personality_cache()
            .set(srid.to_string(), old.clone());

        sqlx::query("DROP TABLE role_runtime")
            .execute(&state.db_manager.pool)
            .await
            .expect("drop role_runtime");

        persist_non_profile_personality_delta(
            &state,
            &vector_role(),
            srid,
            &middle_with_personality(new_personality),
        )
        .await;

        let cached = state
            .session_cache
            .personality_cache()
            .get(srid)
            .expect("cache entry");
        assert_eq!(cached.warmth, old.warmth);
        assert_ne!(cached.warmth, new_warmth);
    }
}
