//! Chat history: Hybrid-only backend (SQLite + optional JSON mirror).

mod backends;
mod chat_messages;
mod chat_search;
mod chat_sessions;
mod cleanup;
mod config;
mod db;
mod export;
mod factory;
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

pub use backends::HybridConversationStore;
pub use cleanup::{apply_auto_cleanup, apply_auto_cleanup_sqlite, AutoCleanupConfig};
pub use config::{
    is_path_writable, migrate_mirror_tree, read_role_chat_storage_location,
    load_max_messages_per_session, load_role_chat_storage_root, load_session_dir,
    load_storage_root, load_storage_root_with_role, set_persisted_storage_root,
    APP_SETTING_CHAT_STORAGE_ROOT, DEFAULT_MAX_MESSAGES, ENV_CHAT_STORAGE_ROOT,
};
pub use db::{highlight_snippet, manifest_sess_glob_pattern, ChatSearchRow};
pub use export::{export_chat_session, export_role_chats, resolve_export_max_messages};
pub use factory::{build_conversation_store, pick_chat_storage_backend_kind, ENV_CHAT_STORAGE_BACKEND};
pub use mirror::delete_mirror_tree_for_role;
pub use replay::{spawn_memory_replay, ReplayTaskRegistry};
pub use role_config::save_role_chat_storage_config;
pub use scheduler::run_global_auto_cleanup;
pub use stats::{
    collect_chat_storage_stats, collect_chat_storage_stats_from_db, delete_mirror_scene_dir,
    enumerate_chat_mirror_roots, role_mirror_tree_bytes,
};
pub use store_trait::ConversationStore;
pub use types::{
    AppendTurnResult, AutoCleanupResult, ChatExportResponse, ChatSearchResult,
    ChatStorageCapabilities, DeleteChatsResult, ImportChatBucket, ImportChatBucketsResult,
    ReplayProgress, ReplayResult, ReplayTarget, RoleStorageStat, SceneStorageStat, SessionMeta,
    StoredMessage, TurnPersistInput,
};

pub use crate::models::role_pack_config::ChatStorageBackendKind;
