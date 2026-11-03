//! A stopped generation archives only its visible prefix, never inferred effects.

use super::super::relation_snapshot::load_relation_snapshot;
use super::super::turn_context::TurnContext;
use super::super::turn_error::{TurnError, TurnResult};
use super::persistence::{append_agent_turn_to_chat_storage, AgentChatTurn};
use super::TurnMode;
use crate::domain::user_identity::resolve_effective_user_relation_key;
use crate::models::dto::{
    DisplayMetricsDto, EmotionDto, PresenceMode, SendMessageResponse, API_VERSION, SCHEMA_VERSION,
};

pub(super) async fn persist_stopped(
    ctx: &TurnContext<'_>,
    mode: TurnMode,
    reply: &str,
) -> TurnResult<SendMessageResponse> {
    let identity =
        resolve_effective_user_relation_key(ctx.state, ctx.role, ctx.srid, Some(ctx.scene_id))
            .await
            .map_err(|e| TurnError::wrap("stopped_identity", e))?;
    let snapshot =
        load_relation_snapshot(ctx.state, ctx.srid, &identity, Some(&ctx.runtime_snapshot))
            .await
            .map_err(|e| TurnError::wrap("stopped_snapshot", e))?;
    // Tokens already passed through the common status-marker filter. Keep their
    // exact spelling; do not fabricate a fallback or run normal reply post-effects.
    let ids = append_agent_turn_to_chat_storage(
        ctx.state,
        ctx.ids(),
        ctx.role,
        AgentChatTurn {
            user_message: ctx.req.user_message.as_str(),
            reply,
            user_emotion: "neutral",
            bot_emotion: &snapshot.portrait_emotion,
            chat_thread_id: ctx.req.chat_thread_id.as_deref(),
            reply_stopped: true,
        },
    )
    .await;
    Ok(SendMessageResponse {
        api_version: API_VERSION,
        schema: SCHEMA_VERSION,
        presence_mode: match mode {
            TurnMode::CoPresent => PresenceMode::CoPresent,
            TurnMode::RemoteLife => PresenceMode::RemoteLife,
        },
        display_metrics: Some(DisplayMetricsDto {
            favor: snapshot.favorability,
            relation_summary: snapshot.relation_state.clone(),
            traits: vec![],
        }),
        relation_state: snapshot.relation_state,
        reply: reply.to_owned(),
        reply_stopped: true,
        adult_beat: None,
        reply_presentation: None,
        emotion: EmotionDto {
            joy: 0.0,
            sadness: 0.0,
            anger: 0.0,
            fear: 0.0,
            surprise: 0.0,
            disgust: 0.0,
            neutral: 1.0,
        },
        bot_emotion: snapshot.portrait_emotion.clone(),
        portrait_emotion: snapshot.portrait_emotion,
        visual_state_id: None,
        performance_directive: None,
        favorability_delta: 0.0,
        favorability_current: snapshot.favorability as f32,
        events: vec![],
        scene_id: ctx.scene_id.to_owned(),
        offer_destination_picker: false,
        offer_together_travel: false,
        reply_is_fallback: false,
        llm_fallback_reason: None,
        knowledge_chunks_in_prompt: 0,
        timestamp: chrono::Utc::now().timestamp_millis(),
        user_message_id: ids.user_message_id,
        assistant_message_id: ids.assistant_message_id,
        user_message_timestamp: ids.user_message_timestamp,
        assistant_message_timestamp: ids.assistant_message_timestamp,
        chat_persist_failed: ids.chat_persist_failed,
        chat_persist_error: ids.chat_persist_error,
        dual_core_degraded: ctx.dual_core_degraded.then_some(true),
        raw_reply: None,
        llm_prompt_eval_ms: None,
    })
}
