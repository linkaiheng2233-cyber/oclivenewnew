//! Kernel connection state shared by HTTP client and watchdog.

use super::reconnect::AutoReconnectPolicy;
use super::status::build_ui_status;
use oclive_kernel_runtime::KernelTier;
use parking_lot::{Mutex, RwLock};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde::Serialize;
use std::process::Child;
use std::sync::Arc;
use std::time::Duration;

/// How the desktop host reached the loopback kernel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum DesktopKernelMode {
    Attached,
    Spawned,
    Offline,
    Reconnecting,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KernelConnectionStatus {
    pub mode: DesktopKernelMode,
    pub base_url: String,
    pub port: u16,
    pub binary_path: Option<String>,
    pub kernel_tier: Option<String>,
    pub healthy: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub degraded: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_message: Option<String>,
    /// UX hint key for profile scheduling (`profile_compatible`, `degraded`, …).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile_hint_key: Option<String>,
}

/// Active kernel HTTP upstream; optionally owns a spawned child process.
pub struct KernelConnection {
    pub mode: RwLock<DesktopKernelMode>,
    pub base_url: String,
    pub port: u16,
    pub binary_path: RwLock<Option<String>>,
    pub kernel_tier: RwLock<Option<KernelTier>>,
    pub degraded: RwLock<bool>,
    pub status_message: RwLock<Option<String>>,
    profile_hint_key: RwLock<Option<String>>,
    client: RwLock<reqwest::Client>,
    api_token: RwLock<Option<String>>,
    spawned_child: Mutex<Option<Child>>,
    pub auto_reconnect: Mutex<AutoReconnectPolicy>,
}

impl KernelConnection {
    #[must_use]
    pub fn new(base_url: impl Into<String>, port: u16) -> Self {
        let api_token = std::env::var(oclive_kernel_host::http_api::ENV_API_TOKEN)
            .ok()
            .map(|token| token.trim().to_string())
            .filter(|token| !token.is_empty());
        Self {
            mode: RwLock::new(DesktopKernelMode::Offline),
            base_url: base_url.into().trim_end_matches('/').to_string(),
            port,
            binary_path: RwLock::new(None),
            kernel_tier: RwLock::new(None),
            degraded: RwLock::new(false),
            status_message: RwLock::new(None),
            profile_hint_key: RwLock::new(None),
            client: RwLock::new(build_http_client(api_token.as_deref())),
            api_token: RwLock::new(api_token),
            spawned_child: Mutex::new(None),
            auto_reconnect: Mutex::new(AutoReconnectPolicy::default()),
        }
    }

    pub fn http_client(&self) -> reqwest::Client {
        self.client.read().clone()
    }

    #[must_use]
    pub fn api_token_snapshot(&self) -> Option<String> {
        self.api_token.read().clone()
    }

    /// Set the token used for subsequent HTTP requests to the kernel.
    pub fn set_api_token(&self, token: impl Into<String>) {
        let token = token.into().trim().to_string();
        let token = (!token.is_empty()).then_some(token);
        *self.api_token.write() = token.clone();
        *self.client.write() = build_http_client(token.as_deref());
    }

    pub fn set_mode(&self, mode: DesktopKernelMode) {
        *self.mode.write() = mode;
    }

    pub fn mode_snapshot(&self) -> DesktopKernelMode {
        *self.mode.read()
    }

    pub fn set_spawn_metadata(&self, binary: impl Into<String>, tier: KernelTier, child: Child) {
        *self.binary_path.write() = Some(binary.into());
        *self.kernel_tier.write() = Some(tier);
        *self.spawned_child.lock() = Some(child);
    }

    pub fn clear_spawned_child(&self) {
        *self.spawned_child.lock() = None;
    }

    pub fn has_spawned_child(&self) -> bool {
        self.spawned_child.lock().is_some()
    }

    /// Returns `true` when a spawned child has exited (clears handle).
    pub fn try_wait_spawned_child(&self) -> bool {
        let mut guard = self.spawned_child.lock();
        if let Some(child) = guard.as_mut() {
            match child.try_wait() {
                Ok(Some(_)) | Err(_) => {
                    guard.take();
                    return true;
                }
                Ok(None) => {}
            }
        }
        false
    }

    /// Kill only a child this host spawned; never touch an external daemon.
    pub fn kill_spawned_child(&self) {
        if let Some(mut child) = self.spawned_child.lock().take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }

    pub fn set_status_hint(&self, degraded: bool, message: Option<String>) {
        *self.degraded.write() = degraded;
        *self.status_message.write() = message;
    }

    pub fn set_profile_hint_key(&self, key: Option<String>) {
        *self.profile_hint_key.write() = key;
    }

    #[must_use]
    pub fn profile_hint_key_snapshot(&self) -> Option<String> {
        self.profile_hint_key.read().clone()
    }

    pub fn clear_status_hint(&self) {
        *self.degraded.write() = false;
        *self.status_message.write() = None;
        *self.profile_hint_key.write() = None;
    }

    #[must_use]
    pub fn status(&self, healthy: bool) -> KernelConnectionStatus {
        build_ui_status(self, healthy)
    }
}

fn build_http_client(api_token: Option<&str>) -> reqwest::Client {
    let mut headers = HeaderMap::new();
    if let Some(token) = api_token {
        if let Ok(value) = HeaderValue::from_str(token) {
            headers.insert(HeaderName::from_static("x-oclive-api-token"), value);
        }
    }
    reqwest::Client::builder()
        .timeout(Duration::from_secs(120))
        .no_proxy()
        .default_headers(headers)
        .build()
        .unwrap_or_else(|_| reqwest::Client::new())
}

pub type SharedKernelConnection = Arc<KernelConnection>;
