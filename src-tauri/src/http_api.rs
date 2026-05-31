//! Local HTTP API (`--api`): for tools such as the pack editor's try-chat to call, bypassing Tauri IPC.
//!
//! Binds only to `127.0.0.1`; evaluate the exposure surface yourself for production environments.
//!
//! Beyond the flattened `SendMessageResponse` fields, a successful `POST /chat` response also includes **`personality_source`**
//! (consistent with the pack's `settings.json` → `evolution.personality_source`: `vector` | `profile`), so try-chat tools can distinguish personality modes.
//!
//! **Error body**: `{ "error": KernelErrorBody }` shares the same source as the Tauri `invoke` failure string (see `oclive_kernel_runtime::KernelErrorBody`);
//! `code` is consistent with [`AppError::code`] (`SCREAMING_SNAKE_CASE`); HTTP-specific errors use [`oclive_kernel_runtime::http_chat_codes`] constants (same source as the kernel crate, avoiding literal drift).

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

/// Inside `spawn_blocking`: `load_role_from_dir` and directory probing are both blocking I/O; do not call them directly on an async thread.
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
    /// Optional: consistent with the main app's `send_message`; if omitted, the engine infers it from session state.
    #[serde(default)]
    pub scene_id: Option<String>,
}

/// Mirrors the `SendMessageResponse` fields and additionally echoes back `session_id` and `personality_source`; used by the pack editor's try-chat to display a status bar.
#[derive(Debug, Serialize)]
pub struct ChatApiResponse {
    #[serde(flatten)]
    pub data: SendMessageResponse,
    /// `evolution.personality_source`: aligned with `get_role_info` / the pack's settings.
    pub personality_source: PersonalitySource,
    /// Echoes back the session id submitted by the client (helps align the pack editor with logs; `null` if not submitted).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ApiErrorResponse {
    pub error: KernelErrorBody,
}

type ApiError = (axum::http::StatusCode, Json<ApiErrorResponse>);

struct ApiTempCleanup {
    db_path: PathBuf,
    app_data_dir: PathBuf,
}

impl Drop for ApiTempCleanup {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.db_path);
        let _ = std::fs::remove_dir_all(&self.app_data_dir);
    }
}

fn api_error(status: axum::http::StatusCode, error: KernelErrorBody) -> ApiError {
    (status, Json(ApiErrorResponse { error }))
}

#[must_use]
fn kernel_http_error(
    code: &str,
    message: impl Into<String>,
    hint: Option<String>,
) -> KernelErrorBody {
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

    state.http_api_roles.insert(role.id.clone(), Arc::clone(&role));

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

/// The same route tree as [`serve_api`], for integration tests to use via `tower::ServiceExt::oneshot` (no port binding required).
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
/// Runs the HTTP service in a blocking manner until the process exits.
///
/// CI / protocol black-box: when `OCLIVE_HTTP_API_MOCK_LLM=1` is set, uses an in-memory DB + [`MockLlmClient`], not depending on a local Ollama.
pub async fn serve_api(port: u16) -> Result<(), String> {
    let db_path = std::env::temp_dir().join(format!("oclive_api_{}.db", port));
    let roles_dir = crate::state::resolve_roles_dir(None);
    let app_data_dir = db_path
        .parent()
        .map(|p| p.join("oclive_api_app_data"))
        .unwrap_or_else(|| std::env::temp_dir().join("oclive_api_app_data"));
    let _ = std::fs::create_dir_all(&app_data_dir);
    let _api_temp_cleanup = ApiTempCleanup {
        db_path: db_path.clone(),
        app_data_dir: app_data_dir.clone(),
    };
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
