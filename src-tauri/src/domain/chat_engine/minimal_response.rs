//! Agent 短路与共景路径共用的最小响应字段（关系 / 好感 / 肖像情绪）。

use crate::domain::plugin_host::ResolvedRolePlugins;
use crate::domain::slot_runner::SlotRunner;
use crate::domain::user_identity::resolve_effective_user_relation_key;
use crate::error::Result;
use crate::models::dto::{
    PresenceMode, SendMessageResponse, API_VERSION, SCHEMA_VERSION,
};
use crate::models::Role;
use crate::state::AppState;

use super::emotion_to_dto;

/// 并行加载关系 / 好感 / 肖像情绪，组装 Agent 短路等最小 [`SendMessageResponse`]。
pub(crate) async fn build_minimal_response(
    state: &AppState,
    pl: &ResolvedRolePlugins,
    role: &Role,
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

    let (rel_id, rel_global, favor_current, portrait_emotion) = tokio::try_join!(
        state
            .db_manager
            .get_relation_state_for_identity(srid, user_relation_key.as_str()),
        state.db_manager.get_relation_state(srid),
        state
            .db_manager
            .favorability_for_identity_with_runtime_fallback(srid, user_relation_key.as_str()),
        async {
            state
                .db_manager
                .get_current_emotion(srid)
                .await
                .map(|e| e.unwrap_or_else(|| "neutral".to_string()))
        },
    )?;

    let relation_state = rel_id
        .or(rel_global)
        .unwrap_or_else(|| "Stranger".to_string());

    Ok(SendMessageResponse {
        api_version: API_VERSION,
        schema: SCHEMA_VERSION,
        presence_mode: PresenceMode::CoPresent,
        relation_state,
        reply,
        emotion: emotion_to_dto(&emotion_result),
        bot_emotion: portrait_emotion.clone(),
        portrait_emotion,
        favorability_delta: 0.0,
        favorability_current: favor_current as f32,
        events: vec![],
        scene_id,
        offer_destination_picker: false,
        offer_together_travel: false,
        reply_is_fallback: false,
        knowledge_chunks_in_prompt: 0,
        timestamp: chrono::Utc::now().timestamp_millis(),
    })
}
