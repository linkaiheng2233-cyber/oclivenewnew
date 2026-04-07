//! 场景校验与近期上下文加载

use crate::error::Result;
use crate::models::Event;
use crate::state::AppState;

pub(super) async fn load_recent_context(
    state: &AppState,
    role_id: &str,
) -> Result<(Vec<(String, String)>, Vec<(String, String)>, Vec<Event>)> {
    let recent_turns = state
        .db_manager
        .list_short_term_recent_turns(role_id, 6)
        .await
        .unwrap_or_default();
    let recent_turns_for_event: Vec<(String, String)> = recent_turns
        .iter()
        .rev()
        .take(5)
        .cloned()
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();
    let recent_events_for_event = state.db_manager.get_events(role_id, 8).await?;
    Ok((
        recent_turns,
        recent_turns_for_event,
        recent_events_for_event,
    ))
}

pub(super) fn validate_scene_id(
    state: &AppState,
    role_id: &str,
    requested_scene_id: String,
) -> Result<(String, Vec<String>)> {
    let mut scene_id = requested_scene_id;
    let scenes = state.storage.list_scene_ids(role_id)?;
    if !scenes.iter().any(|s| s == &scene_id) {
        log::warn!(
            "send_message: invalid scene_id={} for role={}, fallback",
            scene_id,
            role_id
        );
        if scenes.iter().any(|s| s == "default") {
            scene_id = "default".to_string();
        } else {
            scene_id = scenes
                .first()
                .cloned()
                .unwrap_or_else(|| "default".to_string());
        }
    }
    Ok((scene_id, scenes))
}
