//! App-level settings (`app_settings`), updatable via controlled bridge.

use crate::api::error::CommandError;
use oclive_kernel_host::state::SharedAppState;
use serde::Serialize;
use tauri::State;

pub use oclive_kernel_host::service::settings_bridge::update_settings_impl;

/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
#[tauri::command]
pub async fn set_remote_fallback_to_builtin(
    state: State<'_, SharedAppState>,
    allow: bool,
) -> Result<(), CommandError> {
    let raw = if allow { "1" } else { "0" };
    state
        .db_manager
        .upsert_app_setting("remote_fallback_to_builtin", raw)
        .await?;
    let fresh = state
        .db_manager
        .get_app_setting("remote_fallback_to_builtin")
        .await?;
    state.sync_remote_fallback_from_db_value(fresh);
    Ok(())
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteFallbackAppSettings {
    /// `app_settings.remote_fallback_to_builtin` in DB (`"0"` / `"1"`).
    pub remote_fallback_to_builtin: String,
    /// When `OCLIVE_REMOTE_FALLBACK_TO_BUILTIN` is set, env drives in-process value; UI should lock the toggle.
    pub remote_fallback_env_override_active: bool,
}

/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
#[tauri::command]
pub async fn get_remote_fallback_app_settings(
    state: State<'_, SharedAppState>,
) -> Result<RemoteFallbackAppSettings, CommandError> {
    let remote_fallback_to_builtin = state
        .db_manager
        .get_app_setting("remote_fallback_to_builtin")
        .await?
        .unwrap_or_else(|| "1".to_string());
    let remote_fallback_env_override_active =
        oclive_kernel_host::infrastructure::remote_fallback_policy::remote_fallback_env_override().is_some();
    Ok(RemoteFallbackAppSettings {
        remote_fallback_to_builtin,
        remote_fallback_env_override_active,
    })
}
