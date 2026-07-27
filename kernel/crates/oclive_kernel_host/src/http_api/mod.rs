//! Local HTTP API (`--api`): for tools such as the pack editor's try-chat to call, bypassing Tauri IPC.
//!
//! Binds only to `127.0.0.1`; evaluate the exposure surface yourself for production environments.
//!
//! Beyond the flattened `SendMessageResponse` fields, a successful `POST /chat` response also includes **`personality_source`**
//! (consistent with the pack's `settings.json` → `evolution.personality_source`: `vector` | `profile`), so try-chat tools can distinguish personality modes.
//!
//! **Error body**: `{ "error": KernelErrorBody }` shares the same source as the Tauri `invoke` failure string (see `oclive_kernel_types::KernelErrorBody`);
//! `code` is consistent with [`AppError::code`] (`SCREAMING_SNAKE_CASE`); HTTP-specific errors use [`oclive_kernel_types::http_chat_codes`] constants (same source as the kernel crate, avoiding literal drift).

mod bridge;
mod chat;
mod health;
mod llm;
mod mcp;
mod role;
mod theater;

#[cfg(test)]
mod tests;

use crate::infrastructure::MockLlmClient;
use crate::models::dto::SendMessageResponse;
use crate::models::role::PersonalitySource;
use crate::state::AppState;
use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use axum::middleware::{self, Next};
use axum::response::Response;
use axum::routing::{get, post};
use axum::Router;
use oclive_kernel_runtime::{
    ensure_app_data_dir, find_app_data_dir_for_api, find_db_path, temp_api_db_path, AppDataMode,
};
use oclive_kernel_types::KernelErrorBody;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::cors::{AllowOrigin, CorsLayer};

/// Environment variable used by headless API hosts to provide API-token authentication.
pub const ENV_API_TOKEN: &str = "OCLIVE_API_TOKEN";
/// Explicit local-development escape hatch for running protected routes without a token.
pub const ENV_API_ALLOW_UNAUTHENTICATED: &str = "OCLIVE_API_ALLOW_UNAUTHENTICATED";
const API_TOKEN_HEADER: &str = "x-oclive-api-token";

#[derive(Debug, Deserialize)]
pub struct ChatApiRequest {
    pub role_path: String,
    pub message: String,
    #[serde(default)]
    pub session_id: Option<String>,
    /// Optional: consistent with the main app's `send_message`; if omitted, the engine infers it from session state.
    #[serde(default)]
    pub scene_id: Option<String>,
    #[serde(default)]
    pub include_raw_reply: Option<bool>,
    #[serde(default)]
    pub adult: Option<crate::models::dto::AdultInteractionRequest>,
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

pub(crate) type ApiError = (axum::http::StatusCode, axum::Json<ApiErrorResponse>);

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

pub(crate) fn api_error(status: axum::http::StatusCode, error: KernelErrorBody) -> ApiError {
    (status, axum::Json(ApiErrorResponse { error }))
}

#[must_use]
pub(crate) fn kernel_http_error(
    code: &str,
    message: impl Into<String>,
    hint: Option<String>,
) -> KernelErrorBody {
    KernelErrorBody {
        code: code.to_string(),
        message: message.into(),
        hint,
        context: None,
    }
}

/// The same route tree as [`serve_api`], for integration tests to use via `tower::ServiceExt::oneshot` (no port binding required).
pub fn api_router(app_state: Arc<AppState>) -> Router {
    api_router_with_auth(app_state, None)
}

/// Build the HTTP API route tree with optional bearer-like header authentication.
///
/// `/health` intentionally remains public so supervisors can probe readiness without
/// needing to know the application token. All other routes are protected when a
/// non-empty token is supplied.
pub fn api_router_with_auth(app_state: Arc<AppState>, api_token: Option<String>) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::predicate(|origin, _| {
            is_allowed_api_origin(origin)
        }))
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers([
            axum::http::header::ACCEPT,
            axum::http::header::CONTENT_TYPE,
            axum::http::HeaderName::from_static(API_TOKEN_HEADER),
        ]);

    let protected = Router::new()
        .route("/chat", post(chat::chat))
        .route("/chat/stream", post(chat::chat_stream))
        .route(
            "/chat/adult-stage/begin",
            post(chat::begin_adult_stage_route),
        )
        .route(
            "/chat/adult-stage/beat",
            post(chat::generate_adult_staged_beat_route),
        )
        .route(
            "/chat/adult-stage/commit",
            post(chat::commit_adult_staged_beat_route),
        )
        .route(
            "/chat/adult-stage/cancel",
            post(chat::cancel_adult_stage_route),
        )
        .route(
            "/chat/adult-stage/list",
            post(chat::list_adult_staged_beats_route),
        )
        .route("/role_info", get(role::role_info_route))
        .route("/role_snapshot", get(role::role_snapshot_route))
        .route("/display_metrics", get(role::display_metrics_route))
        .route("/role/load", post(role::load_role_route))
        .route(
            "/role/interaction_mode",
            post(role::set_role_interaction_mode_route),
        )
        .route("/chat/sessions", get(chat::chat_sessions_route))
        .route("/chat/messages", get(chat::chat_messages_route))
        .route("/chat/storage", post(chat::chat_storage_proxy_route))
        .route("/time/state", get(role::time_state_route))
        .route("/time/jump", post(role::jump_time_route))
        .route("/scene/switch", post(role::switch_scene_route))
        .route("/user_identity/set", post(role::set_user_identity_route))
        .route(
            "/user_identity/scene_set",
            post(role::set_scene_user_identity_route),
        )
        .route(
            "/user_identity/state",
            get(role::get_user_identity_state_route),
        )
        .route(
            "/scene/user_presence",
            post(role::set_user_presence_scene_route),
        )
        .route("/event/create", post(role::create_event_route))
        .route(
            "/high_risk/grants",
            get(bridge::list_high_risk_grants_route),
        )
        .route("/high_risk/grant", post(bridge::grant_high_risk_route))
        .route("/high_risk/revoke", post(bridge::revoke_high_risk_route))
        .route("/mcp/servers", get(mcp::list_mcp_servers_route))
        .route("/mcp/tools", get(mcp::list_mcp_tools_route))
        .route("/mcp/call", post(mcp::call_mcp_tool_route))
        .route("/bridge/dispatch", post(bridge::bridge_dispatch_route))
        .route("/llm/reload", post(llm::llm_reload_route))
        .route(
            "/llm/user_settings",
            get(llm::llm_user_settings_get_route).post(llm::llm_user_settings_post_route),
        )
        .route("/llm/probe_cloud", post(llm::llm_probe_cloud_route))
        .route("/llm/lora/activate", post(llm::llm_lora_activate_route))
        .route("/llm/lora/delete", post(llm::llm_lora_delete_route))
        .route("/llm/ollama_models", get(llm::llm_ollama_models_route))
        .route("/llm/cloud_models", post(llm::llm_cloud_models_route))
        .route("/llm/session_model", post(llm::llm_session_model_route))
        .route(
            "/llm/global_ollama_model",
            get(llm::llm_global_ollama_model_get_route)
                .post(llm::llm_global_ollama_model_post_route),
        )
        .route("/theater/scene", post(theater::scene_route))
        .layer(middleware::from_fn(
            move |request: Request<Body>, next: Next| {
                let expected = api_token.clone();
                async move { authorize_api_request(expected.as_deref(), request, next).await }
            },
        ));

    Router::new()
        .route("/health", get(health::health_route))
        .merge(protected)
        .layer(cors)
        .with_state(app_state)
}

fn is_allowed_api_origin(origin: &axum::http::HeaderValue) -> bool {
    let Ok(origin) = origin.to_str() else {
        return false;
    };
    if matches!(
        origin,
        "https://tauri.localhost" | "https://ocliveplugin.localhost"
    ) {
        return true;
    }
    let Ok(url) = reqwest::Url::parse(origin) else {
        return false;
    };
    if !matches!(url.scheme(), "http" | "https" | "tauri") {
        return false;
    }
    crate::infrastructure::is_loopback_endpoint(origin)
}

fn validate_api_auth_configuration(
    api_token: Option<&str>,
    allow_unauthenticated: bool,
) -> Result<(), String> {
    if api_token.is_some_and(|token| !token.trim().is_empty()) || allow_unauthenticated {
        return Ok(());
    }
    Err(format!(
        "{ENV_API_TOKEN} is required for HTTP API protected routes; set a strong random token, or explicitly set {ENV_API_ALLOW_UNAUTHENTICATED}=1 for isolated local development"
    ))
}

async fn authorize_api_request(
    expected: Option<&str>,
    request: Request<Body>,
    next: Next,
) -> Response {
    if request.method() == Method::OPTIONS {
        return next.run(request).await;
    }
    let Some(expected) = expected.filter(|token| !token.trim().is_empty()) else {
        return next.run(request).await;
    };

    let actual = request
        .headers()
        .get(API_TOKEN_HEADER)
        .and_then(|value| value.to_str().ok());
    if actual == Some(expected) {
        return next.run(request).await;
    }

    Response::builder()
        .status(StatusCode::UNAUTHORIZED)
        .header("www-authenticate", "OCLive-API-Token")
        .body(Body::from("missing or invalid x-oclive-api-token"))
        .unwrap_or_else(|_| Response::new(Body::from("unauthorized")))
}

/// Build [`AppState`] for headless `--api` / `oclive-kernel-server` (single DB writer).
///
/// # Errors
///
/// Returns a human-readable message on migration or DB bootstrap failure.
pub async fn build_api_app_state(port: u16) -> Result<Arc<AppState>, String> {
    let roles_dir = crate::state::find_roles_dir(None);
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
        crate::domain::startup_health::run_global_db_ping(
            &crate::infrastructure::db_ports::DbHealthPortAdapter(app_state.db_manager.as_ref()),
        )
        .await
        .map_err(|e| e.to_string())?;
        return Ok(Arc::new(app_state));
    }

    let (app_data_dir, mode) = find_app_data_dir_for_api(port);
    ensure_app_data_dir(&app_data_dir)?;
    if mode == AppDataMode::Persistent {
        crate::infrastructure::app_data_migration::ensure_canonical_app_data_ready(&app_data_dir)?;
    }
    let db_path = if mode == AppDataMode::Temp {
        temp_api_db_path(port)
    } else {
        find_db_path(&app_data_dir)
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
    crate::domain::startup_health::run_global_db_ping(
        &crate::infrastructure::db_ports::DbHealthPortAdapter(app_state.db_manager.as_ref()),
    )
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
    let api_token = std::env::var(ENV_API_TOKEN)
        .ok()
        .filter(|token| !token.trim().is_empty());
    let allow_unauthenticated = std::env::var(ENV_API_ALLOW_UNAUTHENTICATED)
        .ok()
        .is_some_and(|value| value == "1" || value.eq_ignore_ascii_case("true"));
    validate_api_auth_configuration(api_token.as_deref(), allow_unauthenticated)?;
    if allow_unauthenticated && api_token.is_none() {
        tracing::warn!(
            target: "oclive_api",
            "{} explicitly enabled; HTTP API protected routes are unauthenticated",
            ENV_API_ALLOW_UNAUTHENTICATED
        );
    }
    let app = api_router_with_auth(app_state, api_token);
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
    let (app_data_dir, mode) = find_app_data_dir_for_api(port);
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
