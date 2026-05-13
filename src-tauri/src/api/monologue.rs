use crate::domain::virtual_time;
use crate::models::dto::{GenerateMonologueRequest, GenerateMonologueResponse};
use crate::state::AppState;
use tauri::State;

pub async fn generate_monologue_impl(
    state: &AppState,
    role_id: &str,
) -> Result<GenerateMonologueResponse, String> {
    virtual_time::generate_monologue(state, role_id, None)
        .await
        .map_err(|e| e.to_frontend_error())
}

#[tauri::command]
pub async fn generate_monologue(
    req: GenerateMonologueRequest,
    state: State<'_, AppState>,
) -> Result<GenerateMonologueResponse, String> {
    generate_monologue_impl(&state, &req.role_id).await
}
