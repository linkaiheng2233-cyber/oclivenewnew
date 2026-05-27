//! Chat history: SQLite (authoritative) + user-visible JSON mirror (best-effort).

mod config;
mod db;
mod manager;
mod mirror;
mod stats;
mod types;

pub use config::{
    resolve_max_messages_per_session, resolve_session_dir, resolve_storage_root,
    DEFAULT_MAX_MESSAGES, ENV_CHAT_STORAGE_ROOT, MAX_MESSAGES_PER_SESSION,
};
pub use manager::{ConversationStore, HybridConversationStore};
pub use mirror::delete_mirror_tree_for_role;
pub use stats::{collect_chat_storage_stats, delete_mirror_scene_dir, role_mirror_tree_bytes};
pub use types::{
    AppendTurnResult, DeleteChatsResult, ImportChatBucket, ImportChatBucketsResult,
    RoleStorageStat, SceneStorageStat, SessionMeta, StoredMessage, TurnPersistInput,
};
