use crate::error::AppError;
use crate::models::dto::{
    CreateRoleFeedbackRequest, CreateRoleFeedbackResponse, QueryRoleFeedbackRequest,
    RoleFeedbackItem,
};
use crate::state::AppState;
use serde::Deserialize;
use tauri::State;

#[tauri::command]
pub async fn create_role_feedback(
    req: CreateRoleFeedbackRequest,
    state: State<'_, AppState>,
) -> Result<CreateRoleFeedbackResponse, String> {
    let runtime_version = env!("CARGO_PKG_VERSION");
    let id = state
        .db_manager
        .insert_role_feedback(
            req.role_id.trim(),
            req.session_id.as_deref(),
            req.mood_tag.as_deref(),
            req.message.as_str(),
            req.scene_id.as_deref(),
            req.presence_mode.as_deref(),
            req.role_version.as_deref(),
            Some(runtime_version),
            req.client_version.as_deref(),
            Some("tauri"),
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
            status: Some(r.status),
            read_at: r.read_at,
            handled_at: r.handled_at,
            handled_note: r.handled_note,
            scene_id: r.scene_id,
            presence_mode: r.presence_mode,
            role_version: r.role_version,
            runtime_version: r.runtime_version,
            client_version: r.client_version,
            source: r.source,
        })
        .collect())
}

#[derive(Debug, Clone, Deserialize)]
pub struct MarkRoleFeedbackReadRequest {
    pub role_id: String,
    pub ids: Vec<i64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SetRoleFeedbackHandledRequest {
    pub role_id: String,
    pub id: i64,
    pub handled: bool,
    #[serde(default)]
    pub note: Option<String>,
}

#[tauri::command]
pub async fn mark_role_feedback_read(
    req: MarkRoleFeedbackReadRequest,
    state: State<'_, AppState>,
) -> Result<i64, String> {
    state
        .db_manager
        .mark_role_feedback_read(&req.role_id, &req.ids)
        .await
        .map_err(|e: AppError| e.to_frontend_error())
}

#[tauri::command]
pub async fn set_role_feedback_handled(
    req: SetRoleFeedbackHandledRequest,
    state: State<'_, AppState>,
) -> Result<(), String> {
    state
        .db_manager
        .set_role_feedback_handled(&req.role_id, req.id, req.handled, req.note.as_deref())
        .await
        .map_err(|e: AppError| e.to_frontend_error())
}
