use crate::api::error::CommandError;
use crate::state::SharedAppState;
use tauri::State;
/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
#[tauri::command]
pub async fn reload_policy_plugins(
    state: State<'_, SharedAppState>,
) -> Result<String, CommandError> {
    let count = state.reload_policy_plugins().map_err(CommandError::from)?;
    Ok(format!("policy plugins reloaded: {} scene bindings", count))
}
