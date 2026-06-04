//! Shared manual/auto reconnect and UI status events.

use super::connection::{DesktopKernelMode, KernelConnection, KernelConnectionStatus};
use super::spawn::{spawn_kernel, wait_for_health};
use super::status::{build_ui_status, probe_health_status};
use oclive_kernel_runtime::{
    apply_promote_to_candidate, discover_spawn_kernel_candidates, pick_best_kernel,
};
use parking_lot::Mutex;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, Manager};

/// Inputs for attach-first then spawn reconnect.
pub struct ReconnectOptions {
    pub port: u16,
    pub roles_dir: PathBuf,
    pub anchors: Vec<PathBuf>,
    pub bundled_binary: Option<PathBuf>,
}

/// Bounded automatic reconnect with backoff (manual reconnect resets).
#[derive(Debug)]
pub struct AutoReconnectPolicy {
    pub attempts: u32,
    pub max_attempts: u32,
}

impl Default for AutoReconnectPolicy {
    fn default() -> Self {
        Self {
            attempts: 0,
            max_attempts: 4,
        }
    }
}

impl AutoReconnectPolicy {
    #[must_use]
    pub fn should_try(&self) -> bool {
        self.attempts < self.max_attempts
    }

    pub fn record_attempt(&mut self) {
        self.attempts = self.attempts.saturating_add(1);
    }

    pub fn reset(&mut self) {
        self.attempts = 0;
    }

    /// Backoff before attempt index: 0 / 5 / 15 / 30 seconds.
    #[must_use]
    pub fn delay_before_attempt(attempt_index: u32) -> Duration {
        match attempt_index {
            0 => Duration::from_secs(0),
            1 => Duration::from_secs(5),
            2 => Duration::from_secs(15),
            _ => Duration::from_secs(30),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum StatusEmit {
    None,
    UpstreamLost,
    Reconnected,
}

/// Emit UI-safe status; keeps legacy event names for older frontends.
pub fn emit_kernel_status(
    app: &AppHandle,
    status: &KernelConnectionStatus,
    kind: StatusEmit,
) {
    let _ = app.emit_all("kernel:status_changed", status);
    match kind {
        StatusEmit::UpstreamLost => {
            let _ = app.emit_all("kernel:upstream_lost", status);
        }
        StatusEmit::Reconnected => {
            let _ = app.emit_all("kernel:reconnected", status);
        }
        StatusEmit::None => {}
    }
}

/// One reconnect cycle: attach probe → discover/spawn → final health probe.
///
/// # Errors
///
/// Returns a human-readable message when no kernel becomes healthy.
pub async fn reconnect_once(
    app: &AppHandle,
    conn: &KernelConnection,
    opts: &ReconnectOptions,
) -> Result<KernelConnectionStatus, String> {
    conn.set_mode(DesktopKernelMode::Reconnecting);
    let status = build_ui_status(conn, false);
    emit_kernel_status(app, &status, StatusEmit::None);

    conn.kill_spawned_child();

    if wait_for_health(&conn.base_url).await {
        conn.set_mode(DesktopKernelMode::Attached);
        let status = probe_health_status(conn).await;
        emit_kernel_status(app, &status, StatusEmit::Reconnected);
        return Ok(status);
    }

    let candidates = discover_spawn_kernel_candidates(
        &opts.anchors,
        None,
        opts.bundled_binary.as_deref(),
    );
    let Some(best) = pick_best_kernel(&candidates) else {
        conn.set_mode(DesktopKernelMode::Offline);
        let status = build_ui_status(conn, false);
        emit_kernel_status(app, &status, StatusEmit::None);
        return Err("no kernel binary found".into());
    };

    let mut candidate = best.clone();
    apply_promote_to_candidate(&mut candidate);

    spawn_kernel(conn, &candidate, opts.port, &opts.roles_dir).await?;
    conn.set_mode(DesktopKernelMode::Spawned);

    let status = probe_health_status(conn).await;
    if status.healthy {
        emit_kernel_status(app, &status, StatusEmit::Reconnected);
        Ok(status)
    } else {
        conn.set_mode(DesktopKernelMode::Offline);
        let status = build_ui_status(conn, false);
        emit_kernel_status(app, &status, StatusEmit::None);
        Err("kernel spawned but /health did not become ready".into())
    }
}

pub type SharedAutoReconnectPolicy = Arc<Mutex<AutoReconnectPolicy>>;

#[must_use]
pub fn new_auto_reconnect_policy() -> SharedAutoReconnectPolicy {
    Arc::new(Mutex::new(AutoReconnectPolicy::default()))
}

/// Reset auto-reconnect budget (manual reconnect).
pub fn reset_auto_reconnect(policy: &SharedAutoReconnectPolicy) {
    policy.lock().reset();
}
