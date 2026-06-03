//! Kernel connection status and reconnect for the desktop host.

use crate::api::error::CommandError;
use crate::kernel_attach::KernelHttpClient;
use crate::kernel_lifecycle::{
    spawn::{spawn_kernel, wait_for_health},
    DesktopKernelMode, KernelConnectionStatus, SharedKernelConnection,
};
use crate::state::SharedAppState;
use oclive_kernel_runtime::{
    discover_spawn_kernel_candidates, pick_best_kernel, promote_to_shared_runtime, should_promote,
    KernelCandidate, KernelTier,
};
use tauri::{AppHandle, Manager, State};

/// # Errors
///
/// Returns [`Err`] when kernel state is not managed (should not happen on desktop).
#[tauri::command]
pub async fn get_kernel_connection_status(
    app: AppHandle,
) -> Result<KernelConnectionStatus, CommandError> {
    let conn = app
        .try_state::<SharedKernelConnection>()
        .ok_or_else(|| CommandError(crate::error::AppError::KernelOffline))?;
    let healthy = KernelHttpClient::probe_health(&conn.base_url).await;
    Ok(conn.status(healthy))
}

/// Re-attach or respawn the loopback kernel.
///
/// # Errors
///
/// Returns [`Err`] when discovery/spawn fails.
#[tauri::command]
pub async fn reconnect_kernel(
    app: AppHandle,
    state: State<'_, SharedAppState>,
) -> Result<KernelConnectionStatus, CommandError> {
    let conn = app
        .try_state::<SharedKernelConnection>()
        .ok_or_else(|| CommandError(crate::error::AppError::KernelOffline))?;

    conn.kill_spawned_child();

    if wait_for_health(&conn.base_url).await {
        conn.set_mode(DesktopKernelMode::Attached);
        let status = conn.status(true);
        let _ = app.emit_all("kernel:reconnected", &status);
        return Ok(status);
    }

    conn.set_mode(DesktopKernelMode::Reconnecting);
    let port = conn.port;
    let roles_dir = state.storage.roles_dir().to_path_buf();
    let mut anchors = Vec::new();
    if let Ok(exe) = std::env::current_exe() {
        if let Some(p) = exe.parent() {
            anchors.push(p.to_path_buf());
        }
    }
    if let Ok(cwd) = std::env::current_dir() {
        anchors.push(cwd);
    }

    let candidates = discover_spawn_kernel_candidates(&anchors, None, None);
    let Some(best) = pick_best_kernel(&candidates) else {
        conn.set_mode(DesktopKernelMode::Offline);
        return Err(CommandError(crate::error::AppError::KernelOffline));
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

    spawn_kernel(&conn, &candidate, port, &roles_dir)
        .await
        .map_err(|e| CommandError(crate::error::AppError::OllamaError(e)))?;
    conn.set_mode(DesktopKernelMode::Spawned);
    let status = conn.status(true);
    let _ = app.emit_all("kernel:reconnected", &status);
    Ok(status)
}

/// Extended kernel diagnostics for settings UI.
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KernelDiagnostics {
    pub status: KernelConnectionStatus,
    pub shared_runtime_path: String,
    pub shared_runtime_exists: bool,
    pub shared_runtime_modified_ms: Option<i64>,
    pub health_json: Option<serde_json::Value>,
}

/// # Errors
///
/// Returns [`Err`] when kernel state is missing.
#[tauri::command]
pub async fn get_kernel_diagnostics(app: AppHandle) -> Result<KernelDiagnostics, CommandError> {
    use oclive_kernel_runtime::shared_kernel_binary_path;

    let conn = app
        .try_state::<SharedKernelConnection>()
        .ok_or_else(|| CommandError(crate::error::AppError::KernelOffline))?;
    let healthy = KernelHttpClient::probe_health(&conn.base_url).await;
    let status = conn.status(healthy);

    let shared = shared_kernel_binary_path();
    let shared_runtime_exists = shared.is_file();
    let shared_runtime_modified_ms = shared.metadata().ok().and_then(|m| {
        m.modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_millis() as i64)
    });

    let health_json = if healthy {
        match conn
            .http_client()
            .get(format!("{}/health", conn.base_url))
            .header("accept", "application/json")
            .send()
            .await
        {
            Ok(res) => res.json().await.ok(),
            Err(_) => None,
        }
    } else {
        None
    };

    Ok(KernelDiagnostics {
        status,
        shared_runtime_path: shared.to_string_lossy().into_owned(),
        shared_runtime_exists,
        shared_runtime_modified_ms,
        health_json,
    })
}
