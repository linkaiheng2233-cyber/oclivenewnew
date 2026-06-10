//! Construct [`ConversationStore`] from backend kind.

use super::backends::HybridConversationStore;
use super::replay::ReplayTaskRegistry;
use super::store_trait::ConversationStore;
use crate::models::role_pack_config::ChatStorageBackendKind;
use crate::models::RolePackChatStorageConfig;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub const ENV_CHAT_STORAGE_BACKEND: &str = "OCLIVE_CHAT_STORAGE_BACKEND";

/// Resolve legacy backend kind: env [`ENV_CHAT_STORAGE_BACKEND`] (`hybrid`|`file`|`sqlite`) >
/// role `config.json` → `chat_storage.backend` > default [`ChatStorageBackendKind::Hybrid`].
///
/// **Semantics (phase 3):** runtime always builds [`HybridConversationStore`] (SQLite authoritative).
/// `file` / `sqlite` / `hybrid` only influence the JSON **mirror** flag via [`resolve_mirror_enabled`];
/// prefer explicit `chat_storage.mirror` in role `config.json` when possible.
#[must_use]
pub fn pick_chat_storage_backend_kind(
    config: Option<&RolePackChatStorageConfig>,
) -> ChatStorageBackendKind {
    if let Ok(raw) = std::env::var(ENV_CHAT_STORAGE_BACKEND) {
        let t = raw.trim().to_ascii_lowercase();
        return match t.as_str() {
            "file" => ChatStorageBackendKind::File,
            "sqlite" => ChatStorageBackendKind::Sqlite,
            _ => ChatStorageBackendKind::Hybrid,
        };
    }
    config.and_then(|c| c.backend).unwrap_or_default()
}

/// Whether JSON mirror files are written. Explicit `chat_storage.mirror` wins; else legacy `backend`.
#[must_use]
pub fn resolve_mirror_enabled(
    config: &RolePackChatStorageConfig,
    kind: ChatStorageBackendKind,
) -> bool {
    if let Some(mirror) = config.mirror {
        return mirror;
    }
    match kind {
        ChatStorageBackendKind::Hybrid | ChatStorageBackendKind::File => {
            if kind == ChatStorageBackendKind::File {
                tracing::warn!(
                    target: "oclive_chat_storage",
                    "chat_storage.backend=file is deprecated; using hybrid store with mirror:on"
                );
            }
            true
        }
        ChatStorageBackendKind::Sqlite => false,
    }
}

/// Build conversation store (always [`HybridConversationStore`] with mirror flag).
#[must_use]
pub fn build_conversation_store(
    kind: ChatStorageBackendKind,
    db: Arc<crate::infrastructure::db::DbManager>,
    app_data_dir: PathBuf,
    roles_dir: PathBuf,
    replay_tasks: Arc<ReplayTaskRegistry>,
    role_config: &RolePackChatStorageConfig,
    role_pack_dir: Option<&Path>,
) -> Arc<dyn ConversationStore> {
    let mirror_enabled = resolve_mirror_enabled(role_config, kind);
    let _ = role_pack_dir;
    Arc::new(HybridConversationStore::new(
        db,
        app_data_dir,
        roles_dir,
        replay_tasks,
        mirror_enabled,
    ))
}
