//! 会话列表元数据（短期记忆命名空间聚合）。

use crate::error::Result;
use crate::state::KernelAppState;
use serde_json::{json, Value};

pub async fn get_conversation_list(state: &KernelAppState) -> Result<Value> {
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
