//! Periodic health probe; respawn when upstream is lost.

use super::connection::{DesktopKernelMode, SharedKernelConnection};
use super::reconnect::{
    emit_kernel_status, reconnect_once, AutoReconnectPolicy, ReconnectOptions, StatusEmit,
};
use super::status::{build_ui_status, probe_health_status};
use crate::kernel_attach::KernelHttpClient;
use std::path::PathBuf;
use std::time::Duration;
use tauri::AppHandle;

const WATCHDOG_INTERVAL_HEALTHY_SECS: u64 = 20;
const WATCHDOG_INTERVAL_UNHEALTHY_SECS: u64 = 3;

fn was_connected(mode: DesktopKernelMode) -> bool {
    matches!(
        mode,
        DesktopKernelMode::Attached | DesktopKernelMode::Spawned
    )
}

/// Start background health polling; emits `kernel:status_changed` (+ legacy events).
pub fn start_kernel_watchdog(
    app: AppHandle,
    conn: SharedKernelConnection,
    roles_dir: PathBuf,
    anchors: Vec<PathBuf>,
    bundled_binary: Option<PathBuf>,
) {
    let opts = ReconnectOptions {
        port: conn.port,
        roles_dir,
        anchors,
        bundled_binary,
    };

    tauri::async_runtime::spawn(async move {
        loop {
            let child_exited = conn.try_wait_spawned_child();
            let healthy = KernelHttpClient::probe_health(&conn.base_url).await;
            let mut internal = conn.mode_snapshot();

            if healthy {
                if child_exited
                    || matches!(
                        internal,
                        DesktopKernelMode::Reconnecting | DesktopKernelMode::Offline
                    )
                {
                    conn.set_mode(if conn.has_spawned_child() {
                        DesktopKernelMode::Spawned
                    } else {
                        DesktopKernelMode::Attached
                    });
                    conn.auto_reconnect.lock().reset();
                    let status = probe_health_status(&conn).await;
                    emit_kernel_status(&app, &status, StatusEmit::Reconnected);
                }
                tokio::time::sleep(Duration::from_secs(WATCHDOG_INTERVAL_HEALTHY_SECS)).await;
                continue;
            }

            if child_exited && was_connected(internal) {
                tracing::warn!(target: "oclive_desktop", "spawned kernel child exited");
                conn.set_mode(DesktopKernelMode::Reconnecting);
                internal = DesktopKernelMode::Reconnecting;
                let status = build_ui_status(&conn, false);
                emit_kernel_status(&app, &status, StatusEmit::UpstreamLost);
            }

            let interval_secs = WATCHDOG_INTERVAL_UNHEALTHY_SECS;

            {
                let exhausted = {
                    let policy = conn.auto_reconnect.lock();
                    !policy.should_try()
                };
                if exhausted {
                    if internal != DesktopKernelMode::Offline {
                        conn.set_mode(DesktopKernelMode::Offline);
                        let status = build_ui_status(&conn, false);
                        emit_kernel_status(&app, &status, StatusEmit::None);
                    }
                    tokio::time::sleep(Duration::from_secs(interval_secs)).await;
                    continue;
                }
            }

            if was_connected(internal) {
                tracing::warn!(target: "oclive_desktop", "kernel upstream lost");
                conn.set_mode(DesktopKernelMode::Reconnecting);
                let status = build_ui_status(&conn, false);
                emit_kernel_status(&app, &status, StatusEmit::UpstreamLost);
            } else if internal != DesktopKernelMode::Offline
                && internal != DesktopKernelMode::Reconnecting
            {
                conn.set_mode(DesktopKernelMode::Offline);
            }

            let attempt_index = {
                let mut policy = conn.auto_reconnect.lock();
                let idx = policy.attempts;
                policy.record_attempt();
                idx
            };

            tokio::time::sleep(AutoReconnectPolicy::delay_before_attempt(attempt_index)).await;

            match reconnect_once(&app, &conn, &opts).await {
                Ok(status) => {
                    conn.auto_reconnect.lock().reset();
                    emit_kernel_status(&app, &status, StatusEmit::Reconnected);
                }
                Err(e) => {
                    tracing::warn!(target: "oclive_desktop", error = %e, "auto reconnect failed");
                    let exhausted = {
                        let policy = conn.auto_reconnect.lock();
                        !policy.should_try()
                    };
                    if exhausted {
                        conn.set_mode(DesktopKernelMode::Offline);
                        let status = build_ui_status(&conn, false);
                        emit_kernel_status(&app, &status, StatusEmit::None);
                    }
                }
            }

            tokio::time::sleep(Duration::from_secs(interval_secs)).await;
        }
    });
}
