//! Attach-first kernel bring-up for the desktop host.

use super::connection::{DesktopKernelMode, KernelConnection, SharedKernelConnection};
use super::spawn::{probe_existing_kernel, spawn_kernel};
use oclive_kernel_runtime::{
    apply_promote_to_candidate, discover_spawn_kernel_candidates, pick_best_kernel,
};
use std::path::PathBuf;
use std::sync::Arc;

/// Bootstrap inputs for [`ensure_kernel_ready`].
pub struct EnsureKernelOptions {
    pub port: u16,
    pub roles_dir: PathBuf,
    pub anchors: Vec<PathBuf>,
    pub bundled_binary: Option<PathBuf>,
}
/// Attach to an existing loopback kernel or spawn one.
///
/// # Errors
///
/// Returns a human-readable message when no kernel is reachable and spawn fails.
pub async fn ensure_kernel_ready(
    opts: EnsureKernelOptions,
) -> Result<SharedKernelConnection, String> {
    let base_url = format!("http://127.0.0.1:{}", opts.port);
    let conn = Arc::new(KernelConnection::new(base_url.clone(), opts.port));

    tracing::info!(
        target: "oclive_desktop",
        port = opts.port,
        "probing for existing kernel on loopback"
    );

    if probe_existing_kernel(&base_url).await {
        conn.set_mode(DesktopKernelMode::Attached);
        tracing::info!(
            target: "oclive_desktop",
            port = opts.port,
            "attached to existing kernel on loopback"
        );
        return Ok(conn);
    }

    let candidates =
        discover_spawn_kernel_candidates(&opts.anchors, None, opts.bundled_binary.as_deref());
    let Some(best) = pick_best_kernel(&candidates) else {
        conn.set_mode(DesktopKernelMode::Offline);
        return Err(format!(
            "no kernel on :{} and no spawn binary found (build oclive-kernel-server or set OCLIVE_KERNEL_BINARY)",
            opts.port
        ));
    };

    let mut candidate = best.clone();
    apply_promote_to_candidate(&mut candidate);

    tracing::info!(
        target: "oclive_desktop",
        binary = %candidate.binary.display(),
        tier = ?candidate.tier,
        port = opts.port,
        "spawning local kernel"
    );

    match spawn_kernel(&conn, &candidate, opts.port, &opts.roles_dir).await {
        Ok(()) => {
            conn.set_mode(DesktopKernelMode::Spawned);
            Ok(conn)
        }
        Err(e) => {
            conn.set_mode(DesktopKernelMode::Offline);
            Err(e)
        }
    }
}
