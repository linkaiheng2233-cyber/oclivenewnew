use crate::error::AppError;
use crate::models::dto::{
    CreateRoleFeedbackRequest, CreateRoleFeedbackResponse, QueryRoleFeedbackRequest,
    RoleFeedbackItem,
};
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub async fn create_role_feedback(
    req: CreateRoleFeedbackRequest,
    state: State<'_, AppState>,
) -> Result<CreateRoleFeedbackResponse, String> {
    let id = state
        .db_manager
        .insert_role_feedback(
            req.role_id.trim(),
            req.session_id.as_deref(),
            req.mood_tag.as_deref(),
            req.message.as_str(),
        )
        .await
        .map_err(|e: AppError| e.to_frontend_error())?;

    Ok(CreateRoleFeedbackResponse { id })
}

#[tauri::command]
pub async fn query_role_feedback(
    req: QueryRoleFeedbackRequest,
    state: State<'_, AppState>,
) -> Result<Vec<RoleFeedbackItem>, String> {
    if req.limit <= 0 || req.limit > 200 {
        return Err(
            AppError::InvalidParameter("limit must be between 1 and 200".to_string())
                .to_frontend_error(),
        );
    }
    if req.offset < 0 {
        return Err(
            AppError::InvalidParameter("offset must be >= 0".to_string()).to_frontend_error(),
        );
    }

    let rows = state
        .db_manager
        .list_role_feedback(&req.role_id, req.limit as i64, req.offset as i64)
        .await
        .map_err(|e: AppError| e.to_frontend_error())?;

    Ok(rows
        .into_iter()
        .map(|r| RoleFeedbackItem {
            id: r.id,
            role_id: r.role_id,
            session_id: r.session_id,
            mood_tag: r.mood_tag,
            message: r.message,
            timestamp: r.created_at,
        })
        .collect())
}
