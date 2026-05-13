//! 本地 HTTP API（`--api`）：供编写器试聊等工具调用，不经 Tauri IPC。
//!
//! **监听地址**：环境变量 **`OOCP_API_BIND`**（默认 `127.0.0.1`）。容器或局域网暴露可设为 `0.0.0.0`，
//! 并建议同时设置 **`OOCP_API_TOKEN`**，要求 REST 与 OOCP WS 使用 `Authorization: Bearer <token>`。
//!
//! `POST /chat` 成功响应在扁平化的 `SendMessageResponse` 字段之外另含 **`personality_source`**
//!（与包内 `settings.json` → `evolution.personality_source` 一致：`vector` | `profile`），便于试聊工具区分人格模式。
//!
//! 另含角色反馈 REST：`/role-feedback` 等（与桌面版一致）。
//!
//! **健康检查**：`GET /health` 默认返回纯文本 `ok`；`GET /health?verbose=true` 返回 JSON（`db` / `roles` / `disk_space` 子检查，各 2s 超时）；`GET /health/db` 仅 `SELECT 1` 验库，供监控专用。

use crate::domain::adapters::oocp_ws;
use crate::domain::chat_engine::process_message;
use crate::error::AppError;
use crate::infrastructure::db::RoleFeedbackRow;
use crate::models::dto::{SendMessageRequest, SendMessageResponse};
use crate::models::role::PersonalitySource;
use crate::state::KernelAppState;
use axum::body::Body;
use axum::extract::Query;
use axum::extract::Request;
use axum::extract::State;
use axum::http::header;
use axum::http::Method;
use axum::http::StatusCode;
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::Json;
use axum::Router;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::task::spawn_blocking;
use tokio::time::timeout;
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

/// `GET /health?verbose=true`：接受 `1` / `true` / `yes` / `on`（不区分大小写）。
#[derive(Debug, Deserialize, Default)]
pub struct HealthQuery {
    #[serde(default)]
    verbose: Option<String>,
}

fn verbose_query_truthy(q: &HealthQuery) -> bool {
    q.verbose
        .as_deref()
        .map(|v| {
            matches!(
                v.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        })
        .unwrap_or(false)
}

const HEALTH_SUBCHECK_TIMEOUT: Duration = Duration::from_secs(2);

#[derive(Debug, Clone, Serialize)]
pub struct HealthChecksJson {
    pub db: String,
    pub roles: String,
    pub disk_space: String,
}

#[derive(Debug, Serialize)]
pub struct HealthVerboseJson {
    pub status: String,
    pub checks: HealthChecksJson,
}

#[cfg(unix)]
fn unix_fs_available_bytes(path: &std::path::Path) -> std::io::Result<u64> {
    use std::ffi::CString;
    use std::os::unix::ffi::OsStrExt;
    let c = CString::new(path.as_os_str().as_bytes())
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?;
    let mut v: libc::statvfs = unsafe { std::mem::zeroed() };
    let rc = unsafe { libc::statvfs(c.as_ptr(), &mut v) };
    if rc != 0 {
        return Err(std::io::Error::last_os_error());
    }
    let bavail = u64::try_from(v.f_bavail).unwrap_or(0);
    let frsize = u64::try_from(v.f_frsize).unwrap_or(0);
    Ok(bavail.saturating_mul(frsize))
}

async fn check_db_ping(state: &KernelAppState) -> String {
    match timeout(HEALTH_SUBCHECK_TIMEOUT, state.db_manager.ping_sqlite()).await {
        Ok(Ok(())) => "ok".to_string(),
        Ok(Err(e)) => format!("error: {}", e),
        Err(_) => "timeout".to_string(),
    }
}

async fn check_roles_readable(roles_root: PathBuf) -> String {
    match timeout(
        HEALTH_SUBCHECK_TIMEOUT,
        spawn_blocking(move || {
            if !roles_root.is_dir() {
                return "error: roles root is not a directory".to_string();
            }
            let rd = std::fs::read_dir(&roles_root);
            match rd {
                Ok(mut it) => {
                    if it.next().is_none() {
                        "error: roles directory is empty".to_string()
                    } else {
                        "ok".to_string()
                    }
                }
                Err(e) => format!("error: {}", e),
            }
        }),
    )
    .await
    {
        Ok(Ok(s)) => s,
        Ok(Err(e)) => format!("error: join {}", e),
        Err(_) => "timeout".to_string(),
    }
}

async fn check_disk_and_app_data_writable(app_data: PathBuf) -> String {
    match timeout(
        HEALTH_SUBCHECK_TIMEOUT,
        spawn_blocking(move || {
            if let Err(e) = std::fs::create_dir_all(&app_data) {
                return format!("error: create_dir_all {}", e);
            }
            #[cfg(unix)]
            {
                const MIN_FREE: u64 = 100 * 1024 * 1024;
                match unix_fs_available_bytes(&app_data) {
                    Ok(free) => {
                        if free < MIN_FREE {
                            return format!(
                                "error: free space {} bytes below {} bytes threshold",
                                free, MIN_FREE
                            );
                        }
                    }
                    Err(e) => return format!("error: statvfs {}", e),
                }
            }
            let probe = app_data.join(".oclive_health_probe");
            if let Err(e) = std::fs::write(&probe, b"ok") {
                return format!("error: probe write {}", e);
            }
            let _ = std::fs::remove_file(&probe);
            "ok".to_string()
        }),
    )
    .await
    {
        Ok(Ok(s)) => s,
        Ok(Err(e)) => format!("error: join {}", e),
        Err(_) => "timeout".to_string(),
    }
}

async fn run_dependency_checks(state: &KernelAppState) -> HealthChecksJson {
    let roles_root = state.storage.roles_dir().to_path_buf();
    let app_data = state.directory_plugins.app_data_dir().to_path_buf();
    let (db, roles, disk_space) = tokio::join!(
        check_db_ping(state),
        check_roles_readable(roles_root),
        check_disk_and_app_data_writable(app_data),
    );
    HealthChecksJson {
        db,
        roles,
        disk_space,
    }
}

fn aggregate_health_status(checks: &HealthChecksJson) -> &'static str {
    let ok = |s: &str| s == "ok";
    if ok(&checks.db) && ok(&checks.roles) && ok(&checks.disk_space) {
        "ok"
    } else {
        "degraded"
    }
}

/// `GET /health`：无 `verbose` 时仍为纯文本 **`ok`**（向后兼容）；`?verbose=true` 时返回 JSON 依赖状态。
async fn health(
    State(state): State<Arc<KernelAppState>>,
    Query(q): Query<HealthQuery>,
) -> Response {
    if !verbose_query_truthy(&q) {
        return (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "text/plain; charset=utf-8")],
            "ok",
        )
            .into_response();
    }
    let checks = run_dependency_checks(state.as_ref()).await;
    let status = aggregate_health_status(&checks).to_string();
    let body = HealthVerboseJson { status, checks };
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json; charset=utf-8")],
        Json(body),
    )
        .into_response()
}

/// `GET /health/db`：仅验证 SQLite（`SELECT 1`），供监控与 `/health?verbose` 解耦。
async fn health_db(State(state): State<Arc<KernelAppState>>) -> Response {
    match timeout(HEALTH_SUBCHECK_TIMEOUT, state.db_manager.ping_sqlite()).await {
        Ok(Ok(())) => (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "text/plain; charset=utf-8")],
            "ok",
        )
            .into_response(),
        Ok(Err(e)) => (
            StatusCode::SERVICE_UNAVAILABLE,
            [(header::CONTENT_TYPE, "text/plain; charset=utf-8")],
            format!("error: {}", e),
        )
            .into_response(),
        Err(_) => (
            StatusCode::SERVICE_UNAVAILABLE,
            [(header::CONTENT_TYPE, "text/plain; charset=utf-8")],
            "timeout",
        )
            .into_response(),
    }
}

fn http_api_bearer_token_from_env() -> Option<String> {
    std::env::var("OOCP_API_TOKEN")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

fn subtle_constant_time_eq(a: &str, b: &str) -> bool {
    let a = a.as_bytes();
    let b = b.as_bytes();
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

/// 当设置了 `OOCP_API_TOKEN` 时，要求 REST 路由携带 `Authorization: Bearer <token>`（与 OOCP WS 一致）。
async fn optional_rest_bearer_middleware(req: Request<Body>, next: Next) -> impl IntoResponse {
    let Some(server_token) = http_api_bearer_token_from_env() else {
        return next.run(req).await;
    };
    let client_token = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(str::trim);
    let ok = match client_token {
        Some(ct) => subtle_constant_time_eq(server_token.as_str(), ct),
        None => false,
    };
    if !ok {
        return (
            axum::http::StatusCode::UNAUTHORIZED,
            [(header::WWW_AUTHENTICATE, "Bearer")],
            "HTTP API 鉴权失败：请提供 Authorization: Bearer <OOCP_API_TOKEN>",
        )
            .into_response();
    }
    next.run(req).await
}

async fn chat(
    State(state): State<Arc<KernelAppState>>,
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

#[derive(Debug, Deserialize)]
pub struct RoleFeedbackPostRequest {
    pub role_id: String,
    #[serde(default)]
    pub session_id: Option<String>,
    #[serde(default)]
    pub mood_tag: Option<String>,
    #[serde(default)]
    pub scene_id: Option<String>,
    #[serde(default)]
    pub presence_mode: Option<String>,
    #[serde(default)]
    pub role_version: Option<String>,
    #[serde(default)]
    pub client_version: Option<String>,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct RoleFeedbackPostResponse {
    pub id: i64,
}

async fn post_role_feedback(
    State(state): State<Arc<KernelAppState>>,
    Json(body): Json<RoleFeedbackPostRequest>,
) -> Result<Json<RoleFeedbackPostResponse>, ApiError> {
    let rid = body.role_id.trim().to_string();
    let msg = body.message.trim().to_string();
    if rid.is_empty() {
        return Err(api_error(
            axum::http::StatusCode::BAD_REQUEST,
            "invalid_role_id",
            "role_id 不能为空",
            Some("请传入角色 manifest.id".to_string()),
        ));
    }
    if msg.is_empty() {
        return Err(api_error(
            axum::http::StatusCode::BAD_REQUEST,
            "empty_message",
            "message 不能为空",
            Some("请至少输入 1 个可见字符".to_string()),
        ));
    }
    let sid = body.session_id.as_deref();
    let mood = body.mood_tag.as_deref();
    let runtime_version = env!("CARGO_PKG_VERSION");

    let id = state
        .db_manager
        .insert_role_feedback(
            &rid,
            sid,
            mood,
            &msg,
            body.scene_id.as_deref(),
            body.presence_mode.as_deref(),
            body.role_version.as_deref(),
            Some(runtime_version),
            body.client_version.as_deref(),
            Some("http_api"),
        )
        .await
        .map_err(|e: AppError| {
            api_error(
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "role_feedback_failed",
                e.to_frontend_error(),
                None,
            )
        })?;

    Ok(Json(RoleFeedbackPostResponse { id }))
}

#[derive(Debug, Deserialize)]
pub struct RoleFeedbackListQuery {
    pub role_id: String,
    #[serde(default)]
    pub limit: Option<i64>,
    #[serde(default)]
    pub offset: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct RoleFeedbackListResponse {
    pub items: Vec<RoleFeedbackItem>,
}

#[derive(Debug, Serialize)]
pub struct RoleFeedbackItem {
    pub id: i64,
    pub role_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mood_tag: Option<String>,
    pub message: String,
    pub created_at: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub read_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub handled_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub handled_note: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scene_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presence_mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runtime_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

impl From<RoleFeedbackRow> for RoleFeedbackItem {
    fn from(r: RoleFeedbackRow) -> Self {
        Self {
            id: r.id,
            role_id: r.role_id,
            session_id: r.session_id,
            mood_tag: r.mood_tag,
            message: r.message,
            created_at: r.created_at,
            status: r.status,
            read_at: r.read_at,
            handled_at: r.handled_at,
            handled_note: r.handled_note,
            scene_id: r.scene_id,
            presence_mode: r.presence_mode,
            role_version: r.role_version,
            runtime_version: r.runtime_version,
            client_version: r.client_version,
            source: r.source,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct RoleFeedbackMarkReadRequest {
    pub role_id: String,
    pub ids: Vec<i64>,
}

#[derive(Debug, Serialize)]
pub struct RoleFeedbackMarkReadResponse {
    pub updated: i64,
}

async fn mark_role_feedback_read(
    State(state): State<Arc<KernelAppState>>,
    Json(body): Json<RoleFeedbackMarkReadRequest>,
) -> Result<Json<RoleFeedbackMarkReadResponse>, ApiError> {
    let rid = body.role_id.trim().to_string();
    if rid.is_empty() {
        return Err(api_error(
            axum::http::StatusCode::BAD_REQUEST,
            "invalid_role_id",
            "role_id 不能为空",
            None,
        ));
    }
    let updated = state
        .db_manager
        .mark_role_feedback_read(&rid, &body.ids)
        .await
        .map_err(|e: AppError| {
            api_error(
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "role_feedback_failed",
                e.to_frontend_error(),
                None,
            )
        })?;
    Ok(Json(RoleFeedbackMarkReadResponse { updated }))
}

#[derive(Debug, Deserialize)]
pub struct RoleFeedbackSetHandledRequest {
    pub role_id: String,
    pub id: i64,
    pub handled: bool,
    #[serde(default)]
    pub note: Option<String>,
}

async fn set_role_feedback_handled(
    State(state): State<Arc<KernelAppState>>,
    Json(body): Json<RoleFeedbackSetHandledRequest>,
) -> Result<&'static str, ApiError> {
    let rid = body.role_id.trim().to_string();
    if rid.is_empty() || body.id <= 0 {
        return Err(api_error(
            axum::http::StatusCode::BAD_REQUEST,
            "invalid_parameter",
            "role_id 与 id 必填",
            None,
        ));
    }
    state
        .db_manager
        .set_role_feedback_handled(&rid, body.id, body.handled, body.note.as_deref())
        .await
        .map_err(|e: AppError| {
            api_error(
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "role_feedback_failed",
                e.to_frontend_error(),
                None,
            )
        })?;
    Ok("ok")
}

async fn list_role_feedback(
    State(state): State<Arc<KernelAppState>>,
    Query(q): Query<RoleFeedbackListQuery>,
) -> Result<Json<RoleFeedbackListResponse>, ApiError> {
    let rid = q.role_id.trim().to_string();
    if rid.is_empty() {
        return Err(api_error(
            axum::http::StatusCode::BAD_REQUEST,
            "invalid_role_id",
            "role_id 不能为空",
            None,
        ));
    }
    let limit = q.limit.unwrap_or(50);
    let offset = q.offset.unwrap_or(0);
    let rows = state
        .db_manager
        .list_role_feedback(&rid, limit, offset)
        .await
        .map_err(|e: AppError| {
            api_error(
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "role_feedback_failed",
                e.to_frontend_error(),
                None,
            )
        })?;
    Ok(Json(RoleFeedbackListResponse {
        items: rows.into_iter().map(RoleFeedbackItem::from).collect(),
    }))
}

/// 与 [`serve_api`] 相同的路由树，供集成测试 `tower::ServiceExt::oneshot` 使用（无需绑端口）。
/// 已合并 OOCP WebSocket 路由（`/oocp`）。
pub fn api_router(app_state: Arc<KernelAppState>) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers(Any);

    let rest = Router::new()
        .route("/chat", post(chat))
        .route(
            "/role-feedback",
            post(post_role_feedback).get(list_role_feedback),
        )
        .route("/role-feedback/mark-read", post(mark_role_feedback_read))
        .route(
            "/role-feedback/set-handled",
            post(set_role_feedback_handled),
        )
        .layer(middleware::from_fn(optional_rest_bearer_middleware))
        .with_state(app_state.clone());

    Router::new()
        .route("/health/db", get(health_db))
        .route("/health", get(health))
        .merge(rest)
        .merge(oocp_ws::oocp_ws_router())
        .layer(cors)
        .with_state(app_state)
}

#[derive(Debug, Clone)]
pub struct ApiServerOptions {
    /// 监听 IP（或可被 [`std::net::ToSocketAddrs`] 解析的主机名），默认 `127.0.0.1`。
    /// 环境变量：`OOCP_API_BIND`。
    pub bind: String,
    pub port: u16,
    pub db_path: PathBuf,
    pub roles_dir: PathBuf,
    pub app_data_dir: PathBuf,
}

impl ApiServerOptions {
    pub fn from_env_or_defaults(port: u16) -> Self {
        let bind = std::env::var("OOCP_API_BIND")
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "127.0.0.1".to_string());

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
            bind,
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
    if let Err(e) = tokio::fs::create_dir_all(&opt.app_data_dir).await {
        log::warn!(
            target: "oclive_api",
            "create_dir_all app_data_dir={} err={}",
            opt.app_data_dir.display(),
            e
        );
    }
    let app_state = KernelAppState::new(&opt.db_path, Some(opt.roles_dir), &opt.app_data_dir)
        .await
        .map_err(|e| e.to_frontend_error())?;
    let app_state = Arc::new(app_state);

    let app = api_router(app_state);

    let addr = format!("{}:{}", opt.bind, opt.port);
    let listener = TcpListener::bind(addr.as_str())
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
    use crate::models::role::PersonalitySource as Ps;

    #[test]
    fn personality_source_json_matches_http_contract() {
        let v = serde_json::to_value(Ps::Vector).unwrap();
        let p = serde_json::to_value(Ps::Profile).unwrap();
        assert_eq!(v, "vector");
        assert_eq!(p, "profile");
    }

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
