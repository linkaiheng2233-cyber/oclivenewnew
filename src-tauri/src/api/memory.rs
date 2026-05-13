use crate::models::dto::{MemoryItem, QueryMemoriesRequest};
use crate::state::AppState;
use tauri::State;

pub async fn query_memories_impl(
    state: &AppState,
    req: &QueryMemoriesRequest,
) -> Result<Vec<MemoryItem>, String> {
    oclive_kernel_runtime::domain::memory_query::query_memories(state, req)
        .await
        .map_err(|e| e.to_frontend_error())
}

#[tauri::command]
pub async fn query_memories(
    req: QueryMemoriesRequest,
    state: State<'_, AppState>,
) -> Result<Vec<MemoryItem>, String> {
    query_memories_impl(&state, &req).await
}
