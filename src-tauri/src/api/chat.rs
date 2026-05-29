use crate::domain::chat_engine::process_message;
use crate::infrastructure::chat_storage::{
    apply_auto_cleanup, collect_chat_storage_stats, delete_mirror_scene_dir,
    delete_mirror_tree_for_role,
    export_chat_session as export_session_file,
    export_role_chats as export_role_file,
    highlight_snippet,
    resolve_export_max_messages, resolve_max_messages_per_session, role_mirror_tree_bytes,
    save_role_chat_storage_config, AutoCleanupConfig, AutoCleanupResult, ChatExportResponse,
    ChatSearchResult, DeleteChatsResult, ImportChatBucket, ImportChatBucketsResult,
    RoleStorageStat, SessionMeta, StoredMessage,
};
use crate::models::dto::{SendMessageRequest, SendMessageResponse};
use crate::models::RolePackChatStorageConfig;
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
    let max = resolve_max_for_session(&state, &session_id).await;
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

/// Export one session as Markdown or JSON (`format`: `markdown` | `json`).
///
/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
#[tauri::command]
pub async fn export_chat_session(
    session_id: String,
    format: String,
    state: State<'_, AppState>,
) -> Result<ChatExportResponse, crate::api::error::CommandError> {
    let session_id = session_id.trim().to_string();
    let max = resolve_max_for_session(&state, &session_id).await;
    let role_name = match state.db_manager.get_chat_session(&session_id).await {
        Ok(Some(s)) => state
            .load_role_cached_async(&s.role_id)
            .await
            .ok()
            .map(|r| r.name.clone()),
        _ => None,
    };
    export_session_file(
        state.db_manager.as_ref(),
        state.directory_plugins.app_data_dir(),
        &session_id,
        &format,
        max,
        role_name.as_deref(),
    )
    .await
    .map_err(Into::into)
}

/// Export all sessions for a role (`format`: `markdown` | `json`; JSON is ZIP base64).
///
/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
#[tauri::command]
pub async fn export_role_chats(
    role_id: String,
    format: String,
    state: State<'_, AppState>,
) -> Result<ChatExportResponse, crate::api::error::CommandError> {
    let rid = role_id.trim();
    let max = match state.load_role_cached_async(rid).await {
        Ok(role) => resolve_export_max_messages(
            role.pack_chat_storage_config.max_messages_per_session,
        ),
        Err(_) => resolve_export_max_messages(None),
    };
    let role_name = state
        .load_role_cached_async(rid)
        .await
        .ok()
        .map(|r| r.name.clone());
    export_role_file(
        state.db_manager.as_ref(),
        state.directory_plugins.app_data_dir(),
        rid,
        &format,
        max,
        role_name.as_deref(),
    )
    .await
    .map_err(Into::into)
}

/// Search stored chat messages (SQLite LIKE; not memory tables).
///
/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
#[tauri::command]
pub async fn search_chat_messages(
    query: String,
    role_id: Option<String>,
    limit: Option<u32>,
    offset: Option<u32>,
    state: State<'_, AppState>,
) -> Result<Vec<ChatSearchResult>, crate::api::error::CommandError> {
    let cap = limit.unwrap_or(100).min(100);
    let rows = state
        .db_manager
        .search_chat_messages(
            query.trim(),
            role_id.as_deref().map(str::trim),
            cap,
            offset.unwrap_or(0),
        )
        .await?;
    Ok(rows
        .into_iter()
        .map(|r| ChatSearchResult {
            session_id: r.session_id.clone(),
            role_id: r.role_id.clone(),
            scene_id: r.scene_id.clone(),
            highlight_snippet: highlight_snippet(&r.content, query.trim(), 40),
            message: StoredMessage {
                id: r.id,
                session_id: r.session_id,
                turn_index: r.turn_index,
                sender: r.sender,
                content: r.content,
                metadata: r.metadata,
                created_at: r.created_at,
            },
        })
        .collect())
}

/// Delete one chat message and refresh mirror for its session.
///
/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
#[tauri::command]
pub async fn delete_chat_message(
    message_id: String,
    state: State<'_, AppState>,
) -> Result<(), crate::api::error::CommandError> {
    let mid = message_id.trim();
    let session_id = state
        .db_manager
        .delete_chat_message(mid)
        .await?
        .ok_or_else(|| {
            crate::error::AppError::InvalidParameter(format!("message not found: {mid}"))
        })?;
    let max = resolve_max_for_session(&state, &session_id).await;
    let _ = state
        .conversation_store
        .rebuild_mirror(&session_id, max)
        .await?;
    Ok(())
}

/// Edit a user chat message and refresh mirror.
///
/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
#[tauri::command]
pub async fn edit_chat_message(
    message_id: String,
    new_content: String,
    state: State<'_, AppState>,
) -> Result<(), crate::api::error::CommandError> {
    let mid = message_id.trim();
    let session_id = state
        .db_manager
        .edit_chat_message(mid, new_content.trim())
        .await?
        .ok_or_else(|| {
            crate::error::AppError::InvalidParameter(format!("message not found: {mid}"))
        })?;
    let max = resolve_max_for_session(&state, &session_id).await;
    let _ = state
        .conversation_store
        .rebuild_mirror(&session_id, max)
        .await?;
    Ok(())
}

/// Read `config.json` → `chat_storage` for a role.
///
/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
#[tauri::command]
pub async fn get_role_chat_storage_config(
    role_id: String,
    state: State<'_, AppState>,
) -> Result<RolePackChatStorageConfig, crate::api::error::CommandError> {
    let role = state.load_role_cached_async(role_id.trim()).await?;
    Ok(role.pack_chat_storage_config.clone())
}

/// Persist `config.json` → `chat_storage` and invalidate role cache.
///
/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
#[tauri::command]
pub async fn save_role_chat_storage_config_cmd(
    role_id: String,
    config: RolePackChatStorageConfig,
    state: State<'_, AppState>,
) -> Result<(), crate::api::error::CommandError> {
    let rid = role_id.trim();
    save_role_chat_storage_config(state.storage.roles_dir(), rid, &config)?;
    state.invalidate_role_cache(rid);
    Ok(())
}

/// Run auto-cleanup immediately for one role (manual trigger / after settings save).
///
/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
#[tauri::command]
pub async fn run_chat_auto_cleanup(
    role_id: String,
    state: State<'_, AppState>,
) -> Result<AutoCleanupResult, crate::api::error::CommandError> {
    let role = state.load_role_cached_async(role_id.trim()).await?;
    let cfg = AutoCleanupConfig::from_role_config(&role.pack_chat_storage_config);
    apply_auto_cleanup(
        state.db_manager.as_ref(),
        state.directory_plugins.app_data_dir(),
        role.id.as_str(),
        &cfg,
    )
    .await
    .map_err(Into::into)
}

async fn resolve_max_for_session(state: &AppState, session_id: &str) -> i64 {
    match state.db_manager.get_chat_session(session_id).await {
        Ok(Some(session)) => match state.load_role_cached_async(&session.role_id).await {
            Ok(role) => resolve_max_messages_per_session(
                role.pack_chat_storage_config.max_messages_per_session,
            ),
            Err(_) => resolve_max_messages_per_session(None),
        },
        _ => resolve_max_messages_per_session(None),
    }
}
