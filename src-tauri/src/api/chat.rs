use crate::domain::chat_engine::process_message;
use crate::infrastructure::chat_storage::{SessionMeta, StoredMessage};
use crate::models::dto::{SendMessageRequest, SendMessageResponse};
use crate::state::AppState;
use tauri::State;

/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
#[tauri::command]
pub async fn send_message(
    req: SendMessageRequest,
    state: State<'_, AppState>,
) -> Result<SendMessageResponse, crate::api::error::CommandError> {
    process_message(&state, &req).await.map_err(Into::into)
}

/// List chat sessions for a role + scene (SQLite authoritative; step-1 API only).
///
/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
#[tauri::command]
pub async fn list_chat_sessions(
    role_id: String,
    scene_id: String,
    limit: Option<u32>,
    offset: Option<u32>,
    state: State<'_, AppState>,
) -> Result<Vec<SessionMeta>, crate::api::error::CommandError> {
    state
        .conversation_store
        .list_sessions(
            role_id.trim(),
            scene_id.trim(),
            limit.unwrap_or(50),
            offset.unwrap_or(0),
        )
        .await
        .map_err(Into::into)
}

/// Fetch paginated messages for a chat session.
///
/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
#[tauri::command]
pub async fn fetch_chat_messages(
    session_id: String,
    limit: Option<u32>,
    offset: Option<u32>,
    state: State<'_, AppState>,
) -> Result<Vec<StoredMessage>, crate::api::error::CommandError> {
    state
        .conversation_store
        .fetch_messages(
            session_id.trim(),
            limit.unwrap_or(100),
            offset.unwrap_or(0),
        )
        .await
        .map_err(Into::into)
}

/// Rebuild JSON mirror for one session from SQLite (admin / repair).
///
/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
#[tauri::command]
pub async fn rebuild_chat_mirror(
    session_id: String,
    state: State<'_, AppState>,
) -> Result<String, crate::api::error::CommandError> {
    state
        .conversation_store
        .rebuild_mirror(session_id.trim())
        .await
        .map_err(Into::into)
}
