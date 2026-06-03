//! Desktop startup: canonical app data, optional remote attach, in-process `:8420`.

use crate::http_api;
use crate::infrastructure::app_data_migration;
use crate::infrastructure::MockLlmClient;
use crate::kernel_attach::KernelAttach;
use crate::state::{AppState, SharedAppState};
use oclive_kernel_runtime::{
    ensure_app_data_dir, resolve_api_port, resolve_app_data_dir_for_host, resolve_db_path,
};
use std::sync::Arc;

fn env_force_local_kernel() -> bool {
    std::env::var("OCLIVE_FORCE_LOCAL_KERNEL")
        .ok()
        .is_some_and(|v| v == "1" || v.eq_ignore_ascii_case("true"))
}

fn env_attach_remote() -> bool {
    std::env::var("OCLIVE_ATTACH_REMOTE_KERNEL")
        .ok()
        .is_some_and(|v| v == "1" || v.eq_ignore_ascii_case("true"))
}

async fn loopback_kernel_healthy(port: u16) -> bool {
    KernelAttach::new(format!("http://127.0.0.1:{port}")).healthy().await
}

/// Bootstrap desktop host state and optionally bind the HTTP API on `127.0.0.1:port`.
///
/// # Errors
///
/// Returns I/O or DB bootstrap errors as `Box<dyn Error>`.
pub async fn bootstrap_desktop(
) -> Result<(SharedAppState, Option<KernelAttach>, u16), Box<dyn std::error::Error + Send + Sync>> {
    let port = resolve_api_port(None);
    let attach_auto = env_attach_remote()
        || (!env_force_local_kernel() && loopback_kernel_healthy(port).await);

    if attach_auto {
        let attach = KernelAttach::new(format!("http://127.0.0.1:{port}"));
        tracing::info!(
            target: "oclive_desktop",
            %port,
            "remote kernel attach mode (existing HTTP API)"
        );
        let roles_dir = crate::state::resolve_roles_dir(None);
        let llm = Arc::new(MockLlmClient {
            reply: String::new(),
        });
        let shell = AppState::new_in_memory_with_llm(llm, roles_dir).await?;
        return Ok((Arc::new(shell), Some(attach), port));
    }

    let app_dir = resolve_app_data_dir_for_host();
    ensure_app_data_dir(&app_dir)?;
    app_data_migration::ensure_canonical_app_data_ready(&app_dir)?;
    let db_path = resolve_db_path(&app_dir);
    let roles_dir = crate::state::resolve_roles_dir(None);
    tracing::info!(
        target: "oclive_desktop",
        app_data = %app_dir.display(),
        db = %db_path.display(),
        "desktop canonical app data"
    );
    let app_state = Arc::new(
        AppState::new(&db_path, Some(roles_dir), &app_dir)
            .await
            .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { Box::new(e) })?,
    );

    let state_bind = Arc::clone(&app_state);
    tauri::async_runtime::spawn(async move {
        if let Err(e) = http_api::serve_api_with_state(state_bind, port).await {
            tracing::warn!(
                target: "oclive_api",
                %port,
                error = %e,
                "in-process HTTP API stopped or failed to bind"
            );
        }
    });

    Ok((app_state, None, port))
}
