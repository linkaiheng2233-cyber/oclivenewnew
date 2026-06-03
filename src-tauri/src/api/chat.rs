use crate::domain::chat_engine::process_message;
use crate::infrastructure::chat_storage::{
    delete_mirror_scene_dir, delete_mirror_tree_for_role, migrate_mirror_tree,
    resolve_export_max_messages, resolve_max_messages_per_session, resolve_role_chat_storage_root,
    resolve_storage_root, role_mirror_tree_bytes, save_role_chat_storage_config,
    set_persisted_storage_root, spawn_memory_replay, APP_SETTING_CHAT_STORAGE_ROOT,
    AutoCleanupConfig, AutoCleanupResult, ChatExportResponse, ChatSearchResult,
    ChatStorageCapabilities, DeleteChatsResult, ImportChatBucket, ImportChatBucketsResult,
    ReplayProgress, ReplayTarget, RoleStorageStat, SessionMeta, StoredMessage,
};
use crate::models::dto::{SendMessageRequest, SendMessageResponse};
use crate::models::RolePackChatStorageConfig;
use crate::kernel_attach::{role_dir_for_id, KernelHttpClient};
use crate::kernel_lifecycle::SharedKernelConnection;
use crate::state::SharedAppState;
use std::path::PathBuf;
use tauri::{Manager, State};

async fn role_mirror_root(state: &crate::state::AppState, role_id: &str) -> PathBuf {
    let location = match state.load_role_cached_async(role_id).await {
        Ok(role) => role.pack_chat_storage_config.location.clone(),
        Err(_) => "global".to_string(),
    };
    resolve_role_chat_storage_root(
        state.directory_plugins.app_data_dir(),
        state.storage.roles_dir(),
        role_id,
        Some(&location),
    )
}

/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
#[tauri::command]
pub async fn send_message(
    req: SendMessageRequest,
    app: tauri::AppHandle,
    state: State<'_, SharedAppState>,
) -> Result<SendMessageResponse, crate::api::error::CommandError> {
    if let Some(conn) = app.try_state::<SharedKernelConnection>() {
        let role_path = role_dir_for_id(state.as_ref(), &req.role_id);
        match KernelHttpClient::send_message_via_http(&conn, &role_path, &req).await {
            Ok(res) => return Ok(res),
            Err(crate::error::AppError::RoleRuntimeNotReady) => {
                KernelHttpClient::load_role_via_http(&conn, req.role_id.trim()).await?;
                return KernelHttpClient::send_message_via_http(&conn, &role_path, &req)
                    .await
                    .map_err(Into::into);
            }
            Err(e) => return Err(e.into()),
        }
    }
    process_message(state.as_ref(), &req).await.map_err(Into::into)
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
    app: tauri::AppHandle,
    state: State<'_, SharedAppState>,
) -> Result<Vec<SessionMeta>, crate::api::error::CommandError> {
    if let Some(conn) = app.try_state::<SharedKernelConnection>() {
        return KernelHttpClient::list_chat_sessions_via_http(
            &conn,
            role_id.trim(),
            scene_id.trim(),
            limit.unwrap_or(50),
            offset.unwrap_or(0),
        )
        .await
        .map_err(Into::into);
    }
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
    app: tauri::AppHandle,
    state: State<'_, SharedAppState>,
) -> Result<Vec<StoredMessage>, crate::api::error::CommandError> {
    if let Some(conn) = app.try_state::<SharedKernelConnection>() {
        return KernelHttpClient::fetch_chat_messages_via_http(
            &conn,
            session_id.trim(),
            limit.unwrap_or(500),
            offset.unwrap_or(0),
        )
        .await
        .map_err(Into::into);
    }
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
    state: State<'_, SharedAppState>,
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
    state: State<'_, SharedAppState>,
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
    state: State<'_, SharedAppState>,
) -> Result<Vec<RoleStorageStat>, crate::api::error::CommandError> {
    state
        .conversation_store
        .get_storage_stats()
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
    state: State<'_, SharedAppState>,
) -> Result<DeleteChatsResult, crate::api::error::CommandError> {
    let rid = role_id.trim();
    let mirror_root = role_mirror_root(state.inner(), rid).await;
    let bytes = role_mirror_tree_bytes(&mirror_root, rid).await?;
    let sessions_deleted = state
        .db_manager
        .count_chat_sessions_for_manifest_role(rid)
        .await?;
    state.db_manager.delete_chat_data_for_manifest_role(rid).await?;
    delete_mirror_tree_for_role(&mirror_root, rid).await?;
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
    state: State<'_, SharedAppState>,
) -> Result<DeleteChatsResult, crate::api::error::CommandError> {
    let rid = role_id.trim();
    let sid = scene_id.trim();
    let mirror_root = role_mirror_root(state.inner(), rid).await;
    let bytes = delete_mirror_scene_dir(&mirror_root, rid, sid).await?;
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
    state: State<'_, SharedAppState>,
) -> Result<ChatExportResponse, crate::api::error::CommandError> {
    let session_id = session_id.trim().to_string();
    let (max, role_name) = match state.db_manager.get_chat_session(&session_id).await {
        Ok(Some(session)) => {
            let role_result = state.load_role_cached_async(&session.role_id).await;
            let max = match &role_result {
                Ok(role) => resolve_max_messages_per_session(
                    role.pack_chat_storage_config.max_messages_per_session,
                ),
                Err(_) => resolve_max_messages_per_session(None),
            };
            let role_name = role_result.ok().map(|r| r.name.clone());
            (max, role_name)
        }
        _ => (resolve_max_messages_per_session(None), None),
    };
    state
        .conversation_store
        .export_session(
            &session_id,
            &format,
            max,
            role_name.as_deref(),
        )
        .await
        .map_err(Into::into)
}

/// Export all sessions for a role (`format`: `markdown` | `json`; JSON is one combined document).
///
/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
#[tauri::command]
pub async fn export_role_chats(
    role_id: String,
    format: String,
    state: State<'_, SharedAppState>,
) -> Result<ChatExportResponse, crate::api::error::CommandError> {
    let rid = role_id.trim();
    let (max, role_name) = match state.load_role_cached_async(rid).await {
        Ok(role) => (
            resolve_export_max_messages(role.pack_chat_storage_config.max_messages_per_session),
            Some(role.name.clone()),
        ),
        Err(_) => (resolve_export_max_messages(None), None),
    };
    state
        .conversation_store
        .export_role(rid, &format, max, role_name.as_deref())
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
    state: State<'_, SharedAppState>,
) -> Result<Vec<ChatSearchResult>, crate::api::error::CommandError> {
    let cap = limit.unwrap_or(100).min(100);
    state
        .conversation_store
        .search_messages(
            query.trim(),
            role_id.as_deref().map(str::trim),
            cap,
            offset.unwrap_or(0),
        )
        .await
        .map_err(Into::into)
}

/// Delete one chat message and refresh mirror for its session.
///
/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
#[tauri::command]
pub async fn delete_chat_message(
    message_id: String,
    state: State<'_, SharedAppState>,
) -> Result<(), crate::api::error::CommandError> {
    state
        .conversation_store
        .delete_message(message_id.trim())
        .await
        .map_err(Into::into)
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
    state: State<'_, SharedAppState>,
) -> Result<(), crate::api::error::CommandError> {
    state
        .conversation_store
        .edit_message(message_id.trim(), new_content.trim())
        .await
        .map_err(Into::into)
}

/// Read `config.json` → `chat_storage` for a role.
///
/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
#[tauri::command]
pub async fn get_role_chat_storage_config(
    role_id: String,
    state: State<'_, SharedAppState>,
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
    state: State<'_, SharedAppState>,
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
    state: State<'_, SharedAppState>,
) -> Result<AutoCleanupResult, crate::api::error::CommandError> {
    let role = state.load_role_cached_async(role_id.trim()).await?;
    let cfg = AutoCleanupConfig::from_role_config(&role.pack_chat_storage_config);
    state
        .conversation_store
        .apply_auto_cleanup(role.id.as_str(), &cfg)
        .await
        .map_err(Into::into)
}

/// Re-extract AI memories from stored chat history (merge, idempotent; runs in background).
///
/// # Errors
///
/// Returns [`Err`] when the task cannot be started.
#[tauri::command]
pub async fn replay_memory_extraction(
    source: String,
    target: ReplayTarget,
    state: State<'_, SharedAppState>,
) -> Result<String, crate::api::error::CommandError> {
    let task_id = spawn_memory_replay(
        state.db_manager.clone(),
        state.conversation_store.clone(),
        source,
        target,
        state.replay_tasks.clone(),
    );
    Ok(task_id)
}

/// Poll progress for a memory replay task started by [`replay_memory_extraction`].
///
/// # Errors
///
/// Returns [`Err`] when the task id is unknown.
#[tauri::command]
pub async fn get_replay_progress(
    task_id: String,
    state: State<'_, SharedAppState>,
) -> Result<ReplayProgress, crate::api::error::CommandError> {
    state
        .replay_tasks
        .get(task_id.trim())
        .ok_or_else(|| {
            crate::error::AppError::InvalidParameter(format!(
                "replay task not found: {}",
                task_id.trim()
            ))
        })
        .map_err(Into::into)
}

/// Query chat storage backend capabilities (search, replay, cleanup).
///
/// # Errors
///
/// Never fails for known backends; reserved for future dynamic backends.
#[tauri::command]
pub async fn get_chat_storage_capabilities(
    state: State<'_, SharedAppState>,
) -> Result<ChatStorageCapabilities, crate::api::error::CommandError> {
    Ok(ChatStorageCapabilities {
        backend_kind: state.conversation_store.backend_kind().to_string(),
        supports_search: state.conversation_store.supports_search().await,
        supports_replay: state.conversation_store.supports_replay().await,
        supports_cleanup: state.conversation_store.supports_cleanup().await,
    })
}

/// Effective chat JSON mirror root (env > app setting > default).
///
/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
#[tauri::command]
pub async fn get_chat_storage_root(
    state: State<'_, SharedAppState>,
) -> Result<String, crate::api::error::CommandError> {
    let app_data = state.directory_plugins.app_data_dir();
    Ok(resolve_storage_root(app_data).to_string_lossy().into_owned())
}

/// Persist custom chat mirror root; optionally copy existing mirror tree.
///
/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
#[tauri::command]
pub async fn set_chat_storage_root(
    path: String,
    migrate: Option<bool>,
    state: State<'_, SharedAppState>,
) -> Result<String, crate::api::error::CommandError> {
    let app_data = state.directory_plugins.app_data_dir();
    let trimmed = path.trim();
    if trimmed.is_empty() {
        state
            .db_manager
            .upsert_app_setting(APP_SETTING_CHAT_STORAGE_ROOT, "")
            .await?;
        set_persisted_storage_root(None);
        return Ok(resolve_storage_root(app_data).to_string_lossy().into_owned());
    }
    let new_root = PathBuf::from(trimmed);
    let old_root = resolve_storage_root(app_data);
    if migrate.unwrap_or(true) && old_root != new_root {
        migrate_mirror_tree(&old_root, &new_root).await?;
    }
    state
        .db_manager
        .upsert_app_setting(APP_SETTING_CHAT_STORAGE_ROOT, trimmed)
        .await?;
    set_persisted_storage_root(Some(new_root.clone()));
    Ok(new_root.to_string_lossy().into_owned())
}
