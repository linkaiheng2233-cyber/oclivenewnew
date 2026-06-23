use crate::api::error::CommandError;
use oclive_kernel_host::state::SharedAppState;
use tauri::State;
/// Returns an error when policy plugin reload fails.
#[tauri::command]
pub async fn reload_policy_plugins(
    state: State<'_, SharedAppState>,
) -> Result<String, CommandError> {
    let count = state.reload_policy_plugins().map_err(CommandError::from)?;
    Ok(format!("policy plugins reloaded: {} scene bindings", count))
}
