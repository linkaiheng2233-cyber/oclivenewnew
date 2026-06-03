//! Periodic health probe; respawn when upstream is lost.

use super::connection::{DesktopKernelMode, SharedKernelConnection};
use super::spawn::{spawn_kernel, wait_for_health};
use oclive_kernel_runtime::{
    discover_spawn_kernel_candidates, pick_best_kernel, promote_to_shared_runtime, should_promote,
    KernelCandidate, KernelTier,
};
use std::path::PathBuf;
use std::time::Duration;
use tauri::{AppHandle, Manager};

const WATCHDOG_INTERVAL_SECS: u64 = 20;

/// Start background health polling; emits `kernel:upstream_lost` / `kernel:reconnected`.
pub fn start_kernel_watchdog(
    app: AppHandle,
    conn: SharedKernelConnection,
    roles_dir: PathBuf,
    anchors: Vec<PathBuf>,
    bundled_binary: Option<PathBuf>,
) {
    tauri::async_runtime::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(WATCHDOG_INTERVAL_SECS)).await;
            let healthy = wait_for_health(&conn.base_url).await;
            let mode = conn.mode_snapshot();

            if healthy {
                if mode == DesktopKernelMode::Reconnecting {
                    conn.set_mode(if conn.has_spawned_child() {
                        DesktopKernelMode::Spawned
                    } else {
                        DesktopKernelMode::Attached
                    });
                    let _ = app.emit_all("kernel:reconnected", conn.status(true));
                }
                continue;
            }

            if mode == DesktopKernelMode::Offline || mode == DesktopKernelMode::Reconnecting {
                continue;
            }

            tracing::warn!(target: "oclive_desktop", "kernel upstream lost");
            conn.set_mode(DesktopKernelMode::Reconnecting);
            let _ = app.emit_all("kernel:upstream_lost", conn.status(false));

            conn.kill_spawned_child();

            if wait_for_health(&conn.base_url).await {
                conn.set_mode(DesktopKernelMode::Attached);
                let _ = app.emit_all("kernel:reconnected", conn.status(true));
                continue;
            }

            let candidates =
                discover_spawn_kernel_candidates(&anchors, None, bundled_binary.as_deref());
            let Some(best) = pick_best_kernel(&candidates) else {
                conn.set_mode(DesktopKernelMode::Offline);
                continue;
            };
            let mut candidate = best.clone();
            if should_promote(&candidate) {
                if let Ok(promoted) = promote_to_shared_runtime(&candidate.binary) {
                    candidate = KernelCandidate {
                        binary: promoted,
                        tier: KernelTier::Shared,
                        score: oclive_kernel_runtime::SCORE_SHARED,
                        extra_args: vec![],
                    };
                }
            }

            if spawn_kernel(&conn, &candidate, conn.port, &roles_dir)
                .await
                .is_ok()
            {
                conn.set_mode(DesktopKernelMode::Spawned);
                let _ = app.emit_all("kernel:reconnected", conn.status(true));
            } else {
                conn.set_mode(DesktopKernelMode::Offline);
            }
        }
    });
}
