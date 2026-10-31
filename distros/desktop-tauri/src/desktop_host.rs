//! Desktop startup: spawn-only kernel client + in-memory UI shell (no local DB writer).

use crate::kernel_lifecycle::{
    emit_kernel_status, ensure_kernel_ready_on_conn, probe_health_status, start_kernel_watchdog,
    DesktopKernelMode, EnsureKernelOptions, KernelConnection, SharedKernelConnection, StatusEmit,
};
use oclive_kernel_host::infrastructure::MockLlmClient;
use oclive_kernel_host::state::{AppState, AppStateBuilder, SharedAppState};
use oclive_kernel_runtime::{resolve_api_port, shared_kernel_binary_path};
use std::path::{Path, PathBuf};
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
    let resource_dir = resource_dir?;
    let name = if cfg!(windows) {
        "oclive-kernel-server.exe"
    } else {
        "oclive-kernel-server"
    };
    [
        resource_dir.join(name),
        resource_dir.join("resources").join(name),
    ]
    .into_iter()
    .find(|candidate| candidate.is_file())
}

async fn build_ui_shell(
    roles_dir: PathBuf,
    app_data: &Path,
) -> oclive_kernel_host::error::Result<AppState> {
    // Keep the shell database in memory, but put its auxiliary files in the
    // writable application-data directory rather than the installed role bundle.
    AppStateBuilder::in_memory_test(
        Arc::new(MockLlmClient {
            reply: String::new(),
        }),
        roles_dir,
        None,
    )
    .with_app_data_dir(app_data)
    .with_host_profile(oclive_kernel_host::domain::host_profile::load_host_profile_from_env())
    .build()
    .await
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
    let resolved_roles = oclive_kernel_host::state::find_roles_dir(resource_dir);
    let app_data = oclive_kernel_runtime::find_app_data_dir_for_host();
    let explicit_roles_override = std::env::var("OCLIVE_ROLES_DIR")
        .ok()
        .filter(|path| !path.trim().is_empty())
        .is_some_and(|path| Path::new(path.trim()).is_dir());
    let distro = oclive_kernel_host::domain::host_profile::load_host_profile_from_env();
    let roles_dir = if matches!(distro.distro_id.as_str(), "default" | "desktop") {
        crate::desktop_library::prepare_desktop_library(
            resource_dir,
            &resolved_roles,
            &app_data,
            explicit_roles_override,
        )?
    } else {
        resolved_roles
    };
    let canonical_models = oclive_kernel_host::state::ensure_models_dir_for_roles(&roles_dir);
    oclive_kernel_host::state::reconcile_legacy_models_layout(&canonical_models, &app_data);
    crate::api::llm_settings::sync_canonical_db_models_dir(&canonical_models, &app_data).await;
    let base_url = format!("http://127.0.0.1:{port}");
    let kernel = Arc::new(KernelConnection::new(base_url, port));
    kernel.set_mode(DesktopKernelMode::Reconnecting);

    tracing::info!(
        target: "oclive_desktop",
        mode = ?kernel.mode_snapshot(),
        port,
        shared_runtime = %shared_kernel_binary_path().display(),
        roles = %roles_dir.display(),
        models = %canonical_models.display(),
        "desktop kernel client initialized; cold bring-up scheduled in background"
    );

    // UI-only in-memory shell — not an authoritative DB/chat writer; kernel HTTP owns persistence.
    let shell = build_ui_shell(roles_dir, &app_data).await?;
    crate::api::llm_settings::seed_shell_llm_from_canonical(&shell).await;
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
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        let bring_up = ensure_kernel_ready_on_conn(
            Arc::clone(&kernel),
            EnsureKernelOptions {
                port: kernel.port,
                roles_dir: roles_dir.clone(),
                anchors: anchors.clone(),
                bundled_binary: bundled.clone(),
            },
        )
        .await;

        match bring_up {
            Ok(_) => {
                if let Err(error) =
                    crate::kernel_attach::KernelHttpClient::reload_llm_via_http(&kernel).await
                {
                    tracing::warn!(
                        target: "oclive_llm",
                        error = %error,
                        "kernel LLM reload after background bring-up failed"
                    );
                }
                let status = probe_health_status(&kernel).await;
                emit_kernel_status(&app, &status, StatusEmit::Reconnected);
            }
            Err(error) => {
                kernel.set_mode(DesktopKernelMode::Offline);
                tracing::warn!(
                    target: "oclive_desktop",
                    error = %error,
                    "background kernel bring-up failed; watchdog will continue recovery"
                );
                let status = probe_health_status(&kernel).await;
                emit_kernel_status(&app, &status, StatusEmit::None);
            }
        }

        start_kernel_watchdog(app, kernel, roles_dir, anchors, bundled);
    });
}

#[cfg(test)]
mod tests {
    use super::{build_ui_shell, bundled_kernel_binary};

    #[tokio::test]
    async fn ui_shell_keeps_auxiliary_state_out_of_the_installed_role_bundle() {
        let temp = tempfile::tempdir().expect("tempdir");
        let bundle = temp.path().join("bundled-role-packs");
        let app_data = temp.path().join("user-data");
        std::fs::create_dir_all(&bundle).expect("create bundle fixture");

        let shell = build_ui_shell(bundle.clone(), &app_data)
            .await
            .expect("initialize shell");

        assert_eq!(shell.directory_plugins.app_data_dir(), app_data.as_path());
        assert!(app_data.is_dir());
        assert_eq!(
            std::fs::read_dir(bundle).expect("inspect bundle").count(),
            0
        );
    }

    fn kernel_name() -> &'static str {
        if cfg!(windows) {
            "oclive-kernel-server.exe"
        } else {
            "oclive-kernel-server"
        }
    }

    #[test]
    fn bundled_kernel_resolves_release_resource_layout() {
        let temp = tempfile::tempdir().expect("tempdir");
        let binary = temp.path().join(kernel_name());
        std::fs::write(&binary, b"kernel").expect("write binary");

        assert_eq!(
            bundled_kernel_binary(Some(temp.path())).as_deref(),
            Some(binary.as_path())
        );
    }

    #[test]
    fn bundled_kernel_resolves_tauri_development_resource_layout() {
        let temp = tempfile::tempdir().expect("tempdir");
        let binary = temp.path().join("resources").join(kernel_name());
        std::fs::create_dir_all(binary.parent().expect("binary parent")).expect("create resources");
        std::fs::write(&binary, b"kernel").expect("write binary");

        assert_eq!(
            bundled_kernel_binary(Some(temp.path())).as_deref(),
            Some(binary.as_path())
        );
    }
}
