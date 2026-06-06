//! Desktop startup: spawn-only kernel client + in-memory UI shell (no local DB writer).

use crate::infrastructure::MockLlmClient;
use crate::kernel_lifecycle::{
    ensure_kernel_ready, start_kernel_watchdog, DesktopKernelMode, EnsureKernelOptions,
    KernelConnection, SharedKernelConnection,
};
use crate::state::{AppState, SharedAppState};
use oclive_kernel_runtime::{resolve_api_port, shared_kernel_binary_path};
use std::path::PathBuf;
use std::sync::Arc;
use tauri::AppHandle;

fn discovery_anchors(resource_dir: Option<&std::path::Path>) -> Vec<PathBuf> {
    let mut anchors = Vec::new();
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            anchors.push(parent.to_path_buf());
        }
    }
    if let Ok(cwd) = std::env::current_dir() {
        anchors.push(cwd);
    }
    if let Some(res) = resource_dir {
        anchors.push(res.to_path_buf());
    }
    anchors
}

fn bundled_kernel_binary(resource_dir: Option<&std::path::Path>) -> Option<PathBuf> {
    resource_dir.map(|res| {
        let name = if cfg!(windows) {
            "oclive-kernel-server.exe"
        } else {
            "oclive-kernel-server"
        };
        res.join(name)
    })
}

/// Run [`bootstrap_desktop`] on a dedicated Tokio runtime (safe inside Tauri `.setup`).
///
/// # Errors
///
/// Returns bootstrap errors when the worker thread panics or bootstrap fails.
pub fn bootstrap_desktop_blocking(
    resource_dir: Option<PathBuf>,
) -> Result<(SharedAppState, SharedKernelConnection, u16), Box<dyn std::error::Error + Send + Sync>>
{
    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .thread_name("oclive-desktop-bootstrap")
            .build()
            .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { Box::new(e) })?;
        rt.block_on(bootstrap_desktop(resource_dir.as_deref()))
    })
    .join()
    .map_err(|_| -> Box<dyn std::error::Error + Send + Sync> {
        "desktop bootstrap thread panicked".into()
    })?
}

/// Bootstrap desktop: attach or spawn loopback kernel; UI shell has no persistent DB writer.
///
/// # Errors
///
/// Returns bootstrap errors when kernel cannot start and shell cannot be created.
pub async fn bootstrap_desktop(
    resource_dir: Option<&std::path::Path>,
) -> Result<(SharedAppState, SharedKernelConnection, u16), Box<dyn std::error::Error + Send + Sync>>
{
    let port = resolve_api_port(None);
    let roles_dir = crate::state::resolve_roles_dir(resource_dir);
    let canonical_models = crate::state::ensure_models_dir_for_roles(&roles_dir);
    let app_data = oclive_kernel_runtime::resolve_app_data_dir_for_host();
    crate::state::reconcile_legacy_models_layout(&canonical_models, &app_data);
    crate::api::llm_settings::sync_canonical_db_models_dir(&canonical_models, &app_data).await;
    let anchors = discovery_anchors(resource_dir);
    let bundled = bundled_kernel_binary(resource_dir);

    let kernel = match ensure_kernel_ready(EnsureKernelOptions {
        port,
        roles_dir: roles_dir.clone(),
        anchors: anchors.clone(),
        bundled_binary: bundled.clone(),
    })
    .await
    {
        Ok(k) => k,
        Err(e) => {
            tracing::warn!(
                target: "oclive_desktop",
                error = %e,
                "kernel not ready at startup; UI will run offline until reconnect"
            );
            let base_url = format!("http://127.0.0.1:{port}");
            let conn = Arc::new(KernelConnection::new(base_url, port));
            conn.set_mode(DesktopKernelMode::Offline);
            conn
        }
    };

    tracing::info!(
        target: "oclive_desktop",
        mode = ?kernel.mode_snapshot(),
        port,
        shared_runtime = %shared_kernel_binary_path().display(),
        roles = %roles_dir.display(),
        models = %canonical_models.display(),
        "desktop kernel client ready"
    );

    let llm = Arc::new(MockLlmClient {
        reply: String::new(),
    });
    // UI-only in-memory shell — not an authoritative DB/chat writer; kernel HTTP owns persistence.
    let shell = AppState::new_in_memory_with_llm(llm, roles_dir).await?;
    crate::api::llm_settings::seed_shell_llm_from_canonical(&shell).await;
    if let Err(e) = crate::kernel_attach::KernelHttpClient::reload_llm_via_http(&kernel).await {
        tracing::warn!(
            target: "oclive_llm",
            error = %e,
            "kernel LLM reload at desktop bootstrap failed"
        );
    }
    Ok((Arc::new(shell), kernel, port))
}

/// Wire watchdog and exit cleanup after Tauri manages state.
pub fn finish_desktop_setup(
    app: &AppHandle,
    kernel: SharedKernelConnection,
    roles_dir: PathBuf,
    resource_dir: Option<PathBuf>,
) {
    let anchors = discovery_anchors(resource_dir.as_deref());
    let bundled = bundled_kernel_binary(resource_dir.as_deref());
    start_kernel_watchdog(
        app.clone(),
        Arc::clone(&kernel),
        roles_dir,
        anchors,
        bundled,
    );
}
