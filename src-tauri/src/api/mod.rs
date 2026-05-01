pub mod agent;
pub mod chat;
pub mod conversation;
pub mod directory_plugin;
pub mod error;
pub mod event;
pub mod expert_models;
pub mod export;
pub mod hotkeys;
pub mod jump_monologue;
pub mod local_imports;
pub mod memory;
pub mod monologue;
pub mod plugin_audit;
pub mod plugin_bridge;
pub mod plugin_config;
pub mod plugin_debug;
pub mod plugin_index;
pub mod plugin_pack;
pub mod plugin_permissions;
pub mod plugin_reviews;
pub mod plugin_scaffold;
pub mod plugin_update;
pub mod policy;
pub mod profile;
pub mod role;
pub mod role_feedback;
pub mod role_market;
pub mod role_pack;
pub mod scene;
pub mod settings;
pub mod time;

pub use chat::send_message;
pub use directory_plugin::{directory_plugin_invoke, get_directory_plugin_bootstrap};
pub use event::{create_event, query_events};
pub use expert_models::{
    expert_models_apply_to_session, expert_models_clear_role_default, expert_models_clear_runs,
    expert_models_clear_session_override, expert_models_get_effective,
    expert_models_get_run_detail, expert_models_import_base_gguf, expert_models_import_lora_gguf,
    expert_models_list_local_base_models, expert_models_list_local_loras, expert_models_list_runs,
    expert_models_rollback_last_run, expert_models_rollback_to_run, expert_models_set_role_default,
    expert_models_set_run_pinned, expert_models_set_session_override, expert_workflows_delete,
    expert_workflows_get, expert_workflows_list, expert_workflows_save,
};
pub use export::export_chat_logs;
pub use local_imports::{list_local_import_candidates_command, read_local_import_text_command};
pub use memory::query_memories;
pub use monologue::generate_monologue;
pub use plugin_bridge::plugin_bridge_invoke;
pub use plugin_reviews::{get_cached_plugin_reviews_index, sync_plugin_reviews_index};
pub use policy::reload_policy_plugins;
pub use role::{get_role_info, list_roles, load_role, switch_role};
pub use role_feedback::{create_role_feedback, query_role_feedback};
pub use role_feedback::{mark_role_feedback_read, set_role_feedback_handled};
pub use scene::switch_scene;
pub use time::{get_time_state, jump_time};
