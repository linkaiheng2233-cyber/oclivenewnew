use crate::models::dto::{
    CreateRoleFeedbackRequest, CreateRoleFeedbackResponse, MarkRoleFeedbackReadRequest,
    QueryRoleFeedbackRequest, RoleFeedbackItem, SetRoleFeedbackHandledRequest,
};
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub async fn create_role_feedback(
    req: CreateRoleFeedbackRequest,
    state: State<'_, AppState>,
) -> Result<CreateRoleFeedbackResponse, String> {
    let runtime_version = env!("CARGO_PKG_VERSION");
    oclive_kernel_runtime::domain::role_feedback_commands::create_role_feedback(
        &state,
        &req,
        runtime_version,
        None,
    )
    .await
    .map_err(|e| e.to_frontend_error())
}

#[tauri::command]
pub async fn query_role_feedback(
    req: QueryRoleFeedbackRequest,
    state: State<'_, AppState>,
) -> Result<Vec<RoleFeedbackItem>, String> {
    oclive_kernel_runtime::domain::role_feedback_commands::query_role_feedback(&state, &req)
        .await
        .map_err(|e| e.to_frontend_error())
}

#[tauri::command]
pub async fn mark_role_feedback_read(
    req: MarkRoleFeedbackReadRequest,
    state: State<'_, AppState>,
) -> Result<i64, String> {
    oclive_kernel_runtime::domain::role_feedback_commands::mark_role_feedback_read(&state, &req)
        .await
        .map_err(|e| e.to_frontend_error())
}

#[tauri::command]
pub async fn set_role_feedback_handled(
    req: SetRoleFeedbackHandledRequest,
    state: State<'_, AppState>,
) -> Result<(), String> {
    oclive_kernel_runtime::domain::role_feedback_commands::set_role_feedback_handled(
        &state, &req,
    )
    .await
    .map_err(|e| e.to_frontend_error())
}
