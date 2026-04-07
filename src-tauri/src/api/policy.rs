use crate::error::AppError;
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub async fn reload_policy_plugins(state: State<'_, AppState>) -> Result<String, String> {
    state
        .reload_policy_plugins()
        .map(|count| format!("policy plugins reloaded: {} scene bindings", count))
        .map_err(|e: AppError| e.to_frontend_error())
}
