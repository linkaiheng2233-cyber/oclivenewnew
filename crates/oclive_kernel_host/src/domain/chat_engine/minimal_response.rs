//! Minimal response fields shared by Agent shortcut and co-present paths (relation / favorability / portrait emotion).

use crate::domain::plugin_host::ResolvedRolePlugins;
use crate::domain::ports::turn_persistence::ChatTurnAtomicInput;
use crate::domain::slot_runner::SlotRunner;
use crate::domain::user_identity::resolve_effective_user_relation_key;
use crate::error::Result;
use crate::models::dto::{PresenceMode, SendMessageResponse, API_VERSION, SCHEMA_VERSION};
use crate::models::{Event, EventType, PersonalityVector, Role};
use crate::state::AppState;

use super::emotion_to_dto;
use super::relation_snapshot::load_relation_snapshot;
use super::turn_context::TurnIds;
use super::turn_pipeline::persistence::append_agent_turn_to_chat_storage;

/// Loads relation / favorability / portrait emotion in parallel and builds a minimal [`SendMessageResponse`] for Agent shortcut, etc.
#[allow(clippy::too_many_arguments)]
pub(crate) async fn build_minimal_response(
    state: &AppState,
    pl: &ResolvedRolePlugins,
    role: &Role,
    mrid: &str,
    srid: &str,
    scene_id: String,
    user_message: &str,
    reply: String,
) -> Result<SendMessageResponse> {
    let (_, emotion_result, user_relation_key) = tokio::try_join!(
        state
            .db_manager
            .set_user_presence_scene(srid, scene_id.as_str()),
        async { SlotRunner::analyze_emotion(pl, user_message) },
        resolve_effective_user_relation_key(state, role, srid, Some(scene_id.as_str())),
    )?;

    let snapshot = load_relation_snapshot(state, srid, user_relation_key.as_str()).await?;

    let user_emotion_str = emotion_result.to_emotion().to_string();
    let bot_emotion = snapshot.portrait_emotion.clone();
    let personality = PersonalityVector::from(&role.default_personality);
    let turn_policies = state.turn_policies_for_scene(Some(scene_id.as_str()));
    let turn_persistence = state.chat_turn_persistence_port();
    let neutral_event = Event {
        event_type: EventType::Ignore,
        user_emotion: user_emotion_str.clone(),
        bot_emotion: bot_emotion.clone(),
    };

    let _favor_current = turn_persistence
        .apply_chat_turn_atomic(ChatTurnAtomicInput {
            role_id: srid,
            personality: &personality,
            current_emotion: bot_emotion.as_str(),
            relation_state: snapshot.relation_state.as_str(),
            user_relation_key: user_relation_key.as_str(),
            favor_delta: 0.0,
            memory_content: "",
            memory_importance: 0.0,
            memory_fifo_limit: turn_policies.memory_fifo_limit,
            memory_similarity_threshold: role.pack_memory_config.similarity_threshold,
            event: &neutral_event,
            user_message,
            bot_reply: reply.as_str(),
            scene_id: scene_id.as_str(),
        })
        .await?;

    let chat_ids = append_agent_turn_to_chat_storage(
        state,
        TurnIds {
            mrid,
            srid,
            scene_id: scene_id.as_str(),
        },
        role,
        user_message,
        reply.as_str(),
        user_emotion_str.as_str(),
        bot_emotion.as_str(),
    )
    .await;

    Ok(SendMessageResponse {
        api_version: API_VERSION,
        schema: SCHEMA_VERSION,
        presence_mode: PresenceMode::CoPresent,
        relation_state: snapshot.relation_state,
        reply,
        emotion: emotion_to_dto(&emotion_result),
        bot_emotion: snapshot.portrait_emotion.clone(),
        portrait_emotion: snapshot.portrait_emotion,
        favorability_delta: 0.0,
        favorability_current: snapshot.favorability as f32,
        events: vec![],
        scene_id,
        offer_destination_picker: false,
        offer_together_travel: false,
        reply_is_fallback: false,
        llm_fallback_reason: None,
        knowledge_chunks_in_prompt: 0,
        timestamp: chrono::Utc::now().timestamp_millis(),
        user_message_id: chat_ids.user_message_id,
        assistant_message_id: chat_ids.assistant_message_id,
        user_message_timestamp: chat_ids.user_message_timestamp,
        assistant_message_timestamp: chat_ids.assistant_message_timestamp,
        chat_persist_failed: chat_ids.chat_persist_failed,
        chat_persist_error: chat_ids.chat_persist_error,
        dual_core_degraded: None,
        raw_reply: None,
    })
}
