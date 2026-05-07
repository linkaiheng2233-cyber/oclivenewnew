//! Module 9: EventTrigger nodes — after a chat turn, write structured memories via [`MemoryRepository`].

use crate::error::Result;
use crate::models::expert_models::{ExpertGraph, ExpertNode};
use crate::state::KernelAppState;

/// After `apply_chat_turn_atomic`, evaluate `EventTrigger` nodes on the **effective** expert graph
/// (session override JSON → role default) and persist matching memories.
pub async fn apply_expert_graph_event_triggers_after_turn(
    state: &KernelAppState,
    manifest_role_id: &str,
    session_namespace: &str,
    user_message: &str,
    bot_reply: &str,
) -> Result<()> {
    let raw_sess = state
        .expert_models_repo
        .get_expert_models_session_override_json(session_namespace)
        .await?;
    let graph = if let Some(s) = raw_sess.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        serde_json::from_str::<ExpertGraph>(s).ok()
    } else {
        None
    };
    let graph = if let Some(g) = graph {
        g
    } else {
        let raw_role = state
            .expert_models_repo
            .get_expert_models_role_default_json(manifest_role_id)
            .await?;
        let Some(s) = raw_role.as_deref().map(str::trim).filter(|s| !s.is_empty()) else {
            return Ok(());
        };
        serde_json::from_str::<ExpertGraph>(s).unwrap_or_default()
    };

    for n in &graph.nodes {
        let ExpertNode::EventTrigger {
            match_substring,
            memory_content,
            importance,
            enabled,
            ..
        } = n
        else {
            continue;
        };
        if !*enabled {
            continue;
        }
        let needle = match_substring.trim();
        if needle.is_empty() || memory_content.trim().is_empty() {
            continue;
        }
        let hit_user = user_message.contains(needle);
        let hit_bot = bot_reply.contains(needle);
        if !hit_user && !hit_bot {
            continue;
        }
        let imp = if importance.is_finite() && *importance >= 0.0 {
            *importance as f64
        } else {
            0.5
        };
        let _ = state
            .memory_repo
            .save_memory(session_namespace, memory_content.trim(), imp)
            .await?;
    }
    Ok(())
}
