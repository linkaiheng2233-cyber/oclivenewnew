//! Local HTTP API + OOCP WS entrypoint (kernel side).
//!
//! This module is migrated from `src-tauri/src/http_api.rs` and intentionally contains
//! **no Tauri IPC**. It binds to `127.0.0.1` and exposes:
//! - `GET /health`
//! - `GET /oocp` (WebSocket)
//! - `POST /chat` (tooling endpoint)

use crate::domain::adapters::oocp_ws;
use crate::domain::chat_engine::process_message;
use crate::error::AppError;
use crate::models::dto::{SendMessageRequest, SendMessageResponse};
use crate::models::role::PersonalitySource;
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
use tokio::task::spawn_blocking;
use tower_http::cors::{Any, CorsLayer};

/// `spawn_blocking` 内：`load_role_from_dir` 与目录探测均为阻塞 I/O，勿在异步线程直接调用。
enum ChatRoleLoadError {
    NotDirectory(String),
    Load(crate::error::AppError),
}

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

/// 与 `SendMessageResponse` 字段一致，并额外回显 `session_id`、`personality_source`；供编写器试聊展示状态条。
#[derive(Debug, Serialize)]
pub struct ChatApiResponse {
    #[serde(flatten)]
    pub data: SendMessageResponse,
    /// `evolution.personality_source`：与 `get_role_info` / 包内 settings 对齐。
    pub personality_source: PersonalitySource,
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
            "load_role_task_panic",
            format!("加载角色任务异常: {}", e),
            None,
        )
    })?;

    let role = match blocked {
        Err(ChatRoleLoadError::NotDirectory(display)) => {
            return Err(api_error(
                axum::http::StatusCode::BAD_REQUEST,
                "invalid_role_path",
                format!("role_path 不是目录：{}", display),
                Some("请传入包含 manifest.json 的角色目录绝对路径".to_string()),
            ));
        }
        Err(ChatRoleLoadError::Load(e)) => {
            return Err(api_error(
                axum::http::StatusCode::BAD_REQUEST,
                "load_role_failed",
                e.to_frontend_error(),
                Some("请检查角色目录结构与 manifest/settings 是否完整".to_string()),
            ));
        }
        Ok(r) => r,
    };

    let personality_source = role.evolution_config.personality_source;
    let role = Arc::new(role);

    state.invalidate_personality_cache_for_role(role.id.as_str());

    state
        .role_cache
        .write()
        .insert(role.id.clone(), Arc::clone(&role));

    let req = SendMessageRequest {
        role_id: role.id.clone(),
        user_message,
        scene_id: body.scene_id,
        session_id: body.session_id,
    };

    let res: SendMessageResponse = process_message(&state, &req).await.map_err(|e: AppError| {
        api_error(
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "chat_engine_failed",
            e.to_frontend_error(),
            Some("请查看 oclive 日志（target: oclive_chat / oclive_plugin）".to_string()),
        )
    })?;

    Ok(Json(ChatApiResponse {
        data: res,
        personality_source,
        session_id: session_echo,
    }))
}

/// 与 `serve_api` 相同的路由树，供集成测试使用（无需绑端口）。
/// 已合并 OOCP WebSocket 路由（`/oocp`）。
pub fn api_router(app_state: Arc<AppState>) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers(Any);

    Router::new()
        .route("/health", get(health))
        .route("/chat", post(chat))
        .merge(oocp_ws::oocp_ws_router())
        .layer(cors)
        .with_state(app_state)
}

#[derive(Debug, Clone)]
pub struct ApiServerOptions {
    pub port: u16,
    pub db_path: PathBuf,
    pub roles_dir: PathBuf,
    pub app_data_dir: PathBuf,
}

impl ApiServerOptions {
    pub fn from_env_or_defaults(port: u16) -> Self {
        let db_path = std::env::var("OCLIVE_DB_PATH")
            .ok()
            .filter(|s| !s.trim().is_empty())
            .map(PathBuf::from)
            .unwrap_or_else(|| std::env::temp_dir().join(format!("oclive_api_{}.db", port)));

        let roles_dir = std::env::var("OCLIVE_ROLES_DIR")
            .ok()
            .filter(|s| !s.trim().is_empty())
            .map(PathBuf::from)
            .unwrap_or_else(crate::state::resolve_roles_dir);

        let app_data_dir = std::env::var("OCLIVE_APP_DATA_DIR")
            .ok()
            .filter(|s| !s.trim().is_empty())
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                db_path
                    .parent()
                    .map(|p| p.join("oclive_api_app_data"))
                    .unwrap_or_else(|| std::env::temp_dir().join("oclive_api_app_data"))
            });

        Self {
            port,
            db_path,
            roles_dir,
            app_data_dir,
        }
    }
}

/// 阻塞运行 HTTP 服务，直到进程结束。
pub async fn serve_api(port: u16) -> Result<(), String> {
    serve_api_with_options(ApiServerOptions::from_env_or_defaults(port)).await
}

pub async fn serve_api_with_options(opt: ApiServerOptions) -> Result<(), String> {
    let _ = std::fs::create_dir_all(&opt.app_data_dir);
    let app_state = AppState::new(&opt.db_path, Some(opt.roles_dir), &opt.app_data_dir)
        .await
        .map_err(|e| e.to_string())?;
    let app_state = Arc::new(app_state);

    let app = api_router(app_state);

    let addr = format!("127.0.0.1:{}", opt.port);
    let listener = TcpListener::bind(&addr)
        .await
        .map_err(|e| format!("绑定 {} 失败：{}", addr, e))?;
    log::info!(target: "oclive_api", "HTTP API listening http://{}", addr);
    axum::serve(listener, app)
        .await
        .map_err(|e| format!("HTTP 服务异常：{}", e))?;
    Ok(())
}
