//! 长期记忆分页查询（无 Tauri 依赖）。

use crate::error::{AppError, Result};
use crate::models::dto::{MemoryItem, QueryMemoriesRequest};
use crate::state::KernelAppState;

pub async fn query_memories(
    state: &KernelAppState,
    req: &QueryMemoriesRequest,
) -> Result<Vec<MemoryItem>> {
    if req.limit <= 0 || req.limit > 100 {
        return Err(AppError::InvalidParameter(
            "limit must be between 1 and 100".to_string(),
        ));
    }
    if req.offset < 0 {
        return Err(AppError::InvalidParameter(
            "offset must be >= 0".to_string(),
        ));
    }

    let memories = state
        .memory_repo
        .load_memories_paged(&req.role_id, req.limit, req.offset)
        .await?;

    Ok(memories
        .into_iter()
        .map(|m| MemoryItem {
            id: m.id,
            role_id: m.role_id,
            content: m.content,
            memory_type: "long_term".to_string(),
            timestamp: m.created_at.to_rfc3339(),
            importance: m.importance,
        })
        .collect())
}
