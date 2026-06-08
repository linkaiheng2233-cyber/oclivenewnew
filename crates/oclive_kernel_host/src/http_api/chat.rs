use super::{api_error, kernel_http_error, ApiError, ChatApiRequest, ChatApiResponse};
use crate::domain::chat_engine::process_message;
use crate::error::{http_chat_codes, AppError};
use crate::infrastructure::chat_storage::{SessionMeta, StoredMessage};
use crate::models::dto::{SendMessageRequest, SendMessageResponse};
use crate::service::{execute_chat_storage_proxy, ChatStorageProxyOp};
use crate::state::AppState;
use axum::extract::{Query, State};
use axum::Json;
use serde::Deserialize;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::task::spawn_blocking;

/// Inside `spawn_blocking`: `load_role_from_dir` and directory probing are both blocking I/O; do not call them directly on an async thread.
enum ChatRoleLoadError {
    NotDirectory(String),
    Load(crate::error::AppError),
}

#[derive(Debug, Deserialize)]
pub(crate) struct ChatSessionsQuery {
    role_id: String,
    scene_id: String,
    limit: Option<u32>,
    offset: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ChatMessagesQuery {
    session_id: String,
    limit: Option<u32>,
    offset: Option<u32>,
}

pub(crate) async fn chat(
    State(state): State<Arc<AppState>>,
    Json(body): Json<ChatApiRequest>,
) -> Result<Json<ChatApiResponse>, ApiError> {
    let session_echo = body.session_id.clone();
    let user_message = body.message.trim().to_string();
    if user_message.is_empty() {
        return Err(api_error(
            axum::http::StatusCode::BAD_REQUEST,
            kernel_http_error(
                http_chat_codes::EMPTY_MESSAGE,
                "message must not be empty or whitespace-only",
                Some("请至少输入 1 个可见字符".into()),
            ),
        ));
    }
    let path = PathBuf::from(body.role_path.trim());
    let storage = state.storage.clone();
    let blocked = spawn_blocking(move || {
        if !path.is_dir() {
            return Err(ChatRoleLoadError::NotDirectory(path.display().to_string()));
        }
        storage
            .load_role_from_dir(&path)
            .map_err(ChatRoleLoadError::Load)
    })
    .await
    .map_err(|e| {
        api_error(
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            kernel_http_error(
                http_chat_codes::LOAD_ROLE_TASK_PANIC,
                format!("load_role task panicked: {e}"),
                None,
            ),
        )
    })?;

    let role = match blocked {
        Err(ChatRoleLoadError::NotDirectory(display)) => {
            return Err(api_error(
                axum::http::StatusCode::BAD_REQUEST,
                kernel_http_error(
                    http_chat_codes::INVALID_ROLE_PATH,
                    format!("role_path is not a directory: {display}"),
                    Some("请传入包含 manifest.json 的角色目录绝对路径".into()),
                ),
            ));
        }
        Err(ChatRoleLoadError::Load(e)) => {
            let mut k = e.kernel_error_body();
            k.hint = Some("请检查角色目录结构与 manifest/settings 是否完整".into());
            return Err(api_error(axum::http::StatusCode::BAD_REQUEST, k));
        }
        Ok(r) => r,
    };

    let personality_source = role.evolution_config.personality_source;
    let role = Arc::new(role);

    state.invalidate_personality_cache_for_role(role.id.as_str());

    state
        .http_api_roles
        .insert(role.id.clone(), Arc::clone(&role));

    let req = SendMessageRequest {
        role_id: role.id.clone(),
        user_message,
        scene_id: body.scene_id,
        session_id: body.session_id,
        include_raw_reply: body.include_raw_reply,
    };

    let res: SendMessageResponse = process_message(&state, &req).await.map_err(|e: AppError| {
        let mut k = e.kernel_error_body();
        k.hint = Some("请查看 oclive 日志（target: oclive_chat / oclive_plugin）".into());
        api_error(axum::http::StatusCode::INTERNAL_SERVER_ERROR, k)
    })?;

    Ok(Json(ChatApiResponse {
        data: res,
        personality_source,
        session_id: session_echo,
    }))
}

pub(crate) async fn chat_sessions_route(
    State(state): State<Arc<AppState>>,
    Query(q): Query<ChatSessionsQuery>,
) -> Result<Json<Vec<SessionMeta>>, ApiError> {
    state
        .conversation_store
        .list_sessions(
            q.role_id.trim(),
            q.scene_id.trim(),
            q.limit.unwrap_or(50),
            q.offset.unwrap_or(0),
        )
        .await
        .map(Json)
        .map_err(|e| {
            let k = e.kernel_error_body();
            api_error(axum::http::StatusCode::INTERNAL_SERVER_ERROR, k)
        })
}

pub(crate) async fn chat_messages_route(
    State(state): State<Arc<AppState>>,
    Query(q): Query<ChatMessagesQuery>,
) -> Result<Json<Vec<StoredMessage>>, ApiError> {
    state
        .conversation_store
        .fetch_messages(
            q.session_id.trim(),
            q.limit.unwrap_or(500),
            q.offset.unwrap_or(0),
        )
        .await
        .map(Json)
        .map_err(|e| {
            let k = e.kernel_error_body();
            api_error(axum::http::StatusCode::INTERNAL_SERVER_ERROR, k)
        })
}

pub(crate) async fn chat_storage_proxy_route(
    State(state): State<Arc<AppState>>,
    Json(op): Json<ChatStorageProxyOp>,
) -> Result<Json<serde_json::Value>, ApiError> {
    execute_chat_storage_proxy(state.as_ref(), op)
        .await
        .map(Json)
        .map_err(|e| {
            let k = e.kernel_error_body();
            api_error(axum::http::StatusCode::INTERNAL_SERVER_ERROR, k)
        })
}
