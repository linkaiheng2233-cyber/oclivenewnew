//! Post-LLM persistence: atomic DB writes, chat storage, profile evolution.

use crate::domain::portrait_emotion_engine::pick_portrait_emotion;
use crate::domain::portrait_facility::{
    pick_portrait_with_catalog, portrait_catalog_active, resolve_visual_state_rule,
};
use crate::domain::ports::conversation_persist::{TurnAutoCleanupConfig, TurnPersistRequest};
use crate::domain::ports::turn_persistence::ChatTurnAtomicInput;
use crate::models::{Event, PersonalitySource, PersonalityVector, Role};
use std::sync::Arc;

use super::super::scene::detect_movement_intent;
use super::super::turn_context::TurnIds;
use super::super::turn_error::TurnResult;
use super::pre::{MainLlmOutput, MiddleOutput, PreLlmOutput, STAGES};
use super::TurnMode;
use crate::domain::chat_engine::chat_stage::ChatStage;

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
    pub visual_state_id: Option<String>,
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
    let turn_policies = state.turn_policies_for_scene(Some(scene_id));
    let turn_persistence = state.chat_turn_persistence_port();
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
        pre.memory.ollama_model.as_str(),
    );
    let portrait_fut = if matches!(mode, TurnMode::CoPresent) {
        None
    } else {
        Some(STAGES.stage(
            ChatStage::PortraitEmotionLlm,
            async {
                if role.pack_portrait_catalog.enabled {
                    let Some(catalog) = role.portrait_catalog.as_ref() else {
                        tracing::warn!(
                            target: "oclive_chat",
                            role_id = %role.id,
                            "portrait catalog enabled but catalog file missing; falling back to legacy portrait picker"
                        );
                        let tag = pick_portrait_emotion(
                            &primary_llm,
                            pre.memory.ollama_model.as_str(),
                            role,
                            &core_v,
                            &middle.personality,
                            pre.relation.favorability_before,
                            user_message,
                            &reply_for_portrait,
                            pre.hints.user_emotion_str.as_str(),
                            &policy.bot_emotion,
                            &policy.recent_events,
                            &pre.memory.recent_turns,
                        )
                        .await?;
                        return Ok((tag, None));
                    };
                    let narrative_hint_owned = {
                        let current = middle.complex_emotion_out.narrative_hint.trim();
                        if !current.is_empty() {
                            Some(current.to_string())
                        } else {
                            let stored = state
                                .session_cache
                                .stored_complex_emotion_narrative_hint(srid);
                            let t = stored.trim();
                            if t.is_empty() {
                                None
                            } else {
                                Some(t.to_string())
                            }
                        }
                    };
                    let (tag, vsid) = pick_portrait_with_catalog(
                        &primary_llm,
                        pre.memory.ollama_model.as_str(),
                        role,
                        catalog,
                        &core_v,
                        &middle.personality,
                        pre.relation.favorability_before,
                        user_message,
                        &reply_for_portrait,
                        pre.hints.user_emotion_str.as_str(),
                        &policy.bot_emotion,
                        &policy.recent_events,
                        &pre.memory.recent_turns,
                        narrative_hint_owned.as_deref(),
                    )
                    .await?;
                    Ok((tag, Some(vsid)))
                } else {
                    let tag = pick_portrait_emotion(
                        &primary_llm,
                        pre.memory.ollama_model.as_str(),
                        role,
                        &core_v,
                        &middle.personality,
                        pre.relation.favorability_before,
                        user_message,
                        &reply_for_portrait,
                        pre.hints.user_emotion_str.as_str(),
                        &policy.bot_emotion,
                        &policy.recent_events,
                        &pre.memory.recent_turns,
                    )
                    .await?;
                    Ok((tag, None))
                }
            },
        ))
    };
    let atomic_fut = STAGES.stage(
        ChatStage::ApplyChatTurnAtomic,
        turn_persistence.apply_chat_turn_atomic(ChatTurnAtomicInput {
            role_id: srid,
            personality: &middle.personality,
            current_emotion: policy.bot_emotion_str.as_str(),
            relation_state: middle.relation_after.as_str(),
            user_relation_key: pre.relation.user_relation_key.as_str(),
            favor_delta: middle.favor_delta,
            memory_content: &policy.memory_line,
            memory_importance: policy.memory_importance,
            memory_fifo_limit: turn_policies.memory_fifo_limit,
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
        let favor_current = favor_current?;
        if matches!(mode, TurnMode::CoPresent) {
            crate::domain::relation_transition::maybe_start_relation_transition(
                &state.session_cache,
                state.db_manager.as_ref(),
                role,
                srid,
                pre.relation.relation_before.as_str(),
                middle.relation_after.as_str(),
                middle.favor_delta,
            )
            .await
            .map_err(|e| super::super::turn_error::TurnError::wrap("relation_transition", e))?;
        }
        let (portrait_emotion_str, visual_state_id) = portrait_res?;
        Ok(PostPersistOutcome {
            favor_current,
            movement,
            portrait_emotion_str,
            visual_state_id,
        })
    } else {
        let (favor_current, movement) = tokio::join!(atomic_fut, movement_fut);
        let favor_current = favor_current?;
        if matches!(mode, TurnMode::CoPresent) {
            crate::domain::relation_transition::maybe_start_relation_transition(
                &state.session_cache,
                state.db_manager.as_ref(),
                role,
                srid,
                pre.relation.relation_before.as_str(),
                middle.relation_after.as_str(),
                middle.favor_delta,
            )
            .await
            .map_err(|e| super::super::turn_error::TurnError::wrap("relation_transition", e))?;
        }
        Ok(PostPersistOutcome {
            favor_current,
            movement,
            portrait_emotion_str: policy.bot_emotion_str.clone(),
            visual_state_id: resolve_visual_state_for_role(role, policy.bot_emotion_str.as_str()),
        })
    }
}

async fn append_turn_inner(
    state: &crate::state::AppState,
    srid: &str,
    persist: TurnPersistRequest,
    log_label: &str,
) -> ChatAppendIds {
    let store = state.conversation_persist_port();
    let mut ids = ChatAppendIds::default();
    match store.append_turn(persist).await {
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
    let persist = TurnPersistRequest {
        session_id: srid.to_string(),
        role_id: mrid.to_string(),
        scene_id: scene_id.to_string(),
        user_message: user_message.to_string(),
        assistant_reply: reply.to_string(),
        reply_is_fallback: llm.main_llm_fallback,
        model_name: Some(pre.memory.ollama_model.clone()),
        response_ms: llm.main_llm_ms,
        user_emotion: Some(pre.hints.user_emotion_str.clone()),
        bot_emotion: Some(policy.bot_emotion_str.clone()),
        max_messages_per_session: role.pack_chat_storage_config.max_messages_per_session,
        auto_cleanup_config: TurnAutoCleanupConfig::from_role_config(
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
    let persist = TurnPersistRequest {
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
        auto_cleanup_config: TurnAutoCleanupConfig::from_role_config(
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

fn resolve_visual_state_for_role(role: &Role, emotion_tag: &str) -> Option<String> {
    if !portrait_catalog_active(role) {
        return None;
    }
    let catalog = role.portrait_catalog.as_ref()?;
    resolve_visual_state_rule(catalog, emotion_tag)
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
            featured: false,
            deep_capsule_enabled: false,
            deep_capsule: None,
            preset_order: 999,
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
            pack_portrait_catalog: Default::default(),
            portrait_catalog: None,
            pack_visual_presentation_config: Default::default(),
            source_dir: None,
        }
    }

    fn middle_with_personality(personality: PersonalityVector) -> MiddleOutput {
        MiddleOutput {
            turn_thinking: crate::domain::turn_thinking::TurnThinkingPlan {
                mode: crate::domain::turn_thinking::TurnThinkingMode::Deep,
                reasons: vec![],
            },
            complex_emotion_out: ComplexEmotionOutput {
                source: "builtin".to_string(),
                narrative_hint: String::new(),
                labels: vec![],
                pattern: None,
                confidence: 0.0,
                intensity: 0.0,
                dissonance_score: 0.0,
                degraded_to_builtin: false,
                extension: None,
            },
            knowledge_chunk_count: 0,
            ai_event_type: EventType::Praise,
            ai_impact_factor_final: 0.0,
            ai_event_confidence: 0.0,
            personality,
            prompt: String::new(),
            favor_delta: 0.0,
            relation_after: RelationState::Stranger,
            prompt_stable_hash: None,
            prompt_stable_len: None,
            prefix_cache_expected_hit: None,
            use_ollama_prefix_opts: false,
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

    #[test]
    fn portrait_catalog_enabled_without_file_does_not_panic() {
        let mut role = vector_role();
        role.pack_portrait_catalog.enabled = true;
        role.portrait_catalog = None;
        assert!(!portrait_catalog_active(&role));
        assert_eq!(resolve_visual_state_for_role(&role, "happy"), None);
    }
}
