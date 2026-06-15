//! Kernel connection status and reconnect for the desktop host.

use crate::api::error::CommandError;
use crate::kernel_lifecycle::{
    probe_health_status, reconnect_once, KernelConnectionStatus, ReconnectOptions,
    SharedKernelConnection,
};
use oclive_kernel_host::state::SharedAppState;
use oclive_kernel_runtime::shared_kernel_binary_path;
use tauri::{AppHandle, Manager, State};

/// Returns kernel connection status for the desktop shell (attach/spawn/offline).
#[tauri::command]
pub async fn get_kernel_connection_status(
    app: AppHandle,
) -> Result<KernelConnectionStatus, CommandError> {
    let conn = app
        .try_state::<SharedKernelConnection>()
        .ok_or(crate::error::AppError::KernelOffline)?;
    Ok(probe_health_status(&conn).await)
}

/// Re-attach or respawn the loopback kernel; fails if discovery/spawn errors.
#[tauri::command]
pub async fn reconnect_kernel(
    app: AppHandle,
    state: State<'_, SharedAppState>,
) -> Result<KernelConnectionStatus, CommandError> {
    let conn = app
        .try_state::<SharedKernelConnection>()
        .ok_or(crate::error::AppError::KernelOffline)?;

    conn.auto_reconnect.lock().reset();

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

    let bundled_binary = app
        .path_resolver()
        .resource_dir()
        .and_then(|res| {
            let name = if cfg!(windows) {
                "oclive-kernel-server.exe"
            } else {
                "oclive-kernel-server"
            };
            let path = res.join(name);
            if path.is_file() { Some(path) } else { None }
        })
        .or_else(|| {
            std::env::var("OCLIVE_KERNEL_BINARY")
                .ok()
                .map(std::path::PathBuf::from)
        });

    let opts = ReconnectOptions {
        port,
        roles_dir,
        anchors,
        bundled_binary,
    };

    Ok(reconnect_once(&app, &conn, &opts)
        .await
        .map_err(crate::error::AppError::OllamaError)?)
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

/// Returns extended kernel diagnostics for settings; fails when kernel connection state is missing.
#[tauri::command]
pub async fn get_kernel_diagnostics(app: AppHandle) -> Result<KernelDiagnostics, CommandError> {
    let conn = app
        .try_state::<SharedKernelConnection>()
        .ok_or(crate::error::AppError::KernelOffline)?;
    let status = probe_health_status(&conn).await;

    let shared = shared_kernel_binary_path();
    let shared_runtime_exists = shared.is_file();
    let shared_runtime_modified_ms = shared.metadata().ok().and_then(|m| {
        m.modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_millis() as i64)
    });

    let health_json = if status.healthy {
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
