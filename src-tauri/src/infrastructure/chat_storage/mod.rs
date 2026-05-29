//! Chat history: SQLite (authoritative) + user-visible JSON mirror (best-effort).

mod config;
mod cleanup;
mod db;
mod export;
mod manager;
mod mirror;
mod role_config;
mod stats;
mod types;

pub use cleanup::{apply_auto_cleanup, AutoCleanupConfig};
pub use config::{
    resolve_max_messages_per_session, resolve_session_dir, resolve_storage_root,
    DEFAULT_MAX_MESSAGES, ENV_CHAT_STORAGE_ROOT, MAX_MESSAGES_PER_SESSION,
};
pub use export::{export_chat_session, export_role_chats, resolve_export_max_messages};
pub use manager::{ConversationStore, HybridConversationStore};
pub use mirror::delete_mirror_tree_for_role;
pub use role_config::save_role_chat_storage_config;
pub use stats::{collect_chat_storage_stats, delete_mirror_scene_dir, role_mirror_tree_bytes};
pub use types::{
    AppendTurnResult, AutoCleanupResult, ChatExportResponse, ChatSearchResult,
    DeleteChatsResult, ImportChatBucket, ImportChatBucketsResult,
    RoleStorageStat, SceneStorageStat, SessionMeta, StoredMessage, TurnPersistInput,
};
pub use db::{highlight_snippet, ChatSearchRow};
