//! Conversation session list (short-term memory namespace aggregation).

use crate::command_error::CommandError;
use crate::state::AppState;
use serde_json::{json, Value};

/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
pub async fn get_conversation_list_impl(state: &AppState) -> Result<Value, CommandError> {
    let rows = state.db_manager.list_conversation_sessions().await?;
    let items: Vec<Value> = rows
        .into_iter()
        .map(|(session_namespace, turn_count, last_at)| {
            json!({
                "session_namespace": session_namespace,
                "turn_count": turn_count,
                "last_at": last_at,
            })
        })
        .collect();
    Ok(json!({ "items": items }))
}
