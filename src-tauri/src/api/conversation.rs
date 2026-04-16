//! 对话会话列表（短期记忆命名空间聚合）。

use crate::state::AppState;
use serde_json::{json, Value};

pub async fn get_conversation_list_impl(state: &AppState) -> Result<Value, String> {
    let rows = state
        .db_manager
        .list_conversation_sessions()
        .await
        .map_err(|e| e.to_frontend_error())?;
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
