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
use crate::infrastructure::chat_storage::{SessionMeta, StoredMessage};
use crate::infrastructure::MockLlmClient;
use crate::models::dto::{
    CreateEventRequest, CreateEventResponse, SetUserPresenceSceneRequest, SwitchSceneRequest,
    SwitchSceneResponse,
};
use crate::models::dto::{
    GetRoleInfoRequest, JumpTimeRequest, JumpTimeResponse, RoleInfo, TimeStateResponse,
};
use crate::models::dto::{SendMessageRequest, SendMessageResponse};
use crate::models::role::PersonalitySource;
use crate::service::{
    dispatch_bridge_command, execute_chat_storage_proxy, get_role_info_impl, get_time_state_impl,
    grant_high_risk_capability_impl, jump_time_impl, list_high_risk_grants_impl, load_role_impl,
    revoke_high_risk_capability_impl, set_user_presence_scene_impl, switch_scene_impl,
    ChatStorageProxyOp, MutateHighRiskGrantRequest,
};
use crate::state::AppState;
use axum::extract::{Query, State};
use axum::http::HeaderMap;
use axum::http::Method;
use axum::routing::{get, post};
use axum::Json;
use axum::Router;
use oclive_kernel_runtime::{
    ensure_app_data_dir, http_chat_codes, resolve_app_data_dir_for_api, resolve_db_path,
    temp_api_db_path, AppDataMode, KernelErrorBody,
};
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

#[derive(Debug, Serialize)]
struct HealthJson {
    ok: bool,
    runtime_api_version: &'static str,
    schema_migration_version: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    kernel_manifest: Option<oclive_kernel_runtime::KernelBinaryManifest>,
}

async fn health(State(state): State<Arc<AppState>>) -> axum::response::Response {
    use axum::http::header::CONTENT_TYPE;
    use axum::http::StatusCode;
    use axum::response::IntoResponse;
    use oclive_kernel_runtime::RUNTIME_API_VERSION;

    let version = state
        .db_manager
        .max_applied_migration_version()
        .await
        .ok()
        .flatten();
    let json = HealthJson {
        ok: true,
        runtime_api_version: RUNTIME_API_VERSION,
        schema_migration_version: version,
        kernel_manifest: Some(oclive_kernel_runtime::KernelBinaryManifest::from_compile_time_env()),
    };
    (
        StatusCode::OK,
        [(CONTENT_TYPE, "application/json")],
        axum::Json(json),
    )
        .into_response()
}

async fn health_plain() -> &'static str {
    "ok"
}

async fn health_route(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
) -> axum::response::Response {
    use axum::http::header::ACCEPT;
    use axum::response::IntoResponse;

    let wants_json = headers
        .get(ACCEPT)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|a| a.contains("application/json"))
        || std::env::var("OCLIVE_HEALTH_JSON")
            .ok()
            .is_some_and(|v| v == "1" || v.eq_ignore_ascii_case("true"));
    if wants_json {
        health(State(state)).await
    } else {
        (axum::http::StatusCode::OK, health_plain().await).into_response()
    }
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
        .http_api_roles
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

#[derive(Debug, Deserialize)]
struct RoleIdQuery {
    role_id: String,
    session_id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RoleSnapshotQuery {
    role_id: String,
    #[serde(default)]
    scene_id: Option<String>,
}

#[derive(Debug, Serialize)]
struct RoleSnapshotResponse {
    role_id: String,
    current_favorability: f64,
    current_emotion: String,
    portrait_emotion: String,
    relation_state: String,
    personality_source: PersonalitySource,
    current_scene: Option<String>,
    user_presence_scene: Option<String>,
}

#[derive(Debug, Deserialize)]
struct LoadRoleBody {
    role_id: String,
}

#[derive(Debug, Deserialize)]
struct ChatSessionsQuery {
    role_id: String,
    scene_id: String,
    limit: Option<u32>,
    offset: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct ChatMessagesQuery {
    session_id: String,
    limit: Option<u32>,
    offset: Option<u32>,
}

async fn role_info_route(
    State(state): State<Arc<AppState>>,
    Query(q): Query<RoleIdQuery>,
) -> Result<Json<RoleInfo>, ApiError> {
    let req = GetRoleInfoRequest {
        role_id: q.role_id.trim().to_string(),
        session_id: q.session_id,
    };
    get_role_info_impl(&state, &req.role_id, req.session_id.as_deref())
        .await
        .map(Json)
        .map_err(|e| {
            let k = e.kernel_error_body();
            api_error(axum::http::StatusCode::BAD_REQUEST, k)
        })
}

async fn role_snapshot_route(
    State(state): State<Arc<AppState>>,
    Query(q): Query<RoleSnapshotQuery>,
) -> Result<Json<RoleSnapshotResponse>, ApiError> {
    let role_id = q.role_id.trim();
    let info = get_role_info_impl(&state, role_id, None)
        .await
        .map_err(|e| {
            let k = e.kernel_error_body();
            api_error(axum::http::StatusCode::BAD_REQUEST, k)
        })?;
    let _scene = q.scene_id.as_deref();
    Ok(Json(RoleSnapshotResponse {
        role_id: info.role_id,
        current_favorability: info.current_favorability,
        current_emotion: info.current_emotion.clone(),
        portrait_emotion: info.current_emotion,
        relation_state: info.relation_state,
        personality_source: info.personality_source,
        current_scene: info.current_scene,
        user_presence_scene: info.user_presence_scene,
    }))
}

async fn load_role_route(
    State(state): State<Arc<AppState>>,
    Json(body): Json<LoadRoleBody>,
) -> Result<axum::http::StatusCode, ApiError> {
    load_role_impl(&state, body.role_id.trim(), false)
        .await
        .map(|_| axum::http::StatusCode::NO_CONTENT)
        .map_err(|e| {
            let k = e.kernel_error_body();
            api_error(axum::http::StatusCode::BAD_REQUEST, k)
        })
}

async fn chat_sessions_route(
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

async fn chat_messages_route(
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

async fn time_state_route(
    State(state): State<Arc<AppState>>,
    Query(q): Query<RoleIdQuery>,
) -> Result<Json<TimeStateResponse>, ApiError> {
    get_time_state_impl(&state, q.role_id.trim())
        .await
        .map(Json)
        .map_err(|e| {
            let k = e.kernel_error_body();
            api_error(axum::http::StatusCode::BAD_REQUEST, k)
        })
}

async fn jump_time_route(
    State(state): State<Arc<AppState>>,
    Json(req): Json<JumpTimeRequest>,
) -> Result<Json<JumpTimeResponse>, ApiError> {
    jump_time_impl(&state, &req).await.map(Json).map_err(|e| {
        let k = e.kernel_error_body();
        api_error(axum::http::StatusCode::BAD_REQUEST, k)
    })
}

async fn switch_scene_route(
    State(state): State<Arc<AppState>>,
    Json(req): Json<SwitchSceneRequest>,
) -> Result<Json<SwitchSceneResponse>, ApiError> {
    switch_scene_impl(&state, &req)
        .await
        .map(Json)
        .map_err(|e| {
            let k = e.kernel_error_body();
            api_error(axum::http::StatusCode::BAD_REQUEST, k)
        })
}

async fn set_user_presence_scene_route(
    State(state): State<Arc<AppState>>,
    Json(req): Json<SetUserPresenceSceneRequest>,
) -> Result<Json<crate::models::dto::RoleInfo>, ApiError> {
    set_user_presence_scene_impl(&state, &req)
        .await
        .map(Json)
        .map_err(|e| {
            let k = e.kernel_error_body();
            api_error(axum::http::StatusCode::BAD_REQUEST, k)
        })
}

async fn create_event_route(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateEventRequest>,
) -> Result<Json<CreateEventResponse>, ApiError> {
    crate::service::plugin_bridge::create_event_impl(&state, &req)
        .await
        .map(Json)
        .map_err(|e| {
            let k = e.kernel_error_body();
            api_error(axum::http::StatusCode::BAD_REQUEST, k)
        })
}

async fn list_high_risk_grants_route(
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, ApiError> {
    list_high_risk_grants_impl(&state).map(Json).map_err(|e| {
        let k = e.kernel_error_body();
        api_error(axum::http::StatusCode::INTERNAL_SERVER_ERROR, k)
    })
}

async fn grant_high_risk_route(
    State(state): State<Arc<AppState>>,
    Json(req): Json<MutateHighRiskGrantRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    grant_high_risk_capability_impl(&state, &req)
        .map(|_| Json(serde_json::json!({ "ok": true })))
        .map_err(|e| {
            let k = e.kernel_error_body();
            api_error(axum::http::StatusCode::BAD_REQUEST, k)
        })
}

async fn revoke_high_risk_route(
    State(state): State<Arc<AppState>>,
    Json(req): Json<MutateHighRiskGrantRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    revoke_high_risk_capability_impl(&state, &req)
        .map(|_| Json(serde_json::json!({ "ok": true })))
        .map_err(|e| {
            let k = e.kernel_error_body();
            api_error(axum::http::StatusCode::BAD_REQUEST, k)
        })
}

#[derive(Debug, Deserialize)]
struct BridgeDispatchRequest {
    command: String,
    #[serde(default)]
    params: serde_json::Value,
}

const BRIDGE_TOKEN_HEADER: &str = "x-oclive-bridge-token";

fn bridge_dispatch_authorized(headers: &HeaderMap) -> bool {
    let Some(expected) = std::env::var("OCLIVE_BRIDGE_TOKEN")
        .ok()
        .filter(|s| !s.trim().is_empty())
    else {
        return true;
    };
    headers
        .get(BRIDGE_TOKEN_HEADER)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|t| t == expected)
}

async fn bridge_dispatch_route(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<BridgeDispatchRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    if !bridge_dispatch_authorized(&headers) {
        let k = kernel_http_error(
            "INVALID_PARAMETER",
            "bridge dispatch: missing or invalid x-oclive-bridge-token",
            Some("Set OCLIVE_BRIDGE_TOKEN on kernel and pass the same value in the header.".into()),
        );
        return Err(api_error(axum::http::StatusCode::UNAUTHORIZED, k));
    }
    let cmd = req.command.trim();
    if cmd.is_empty() {
        let k = KernelErrorBody {
            code: "INVALID_PARAMETER".to_string(),
            message: "bridge dispatch: command required".into(),
            hint: None,
        };
        return Err(api_error(axum::http::StatusCode::BAD_REQUEST, k));
    }
    dispatch_bridge_command(state.as_ref(), cmd, req.params)
        .await
        .map(Json)
        .map_err(|e| {
            let k = e.kernel_error_body();
            api_error(axum::http::StatusCode::BAD_REQUEST, k)
        })
}

#[derive(Serialize)]
struct LlmReloadResponse {
    ok: bool,
    provider: String,
}

async fn chat_storage_proxy_route(
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

async fn llm_reload_route(
    State(state): State<Arc<AppState>>,
) -> Result<Json<LlmReloadResponse>, ApiError> {
    state.mark_user_llm_env_dirty();
    crate::domain::user_llm_env::apply_user_llm_env(state.as_ref())
        .await
        .map_err(|e| {
            let k = e.kernel_error_body();
            api_error(axum::http::StatusCode::INTERNAL_SERVER_ERROR, k)
        })?;
    let provider = state.user_llm_provider.read().clone();
    Ok(Json(LlmReloadResponse { ok: true, provider }))
}

/// The same route tree as [`serve_api`], for integration tests to use via `tower::ServiceExt::oneshot` (no port binding required).
pub fn api_router(app_state: Arc<AppState>) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers(Any);

    Router::new()
        .route("/health", get(health_route))
        .route("/chat", post(chat))
        .route("/role_info", get(role_info_route))
        .route("/role_snapshot", get(role_snapshot_route))
        .route("/role/load", post(load_role_route))
        .route("/chat/sessions", get(chat_sessions_route))
        .route("/chat/messages", get(chat_messages_route))
        .route("/chat/storage", post(chat_storage_proxy_route))
        .route("/time/state", get(time_state_route))
        .route("/time/jump", post(jump_time_route))
        .route("/scene/switch", post(switch_scene_route))
        .route("/scene/user_presence", post(set_user_presence_scene_route))
        .route("/event/create", post(create_event_route))
        .route("/high_risk/grants", get(list_high_risk_grants_route))
        .route("/high_risk/grant", post(grant_high_risk_route))
        .route("/high_risk/revoke", post(revoke_high_risk_route))
        .route("/bridge/dispatch", post(bridge_dispatch_route))
        .route("/llm/reload", post(llm_reload_route))
        .layer(cors)
        .with_state(app_state)
}
/// Build [`AppState`] for headless `--api` / `oclive-kernel-server` (single DB writer).
///
/// # Errors
///
/// Returns a human-readable message on migration or DB bootstrap failure.
pub async fn build_api_app_state(port: u16) -> Result<Arc<AppState>, String> {
    let roles_dir = crate::state::resolve_roles_dir(None);
    let mock_llm = std::env::var("OCLIVE_HTTP_API_MOCK_LLM")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);

    if mock_llm {
        tracing::warn!(target: "oclive_api", "OCLIVE_HTTP_API_MOCK_LLM enabled: using in-memory DB + mock LLM");
        let llm = Arc::new(MockLlmClient {
            reply: "OOCP mock reply".to_string(),
        });
        let app_state = AppState::new_in_memory_with_llm(llm, roles_dir)
            .await
            .map_err(|e| e.to_string())?;
        crate::domain::startup_health::run_global_db_ping(&app_state.db_manager)
            .await
            .map_err(|e| e.to_string())?;
        return Ok(Arc::new(app_state));
    }

    let (app_data_dir, mode) = resolve_app_data_dir_for_api(port);
    ensure_app_data_dir(&app_data_dir)?;
    if mode == AppDataMode::Persistent {
        crate::infrastructure::app_data_migration::ensure_canonical_app_data_ready(&app_data_dir)?;
    }
    let db_path = if mode == AppDataMode::Temp {
        temp_api_db_path(port)
    } else {
        resolve_db_path(&app_data_dir)
    };
    tracing::info!(
        target: "oclive_api",
        app_data = %app_data_dir.display(),
        db = %db_path.display(),
        mode = ?mode,
        "resolved HTTP API app data"
    );
    let app_state = AppState::new(&db_path, Some(roles_dir), &app_data_dir)
        .await
        .map_err(|e| e.to_string())?;
    crate::domain::startup_health::run_global_db_ping(&app_state.db_manager)
        .await
        .map_err(|e| e.to_string())?;
    Ok(Arc::new(app_state))
}

/// Bind `api_router` on `127.0.0.1:port` until the process exits.
///
/// # Errors
///
/// Returns [`Err`] when listen or serve fails.
pub async fn serve_api_with_state(app_state: Arc<AppState>, port: u16) -> Result<(), String> {
    let plugins = Arc::clone(&app_state.directory_plugins);
    let app = api_router(app_state);
    let addr = format!("127.0.0.1:{port}");
    let listener = TcpListener::bind(&addr)
        .await
        .map_err(|e| format!("绑定 {} 失败：{}", addr, e))?;
    tracing::info!(target: "oclive_api", "HTTP API listening http://{addr}");
    axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            if tokio::signal::ctrl_c().await.is_ok() {
                tracing::info!(target: "oclive_api", "shutdown signal received");
            }
            plugins.shutdown_all();
        })
        .await
        .map_err(|e| format!("HTTP 服务异常：{e}"))?;
    Ok(())
}

/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
/// Runs the HTTP service in a blocking manner until the process exits.
///
/// CI / protocol black-box: when `OCLIVE_HTTP_API_MOCK_LLM=1` is set, uses an in-memory DB + [`MockLlmClient`], not depending on a local Ollama.
pub async fn serve_api(port: u16) -> Result<(), String> {
    let (app_data_dir, mode) = resolve_app_data_dir_for_api(port);
    let _api_temp_cleanup = if mode == AppDataMode::Temp {
        Some(ApiTempCleanup {
            db_path: temp_api_db_path(port),
            app_data_dir: app_data_dir.clone(),
        })
    } else {
        None
    };
    let app_state = build_api_app_state(port).await?;
    serve_api_with_state(app_state, port).await
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
