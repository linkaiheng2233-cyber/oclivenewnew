//! 角色包使用后反馈（半私密）：创建、分页查询、已读与处理标记。

use crate::error::{AppError, Result};
use crate::models::dto::{
    CreateRoleFeedbackRequest, CreateRoleFeedbackResponse, MarkRoleFeedbackReadRequest,
    QueryRoleFeedbackRequest, RoleFeedbackItem, SetRoleFeedbackHandledRequest,
};
use crate::state::KernelAppState;

/// `source` 为写入 `role_feedback.source` 的审计来源；`None` 时默认 `"tauri"`（桌面宿主）。
pub async fn create_role_feedback(
    state: &KernelAppState,
    req: &CreateRoleFeedbackRequest,
    runtime_version: &str,
    source: Option<&str>,
) -> Result<CreateRoleFeedbackResponse> {
    let src = source.unwrap_or("tauri");
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
            Some(src),
        )
        .await?;

    Ok(CreateRoleFeedbackResponse { id })
}

pub async fn query_role_feedback(
    state: &KernelAppState,
    req: &QueryRoleFeedbackRequest,
) -> Result<Vec<RoleFeedbackItem>> {
    if req.limit <= 0 || req.limit > 200 {
        return Err(AppError::InvalidParameter(
            "limit must be between 1 and 200".to_string(),
        ));
    }
    if req.offset < 0 {
        return Err(AppError::InvalidParameter(
            "offset must be >= 0".to_string(),
        ));
    }

    let rows = state
        .db_manager
        .list_role_feedback(&req.role_id, req.limit as i64, req.offset as i64)
        .await?;

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

pub async fn mark_role_feedback_read(
    state: &KernelAppState,
    req: &MarkRoleFeedbackReadRequest,
) -> Result<i64> {
    state
        .db_manager
        .mark_role_feedback_read(&req.role_id, &req.ids)
        .await
}

pub async fn set_role_feedback_handled(
    state: &KernelAppState,
    req: &SetRoleFeedbackHandledRequest,
) -> Result<()> {
    state
        .db_manager
        .set_role_feedback_handled(&req.role_id, req.id, req.handled, req.note.as_deref())
        .await
}
