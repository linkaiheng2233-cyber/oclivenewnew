use crate::api::chat_backend::ChatBackend;
use crate::kernel_attach::role_dir_for_id;
use oclive_kernel_host::infrastructure::chat_storage::{
    AutoCleanupResult, ChatExportResponse, ChatSearchResult, ChatStorageCapabilities,
    DeleteChatsResult, ImportChatBucket, ImportChatBucketsResult, ReplayProgress, ReplayTarget,
    SessionMeta, StoredMessage,
};
use oclive_kernel_host::service::ChatStorageProxyOp;
use oclive_kernel_host::state::SharedAppState;
use oclive_kernel_types::models::dto::{SendMessageRequest, SendMessageResponse};
use oclive_kernel_types::models::RolePackChatStorageConfig;
use serde::de::DeserializeOwned;
use tauri::{AppHandle, State};

async fn storage_proxy_json<T: DeserializeOwned>(
    app: &AppHandle,
    state: &SharedAppState,
    op: ChatStorageProxyOp,
) -> Result<T, crate::api::error::CommandError> {
    let v = ChatBackend::from_app(app, state.clone())
        .storage_proxy(op)
        .await
        .map_err(crate::api::error::CommandError::from)?;
    serde_json::from_value(v).map_err(|e| {
        crate::error::AppError::InvalidParameter(format!("chat storage proxy decode: {e}")).into()
    })
}

async fn storage_proxy_ok(
    app: &AppHandle,
    state: &SharedAppState,
    op: ChatStorageProxyOp,
) -> Result<(), crate::api::error::CommandError> {
    let _: serde_json::Value = storage_proxy_json(app, state, op).await?;
    Ok(())
}

/// Runs one chat turn via [`ChatBackend`] (in-process or attach HTTP); rejects empty `user_message`.
#[tauri::command]
pub async fn send_message(
    req: SendMessageRequest,
    app: AppHandle,
    state: State<'_, SharedAppState>,
) -> Result<SendMessageResponse, crate::api::error::CommandError> {
    let user_message = req.user_message.trim().to_string();
    if user_message.is_empty() {
        return Err(crate::error::AppError::EmptyMessage.into());
    }
    let mut req = req;
    req.user_message = user_message;
    let role_path = role_dir_for_id(state.as_ref(), &req.role_id);
    ChatBackend::from_app(&app, state.inner().clone())
        .send_message(&role_path, &req)
        .await
        .map_err(Into::into)
}

/// Returns absolute on-disk path for a role pack (for kernel HTTP `role_path`).
#[tauri::command]
pub fn get_role_pack_path(
    role_id: String,
    state: State<'_, SharedAppState>,
) -> Result<String, crate::api::error::CommandError> {
    Ok(role_dir_for_id(state.as_ref(), role_id.trim())
        .to_string_lossy()
        .into_owned())
}

/// Lists chat sessions for a role/scene through the active storage backend.
#[tauri::command]
pub async fn list_chat_sessions(
    role_id: String,
    scene_id: String,
    limit: Option<u32>,
    offset: Option<u32>,
    app: AppHandle,
    state: State<'_, SharedAppState>,
) -> Result<Vec<SessionMeta>, crate::api::error::CommandError> {
    ChatBackend::from_app(&app, state.inner().clone())
        .list_chat_sessions(
            role_id.trim(),
            scene_id.trim(),
            limit.unwrap_or(50),
            offset.unwrap_or(0),
        )
        .await
        .map_err(Into::into)
}

/// Loads paginated messages for a session id.
#[tauri::command]
pub async fn fetch_chat_messages(
    session_id: String,
    limit: Option<u32>,
    offset: Option<u32>,
    app: AppHandle,
    state: State<'_, SharedAppState>,
) -> Result<Vec<StoredMessage>, crate::api::error::CommandError> {
    ChatBackend::from_app(&app, state.inner().clone())
        .fetch_chat_messages(session_id.trim(), limit.unwrap_or(500), offset.unwrap_or(0))
        .await
        .map_err(Into::into)
}

/// Rebuilds the JSON mirror for a session via storage proxy (attach-safe).
#[tauri::command]
pub async fn rebuild_chat_mirror(
    session_id: String,
    app: AppHandle,
    state: State<'_, SharedAppState>,
) -> Result<String, crate::api::error::CommandError> {
    #[derive(serde::Deserialize)]
    struct Out {
        path: String,
    }
    let out: Out = storage_proxy_json(
        &app,
        state.inner(),
        ChatStorageProxyOp::RebuildMirror {
            session_id: session_id.trim().to_string(),
        },
    )
    .await?;
    Ok(out.path)
}

/// Imports legacy IndexedDB chat buckets into the active backend via storage proxy.
#[tauri::command]
pub async fn migrate_indexeddb_to_backend(
    buckets: Vec<ImportChatBucket>,
    app: AppHandle,
    state: State<'_, SharedAppState>,
) -> Result<ImportChatBucketsResult, crate::api::error::CommandError> {
    storage_proxy_json(
        &app,
        state.inner(),
        ChatStorageProxyOp::ImportBuckets { buckets },
    )
    .await
}

/// Returns per-role chat storage usage stats from the storage proxy.
#[tauri::command]
pub async fn get_chat_storage_stats(
    app: AppHandle,
    state: State<'_, SharedAppState>,
) -> Result<
    Vec<oclive_kernel_host::infrastructure::chat_storage::RoleStorageStat>,
    crate::api::error::CommandError,
> {
    storage_proxy_json(&app, state.inner(), ChatStorageProxyOp::StorageStats).await
}

/// Deletes all chat sessions for a role (SQLite + optional mirror).
#[tauri::command]
pub async fn delete_role_chats(
    role_id: String,
    app: AppHandle,
    state: State<'_, SharedAppState>,
) -> Result<DeleteChatsResult, crate::api::error::CommandError> {
    storage_proxy_json(
        &app,
        state.inner(),
        ChatStorageProxyOp::DeleteRoleChats {
            role_id: role_id.trim().to_string(),
        },
    )
    .await
}

/// Deletes chat sessions for one role/scene pair.
#[tauri::command]
pub async fn delete_scene_chats(
    role_id: String,
    scene_id: String,
    app: AppHandle,
    state: State<'_, SharedAppState>,
) -> Result<DeleteChatsResult, crate::api::error::CommandError> {
    storage_proxy_json(
        &app,
        state.inner(),
        ChatStorageProxyOp::DeleteSceneChats {
            role_id: role_id.trim().to_string(),
            scene_id: scene_id.trim().to_string(),
        },
    )
    .await
}

/// Exports one session to JSON or plain text.
#[tauri::command]
pub async fn export_chat_session(
    session_id: String,
    format: String,
    app: AppHandle,
    state: State<'_, SharedAppState>,
) -> Result<ChatExportResponse, crate::api::error::CommandError> {
    storage_proxy_json(
        &app,
        state.inner(),
        ChatStorageProxyOp::ExportSession {
            session_id: session_id.trim().to_string(),
            format,
        },
    )
    .await
}

/// Exports all sessions for a role.
#[tauri::command]
pub async fn export_role_chats(
    role_id: String,
    format: String,
    app: AppHandle,
    state: State<'_, SharedAppState>,
) -> Result<ChatExportResponse, crate::api::error::CommandError> {
    storage_proxy_json(
        &app,
        state.inner(),
        ChatStorageProxyOp::ExportRole {
            role_id: role_id.trim().to_string(),
            format,
        },
    )
    .await
}

/// Full-text search across stored chat messages.
#[tauri::command]
pub async fn search_chat_messages(
    query: String,
    role_id: Option<String>,
    limit: Option<u32>,
    offset: Option<u32>,
    app: AppHandle,
    state: State<'_, SharedAppState>,
) -> Result<Vec<ChatSearchResult>, crate::api::error::CommandError> {
    storage_proxy_json(
        &app,
        state.inner(),
        ChatStorageProxyOp::SearchMessages {
            query,
            role_id,
            limit: limit.unwrap_or(100),
            offset: offset.unwrap_or(0),
        },
    )
    .await
}

/// Deletes a single chat message by id.
#[tauri::command]
pub async fn delete_chat_message(
    message_id: String,
    app: AppHandle,
    state: State<'_, SharedAppState>,
) -> Result<(), crate::api::error::CommandError> {
    storage_proxy_ok(
        &app,
        state.inner(),
        ChatStorageProxyOp::DeleteMessage {
            message_id: message_id.trim().to_string(),
        },
    )
    .await
}

/// Edits assistant/user message content in place.
#[tauri::command]
pub async fn edit_chat_message(
    message_id: String,
    new_content: String,
    app: AppHandle,
    state: State<'_, SharedAppState>,
) -> Result<(), crate::api::error::CommandError> {
    storage_proxy_ok(
        &app,
        state.inner(),
        ChatStorageProxyOp::EditMessage {
            message_id: message_id.trim().to_string(),
            new_content,
        },
    )
    .await
}

/// Role pack `config.json` → `chat_storage` (local roles dir; shared with kernel).
#[tauri::command]
pub async fn get_role_chat_storage_config(
    role_id: String,
    state: State<'_, SharedAppState>,
) -> Result<RolePackChatStorageConfig, crate::api::error::CommandError> {
    let role = state.load_role_cached_async(role_id.trim()).await?;
    Ok(role.pack_chat_storage_config.clone())
}

/// Persists role pack `chat_storage` config and invalidates the role cache.
#[tauri::command]
pub async fn save_role_chat_storage_config_cmd(
    role_id: String,
    config: RolePackChatStorageConfig,
    state: State<'_, SharedAppState>,
) -> Result<(), crate::api::error::CommandError> {
    let rid = role_id.trim();
    oclive_kernel_host::infrastructure::chat_storage::save_role_chat_storage_config(
        state.storage.roles_dir(),
        rid,
        &config,
    )?;
    state.invalidate_role_cache(rid);
    Ok(())
}

/// Runs auto-cleanup for a role per pack `chat_storage` retention rules.
#[tauri::command]
pub async fn run_chat_auto_cleanup(
    role_id: String,
    app: AppHandle,
    state: State<'_, SharedAppState>,
) -> Result<AutoCleanupResult, crate::api::error::CommandError> {
    storage_proxy_json(
        &app,
        state.inner(),
        ChatStorageProxyOp::AutoCleanup {
            role_id: role_id.trim().to_string(),
        },
    )
    .await
}

/// Starts async memory replay from chat history; returns task id.
#[tauri::command]
pub async fn replay_memory_extraction(
    source: String,
    target: ReplayTarget,
    app: AppHandle,
    state: State<'_, SharedAppState>,
) -> Result<String, crate::api::error::CommandError> {
    #[derive(serde::Deserialize)]
    struct Out {
        task_id: String,
    }
    let out: Out = storage_proxy_json(
        &app,
        state.inner(),
        ChatStorageProxyOp::ReplayMemory { source, target },
    )
    .await?;
    Ok(out.task_id)
}

/// Polls replay task progress by task id.
#[tauri::command]
pub async fn get_replay_progress(
    task_id: String,
    app: AppHandle,
    state: State<'_, SharedAppState>,
) -> Result<ReplayProgress, crate::api::error::CommandError> {
    storage_proxy_json(
        &app,
        state.inner(),
        ChatStorageProxyOp::ReplayProgress {
            task_id: task_id.trim().to_string(),
        },
    )
    .await
}

/// Reports hybrid storage capabilities (search, replay, cleanup flags).
#[tauri::command]
pub async fn get_chat_storage_capabilities(
    app: AppHandle,
    state: State<'_, SharedAppState>,
) -> Result<ChatStorageCapabilities, crate::api::error::CommandError> {
    storage_proxy_json(&app, state.inner(), ChatStorageProxyOp::Capabilities).await
}

/// Returns the configured chat storage root directory path.
#[tauri::command]
pub async fn get_chat_storage_root(
    app: AppHandle,
    state: State<'_, SharedAppState>,
) -> Result<String, crate::api::error::CommandError> {
    #[derive(serde::Deserialize)]
    struct Out {
        path: String,
    }
    let out: Out =
        storage_proxy_json(&app, state.inner(), ChatStorageProxyOp::GetStorageRoot).await?;
    Ok(out.path)
}

/// Sets chat storage root; optional migrate moves existing sessions.
#[tauri::command]
pub async fn set_chat_storage_root(
    path: String,
    migrate: Option<bool>,
    app: AppHandle,
    state: State<'_, SharedAppState>,
) -> Result<String, crate::api::error::CommandError> {
    #[derive(serde::Deserialize)]
    struct Out {
        path: String,
    }
    let out: Out = storage_proxy_json(
        &app,
        state.inner(),
        ChatStorageProxyOp::SetStorageRoot { path, migrate },
    )
    .await?;
    Ok(out.path)
}
