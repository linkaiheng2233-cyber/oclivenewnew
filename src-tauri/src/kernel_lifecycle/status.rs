//! Health probe and UI-safe kernel connection status derivation.

use super::connection::{DesktopKernelMode, KernelConnection, KernelConnectionStatus};
use crate::kernel_attach::KernelHttpClient;

/// Probe `GET /health` (3s timeout) and return a UI-safe [`KernelConnectionStatus`].
pub async fn probe_health_status(conn: &KernelConnection) -> KernelConnectionStatus {
    let healthy = KernelHttpClient::probe_health(&conn.base_url).await;
    if healthy {
        sync_internal_mode_on_healthy(conn);
    }
    build_ui_status(conn, healthy)
}

/// Map internal connection mode + health probe to UI-facing mode.
#[must_use]
pub fn to_ui_mode(
    internal: DesktopKernelMode,
    healthy: bool,
    has_spawned_child: bool,
) -> DesktopKernelMode {
    if healthy {
        match internal {
            DesktopKernelMode::Attached | DesktopKernelMode::Spawned => internal,
            DesktopKernelMode::Reconnecting | DesktopKernelMode::Offline => {
                if has_spawned_child {
                    DesktopKernelMode::Spawned
                } else {
                    DesktopKernelMode::Attached
                }
            }
        }
    } else if internal == DesktopKernelMode::Reconnecting {
        DesktopKernelMode::Reconnecting
    } else {
        DesktopKernelMode::Offline
    }
}

/// Build status snapshot for API/events (does not probe).
#[must_use]
pub fn build_ui_status(conn: &KernelConnection, healthy: bool) -> KernelConnectionStatus {
    let internal = conn.mode_snapshot();
    let ui_mode = to_ui_mode(internal, healthy, conn.has_spawned_child());
    if internal != ui_mode || !healthy {
        tracing::info!(
            target: "oclive_desktop",
            healthy,
            ?internal,
            ui_mode = ?ui_mode,
            "kernel ui status"
        );
    }
    KernelConnectionStatus {
        mode: ui_mode,
        base_url: conn.base_url.clone(),
        port: conn.port,
        binary_path: conn.binary_path.read().clone(),
        kernel_tier: conn
            .kernel_tier
            .read()
            .map(|t| format!("{t:?}").to_lowercase().replace('_', "-")),
        healthy,
        degraded: (*conn.degraded.read()).then_some(true),
        status_message: conn.status_message.read().clone(),
    }
}

fn sync_internal_mode_on_healthy(conn: &KernelConnection) {
    let internal = conn.mode_snapshot();
    if matches!(
        internal,
        DesktopKernelMode::Offline | DesktopKernelMode::Reconnecting
    ) {
        let next = if conn.has_spawned_child() {
            DesktopKernelMode::Spawned
        } else {
            DesktopKernelMode::Attached
        };
        conn.set_mode(next);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_ui_mode_healthy_matrix() {
        assert_eq!(
            to_ui_mode(DesktopKernelMode::Attached, true, false),
            DesktopKernelMode::Attached
        );
        assert_eq!(
            to_ui_mode(DesktopKernelMode::Spawned, true, true),
            DesktopKernelMode::Spawned
        );
        assert_eq!(
            to_ui_mode(DesktopKernelMode::Offline, true, false),
            DesktopKernelMode::Attached
        );
        assert_eq!(
            to_ui_mode(DesktopKernelMode::Offline, true, true),
            DesktopKernelMode::Spawned
        );
    }

    #[test]
    fn to_ui_mode_unhealthy_never_attached() {
        assert_eq!(
            to_ui_mode(DesktopKernelMode::Attached, false, false),
            DesktopKernelMode::Offline
        );
        assert_eq!(
            to_ui_mode(DesktopKernelMode::Spawned, false, true),
            DesktopKernelMode::Offline
        );
        assert_eq!(
            to_ui_mode(DesktopKernelMode::Reconnecting, false, false),
            DesktopKernelMode::Reconnecting
        );
    }
}
