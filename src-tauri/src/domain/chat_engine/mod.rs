//! 对话编排：串联各 domain 模块与 Repository / LLM
//!
//! 具体步骤中的纯逻辑见 [`super::chat_turn`]，本模块负责异步编排与 `AppState` 交互。
//! 场景与好感子逻辑见 [`context`]、[`scene`]、[`favor`]。

mod co_present;
mod context;
mod favor;
mod presence;
mod scene;

use crate::domain::chat_llm_fallback::{fallback_reply_for_llm_failure, FallbackReplyContext};
use crate::domain::chat_turn::{relation_favor_for_key, weight_memories_for_scene};
use crate::domain::chat_turn_rules::{soft_append_guard, strip_hallucination_tokens};
use crate::domain::life_schedule::{format_life_prompt_line, resolve_life_state};
use crate::domain::memory_retrieval::MemoryRetrievalInput;
use crate::domain::personality_engine::PersonalityEngine;
use crate::domain::policy::PolicyContext;
use crate::domain::portrait_emotion_engine::resolve_portrait_emotion;
use crate::domain::remote_life_prompt::{build_remote_life_prompt, compose_remote_stub_reply};
use crate::domain::user_identity::resolve_effective_user_relation_key;
use crate::error::Result;
use crate::models::dto::{
    EmotionDto, PresenceMode, SendMessageRequest, SendMessageResponse, API_VERSION, SCHEMA_VERSION,
};
use crate::models::{
    Event, EventType, KnowledgeIndex, Memory, PersonalitySource, PersonalityVector, Role,
};
use crate::state::AppState;
use chrono::Utc;
use context::{load_recent_context, validate_scene_id};
use favor::{compute_favor_and_relation, FavorRelationInput};
use presence::user_is_remote_from_character;
use scene::{detect_movement_intent, movement_ui_flags};
use std::sync::Arc;
use std::time::Instant;

pub(super) fn emotion_to_dto(r: &crate::domain::emotion_analyzer::EmotionResult) -> EmotionDto {
    EmotionDto {
        joy: r.joy as f32,
        sadness: r.sadness as f32,
        anger: r.anger as f32,
        fear: r.fear as f32,
        surprise: r.surprise as f32,
        disgust: r.disgust as f32,
        neutral: r.neutral as f32,
    }
}

/// 会话级 SQLite 命名空间：HTTP 试聊传入 `session_id` 时与无 `session_id` 的默认对话隔离。
pub(crate) fn conversation_state_role_id(
    manifest_role_id: &str,
    session_id: Option<&str>,
) -> String {
    /// 控制 SQLite 键与日志长度，避免异常长 `session_id` 撑爆存储。
    const MAX_SUFFIX_CHARS: usize = 64;
    const MAX_TOTAL_CHARS: usize = 256;

    let sid = session_id.map(str::trim).filter(|s| !s.is_empty());
    match sid {
        None => manifest_role_id.chars().take(MAX_TOTAL_CHARS).collect(),
        Some(s) => {
            let safe: String = s
                .chars()
                .map(|c| {
                    if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                        c
                    } else {
                        '_'
                    }
                })
                .take(MAX_SUFFIX_CHARS)
                .collect();
            let out = format!("{}__sess__{}", manifest_role_id, safe);
            out.chars().take(MAX_TOTAL_CHARS).collect()
        }
    }
}

/// 异地 + 关：占位文案，**不**写入短期记忆 / 事件 / 好感事务（避免无对话却涨好感）
async fn process_remote_stub(
    state: &AppState,
    req: &SendMessageRequest,
    role: &Role,
    scene_id: &str,
    t0: Instant,
    srid: &str,
    preflight_ms: u64,
) -> Result<SendMessageResponse> {
    let role_id = req.role_id.as_str();
    let user_message = req.user_message.as_str();
    let pl = state.resolved_plugins_for(role);
    let emotion_result = pl.emotion.analyze(user_message)?;
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

/// 异地 + 开：专用 LLM；跳过事件影响探测，以 `Ignore` + 零振幅参与好感事务（仍更新短期记忆等）
#[allow(clippy::too_many_arguments)] // 与 `process_co_present` 同级编排入口
async fn process_remote_life(
    state: &AppState,
    req: &SendMessageRequest,
    role: &Role,
    scene_id: &str,
    character_scene_id: &str,
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

    let pl = state.resolved_plugins_for(role);
    let emotion_result = pl.emotion.analyze(user_message)?;
    let user_emotion = emotion_result.to_emotion();
    let user_emotion_str = user_emotion.to_string();

    let ollama_model = role.resolve_ollama_model(state.ollama_model.as_str());
    let (recent_turns, _recent_turns_for_event, recent_events_for_event) =
        load_recent_context(state, srid).await?;

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
    let favor_relation_input = FavorRelationInput {
        relation_before: relation_before.as_str(),
        favorability_before,
        ai_event_type: &ai_event_type,
        ai_impact_factor_final,
        event_runtime,
        favor_mult: rf.favor_mult,
        event_confidence: ai_event_confidence,
        recent_events_for_event: &recent_events_for_event,
    };
    let (favor_delta, relation_after) = compute_favor_and_relation(&favor_relation_input);

    let char_label = state.storage.scene_display_name(mrid, character_scene_id);
    let user_label = state.storage.scene_display_name(mrid, scene_id);
    let away_material = state
        .storage
        .away_life_material(mrid, character_scene_id, scene_id);
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

    // 与共景一致：按角色当前所在场景过滤世界观知识（异地心声以角色侧场景为准）
    let knowledge_chunks = role
        .knowledge_index
        .as_ref()
        .map(|idx| idx.retrieve(user_message, Some(character_scene_id), 8))
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
    );

    let pre_main_llm_ms = t_path.elapsed().as_millis() as u64;
    let t_main_llm = Instant::now();
    let mut main_llm_fallback = false;
    let reply_raw = match pl.llm.generate(ollama_model.as_str(), &prompt).await {
        Ok(s) => s,
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
    let bot_emotion_result = pl.emotion.analyze(&reply)?;
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
    .await?;

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

    if role.evolution_config.personality_source == PersonalitySource::Profile {
        let prev = state.db_manager.get_mutable_personality(srid).await?;
        let impact_scaled = (ai_impact_factor_final * event_runtime).clamp(-1.0, 1.0);
        let next = match crate::domain::mutable_profile_llm::evolve_mutable_personality_with_llm(
            &pl.llm,
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
    let movement = detect_movement_intent(
        state,
        &pl.llm,
        mrid,
        srid,
        scene_id,
        &scenes,
        user_message,
        ollama_model.as_str(),
    )
    .await;
    let (offer_destination_picker, offer_together_travel) =
        movement_ui_flags(movement, user_message);

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

/// 处理一条用户消息：分析情绪 → 检测事件 → 演化性格 → 构建 Prompt → 调用 LLM → 持久化
pub async fn process_message(
    state: &AppState,
    req: &SendMessageRequest,
) -> Result<SendMessageResponse> {
    let mrid = req.role_id.as_str();
    let state_rid = conversation_state_role_id(mrid, req.session_id.as_deref());
    let srid = state_rid.as_str();
    let requested_scene_id = req
        .scene_id
        .clone()
        .unwrap_or_else(|| "default".to_string());
    let (scene_id, scenes) = validate_scene_id(state, mrid, requested_scene_id)?;
    let t0 = Instant::now();
    log::debug!(
        target: "oclive_chat",
        "send_message start role_id={} scene_id={} session_ns={}",
        mrid,
        scene_id,
        srid
    );

    state.db_manager.ensure_role_runtime(srid).await?;

    let role = ensure_role_loaded(state, mrid).await?;
    state
        .db_manager
        .ensure_interaction_mode_seeded(srid, role.interaction_mode.as_deref())
        .await?;

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

async fn ensure_role_loaded(state: &AppState, role_id: &str) -> Result<Arc<Role>> {
    state.load_role_cached(role_id)
}

#[cfg(test)]
mod tests {
    use crate::domain::chat_engine::conversation_state_role_id;
    use crate::domain::chat_turn_rules::{
        avoid_fast_promote_score, smooth_favor_delta_for_short_streak, soft_append_guard,
    };
    use crate::models::{Event, EventType};

    #[test]
    fn conversation_state_role_id_none_matches_manifest_id() {
        assert_eq!(conversation_state_role_id("role_a", None), "role_a");
    }

    #[test]
    fn conversation_state_role_id_distinct_sessions() {
        let a = conversation_state_role_id("role_a", Some("sess-1"));
        let b = conversation_state_role_id("role_a", Some("sess-2"));
        assert_ne!(a, b);
        assert!(a.contains("__sess__"));
    }

    #[test]
    fn conversation_state_role_id_caps_total_length() {
        let long = "x".repeat(400);
        let out = conversation_state_role_id("r", Some(&long));
        assert!(out.chars().count() <= 256);
    }

    #[test]
    fn soft_append_triggers_for_quarrel_with_sweet_words() {
        let reply = "宝贝别生气呀，抱抱你，我最想你了";
        let out = soft_append_guard(reply, &EventType::Quarrel, -0.3, "Friend");
        assert!(out.len() > reply.len());
        assert!(out.contains("先"));
    }

    #[test]
    fn soft_append_triggers_for_low_stage_with_strong_promise() {
        let reply = "我想和你永远在一起，这辈子都不离不弃";
        let out = soft_append_guard(reply, &EventType::Praise, 0.4, "Stranger");
        assert!(out.len() > reply.len());
        assert!(out.contains("慢慢"));
    }

    #[test]
    fn soft_append_not_triggered_for_normal_positive_reply() {
        let reply = "今天和你聊天很开心，我们继续聊聊最近的事吧。";
        let out = soft_append_guard(reply, &EventType::Praise, 0.5, "Friend");
        assert_eq!(out, reply);
    }

    #[test]
    fn avoid_fast_promote_detects_consecutive_positive_streak() {
        let recent = vec![
            Event {
                event_type: EventType::Praise,
                user_emotion: "joy".to_string(),
                bot_emotion: "joy".to_string(),
            },
            Event {
                event_type: EventType::Confession,
                user_emotion: "joy".to_string(),
                bot_emotion: "surprise".to_string(),
            },
        ];
        let score = avoid_fast_promote_score(&EventType::Praise, 0.7, &recent);
        assert!(score >= 0.7);
    }

    #[test]
    fn avoid_fast_promote_ignores_weak_or_broken_streak() {
        let weak = avoid_fast_promote_score(&EventType::Praise, 0.3, &[]);
        assert_eq!(weak, 0.0);

        let broken = vec![
            Event {
                event_type: EventType::Joke,
                user_emotion: "neutral".to_string(),
                bot_emotion: "joy".to_string(),
            },
            Event {
                event_type: EventType::Praise,
                user_emotion: "joy".to_string(),
                bot_emotion: "joy".to_string(),
            },
        ];
        let broken_score = avoid_fast_promote_score(&EventType::Confession, 0.8, &broken);
        assert_eq!(broken_score, 0.0);
    }

    #[test]
    fn avoid_fast_promote_does_not_apply_to_negative_events() {
        let recent = vec![Event {
            event_type: EventType::Praise,
            user_emotion: "joy".to_string(),
            bot_emotion: "joy".to_string(),
        }];
        let score = avoid_fast_promote_score(&EventType::Quarrel, -0.8, &recent);
        assert_eq!(score, 0.0);
    }

    #[test]
    fn favor_delta_smoothing_applies_on_consecutive_positive_streak() {
        let recent = vec![
            Event {
                event_type: EventType::Praise,
                user_emotion: "joy".to_string(),
                bot_emotion: "joy".to_string(),
            },
            Event {
                event_type: EventType::Confession,
                user_emotion: "joy".to_string(),
                bot_emotion: "joy".to_string(),
            },
        ];
        let out = smooth_favor_delta_for_short_streak(0.1, &recent);
        assert!(out < 0.1);
        assert!(out > 0.08);
    }

    #[test]
    fn favor_delta_smoothing_keeps_non_streak_or_low_amplitude_nearly_same() {
        let broken = vec![
            Event {
                event_type: EventType::Quarrel,
                user_emotion: "anger".to_string(),
                bot_emotion: "anger".to_string(),
            },
            Event {
                event_type: EventType::Praise,
                user_emotion: "joy".to_string(),
                bot_emotion: "joy".to_string(),
            },
        ];
        let unchanged = smooth_favor_delta_for_short_streak(0.1, &broken);
        assert_eq!(unchanged, 0.1);

        let low_amp = smooth_favor_delta_for_short_streak(0.02, &broken);
        assert_eq!(low_amp, 0.02);
    }

    #[test]
    fn favor_delta_smoothing_supports_negative_streak() {
        let recent = vec![
            Event {
                event_type: EventType::Quarrel,
                user_emotion: "anger".to_string(),
                bot_emotion: "anger".to_string(),
            },
            Event {
                event_type: EventType::Complaint,
                user_emotion: "sadness".to_string(),
                bot_emotion: "sadness".to_string(),
            },
        ];
        let out = smooth_favor_delta_for_short_streak(-0.1, &recent);
        assert!(out > -0.1);
        assert!(out < -0.08);
    }
}
