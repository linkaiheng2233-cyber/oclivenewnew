//! Virtual time impl shared by HTTP routes and Tauri invoke.

use crate::command_error::CommandError;
use crate::domain::virtual_time_sync::{sync_and_persist_virtual_time};
use crate::models::dto::TimeStateResponse;
use crate::service::role::ensure_manifest_role_ready;
use crate::state::AppState;
use chrono::{DateTime, Utc};

/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
pub async fn get_time_state_impl(
    state: &AppState,
    role_id: &str,
) -> Result<TimeStateResponse, CommandError> {
    ensure_manifest_role_ready(state, role_id).await?;

    let role = state.load_role_cached_async(role_id).await?;
    let immersive = state
        .db_manager
        .get_interaction_mode(role_id)
        .await?
        .is_immersive();
    let ms = sync_and_persist_virtual_time(
        state.db_manager.as_ref(),
        role.as_ref(),
        role_id,
        immersive,
    )
    .await?;
    let dt = DateTime::from_timestamp_millis(ms).unwrap_or_else(Utc::now);
    Ok(TimeStateResponse {
        virtual_time_ms: ms,
        iso_datetime: dt.to_rfc3339(),
    })
}
