//! Remote kernel HTTP client (desktop → `:8420` single writer).

use crate::error::AppError;
use crate::infrastructure::chat_storage::{SessionMeta, StoredMessage};
use crate::kernel_lifecycle::KernelConnection;
use crate::models::dto::{
    CreateEventRequest, CreateEventResponse, GetRoleInfoRequest, JumpTimeRequest, JumpTimeResponse,
    RoleInfo, SendMessageRequest, SendMessageResponse, SetUserPresenceSceneRequest,
    SwitchSceneRequest, SwitchSceneResponse, TimeStateResponse,
};
use crate::state::AppState;
pub(crate) use oclive_kernel_runtime::app_error_from_http_response;
use oclive_kernel_runtime::RUNTIME_API_VERSION;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

const HEALTH_GATE_TTL: Duration = Duration::from_millis(1500);

#[derive(Debug, Deserialize)]
struct HealthProbeJson {
    ok: bool,
    runtime_api_version: String,
}

/// Lightweight UI snapshot from `GET /role_snapshot`.
#[derive(Debug, Clone, Deserialize)]
pub struct RoleSnapshot {
    pub role_id: String,
    pub current_favorability: f64,
    pub current_emotion: String,
    pub portrait_emotion: String,
    pub relation_state: String,
    pub personality_source: String,
    pub current_scene: Option<String>,
    pub user_presence_scene: Option<String>,
}

/// TTL cache for successful `/health` probes on the IPC hot path.
struct HealthGate;

impl HealthGate {
    fn cache() -> &'static Mutex<HashMap<String, Instant>> {
        static GATE: OnceLock<Mutex<HashMap<String, Instant>>> = OnceLock::new();
        GATE.get_or_init(|| Mutex::new(HashMap::new()))
    }

    fn normalize(base_url: &str) -> String {
        base_url.trim_end_matches('/').to_string()
    }

    fn is_fresh(base_url: &str) -> bool {
        let key = Self::normalize(base_url);
        let Ok(guard) = Self::cache().lock() else {
            return false;
        };
        guard
            .get(&key)
            .is_some_and(|t| t.elapsed() < HEALTH_GATE_TTL)
    }

    fn mark_ok(base_url: &str) {
        let key = Self::normalize(base_url);
        if let Ok(mut guard) = Self::cache().lock() {
            guard.insert(key, Instant::now());
        }
    }

    fn invalidate(base_url: &str) {
        let key = Self::normalize(base_url);
        if let Ok(mut guard) = Self::cache().lock() {
            guard.remove(&key);
        }
    }
}

fn probe_http_client() -> &'static reqwest::Client {
    static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .build()
            .unwrap_or_else(|_| reqwest::Client::new())
    })
}

/// HTTP proxy for kernel routes when desktop is a thin client.
pub struct KernelHttpClient;

impl KernelHttpClient {
    pub async fn probe_health(base_url: &str) -> bool {
        Self::probe_health_timeout(base_url, Duration::from_secs(3)).await
    }

    pub async fn probe_health_timeout(base_url: &str, timeout: Duration) -> bool {
        let url = format!("{}/health", base_url.trim_end_matches('/'));
        let Ok(res) = probe_http_client().get(&url).timeout(timeout).send().await else {
            return false;
        };
        if !res.status().is_success() {
            return false;
        }
        let Ok(text) = res.text().await else {
            return false;
        };
        let t = text.trim();
        if t == "ok" {
            return true;
        }
        let Ok(parsed) = serde_json::from_str::<HealthProbeJson>(t) else {
            return t.contains("\"ok\":true") || t.contains("\"ok\": true");
        };
        if !parsed.ok {
            return false;
        }
        if parsed.runtime_api_version != RUNTIME_API_VERSION {
            tracing::warn!(
                target: "oclive_desktop",
                expected = RUNTIME_API_VERSION,
                actual = %parsed.runtime_api_version,
                "kernel health runtime_api_version mismatch"
            );
            return false;
        }
        true
    }

    async fn ensure_healthy(conn: &KernelConnection) -> bool {
        let base = &conn.base_url;
        if HealthGate::is_fresh(base) {
            return true;
        }
        let ok = Self::probe_health_timeout(base, Duration::from_secs(3)).await;
        if ok {
            HealthGate::mark_ok(base);
        }
        ok
    }

    fn map_send_err(base_url: &str, context: &str, e: reqwest::Error) -> AppError {
        HealthGate::invalidate(base_url);
        AppError::OllamaError(format!("{context}: {e}"))
    }

    fn offline_err() -> AppError {
        AppError::KernelOffline
    }

    pub async fn send_message_via_http(
        conn: &KernelConnection,
        role_path: &Path,
        req: &SendMessageRequest,
    ) -> Result<SendMessageResponse, AppError> {
        if !Self::ensure_healthy(conn).await {
            return Err(Self::offline_err());
        }
        let url = format!("{}/chat", conn.base_url);
        let body = serde_json::json!({
            "role_path": role_path.to_string_lossy(),
            "message": req.user_message,
            "session_id": req.session_id,
            "scene_id": req.scene_id,
        });
        let res = conn
            .http_client()
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| Self::map_send_err(&conn.base_url, "remote chat request", e))?;
        let status = res.status();
        let text = res
            .text()
            .await
            .map_err(|e| AppError::OllamaError(format!("remote chat body: {e}")))?;
        if !status.is_success() {
            return Err(app_error_from_http_response(status.as_u16(), &text));
        }
        serde_json::from_str(&text)
            .map_err(|e| AppError::OllamaError(format!("remote chat JSON: {e}")))
    }

    pub async fn get_role_info_via_http(
        conn: &KernelConnection,
        req: &GetRoleInfoRequest,
    ) -> Result<RoleInfo, AppError> {
        if !Self::ensure_healthy(conn).await {
            return Err(Self::offline_err());
        }
        let mut req_builder = conn
            .http_client()
            .get(format!("{}/role_info", conn.base_url))
            .query(&[("role_id", req.role_id.as_str())]);
        if let Some(sid) = req.session_id.as_deref().filter(|s| !s.is_empty()) {
            req_builder = req_builder.query(&[("session_id", sid)]);
        }
        let res = req_builder
            .send()
            .await
            .map_err(|e| Self::map_send_err(&conn.base_url, "role_info request", e))?;
        let status = res.status();
        let text = res
            .text()
            .await
            .map_err(|e| AppError::OllamaError(format!("role_info body: {e}")))?;
        if !status.is_success() {
            return Err(app_error_from_http_response(status.as_u16(), &text));
        }
        serde_json::from_str(&text)
            .map_err(|e| AppError::OllamaError(format!("role_info JSON: {e}")))
    }

    pub async fn get_time_state_via_http(
        conn: &KernelConnection,
        role_id: &str,
    ) -> Result<TimeStateResponse, AppError> {
        if !Self::ensure_healthy(conn).await {
            return Err(Self::offline_err());
        }
        let res = conn
            .http_client()
            .get(format!("{}/time/state", conn.base_url))
            .query(&[("role_id", role_id.trim())])
            .send()
            .await
            .map_err(|e| Self::map_send_err(&conn.base_url, "time/state request", e))?;
        let status = res.status();
        let text = res
            .text()
            .await
            .map_err(|e| AppError::OllamaError(format!("time/state body: {e}")))?;
        if !status.is_success() {
            return Err(app_error_from_http_response(status.as_u16(), &text));
        }
        serde_json::from_str(&text)
            .map_err(|e| AppError::OllamaError(format!("time/state JSON: {e}")))
    }

    pub async fn jump_time_via_http(
        conn: &KernelConnection,
        req: &JumpTimeRequest,
    ) -> Result<JumpTimeResponse, AppError> {
        if !Self::ensure_healthy(conn).await {
            return Err(Self::offline_err());
        }
        let res = conn
            .http_client()
            .post(format!("{}/time/jump", conn.base_url))
            .json(req)
            .send()
            .await
            .map_err(|e| Self::map_send_err(&conn.base_url, "time/jump request", e))?;
        let status = res.status();
        let text = res
            .text()
            .await
            .map_err(|e| AppError::OllamaError(format!("time/jump body: {e}")))?;
        if !status.is_success() {
            return Err(app_error_from_http_response(status.as_u16(), &text));
        }
        serde_json::from_str(&text)
            .map_err(|e| AppError::OllamaError(format!("time/jump JSON: {e}")))
    }

    pub async fn switch_scene_via_http(
        conn: &KernelConnection,
        req: &SwitchSceneRequest,
    ) -> Result<SwitchSceneResponse, AppError> {
        if !Self::ensure_healthy(conn).await {
            return Err(Self::offline_err());
        }
        let res = conn
            .http_client()
            .post(format!("{}/scene/switch", conn.base_url))
            .json(req)
            .send()
            .await
            .map_err(|e| Self::map_send_err(&conn.base_url, "scene/switch request", e))?;
        let status = res.status();
        let text = res
            .text()
            .await
            .map_err(|e| AppError::OllamaError(format!("scene/switch body: {e}")))?;
        if !status.is_success() {
            return Err(app_error_from_http_response(status.as_u16(), &text));
        }
        serde_json::from_str(&text)
            .map_err(|e| AppError::OllamaError(format!("scene/switch JSON: {e}")))
    }

    pub async fn set_user_presence_scene_via_http(
        conn: &KernelConnection,
        req: &SetUserPresenceSceneRequest,
    ) -> Result<RoleInfo, AppError> {
        if !Self::ensure_healthy(conn).await {
            return Err(Self::offline_err());
        }
        let res = conn
            .http_client()
            .post(format!("{}/scene/user_presence", conn.base_url))
            .json(req)
            .send()
            .await
            .map_err(|e| Self::map_send_err(&conn.base_url, "scene/user_presence request", e))?;
        let status = res.status();
        let text = res
            .text()
            .await
            .map_err(|e| AppError::OllamaError(format!("scene/user_presence body: {e}")))?;
        if !status.is_success() {
            return Err(app_error_from_http_response(status.as_u16(), &text));
        }
        serde_json::from_str(&text)
            .map_err(|e| AppError::OllamaError(format!("scene/user_presence JSON: {e}")))
    }

    pub async fn create_event_via_http(
        conn: &KernelConnection,
        req: &CreateEventRequest,
    ) -> Result<CreateEventResponse, AppError> {
        if !Self::ensure_healthy(conn).await {
            return Err(Self::offline_err());
        }
        let res = conn
            .http_client()
            .post(format!("{}/event/create", conn.base_url))
            .json(req)
            .send()
            .await
            .map_err(|e| Self::map_send_err(&conn.base_url, "event/create request", e))?;
        let status = res.status();
        let text = res
            .text()
            .await
            .map_err(|e| AppError::OllamaError(format!("event/create body: {e}")))?;
        if !status.is_success() {
            return Err(app_error_from_http_response(status.as_u16(), &text));
        }
        serde_json::from_str(&text)
            .map_err(|e| AppError::OllamaError(format!("event/create JSON: {e}")))
    }

    pub async fn bridge_dispatch_via_http(
        conn: &KernelConnection,
        command: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, AppError> {
        if !Self::ensure_healthy(conn).await {
            return Err(Self::offline_err());
        }
        let res = conn
            .http_client()
            .post(format!("{}/bridge/dispatch", conn.base_url))
            .json(&serde_json::json!({
                "command": command,
                "params": params,
            }))
            .send()
            .await
            .map_err(|e| Self::map_send_err(&conn.base_url, "bridge/dispatch request", e))?;
        let status = res.status();
        let text = res
            .text()
            .await
            .map_err(|e| AppError::OllamaError(format!("bridge/dispatch body: {e}")))?;
        if !status.is_success() {
            return Err(app_error_from_http_response(status.as_u16(), &text));
        }
        serde_json::from_str(&text)
            .map_err(|e| AppError::OllamaError(format!("bridge/dispatch JSON: {e}")))
    }

    pub async fn load_role_via_http(
        conn: &KernelConnection,
        role_id: &str,
    ) -> Result<(), AppError> {
        if !Self::ensure_healthy(conn).await {
            return Err(Self::offline_err());
        }
        let url = format!("{}/role/load", conn.base_url);
        let res = conn
            .http_client()
            .post(&url)
            .json(&serde_json::json!({ "role_id": role_id }))
            .send()
            .await
            .map_err(|e| Self::map_send_err(&conn.base_url, "role/load request", e))?;
        let status = res.status();
        if status.is_success() {
            Ok(())
        } else {
            let text = res.text().await.unwrap_or_default();
            Err(app_error_from_http_response(status.as_u16(), &text))
        }
    }

    pub async fn role_snapshot_via_http(
        conn: &KernelConnection,
        role_id: &str,
        scene_id: Option<&str>,
    ) -> Result<RoleSnapshot, AppError> {
        if !Self::ensure_healthy(conn).await {
            return Err(Self::offline_err());
        }
        let mut req_builder = conn
            .http_client()
            .get(format!("{}/role_snapshot", conn.base_url))
            .query(&[("role_id", role_id)]);
        if let Some(s) = scene_id.filter(|s| !s.is_empty()) {
            req_builder = req_builder.query(&[("scene_id", s)]);
        }
        let res = req_builder
            .send()
            .await
            .map_err(|e| Self::map_send_err(&conn.base_url, "role_snapshot request", e))?;
        let status = res.status();
        let text = res
            .text()
            .await
            .map_err(|e| AppError::OllamaError(format!("role_snapshot body: {e}")))?;
        if !status.is_success() {
            return Err(app_error_from_http_response(status.as_u16(), &text));
        }
        serde_json::from_str(&text)
            .map_err(|e| AppError::OllamaError(format!("role_snapshot JSON: {e}")))
    }

    pub async fn list_chat_sessions_via_http(
        conn: &KernelConnection,
        role_id: &str,
        scene_id: &str,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<SessionMeta>, AppError> {
        if !Self::ensure_healthy(conn).await {
            return Err(Self::offline_err());
        }
        let res = conn
            .http_client()
            .get(format!("{}/chat/sessions", conn.base_url))
            .query(&[
                ("role_id", role_id),
                ("scene_id", scene_id),
                ("limit", &limit.to_string()),
                ("offset", &offset.to_string()),
            ])
            .send()
            .await
            .map_err(|e| Self::map_send_err(&conn.base_url, "chat/sessions request", e))?;
        let status = res.status();
        let text = res
            .text()
            .await
            .map_err(|e| AppError::OllamaError(format!("chat/sessions body: {e}")))?;
        if !status.is_success() {
            return Err(AppError::OllamaError(format!(
                "chat/sessions HTTP {status}: {text}"
            )));
        }
        serde_json::from_str(&text)
            .map_err(|e| AppError::OllamaError(format!("chat/sessions JSON: {e}")))
    }

    /// Tell the kernel process to re-read LLM settings from canonical DB.
    pub async fn reload_llm_via_http(conn: &KernelConnection) -> Result<(), AppError> {
        if !Self::ensure_healthy(conn).await {
            return Err(Self::offline_err());
        }
        let res = conn
            .http_client()
            .post(format!("{}/llm/reload", conn.base_url))
            .json(&serde_json::json!({}))
            .send()
            .await
            .map_err(|e| Self::map_send_err(&conn.base_url, "llm/reload request", e))?;
        let status = res.status();
        if status.is_success() {
            Ok(())
        } else {
            let text = res.text().await.unwrap_or_default();
            Err(app_error_from_http_response(status.as_u16(), &text))
        }
    }

    pub async fn fetch_chat_messages_via_http(
        conn: &KernelConnection,
        session_id: &str,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<StoredMessage>, AppError> {
        if !Self::ensure_healthy(conn).await {
            return Err(Self::offline_err());
        }
        let res = conn
            .http_client()
            .get(format!("{}/chat/messages", conn.base_url))
            .query(&[
                ("session_id", session_id),
                ("limit", &limit.to_string()),
                ("offset", &offset.to_string()),
            ])
            .send()
            .await
            .map_err(|e| Self::map_send_err(&conn.base_url, "chat/messages request", e))?;
        let status = res.status();
        let text = res
            .text()
            .await
            .map_err(|e| AppError::OllamaError(format!("chat/messages body: {e}")))?;
        if !status.is_success() {
            return Err(AppError::OllamaError(format!(
                "chat/messages HTTP {status}: {text}"
            )));
        }
        serde_json::from_str(&text)
            .map_err(|e| AppError::OllamaError(format!("chat/messages JSON: {e}")))
    }

    pub async fn chat_storage_proxy_via_http(
        conn: &KernelConnection,
        op: &oclive_kernel_host::service::ChatStorageProxyOp,
    ) -> Result<serde_json::Value, AppError> {
        if !Self::ensure_healthy(conn).await {
            return Err(Self::offline_err());
        }
        let res = conn
            .http_client()
            .post(format!("{}/chat/storage", conn.base_url))
            .json(op)
            .send()
            .await
            .map_err(|e| Self::map_send_err(&conn.base_url, "chat/storage request", e))?;
        let status = res.status();
        let text = res
            .text()
            .await
            .map_err(|e| AppError::OllamaError(format!("chat/storage body: {e}")))?;
        if !status.is_success() {
            return Err(app_error_from_http_response(status.as_u16(), &text));
        }
        serde_json::from_str(&text)
            .map_err(|e| AppError::OllamaError(format!("chat/storage JSON: {e}")))
    }
}

/// Resolve on-disk role directory for `role_id`.
pub fn role_dir_for_id(state: &AppState, role_id: &str) -> PathBuf {
    state.storage.roles_dir().join(role_id)
}
