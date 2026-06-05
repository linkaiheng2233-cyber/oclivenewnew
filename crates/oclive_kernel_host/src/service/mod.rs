//! Headless service layer shared by HTTP routes and (via re-export) Tauri invoke impls.

pub mod chat_storage_proxy;
pub mod conversation;
pub mod export;
pub mod plugin_bridge;
pub mod role;
pub mod settings_bridge;
pub mod time;

pub use chat_storage_proxy::{execute_chat_storage_proxy, ChatStorageProxyOp};
pub use conversation::get_conversation_list_impl;
pub use export::export_chat_logs_impl;
pub use plugin_bridge::{
    bridge_command_needs_kernel_writer, dispatch_bridge_command, parse_send_message_request,
};
pub use role::{delete_role_impl, get_role_info_impl, load_role_impl};
pub use settings_bridge::update_settings_impl;
pub use time::{generate_monologue_impl, get_time_state_impl, jump_time_impl};
