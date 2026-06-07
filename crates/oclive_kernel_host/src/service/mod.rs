//! Headless service layer shared by HTTP routes and (via re-export) Tauri invoke impls.

pub mod chat_storage_proxy;
pub mod conversation;
pub mod export;
pub mod high_risk;
pub mod llm_settings;
pub mod plugin_bridge;
pub mod role;
pub mod scene;
pub mod settings_bridge;
pub mod time;

pub use chat_storage_proxy::{execute_chat_storage_proxy, ChatStorageProxyOp};
pub use conversation::get_conversation_list_impl;
pub use export::export_chat_logs_impl;
pub use high_risk::{
    grant_high_risk_capability_impl, list_high_risk_grants_impl, revoke_high_risk_capability_impl,
    MutateHighRiskGrantRequest,
};
pub use llm_settings::{
    get_llm_user_settings_impl, list_ollama_models_impl, probe_cloud_llm_impl,
    save_llm_user_settings_impl, set_session_llm_model_impl, LlmUserSettingsDto,
    SaveLlmUserSettingsRequest, SetSessionLlmModelRequest,
};
pub use plugin_bridge::{
    bridge_command_needs_kernel_writer, dispatch_bridge_command, parse_send_message_request,
};
pub use role::{
    delete_role_impl, get_role_info_impl, get_user_identity_state_impl, load_role_impl,
    session_namespace, set_scene_user_identity_impl, set_user_identity_impl,
};
pub use scene::{set_user_presence_scene_impl, switch_scene_impl};
pub use settings_bridge::update_settings_impl;
pub use time::{generate_monologue_impl, get_time_state_impl, jump_time_impl};
