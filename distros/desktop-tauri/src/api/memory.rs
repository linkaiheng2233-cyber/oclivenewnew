use crate::api::error::CommandError;
use crate::error::AppError;
use oclive_kernel_host::state::{AppState, SharedAppState};
use oclive_kernel_types::models::dto::{MemoryItem, QueryMemoriesRequest};
use tauri::State;
/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
pub async fn query_memories_impl(
    state: &AppState,
    req: &QueryMemoriesRequest,
) -> Result<Vec<MemoryItem>, CommandError> {
    if req.limit <= 0 || req.limit > 100 {
        return Err(
            AppError::InvalidParameter("limit must be between 1 and 100".to_string()).into(),
        );
    }
    if req.offset < 0 {
        return Err(AppError::InvalidParameter("offset must be >= 0".to_string()).into());
    }
    let content_scope = req.content_scope.as_deref().map(str::trim);
    if content_scope.is_some_and(|scope| !matches!(scope, "ordinary" | "adult")) {
        return Err(AppError::InvalidParameter(
            "content_scope must be ordinary or adult".to_string(),
        )
        .into());
    }

    let memories = state
        .memory_repo
        .load_memories_paged_for_scope(&req.role_id, req.limit, req.offset, content_scope)
        .await?;

    Ok(memories
        .into_iter()
        .map(|(m, content_scope)| MemoryItem {
            id: m.id,
            role_id: m.role_id,
            content: m.content,
            memory_type: "long_term".to_string(),
            timestamp: m.created_at.to_rfc3339(),
            importance: m.importance,
            content_scope,
        })
        .collect())
}
/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
#[tauri::command]
pub async fn query_memories(
    req: QueryMemoriesRequest,
    state: State<'_, SharedAppState>,
) -> Result<Vec<MemoryItem>, CommandError> {
    query_memories_impl(&state, &req).await
}
