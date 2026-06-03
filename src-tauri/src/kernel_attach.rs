//! Remote kernel HTTP client (desktop attach mode when `:8420` is already serving).

use crate::models::dto::{SendMessageRequest, SendMessageResponse};
use crate::state::AppState;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Active when desktop attaches to an existing kernel on loopback (no local `app.db` writer).
pub struct KernelAttach {
    pub base_url: String,
    client: reqwest::Client,
}

impl KernelAttach {
    #[must_use]
    pub fn new(base_url: impl Into<String>) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(120))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        Self {
            base_url: base_url.into().trim_end_matches('/').to_string(),
            client,
        }
    }

    /// `GET /health` — accepts plain `ok` or JSON `{ "ok": true }`.
    pub async fn healthy(&self) -> bool {
        let url = format!("{}/health", self.base_url);
        let Ok(res) = self
            .client
            .get(&url)
            .timeout(Duration::from_secs(3))
            .send()
            .await
        else {
            return false;
        };
        if !res.status().is_success() {
            return false;
        }
        let Ok(text) = res.text().await else {
            return false;
        };
        let t = text.trim();
        t == "ok" || t.contains("\"ok\":true") || t.contains("\"ok\": true")
    }

    /// Proxy `POST /chat` using role directory path.
    ///
    /// # Errors
    ///
    /// Returns a human-readable message on HTTP or contract failure.
    pub async fn send_message_via_http(
        &self,
        role_path: &Path,
        req: &SendMessageRequest,
    ) -> Result<SendMessageResponse, String> {
        let url = format!("{}/chat", self.base_url);
        let body = serde_json::json!({
            "role_path": role_path.to_string_lossy(),
            "message": req.user_message,
            "session_id": req.session_id,
            "scene_id": req.scene_id,
        });
        let res = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("remote chat request: {e}"))?;
        let status = res.status();
        let text = res.text().await.map_err(|e| format!("remote chat body: {e}"))?;
        if !status.is_success() {
            return Err(format!("remote chat HTTP {status}: {text}"));
        }
        serde_json::from_str(&text).map_err(|e| format!("remote chat JSON: {e}"))
    }
}

/// Resolve on-disk role directory for `role_id`.
pub fn role_dir_for_id(state: &AppState, role_id: &str) -> PathBuf {
    state.storage.roles_dir().join(role_id)
}
