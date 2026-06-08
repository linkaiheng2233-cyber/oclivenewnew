//! Capability-first kernel bring-up via shared Rust policy (`resolve_kernel_action`).

use super::connection::{DesktopKernelMode, KernelConnection, SharedKernelConnection};
use super::port_ops::{is_known_distribution_kernel, terminate_listeners_on_port};
use super::spawn::{spawn_kernel, wait_for_health};
use crate::kernel_attach::{KernelHealthJson, KernelHttpClient};
use oclive_kernel_runtime::{
    apply_promote_to_candidate, build_resolve_plan, discover_spawn_kernel_candidates,
    promote_with_backup, PolicyContext, KernelActionKind, KernelActionPlan, KernelBinaryManifest,
    ReplaceReason, ENV_DISTRO_PROFILE,
};
use oclive_kernel_types::AttachReason;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

/// Shared inputs for policy-based bring-up (desktop + reconnect).
#[derive(Debug, Clone)]
pub struct KernelBringUpOptions {
    pub port: u16,
    pub roles_dir: PathBuf,
    pub anchors: Vec<PathBuf>,
    pub bundled_binary: Option<PathBuf>,
    pub caller_distro_id: Option<String>,
    pub distro_profile_path: Option<PathBuf>,
    pub promote_shared: bool,
}

fn kernel_binary_pinned() -> bool {
    std::env::var("OCLIVE_KERNEL_BINARY")
        .ok()
        .is_some_and(|s| !s.trim().is_empty())
}

fn policy_context_from_health(health: Option<&KernelHealthJson>) -> PolicyContext {
    let Some(h) = health else {
        return PolicyContext {
            health_ok: false,
            running_manifest: None,
            running_distro_id: None,
            running_profile: None,
            running_profile_hash: None,
        };
    };
    PolicyContext {
        health_ok: h.ok,
        running_manifest: h.kernel_manifest.clone(),
        running_distro_id: h.distro_id.clone(),
        running_profile: h.active_profile_summary.clone(),
        running_profile_hash: h.distro_profile_hash.clone(),
    }
}

async fn build_plan_async(opts: &KernelBringUpOptions, base_url: &str) -> KernelActionPlan {
    let health = KernelHttpClient::fetch_health_json(base_url).await;
    let ctx = policy_context_from_health(health.as_ref());
    let candidates =
        discover_spawn_kernel_candidates(&opts.anchors, None, opts.bundled_binary.as_deref());
    let allow_replace = is_known_distribution_kernel(opts.port, &candidates);
    let distro_id = opts
        .caller_distro_id
        .as_deref()
        .unwrap_or("desktop");

    build_resolve_plan(
        &ctx,
        &candidates,
        distro_id,
        opts.distro_profile_path.as_deref(),
        kernel_binary_pinned(),
        allow_replace,
        opts.promote_shared,
    )
    .plan
}

fn apply_attach_status_hint(conn: &KernelConnection, plan: &KernelActionPlan) {
    match plan.attach_reason {
        Some(AttachReason::KernelPinnedProfileMismatch) => {
            conn.set_status_hint(
                true,
                Some(
                    "Kernel binary is pinned but its profile does not match desktop requirements"
                        .into(),
                ),
            );
        }
        Some(AttachReason::ProfileMismatchNoReplace) => {
            conn.set_status_hint(
                true,
                Some(
                    "Running kernel profile does not match desktop requirements (replace not allowed)"
                        .into(),
                ),
            );
        }
        Some(AttachReason::LegacyFallback) => {
            conn.set_status_hint(
                true,
                Some("Attached via legacy fallback; profile may not match".into()),
            );
        }
        _ => conn.clear_status_hint(),
    }
}

pub(super) async fn try_profile_aware_attach(
    conn: &KernelConnection,
    opts: &KernelBringUpOptions,
) -> bool {
    let health = KernelHttpClient::fetch_health_json(&conn.base_url).await;
    if !health.as_ref().is_some_and(|h| h.ok) {
        return false;
    }

    let ctx = policy_context_from_health(health.as_ref());

    let candidates =
        discover_spawn_kernel_candidates(&opts.anchors, None, opts.bundled_binary.as_deref());
    let distro_id = opts.caller_distro_id.as_deref().unwrap_or("desktop");
    let resolution = build_resolve_plan(
        &ctx,
        &candidates,
        distro_id,
        opts.distro_profile_path.as_deref(),
        kernel_binary_pinned(),
        false,
        opts.promote_shared,
    );

    if resolution.plan.action != KernelActionKind::Attach {
        return false;
    }

    conn.set_mode(DesktopKernelMode::Attached);
    let mut plan = resolution.plan;
    if plan.attach_reason.is_none() {
        plan.attach_reason = Some(AttachReason::LegacyFallback);
    }
    apply_attach_status_hint(conn, &plan);
    tracing::info!(
        target: "oclive_desktop",
        reason = ?plan.attach_reason,
        compat = ?resolution.profile_compat,
        "profile-aware attach fallback"
    );
    true
}

async fn execute_plan(
    conn: &KernelConnection,
    opts: &KernelBringUpOptions,
    plan: &KernelActionPlan,
) -> Result<(), String> {
    match plan.action {
        KernelActionKind::Attach => {
            conn.set_mode(DesktopKernelMode::Attached);
            apply_attach_status_hint(conn, plan);
            tracing::info!(
                target: "oclive_desktop",
                reason = ?plan.attach_reason,
                replace_reason = ?plan.replace_reason,
                "kernel policy: attach"
            );
            Ok(())
        }
        KernelActionKind::ReplaceAndAttach
        | KernelActionKind::SpawnBest
        | KernelActionKind::FallbackBundled => spawn_from_plan(conn, opts, plan).await,
    }
}

async fn spawn_from_plan(
    conn: &KernelConnection,
    opts: &KernelBringUpOptions,
    plan: &KernelActionPlan,
) -> Result<(), String> {
    let Some(sel) = plan.candidate.as_ref() else {
        return Err("kernel policy spawn action missing candidate".into());
    };

    let candidates =
        discover_spawn_kernel_candidates(&opts.anchors, None, opts.bundled_binary.as_deref());
    let Some(mut candidate) = candidates
        .into_iter()
        .find(|c| c.binary.display().to_string() == sel.binary)
    else {
        return Err(format!("candidate not found: {}", sel.binary));
    };

    if sel.promote_to_shared {
        let manifest = KernelBinaryManifest::read_sidecar(&candidate.binary);
        let _ = promote_with_backup(&candidate.binary, manifest.as_ref());
        apply_promote_to_candidate(&mut candidate);
    }

    if matches!(plan.action, KernelActionKind::ReplaceAndAttach) {
        conn.kill_spawned_child();
        terminate_listeners_on_port(opts.port);
        sleep(Duration::from_millis(400)).await;
        if plan.replace_reason == Some(ReplaceReason::ProfileMismatch) {
            tracing::info!(
                target: "oclive_desktop",
                "kernel policy: replacing running kernel due to profile mismatch"
            );
        }
    }

    if plan.degraded || sel.degraded {
        let msg = plan
            .degrade_reason
            .clone()
            .or_else(|| sel.degrade_reason.clone())
            .unwrap_or_else(|| "using bundled fallback kernel".into());
        conn.set_status_hint(true, Some(msg));
        tracing::warn!(
            target: "oclive_desktop",
            tier = ?candidate.tier,
            "kernel policy: degraded spawn"
        );
    } else {
        conn.clear_status_hint();
    }

    spawn_kernel(
        conn,
        &candidate,
        opts.port,
        &opts.roles_dir,
        opts.distro_profile_path.as_deref(),
    )
    .await?;
    conn.set_mode(DesktopKernelMode::Spawned);
    Ok(())
}

/// Resolve desktop `distro.oclive.toml` for policy + spawn env.
///
/// Priority: `OCLIVE_DISTRO_PROFILE` → `{anchor}/distro.oclive.toml` → monorepo example.
#[must_use]
pub fn resolve_desktop_distro_profile_path(anchors: &[PathBuf]) -> Option<PathBuf> {
    if let Ok(p) = std::env::var(ENV_DISTRO_PROFILE) {
        let path = PathBuf::from(p.trim());
        if path.is_file() {
            return Some(path);
        }
    }
    for anchor in anchors {
        let at_root = anchor.join("distro.oclive.toml");
        if at_root.is_file() {
            return Some(at_root);
        }
    }
    for anchor in anchors {
        let dev = anchor.join("examples/distro-profiles/desktop.oclive.toml");
        if dev.is_file() {
            return Some(dev);
        }
    }
    None
}

/// Policy-first bring-up; graded fallback on failure.
pub async fn ensure_kernel_with_policy(
    opts: KernelBringUpOptions,
) -> Result<SharedKernelConnection, String> {
    let base_url = format!("http://127.0.0.1:{}", opts.port);
    let conn = Arc::new(KernelConnection::new(base_url.clone(), opts.port));

    tracing::info!(
        target: "oclive_desktop",
        port = opts.port,
        "kernel policy bring-up"
    );

    let plan = build_plan_async(&opts, &base_url).await;
    if let Err(e) = execute_plan(&conn, &opts, &plan).await {
        tracing::warn!(
            target: "oclive_desktop",
            error = %e,
            "kernel policy execution failed; trying profile-aware attach"
        );
        if try_profile_aware_attach(&conn, &opts).await {
            return Ok(conn);
        }
        return super::ensure::ensure_kernel_ready_legacy_on_conn(conn, opts).await;
    }

    if !wait_for_health(&base_url).await && !matches!(plan.action, KernelActionKind::Attach) {
        tracing::warn!(
            target: "oclive_desktop",
            "policy spawn did not pass health; trying profile-aware attach"
        );
        if try_profile_aware_attach(&conn, &opts).await {
            return Ok(conn);
        }
        return super::ensure::ensure_kernel_ready_legacy_on_conn(conn, opts).await;
    }

    Ok(conn)
}

/// Reconnect cycle using shared policy (same as ensure).
pub async fn reconnect_with_policy(
    conn: &KernelConnection,
    opts: &KernelBringUpOptions,
) -> Result<(), String> {
    conn.kill_spawned_child();
    let plan = build_plan_async(opts, &conn.base_url).await;
    execute_plan(conn, opts, &plan).await?;
    if wait_for_health(&conn.base_url).await {
        Ok(())
    } else if KernelHttpClient::probe_health(&conn.base_url).await {
        conn.set_mode(DesktopKernelMode::Attached);
        Ok(())
    } else {
        Err("kernel policy reconnect: /health not ready".into())
    }
}
