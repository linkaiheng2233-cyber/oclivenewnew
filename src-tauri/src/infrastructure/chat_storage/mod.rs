//! Chat history: pluggable backends (hybrid / file / sqlite).

mod backends;
mod config;
mod cleanup;
mod db;
mod export;
mod factory;
mod manager;
mod mirror;
mod replay;
mod role_config;
mod scheduler;
mod shared;
mod stats;
mod store_trait;
#[cfg(test)]
mod store_trait_tests;
mod types;

pub use cleanup::{apply_auto_cleanup, apply_auto_cleanup_sqlite, AutoCleanupConfig};
pub use config::{
    migrate_mirror_tree, resolve_max_messages_per_session, resolve_session_dir,
    resolve_storage_root, set_persisted_storage_root, APP_SETTING_CHAT_STORAGE_ROOT,
    DEFAULT_MAX_MESSAGES, ENV_CHAT_STORAGE_ROOT, MAX_MESSAGES_PER_SESSION,
};
pub use export::{export_chat_session, export_role_chats, resolve_export_max_messages};
pub use factory::{build_conversation_store, resolve_backend_kind, ENV_CHAT_STORAGE_BACKEND};
pub use manager::HybridConversationStore;
pub use store_trait::ConversationStore;
pub use mirror::delete_mirror_tree_for_role;
pub use replay::{spawn_memory_replay, ReplayTaskRegistry};
pub use role_config::save_role_chat_storage_config;
pub use scheduler::spawn_auto_cleanup_scheduler;
pub use stats::{
    collect_chat_storage_stats, collect_chat_storage_stats_from_db, delete_mirror_scene_dir,
    role_mirror_tree_bytes,
};
pub use types::{
    AppendTurnResult, AutoCleanupResult, ChatExportResponse, ChatSearchResult,
    ChatStorageCapabilities, DeleteChatsResult, ImportChatBucket, ImportChatBucketsResult,
    ReplayProgress, ReplayResult, ReplayTarget, RoleStorageStat, SceneStorageStat, SessionMeta,
    StoredMessage, TurnPersistInput,
};
pub use db::{highlight_snippet, ChatSearchRow};

pub use crate::models::role_pack_config::ChatStorageBackendKind;
