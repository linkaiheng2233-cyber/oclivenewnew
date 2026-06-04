//! Headless service layer shared by HTTP routes and (via re-export) Tauri invoke impls.

pub mod chat_storage_proxy;
pub mod role;
pub mod time;

pub use chat_storage_proxy::{execute_chat_storage_proxy, ChatStorageProxyOp};
pub use role::{get_role_info_impl, load_role_impl};
pub use time::get_time_state_impl;
