use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub async fn reload_policy_plugins(state: State<'_, AppState>) -> Result<String, String> {
    oclive_kernel_runtime::domain::policy_host::reload_policy_plugins_message(&state)
        .map_err(|e| e.to_frontend_error())
}
