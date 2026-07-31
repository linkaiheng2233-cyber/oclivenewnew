//! Scene validation and recent context loading.

use crate::error::Result;
use crate::models::Event;
use crate::state::AppState;
use std::sync::Arc;

pub(crate) async fn load_recent_context(
    state: &AppState,
    role_id: &str,
    include_adult: bool,
) -> Result<(Vec<(String, String)>, Vec<(String, String)>, Vec<Event>)> {
    let (recent_turns, recent_events_for_event) = tokio::try_join!(
        async {
            Ok(state
                .db_manager
                .list_short_term_recent_turns(role_id, 6, include_adult)
                .await
                .unwrap_or_else(|e| {
                    tracing::warn!(
                        target: "oclive_chat",
                        "failed to load recent turns for role_id={role_id}: {e}"
                    );
                    vec![]
                }))
        },
        state.db_manager.get_events(role_id, 8),
    )?;
    let start = recent_turns.len().saturating_sub(5);
    let recent_turns_for_event = recent_turns[start..].to_vec();
    Ok((
        recent_turns,
        recent_turns_for_event,
        recent_events_for_event,
    ))
}

pub(crate) fn validate_scene_id(
    role_id: &str,
    scene_ids: &Arc<[String]>,
    requested_scene_id: String,
) -> String {
    let mut scene_id = requested_scene_id;
    if !scene_ids.iter().any(|s| s == &scene_id) {
        tracing::warn!(
            "send_message: invalid scene_id={} for role={}, fallback",
            scene_id,
            role_id
        );
        if scene_ids.iter().any(|s| s == "default") {
            scene_id = "default".to_string();
        } else {
            scene_id = scene_ids
                .first()
                .cloned()
                .unwrap_or_else(|| "default".to_string());
        }
    }
    scene_id
}
