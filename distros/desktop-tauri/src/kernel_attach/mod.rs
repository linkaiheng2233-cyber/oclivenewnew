//! Remote kernel HTTP client (desktop → `:8420` single writer).

mod chat;
mod resources;
mod theater;

use crate::error::AppError;
use crate::kernel_lifecycle::KernelConnection;
use oclive_kernel_host::state::AppState;
pub(crate) use oclive_kernel_runtime::app_error_from_http_response;
use oclive_kernel_runtime::KernelBinaryManifest;
use oclive_kernel_runtime::RUNTIME_API_VERSION;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

const HEALTH_GATE_TTL: Duration = Duration::from_millis(1500);

#[derive(Debug, Deserialize)]
struct HealthProbeJson {
    ok: bool,
    runtime_api_version: String,
}

/// Full `/health` JSON (policy + diagnostics).
#[derive(Debug, Clone, Deserialize)]
pub struct KernelHealthJson {
    pub ok: bool,
    #[serde(default)]
    pub kernel_manifest: Option<KernelBinaryManifest>,
    pub distro_id: Option<String>,
    pub distro_profile_hash: Option<String>,
    pub active_profile_summary: Option<oclive_kernel_types::ActiveProfileSummary>,
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

    /// Verify the caller's token against the running kernel (`GET /auth/check`).
    /// `false` means the kernel is stale (token mismatch) and should be replaced.
    pub async fn probe_authenticated(base_url: &str, api_token: &str) -> bool {
        let url = format!("{}/auth/check", base_url.trim_end_matches('/'));
        let Ok(res) = probe_http_client()
            .get(&url)
            .header(oclive_kernel_host::http_api::API_TOKEN_HEADER, api_token)
            .timeout(Duration::from_secs(3))
            .send()
            .await
        else {
            return false;
        };
        res.status().is_success()
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

    pub async fn fetch_health_json(base_url: &str) -> Option<KernelHealthJson> {
        let url = format!("{}/health", base_url.trim_end_matches('/'));
        let Ok(res) = probe_http_client()
            .get(&url)
            .header("Accept", "application/json")
            .timeout(Duration::from_secs(5))
            .send()
            .await
        else {
            return None;
        };
        if !res.status().is_success() {
            return None;
        }
        res.json().await.ok()
    }

    pub(super) async fn ensure_healthy(conn: &KernelConnection) -> bool {
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

    pub(super) fn map_send_err(base_url: &str, context: &str, e: reqwest::Error) -> AppError {
        HealthGate::invalidate(base_url);
        AppError::OllamaError(format!("{context}: {e}"))
    }

    pub(super) fn offline_err() -> AppError {
        AppError::KernelOffline
    }
}

/// Resolve on-disk role directory for `role_id`.
pub fn role_dir_for_id(state: &AppState, role_id: &str) -> Result<PathBuf, AppError> {
    state.storage.role_dir_path(role_id)
}

fn parse_sse_block(block: &str) -> (String, String) {
    let mut event_name = "message".to_string();
    let mut data_lines = Vec::new();
    for line in block.lines() {
        if let Some(rest) = line.strip_prefix("event:") {
            event_name = rest.trim().to_string();
        } else if let Some(rest) = line.strip_prefix("data:") {
            data_lines.push(rest.trim());
        }
    }
    (event_name, data_lines.join("\n"))
}
