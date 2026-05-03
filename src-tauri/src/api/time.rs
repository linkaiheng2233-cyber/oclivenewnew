use crate::domain::virtual_time;
use crate::models::dto::{JumpTimeRequest, JumpTimeResponse, TimeStateResponse};
use crate::state::AppState;
use tauri::State;

pub use virtual_time::round_to_minute_ms;

pub async fn get_time_state_impl(
    state: &AppState,
    role_id: &str,
) -> Result<TimeStateResponse, String> {
    virtual_time::get_time_state(state, role_id)
        .await
        .map_err(|e| e.to_frontend_error())
}

pub async fn jump_time_impl(
    state: &AppState,
    req: &JumpTimeRequest,
) -> Result<JumpTimeResponse, String> {
    virtual_time::jump_time(state, req)
        .await
        .map_err(|e| e.to_frontend_error())
}

#[tauri::command]
pub async fn get_time_state(
    role_id: String,
    state: State<'_, AppState>,
) -> Result<TimeStateResponse, String> {
    get_time_state_impl(&state, &role_id).await
}

#[tauri::command]
pub async fn jump_time(
    req: JumpTimeRequest,
    state: State<'_, AppState>,
) -> Result<JumpTimeResponse, String> {
    jump_time_impl(&state, &req).await
}
