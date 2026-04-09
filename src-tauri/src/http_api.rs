//! 本地 HTTP API（`--api`）：供编写器试聊等工具调用，不经 Tauri IPC。
//!
//! 仅绑定 `127.0.0.1`；生产环境请自行评估暴露面。

use crate::domain::chat_engine::process_message;
use crate::error::AppError;
use crate::models::dto::{SendMessageRequest, SendMessageResponse};
use crate::state::AppState;
use axum::extract::State;
use axum::http::Method;
use axum::routing::{get, post};
use axum::Json;
use axum::Router;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};

#[derive(Debug, Deserialize)]
pub struct ChatApiRequest {
    pub role_path: String,
    pub message: String,
    #[serde(default)]
    pub session_id: Option<String>,
    /// 可选：与主应用 `send_message` 一致；未传则由引擎按会话状态推断。
    #[serde(default)]
    pub scene_id: Option<String>,
}

/// 与 `SendMessageResponse` 字段一致，并额外回显 `session_id`；供编写器试聊展示状态条。
#[derive(Debug, Serialize)]
pub struct ChatApiResponse {
    #[serde(flatten)]
    pub data: SendMessageResponse,
    /// 回显客户端提交的会话 id（便于编写器与日志对齐；未提交则为 `null`）。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ApiErrorDetail {
    pub code: &'static str,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ApiErrorResponse {
    pub error: ApiErrorDetail,
}

type ApiError = (axum::http::StatusCode, Json<ApiErrorResponse>);

fn api_error(
    status: axum::http::StatusCode,
    code: &'static str,
    message: impl Into<String>,
    hint: Option<String>,
) -> ApiError {
    (
        status,
        Json(ApiErrorResponse {
            error: ApiErrorDetail {
                code,
                message: message.into(),
                hint,
            },
        }),
    )
}

async fn health() -> &'static str {
    "ok"
}

async fn chat(
    State(state): State<Arc<AppState>>,
    Json(body): Json<ChatApiRequest>,
) -> Result<Json<ChatApiResponse>, ApiError> {
    let session_echo = body.session_id.clone();
    let user_message = body.message.trim().to_string();
    if user_message.is_empty() {
        return Err(api_error(
            axum::http::StatusCode::BAD_REQUEST,
            "empty_message",
            "message 不能为空",
            Some("请至少输入 1 个可见字符".to_string()),
        ));
    }
    let path = PathBuf::from(body.role_path.trim());
    if !path.is_dir() {
        return Err(api_error(
            axum::http::StatusCode::BAD_REQUEST,
            "invalid_role_path",
            format!("role_path 不是目录：{}", path.display()),
            Some("请传入包含 manifest.json 的角色目录绝对路径".to_string()),
        ));
    }

    let role = state
        .storage
        .load_role_from_dir(&path)
        .map_err(|e: AppError| {
            api_error(
                axum::http::StatusCode::BAD_REQUEST,
                "load_role_failed",
                e.to_frontend_error(),
                Some("请检查角色目录结构与 manifest/settings 是否完整".to_string()),
            )
        })?;

    state
        .role_cache
        .write()
        .insert(role.id.clone(), role.clone());

    let req = SendMessageRequest {
        role_id: role.id.clone(),
        user_message,
        scene_id: body.scene_id,
        session_id: body.session_id,
    };

    let res: SendMessageResponse = process_message(&state, &req)
        .await
        .map_err(|e: AppError| {
            api_error(
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "chat_engine_failed",
                e.to_frontend_error(),
                Some("请查看 oclive 日志（target: oclive_chat / oclive_plugin）".to_string()),
            )
        })?;

    Ok(Json(ChatApiResponse {
        data: res,
        session_id: session_echo,
    }))
}

/// 阻塞运行 HTTP 服务，直到进程结束。
pub async fn serve_api(port: u16) -> Result<(), String> {
    let db_path = std::env::temp_dir().join(format!("oclive_api_{}.db", port));
    let roles_dir = crate::state::resolve_roles_dir();
    let app_state = AppState::new(&db_path, Some(roles_dir))
        .await
        .map_err(|e| e.to_string())?;
    let app_state = Arc::new(app_state);

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers(Any);

    let app = Router::new()
        .route("/health", get(health))
        .route("/chat", post(chat))
        .layer(cors)
        .with_state(app_state);

    let addr = format!("127.0.0.1:{}", port);
    let listener = TcpListener::bind(&addr)
        .await
        .map_err(|e| format!("绑定 {} 失败：{}", addr, e))?;
    log::info!(target: "oclive_api", "HTTP API listening http://{}", addr);
    axum::serve(listener, app)
        .await
        .map_err(|e| format!("HTTP 服务异常：{}", e))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn api_error_serializes_code_message_hint() {
        let (_, Json(body)) = api_error(
            axum::http::StatusCode::BAD_REQUEST,
            "invalid_role_path",
            "role_path 不是目录",
            Some("请传入绝对路径".to_string()),
        );
        let v = serde_json::to_value(body).expect("serialize");
        assert_eq!(v["error"]["code"], "invalid_role_path");
        assert_eq!(v["error"]["message"], "role_path 不是目录");
        assert_eq!(v["error"]["hint"], "请传入绝对路径");
    }
}
