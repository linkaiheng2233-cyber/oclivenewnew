//! Role load / snapshot impls shared by HTTP routes and Tauri invoke.

pub mod affect;
pub mod delete;
pub mod display;
pub mod identity;
pub mod interaction;
pub mod runtime;
pub mod slot_session;

pub use affect::get_display_metrics_impl;
pub use delete::delete_role_impl;
pub use identity::{
    get_user_identity_state_impl, set_scene_user_identity_impl, set_user_identity_impl,
};
pub use interaction::set_role_interaction_mode_impl;
pub use slot_session::{
    apply_author_suggested_plugin_backends_impl, build_plugin_resolution_debug_info,
    clear_all_session_slot_overrides_impl, clear_session_slot_override_impl,
    get_plugin_resolution_debug_impl, save_role_slot_registry_impl,
    set_session_plugin_backend_impl, set_session_slot_override_impl,
};

use crate::command_error::CommandError;
use crate::domain::role_snapshot::{assemble_role_data, assemble_role_info};
use crate::models::dto::{RoleData, RoleInfo, RoleSummary};
use crate::state::AppState;
use std::sync::Arc;

#[must_use]
pub fn session_namespace(role_id: &str, session_id: Option<&str>) -> String {
    crate::domain::chat_engine::conversation_state_role_id(role_id, session_id)
}

/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
/// When `reset_portrait_emotion` is `true` (app startup `load_role`), portrait emotion resets to `neutral`.
pub async fn load_role_impl(
    state: &AppState,
    role_id: &str,
    reset_portrait_emotion: bool,
) -> Result<RoleData, CommandError> {
    let role = state.storage.load_role(role_id)?;
    let role = Arc::new(role);

    state.directory_plugins.set_active_role_id(role_id);
    state
        .directory_plugins
        .ensure_role_plugin_state(role_id, role.plugin_state_ui_baseline());
    crate::service::execution_plan::ensure_role_execution_plan_activatable(state, role.as_ref())?;

    state.invalidate_personality_cache_for_role(role_id);

    state.db_manager.ensure_role_runtime(role_id).await?;

    if reset_portrait_emotion {
        state
            .db_manager
            .set_current_emotion(role_id, "neutral")
            .await?;
    }

    state
        .role_cache
        .write()
        .insert(role_id.to_string(), Arc::clone(&role));

    assemble_role_data(state, role_id, role.as_ref()).await
}

/// Ensure manifest `role_id` has `role_runtime` (auto [`load_role_impl`] when missing).
///
/// # Errors
///
/// Returns [`Err`] when role loading or runtime initialization fails.
pub async fn ensure_manifest_role_ready(
    state: &AppState,
    role_id: &str,
) -> Result<(), CommandError> {
    ensure_role_info_ready(state, role_id, None).await
}

async fn ensure_role_info_ready(
    state: &AppState,
    role_id: &str,
    session_id: Option<&str>,
) -> Result<(), CommandError> {
    let session_ns = session_namespace(role_id, session_id);
    if state
        .db_manager
        .role_runtime_exists(session_ns.as_str())
        .await?
    {
        return Ok(());
    }
    if session_id
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .is_some()
    {
        state
            .db_manager
            .ensure_role_runtime(session_ns.as_str())
            .await?;
        return Ok(());
    }
    load_role_impl(state, role_id, false).await?;
    Ok(())
}

/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
pub async fn get_role_info_impl(
    state: &AppState,
    role_id: &str,
    session_id: Option<&str>,
) -> Result<RoleInfo, CommandError> {
    ensure_role_info_ready(state, role_id, session_id).await?;

    let role = state.load_role_cached_async(role_id).await?;

    assemble_role_info(state, role_id, role.as_ref(), session_id).await
}

/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
pub async fn list_roles_impl(state: &AppState) -> Result<Vec<RoleSummary>, CommandError> {
    let list_dev = crate::env_flags::list_dev_roles_enabled();
    let roles = state.storage.load_all_roles()?;
    let mut summaries: Vec<RoleSummary> = roles
        .into_iter()
        .filter(|r| list_dev || !r.dev_only)
        .map(|r| RoleSummary {
            id: r.id,
            name: r.name,
            version: r.version,
            author: r.author,
            description: r.description,
            featured: r.featured,
            preset_order: r.preset_order,
            interaction_mode_suggestion: r.interaction_mode.clone(),
            adult_extension_available: r.adult_extension.is_some(),
        })
        .collect();
    summaries.sort_by(|a, b| {
        a.preset_order
            .cmp(&b.preset_order)
            .then_with(|| a.name.cmp(&b.name))
    });
    Ok(summaries)
}

/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
pub async fn switch_role_impl(state: &AppState, role_id: &str) -> Result<RoleInfo, CommandError> {
    load_role_impl(state, role_id, false).await?;
    get_role_info_impl(state, role_id, None).await
}
