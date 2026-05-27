use crate::domain::chat_engine::process_message;
use crate::infrastructure::chat_storage::{
    collect_chat_storage_stats, delete_mirror_scene_dir, delete_mirror_tree_for_role,
    resolve_max_messages_per_session, role_mirror_tree_bytes, DeleteChatsResult,
    ImportChatBucket, ImportChatBucketsResult, RoleStorageStat,
    SessionMeta, StoredMessage,
};
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

/// List chat sessions for a role + scene (SQLite authoritative).
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
            limit.unwrap_or(500),
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
    let session_id = session_id.trim().to_string();
    let max = match state.db_manager.get_chat_session(&session_id).await {
        Ok(Some(session)) => match state.load_role_cached_async(&session.role_id).await {
            Ok(role) => resolve_max_messages_per_session(
                role.pack_chat_storage_config.max_messages_per_session,
            ),
            Err(_) => resolve_max_messages_per_session(None),
        },
        _ => resolve_max_messages_per_session(None),
    };
    state
        .conversation_store
        .rebuild_mirror(&session_id, max)
        .await
        .map_err(Into::into)
}

/// Import chat buckets from frontend IndexedDB (one-time migration).
///
/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
#[tauri::command]
pub async fn migrate_indexeddb_to_backend(
    buckets: Vec<ImportChatBucket>,
    state: State<'_, AppState>,
) -> Result<ImportChatBucketsResult, crate::api::error::CommandError> {
    state
        .conversation_store
        .import_chat_buckets(buckets)
        .await
        .map_err(Into::into)
}

/// Storage usage grouped by role and scene (mirror files + SQLite session metadata).
///
/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
#[tauri::command]
pub async fn get_chat_storage_stats(
    state: State<'_, AppState>,
) -> Result<Vec<RoleStorageStat>, crate::api::error::CommandError> {
    collect_chat_storage_stats(
        state.directory_plugins.app_data_dir(),
        state.db_manager.as_ref(),
    )
    .await
    .map_err(Into::into)
}

/// Delete all chat history for a manifest role (SQLite + mirror tree).
///
/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
#[tauri::command]
pub async fn delete_role_chats(
    role_id: String,
    state: State<'_, AppState>,
) -> Result<DeleteChatsResult, crate::api::error::CommandError> {
    let rid = role_id.trim();
    let app_data = state.directory_plugins.app_data_dir();
    let bytes = role_mirror_tree_bytes(app_data, rid).await?;
    let sessions_deleted = state
        .db_manager
        .count_chat_sessions_for_manifest_role(rid)
        .await?;
    state.db_manager.delete_chat_data_for_manifest_role(rid).await?;
    delete_mirror_tree_for_role(app_data, rid).await?;
    Ok(DeleteChatsResult {
        sessions_deleted,
        bytes_freed: bytes,
    })
}

/// Delete chat history for one role + scene.
///
/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
#[tauri::command]
pub async fn delete_scene_chats(
    role_id: String,
    scene_id: String,
    state: State<'_, AppState>,
) -> Result<DeleteChatsResult, crate::api::error::CommandError> {
    let rid = role_id.trim();
    let sid = scene_id.trim();
    let app_data = state.directory_plugins.app_data_dir();
    let bytes = delete_mirror_scene_dir(app_data, rid, sid).await?;
    let sessions_deleted = state.db_manager.delete_chat_data_for_role_scene(rid, sid).await?;
    Ok(DeleteChatsResult {
        sessions_deleted,
        bytes_freed: bytes,
    })
}
