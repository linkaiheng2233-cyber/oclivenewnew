//! Chat history: SQLite (authoritative) + user-visible JSON mirror (best-effort).

mod config;
mod db;
mod manager;
mod mirror;
mod types;

pub use config::{
    resolve_session_dir, resolve_storage_root, ENV_CHAT_STORAGE_ROOT, MAX_MESSAGES_PER_SESSION,
};
pub use manager::{ConversationStore, HybridConversationStore};
pub use mirror::delete_mirror_tree_for_role;
pub use types::{SessionMeta, StoredMessage, TurnPersistInput};
