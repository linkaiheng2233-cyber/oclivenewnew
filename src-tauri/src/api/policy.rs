use crate::state::AppState;
use tauri::State;
use crate::api::error::CommandError;
/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
#[tauri::command]
pub async fn reload_policy_plugins(state: State<'_, AppState>) -> Result<String, CommandError> {
    let count = state.reload_policy_plugins().map_err(CommandError::from)?;
    Ok(format!("policy plugins reloaded: {} scene bindings", count))
}
