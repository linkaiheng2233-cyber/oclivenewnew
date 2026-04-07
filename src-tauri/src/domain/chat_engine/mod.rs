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
use crate::models::{Event, EventType, KnowledgeIndex, Memory, PersonalityVector, Role};
use crate::state::AppState;
use chrono::Utc;
use context::{load_recent_context, validate_scene_id};
use favor::{compute_favor_and_relation, FavorRelationInput};
use presence::user_is_remote_from_character;
use scene::{detect_movement_intent, movement_ui_flags};
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

/// 异地 + 关：占位文案，**不**写入短期记忆 / 事件 / 好感事务（避免无对话却涨好感）
async fn process_remote_stub(
    state: &AppState,
    req: &SendMessageRequest,
    role: &Role,
    scene_id: &str,
    t0: Instant,
) -> Result<SendMessageResponse> {
    let role_id = req.role_id.as_str();
    let user_message = req.user_message.as_str();
    let pl = state.resolved_plugins_for(role);
    let emotion_result = pl.emotion.analyze(user_message)?;
    let user_relation_key: String =
        resolve_effective_user_relation_key(state, role, role_id, Some(scene_id)).await?;
    let relation_before = state
        .db_manager
        .get_relation_state_for_identity(role_id, user_relation_key.as_str())
        .await?
        .or(state.db_manager.get_relation_state(role_id).await?)
        .unwrap_or_else(|| "Stranger".to_string());
    let favorability_before = state
        .db_manager
        .favorability_for_identity_with_runtime_fallback(role_id, user_relation_key.as_str())
        .await?;
    let portrait_emotion_str = state
        .db_manager
        .get_current_emotion(role_id)
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
async fn process_remote_life(
    state: &AppState,
    req: &SendMessageRequest,
    role: &Role,
    scene_id: &str,
    character_scene_id: &str,
    t0: Instant,
) -> Result<SendMessageResponse> {
    let role_id = req.role_id.as_str();
    let user_message = req.user_message.as_str();
    let event_runtime = state
        .db_manager
        .get_event_impact_factor(role_id)
        .await?
        .unwrap_or(role.evolution_config.event_impact_factor);

    let mut personality = state.get_current_personality(role_id, role).await?;

    let pl = state.resolved_plugins_for(role);
    let emotion_result = pl.emotion.analyze(user_message)?;
    let user_emotion = emotion_result.to_emotion();
    let user_emotion_str = user_emotion.to_string();

    let ollama_model = role.resolve_ollama_model(state.ollama_model.as_str());
    let (recent_turns, _recent_turns_for_event, recent_events_for_event) =
        load_recent_context(state, role_id).await?;

    personality = PersonalityEngine::adjust_by_user_emotion(
        personality,
        &user_emotion_str,
        &role.evolution_bounds,
    );

    let ai_event_type = EventType::Ignore;
    let ai_impact_factor_final = 0.0_f64;
    let ai_event_confidence = 0.0_f32;

    personality = PersonalityEngine::evolve_by_event(
        personality,
        ai_impact_factor_final * event_runtime,
        &role.evolution_bounds,
    );

    let mut memories = state.memory_repo.load_memories(role_id, 10).await?;
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
        resolve_effective_user_relation_key(state, role, role_id, Some(scene_id)).await?;
    let rf = relation_favor_for_key(role, user_relation_key.as_str());

    let relation_before = state
        .db_manager
        .get_relation_state_for_identity(role_id, user_relation_key.as_str())
        .await?
        .or(state.db_manager.get_relation_state(role_id).await?)
        .unwrap_or_else(|| "Stranger".to_string());
    let seed_favor = role.initial_favorability_for_relation(user_relation_key.as_str());
    state
        .db_manager
        .ensure_identity_stats_row(role_id, user_relation_key.as_str(), seed_favor)
        .await?;
    let favorability_before = state
        .db_manager
        .favorability_for_identity_with_runtime_fallback(role_id, user_relation_key.as_str())
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

    let char_label = state
        .storage
        .scene_display_name(role_id, character_scene_id);
    let user_label = state.storage.scene_display_name(role_id, scene_id);
    let away_material = state
        .storage
        .away_life_material(role_id, character_scene_id, scene_id);
    let vt_ms = state
        .db_manager
        .get_virtual_time_ms(role_id)
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
    );

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
    let reply = strip_hallucination_tokens(&soft_append_guard(
        &reply_raw,
        &ai_event_type,
        ai_impact_factor_final,
        relation_after.as_str(),
    ));
    let bot_emotion_result = pl.emotion.analyze(&reply)?;
    let previous_emotion = state.db_manager.get_current_emotion(role_id).await?;
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
            role_id: role_id.to_string(),
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
        role_id,
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
            role_id,
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

    let delta_out = PersonalityVector::sub_components(&personality, &core_v);
    state
        .db_manager
        .set_core_delta_personality_json(role_id, &core_v.to_json_vec(), &delta_out.to_json_vec())
        .await?;

    state
        .personality_cache
        .write()
        .insert(role_id.to_string(), personality.clone());

    let scenes = state.storage.list_scene_ids(role_id)?;
    let movement = detect_movement_intent(
        state,
        &pl.llm,
        role_id,
        scene_id,
        &scenes,
        user_message,
        ollama_model.as_str(),
    )
    .await;
    let (offer_destination_picker, offer_together_travel) =
        movement_ui_flags(movement, user_message);

    let duration_ms = t0.elapsed().as_millis() as u64;
    log::info!(
        target: "oclive_chat",
        "send_message remote_life role_id={} scene_id={} main_llm_fallback={} duration_ms={} offer_destination_picker={} offer_together_travel={}",
        role_id,
        scene_id,
        main_llm_fallback,
        duration_ms,
        offer_destination_picker,
        offer_together_travel
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
    let role_id = req.role_id.as_str();
    let requested_scene_id = req
        .scene_id
        .clone()
        .unwrap_or_else(|| "default".to_string());
    let (scene_id, scenes) = validate_scene_id(state, role_id, requested_scene_id)?;
    let t0 = Instant::now();
    log::info!(
        target: "oclive_chat",
        "send_message start role_id={} scene_id={}",
        role_id,
        scene_id
    );

    state.db_manager.ensure_role_runtime(role_id).await?;

    let role = ensure_role_loaded(state, role_id).await?;
    state
        .db_manager
        .ensure_interaction_mode_seeded(role_id, role.interaction_mode.as_deref())
        .await?;

    state
        .db_manager
        .set_user_presence_scene(role_id, scene_id.as_str())
        .await?;

    let current_scene = state.db_manager.get_current_scene(role_id).await?;
    let immersive = state
        .db_manager
        .get_interaction_mode(role_id)
        .await?
        .is_immersive();
    let remote_life_enabled = state.db_manager.get_remote_life_enabled(role_id).await?;
    let is_remote =
        immersive && user_is_remote_from_character(scene_id.as_str(), current_scene.as_deref());
    if is_remote {
        if !remote_life_enabled {
            return process_remote_stub(state, req, &role, scene_id.as_str(), t0).await;
        }
        let char_scene = current_scene.as_deref().unwrap_or("default");
        return process_remote_life(state, req, &role, scene_id.as_str(), char_scene, t0).await;
    }

    co_present::process_co_present(state, req, &role, scene_id, scenes, immersive, t0).await
}

async fn ensure_role_loaded(state: &AppState, role_id: &str) -> Result<Role> {
    if let Some(r) = state.role_cache.read().get(role_id).cloned() {
        return Ok(r);
    }
    let role = state.storage.load_role(role_id)?;
    state
        .role_cache
        .write()
        .insert(role_id.to_string(), role.clone());
    Ok(role)
}

#[cfg(test)]
mod tests {
    use crate::domain::chat_turn_rules::{
        avoid_fast_promote_score, smooth_favor_delta_for_short_streak, soft_append_guard,
    };
    use crate::models::{Event, EventType};

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
