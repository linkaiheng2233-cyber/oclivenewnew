//! 本地 HTTP API（`--api`）：供编写器试聊等工具调用，不经 Tauri IPC。
//!
//! 仅绑定 `127.0.0.1`；生产环境请自行评估暴露面。
//!
//! `POST /chat` 成功响应在扁平化的 `SendMessageResponse` 字段之外另含 **`personality_source`**
//!（与包内 `settings.json` → `evolution.personality_source` 一致：`vector` | `profile`），便于试聊工具区分人格模式。
//!
//! **错误体**：`{ "error": KernelErrorBody }` 与 Tauri `invoke` 失败字符串 **同源**（见 `oclive_kernel_runtime::KernelErrorBody`），
//! `code` 与 [`AppError::code`] 一致（`SCREAMING_SNAKE_CASE`）；HTTP 专有错误使用 [`oclive_kernel_runtime::http_chat_codes`] 常量（与内核 crate 同源，避免字面量漂移）。

use crate::domain::chat_engine::process_message;
use crate::error::AppError;
use crate::infrastructure::MockLlmClient;
use crate::models::dto::{SendMessageRequest, SendMessageResponse};
use crate::models::role::PersonalitySource;
use crate::state::AppState;
use axum::extract::State;
use axum::http::Method;
use axum::routing::{get, post};
use axum::Json;
use axum::Router;
use oclive_kernel_runtime::{http_chat_codes, KernelErrorBody};
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
pub struct ApiErrorResponse {
    pub error: KernelErrorBody,
}

type ApiError = (axum::http::StatusCode, Json<ApiErrorResponse>);

fn api_error(status: axum::http::StatusCode, error: KernelErrorBody) -> ApiError {
    (status, Json(ApiErrorResponse { error }))
}

#[must_use]
fn kernel_http_error(code: &str, message: impl Into<String>, hint: Option<String>) -> KernelErrorBody {
    KernelErrorBody {
        code: code.to_string(),
        message: message.into(),
        hint,
    }
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

/// 与 [`serve_api`] 相同的路由树，供集成测试 `tower::ServiceExt::oneshot` 使用（无需绑端口）。
pub fn api_router(app_state: Arc<AppState>) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers(Any);

    Router::new()
        .route("/health", get(health))
        .route("/chat", post(chat))
        .layer(cors)
        .with_state(app_state)
}
/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
/// 阻塞运行 HTTP 服务，直到进程结束。
///
/// CI / 协议黑盒：设置 `OCLIVE_HTTP_API_MOCK_LLM=1` 时使用内存库 + [`MockLlmClient`]，不依赖本机 Ollama。
pub async fn serve_api(port: u16) -> Result<(), String> {
    let db_path = std::env::temp_dir().join(format!("oclive_api_{}.db", port));
    let roles_dir = crate::state::resolve_roles_dir();
    let app_data_dir = db_path
        .parent()
        .map(|p| p.join("oclive_api_app_data"))
        .unwrap_or_else(|| std::env::temp_dir().join("oclive_api_app_data"));
    let _ = std::fs::create_dir_all(&app_data_dir);
    let mock_llm = std::env::var("OCLIVE_HTTP_API_MOCK_LLM")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);
    let app_state = if mock_llm {
        tracing::warn!(target: "oclive_api", "OCLIVE_HTTP_API_MOCK_LLM enabled: using in-memory DB + mock LLM");
        let llm = Arc::new(MockLlmClient {
            reply: "OOCP mock reply".to_string(),
        });
        AppState::new_in_memory_with_llm(llm, roles_dir.clone())
            .await
            .map_err(|e| e.to_string())?
    } else {
        AppState::new(&db_path, Some(roles_dir), &app_data_dir)
            .await
            .map_err(|e| e.to_string())?
    };
    crate::domain::startup_health::run_global_db_ping(&app_state.db_manager)
        .await
        .map_err(|e| e.to_string())?;
    let app_state = Arc::new(app_state);

    let app = api_router(app_state);

    let addr = format!("127.0.0.1:{}", port);
    let listener = TcpListener::bind(&addr)
        .await
        .map_err(|e| format!("绑定 {} 失败：{}", addr, e))?;
    tracing::info!(target: "oclive_api", "HTTP API listening http://{}", addr);
    axum::serve(listener, app)
        .await
        .map_err(|e| format!("HTTP 服务异常：{}", e))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::role::PersonalitySource as Ps;

    #[test]
    fn personality_source_json_matches_http_contract() {
        let v = serde_json::to_value(Ps::Vector).unwrap();
        let p = serde_json::to_value(Ps::Profile).unwrap();
        assert_eq!(v, "vector");
        assert_eq!(p, "profile");
    }

    #[test]
    fn api_error_serializes_kernel_error_body() {
        let (_, Json(body)) = api_error(
            axum::http::StatusCode::BAD_REQUEST,
            kernel_http_error(
                http_chat_codes::INVALID_ROLE_PATH,
                "role_path is not a directory: /x",
                Some("请传入绝对路径".into()),
            ),
        );
        let v = serde_json::to_value(body).expect("serialize");
        assert_eq!(v["error"]["code"], "INVALID_ROLE_PATH");
        assert_eq!(v["error"]["message"], "role_path is not a directory: /x");
        assert_eq!(v["error"]["hint"], "请传入绝对路径");
    }
}
