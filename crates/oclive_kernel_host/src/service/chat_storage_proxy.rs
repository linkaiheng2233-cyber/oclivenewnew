//! Shared chat-storage operations for Tauri IPC, HTTP kernel proxy, and [`ChatBackend`].

use crate::error::{AppError, Result};
use crate::infrastructure::chat_storage::{
    delete_mirror_scene_dir, delete_mirror_tree_for_role, resolve_export_max_messages,
    resolve_max_messages_per_session, resolve_role_chat_storage_root, resolve_storage_root,
    role_mirror_tree_bytes, set_persisted_storage_root,
    spawn_memory_replay, APP_SETTING_CHAT_STORAGE_ROOT, AutoCleanupConfig,
    ChatStorageCapabilities, DeleteChatsResult, ImportChatBucket,
    ReplayTarget,
};
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

async fn role_mirror_root(state: &AppState, role_id: &str) -> PathBuf {
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

/// Kernel HTTP + desktop thin-client chat storage dispatch.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum ChatStorageProxyOp {
    RebuildMirror {
        session_id: String,
    },
    ImportBuckets {
        buckets: Vec<ImportChatBucket>,
    },
    StorageStats,
    SearchMessages {
        query: String,
        role_id: Option<String>,
        limit: u32,
        offset: u32,
    },
    DeleteMessage {
        message_id: String,
    },
    EditMessage {
        message_id: String,
        new_content: String,
    },
    ExportSession {
        session_id: String,
        format: String,
    },
    ExportRole {
        role_id: String,
        format: String,
    },
    AutoCleanup {
        role_id: String,
    },
    ReplayMemory {
        source: String,
        target: ReplayTarget,
    },
    ReplayProgress {
        task_id: String,
    },
    Capabilities,
    DeleteRoleChats {
        role_id: String,
    },
    DeleteSceneChats {
        role_id: String,
        scene_id: String,
    },
    GetStorageRoot,
    SetStorageRoot {
        path: String,
        migrate: Option<bool>,
    },
}

/// Execute a chat storage operation on the authoritative [`AppState`] (kernel or full desktop).
pub async fn execute_chat_storage_proxy(
    state: &AppState,
    op: ChatStorageProxyOp,
) -> Result<serde_json::Value> {
    match op {
        ChatStorageProxyOp::RebuildMirror { session_id } => {
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
            let path = state
                .conversation_store
                .rebuild_mirror(&session_id, max)
                .await?;
            Ok(serde_json::json!({ "path": path }))
        }
        ChatStorageProxyOp::ImportBuckets { buckets } => {
            let res = state.conversation_store.import_chat_buckets(buckets).await?;
            Ok(serde_json::to_value(res).map_err(|e| {
                AppError::InvalidParameter(format!("chat storage proxy encode: {e}"))
            })?)
        }
        ChatStorageProxyOp::StorageStats => {
            let stats = state.conversation_store.get_storage_stats().await?;
            Ok(serde_json::to_value(stats).map_err(|e| AppError::InvalidParameter(format!("chat storage proxy encode: {e}")))?)
        }
        ChatStorageProxyOp::SearchMessages {
            query,
            role_id,
            limit,
            offset,
        } => {
            let cap = limit.min(100);
            let hits = state
                .conversation_store
                .search_messages(
                    query.trim(),
                    role_id.as_deref().map(str::trim),
                    cap,
                    offset,
                )
                .await?;
            Ok(serde_json::to_value(hits).map_err(|e| AppError::InvalidParameter(format!("chat storage proxy encode: {e}")))?)
        }
        ChatStorageProxyOp::DeleteMessage { message_id } => {
            state
                .conversation_store
                .delete_message(message_id.trim())
                .await?;
            Ok(serde_json::json!({ "ok": true }))
        }
        ChatStorageProxyOp::EditMessage {
            message_id,
            new_content,
        } => {
            state
                .conversation_store
                .edit_message(message_id.trim(), new_content.trim())
                .await?;
            Ok(serde_json::json!({ "ok": true }))
        }
        ChatStorageProxyOp::ExportSession {
            session_id,
            format,
        } => {
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
            let res = state
                .conversation_store
                .export_session(
                    &session_id,
                    format.trim(),
                    max,
                    role_name.as_deref(),
                )
                .await?;
            Ok(serde_json::to_value(res).map_err(|e| {
                AppError::InvalidParameter(format!("chat storage proxy encode: {e}"))
            })?)
        }
        ChatStorageProxyOp::ExportRole { role_id, format } => {
            let rid = role_id.trim();
            let (max, role_name) = match state.load_role_cached_async(rid).await {
                Ok(role) => (
                    resolve_export_max_messages(role.pack_chat_storage_config.max_messages_per_session),
                    Some(role.name.clone()),
                ),
                Err(_) => (resolve_export_max_messages(None), None),
            };
            let res = state
                .conversation_store
                .export_role(rid, format.trim(), max, role_name.as_deref())
                .await?;
            Ok(serde_json::to_value(res).map_err(|e| {
                AppError::InvalidParameter(format!("chat storage proxy encode: {e}"))
            })?)
        }
        ChatStorageProxyOp::AutoCleanup { role_id } => {
            let role = state.load_role_cached_async(role_id.trim()).await?;
            let cfg = AutoCleanupConfig::from_role_config(&role.pack_chat_storage_config);
            let res = state
                .conversation_store
                .apply_auto_cleanup(role.id.as_str(), &cfg)
                .await?;
            Ok(serde_json::to_value(res).map_err(|e| {
                AppError::InvalidParameter(format!("chat storage proxy encode: {e}"))
            })?)
        }
        ChatStorageProxyOp::ReplayMemory { source, target } => {
            let task_id = spawn_memory_replay(
                state.db_manager.clone(),
                state.conversation_store.clone(),
                source,
                target,
                state.replay_tasks.clone(),
            );
            Ok(serde_json::json!({ "task_id": task_id }))
        }
        ChatStorageProxyOp::ReplayProgress { task_id } => {
            let progress = state
                .replay_tasks
                .get(task_id.trim())
                .ok_or_else(|| {
                    AppError::InvalidParameter(format!(
                        "replay task not found: {}",
                        task_id.trim()
                    ))
                })?;
            Ok(serde_json::to_value(progress).map_err(|e| AppError::InvalidParameter(format!("chat storage proxy encode: {e}")))?)
        }
        ChatStorageProxyOp::Capabilities => {
            let backend_kind = state.conversation_store.backend_kind().to_string();
            let mirror_enabled = backend_kind == "hybrid";
            let caps = ChatStorageCapabilities {
                backend_kind,
                mirror_enabled,
                default_max_messages_per_session: crate::infrastructure::chat_storage::DEFAULT_MAX_MESSAGES
                    as u32,
                supports_search: state.conversation_store.supports_search(),
                supports_replay: state.conversation_store.supports_replay(),
                supports_cleanup: state.conversation_store.supports_cleanup(),
            };
            Ok(serde_json::to_value(caps).map_err(|e| AppError::InvalidParameter(format!("chat storage proxy encode: {e}")))?)
        }
        ChatStorageProxyOp::DeleteRoleChats { role_id } => {
            let rid = role_id.trim();
            let mirror_root = role_mirror_root(state, rid).await;
            let bytes = role_mirror_tree_bytes(&mirror_root, rid).await?;
            let sessions_deleted = state
                .db_manager
                .count_chat_sessions_for_manifest_role(rid)
                .await?;
            state.db_manager.delete_chat_data_for_manifest_role(rid).await?;
            delete_mirror_tree_for_role(&mirror_root, rid).await?;
            let res = DeleteChatsResult {
                sessions_deleted,
                bytes_freed: bytes,
            };
            Ok(serde_json::to_value(res).map_err(|e| {
                AppError::InvalidParameter(format!("chat storage proxy encode: {e}"))
            })?)
        }
        ChatStorageProxyOp::DeleteSceneChats { role_id, scene_id } => {
            let rid = role_id.trim();
            let sid = scene_id.trim();
            let mirror_root = role_mirror_root(state, rid).await;
            let bytes = delete_mirror_scene_dir(&mirror_root, rid, sid).await?;
            let sessions_deleted = state.db_manager.delete_chat_data_for_role_scene(rid, sid).await?;
            let res = DeleteChatsResult {
                sessions_deleted,
                bytes_freed: bytes,
            };
            Ok(serde_json::to_value(res).map_err(|e| {
                AppError::InvalidParameter(format!("chat storage proxy encode: {e}"))
            })?)
        }
        ChatStorageProxyOp::GetStorageRoot => {
            let app_data = state.directory_plugins.app_data_dir();
            let root = resolve_storage_root(app_data).to_string_lossy().into_owned();
            Ok(serde_json::json!({ "path": root }))
        }
        ChatStorageProxyOp::SetStorageRoot { path, migrate } => {
            let app_data = state.directory_plugins.app_data_dir();
            let trimmed = path.trim();
            if trimmed.is_empty() {
                state
                    .db_manager
                    .upsert_app_setting(APP_SETTING_CHAT_STORAGE_ROOT, "")
                    .await?;
                set_persisted_storage_root(None);
                let root = resolve_storage_root(app_data).to_string_lossy().into_owned();
                return Ok(serde_json::json!({ "path": root }));
            }
            let new_root = PathBuf::from(trimmed);
            let old_root = resolve_storage_root(app_data);
            if migrate.unwrap_or(true) && old_root != new_root {
                crate::infrastructure::chat_storage::migrate_mirror_tree(&old_root, &new_root)
                    .await?;
            }
            state
                .db_manager
                .upsert_app_setting(APP_SETTING_CHAT_STORAGE_ROOT, trimmed)
                .await?;
            set_persisted_storage_root(Some(new_root.clone()));
            Ok(serde_json::json!({ "path": new_root.to_string_lossy() }))
        }
    }
}
