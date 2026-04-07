use crate::error::AppError;
use crate::models::dto::{MemoryItem, QueryMemoriesRequest};
use crate::state::AppState;
use tauri::State;

pub async fn query_memories_impl(
    state: &AppState,
    req: &QueryMemoriesRequest,
) -> Result<Vec<MemoryItem>, String> {
    if req.limit <= 0 || req.limit > 100 {
        return Err(
            AppError::InvalidParameter("limit must be between 1 and 100".to_string())
                .to_frontend_error(),
        );
    }
    if req.offset < 0 {
        return Err(
            AppError::InvalidParameter("offset must be >= 0".to_string()).to_frontend_error(),
        );
    }

    let memories = state
        .memory_repo
        .load_memories_paged(&req.role_id, req.limit, req.offset)
        .await
        .map_err(|e| e.to_frontend_error())?;

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

#[tauri::command]
pub async fn query_memories(
    req: QueryMemoriesRequest,
    state: State<'_, AppState>,
) -> Result<Vec<MemoryItem>, String> {
    query_memories_impl(&state, &req).await
}
