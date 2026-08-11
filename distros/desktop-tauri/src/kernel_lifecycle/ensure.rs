//! Attach-first fallback when shared policy execution fails.

use super::connection::{DesktopKernelMode, SharedKernelConnection};
use super::policy::{find_desktop_distro_profile_path, KernelBringUpOptions};
use super::spawn::{probe_existing_kernel, spawn_kernel};
use oclive_kernel_runtime::{
    apply_promote_to_candidate, discover_spawn_kernel_candidates, parse_distro_requirements_file,
    pick_best_for_spawn,
};
use std::path::PathBuf;

/// Bootstrap inputs for [`ensure_kernel_ready`].
pub struct EnsureKernelOptions {
    pub port: u16,
    pub roles_dir: PathBuf,
    pub anchors: Vec<PathBuf>,
    pub bundled_binary: Option<PathBuf>,
}

/// Policy-first bring-up (shared Rust SSOT); legacy attach-first on failure.
pub async fn ensure_kernel_ready(
    opts: EnsureKernelOptions,
) -> Result<SharedKernelConnection, String> {
    let distro_profile_path = find_desktop_distro_profile_path(&opts.anchors);
    let caller_distro_id = distro_profile_path
        .as_ref()
        .and_then(|p| parse_distro_requirements_file(p).ok())
        .map(|req| req.distro_id)
        .unwrap_or_else(|| "desktop".into());
    super::policy::ensure_kernel_with_policy(KernelBringUpOptions {
        port: opts.port,
        roles_dir: opts.roles_dir,
        anchors: opts.anchors,
        bundled_binary: opts.bundled_binary,
        caller_distro_id: Some(caller_distro_id),
        distro_profile_path,
        promote_shared: true,
    })
    .await
}

/// Legacy attach-first path used as fallback from [`super::policy`].
pub(super) async fn ensure_kernel_ready_legacy_on_conn(
    conn: SharedKernelConnection,
    opts: KernelBringUpOptions,
) -> Result<SharedKernelConnection, String> {
    let base_url = conn.base_url.clone();
    if super::policy::try_profile_aware_attach(&conn, &opts).await {
        return Ok(conn);
    }
    if probe_existing_kernel(&base_url).await {
        if super::policy::attach_auth_ok(&conn, &base_url).await {
            conn.set_mode(DesktopKernelMode::Attached);
            conn.clear_status_hint();
            tracing::info!(
                target: "oclive_desktop",
                port = opts.port,
                "legacy attach-first: attached to existing kernel"
            );
            return Ok(conn);
        }
        tracing::warn!(
            target: "oclive_desktop",
            port = opts.port,
            "existing kernel rejected our token; replacing stale kernel"
        );
        if super::policy::replace_stale_kernel(&conn, &opts).await {
            return Ok(conn);
        }
    }

    let candidates =
        discover_spawn_kernel_candidates(&opts.anchors, None, opts.bundled_binary.as_deref());
    let Some(best) = pick_best_for_spawn(&candidates) else {
        conn.set_mode(DesktopKernelMode::Offline);
        return Err(format!(
            "no kernel on :{} and no spawn binary found (build oclive-kernel-server or set OCLIVE_KERNEL_BINARY)",
            opts.port
        ));
    };

    let mut candidate = best.clone();
    apply_promote_to_candidate(&mut candidate);

    match spawn_kernel(
        &conn,
        &candidate,
        opts.port,
        &opts.roles_dir,
        opts.distro_profile_path.as_deref(),
    )
    .await
    {
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
