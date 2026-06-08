//! Spawn `oclive-kernel-server --api` with canonical cross-host env.

use oclive_kernel_host::domain::host_profile::{
    load_host_profile_from_env, HostProfile, ENV_DISTRO_ID, ENV_DISTRO_PROFILE,
};
use crate::kernel_lifecycle::connection::KernelConnection;
use oclive_kernel_runtime::{
    ensure_app_data_dir, resolve_app_data_dir_for_host, KernelCandidate, KernelTier,
    ENV_HTTP_API_MOCK_LLM, ENV_ROLES_DIR,
};
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Duration;

const HEALTH_POLL_MS: u64 = 500;
const HEALTH_POLL_MAX: u32 = 30;
const ATTACH_PROBE_MS: u64 = 300;
const ATTACH_PROBE_MAX: u32 = 6;
const ATTACH_POLL_MS: u64 = 100;

/// Fast attach probe before spawn (avoid blocking Tauri setup on an idle port).
pub async fn probe_existing_kernel(base_url: &str) -> bool {
    for _ in 0..ATTACH_PROBE_MAX {
        if crate::kernel_attach::KernelHttpClient::probe_health_timeout(
            base_url,
            Duration::from_millis(ATTACH_PROBE_MS),
        )
        .await
        {
            return true;
        }
        tokio::time::sleep(Duration::from_millis(ATTACH_POLL_MS)).await;
    }
    false
}

/// Poll `GET /health` until success or timeout (post-spawn readiness).
pub async fn wait_for_health(base_url: &str) -> bool {
    for _ in 0..HEALTH_POLL_MAX {
        if crate::kernel_attach::KernelHttpClient::probe_health(base_url).await {
            return true;
        }
        tokio::time::sleep(Duration::from_millis(HEALTH_POLL_MS)).await;
    }
    false
}

fn spawn_env(
    port: u16,
    roles_dir: &Path,
    app_data: &Path,
    profile_override: Option<&Path>,
) -> Vec<(String, String)> {
    let mut pairs = vec![
        ("OCLIVE_API_PORT".into(), port.to_string()),
        (
            "OCLIVE_APP_DATA".into(),
            app_data.to_string_lossy().into_owned(),
        ),
        ("OCLIVE_USE_CANONICAL_APP_DATA".into(), "1".into()),
        (
            ENV_ROLES_DIR.into(),
            roles_dir.to_string_lossy().into_owned(),
        ),
    ];
    if std::env::var(ENV_HTTP_API_MOCK_LLM)
        .ok()
        .is_some_and(|v| v == "1" || v.eq_ignore_ascii_case("true"))
    {
        pairs.push((ENV_HTTP_API_MOCK_LLM.into(), "1".into()));
    }
    append_distro_env(&mut pairs, &load_host_profile_from_env(), profile_override);
    pairs
}

fn append_distro_env(
    pairs: &mut Vec<(String, String)>,
    host: &HostProfile,
    profile_override: Option<&Path>,
) {
    let distro_id = if host.distro_id != "default" {
        host.distro_id.clone()
    } else {
        "desktop".into()
    };
    pairs.push((ENV_DISTRO_ID.into(), distro_id));
    if let Some(p) = profile_override.filter(|p| p.is_file()) {
        pairs.push((
            ENV_DISTRO_PROFILE.into(),
            p.to_string_lossy().into_owned(),
        ));
    } else if let Some(ref p) = host.profile_path {
        pairs.push((ENV_DISTRO_PROFILE.into(), p.to_string_lossy().into_owned()));
    } else if let Ok(p) = std::env::var(ENV_DISTRO_PROFILE) {
        let t = p.trim().to_string();
        if !t.is_empty() {
            pairs.push((ENV_DISTRO_PROFILE.into(), t));
        }
    }
}

/// Spawn kernel binary; store child on `conn` when successful.
///
/// # Errors
///
/// Returns a human-readable message when spawn or health poll fails.
pub async fn spawn_kernel(
    conn: &KernelConnection,
    candidate: &KernelCandidate,
    port: u16,
    roles_dir: &Path,
    distro_profile_override: Option<&Path>,
) -> Result<(), String> {
    let app_data = resolve_app_data_dir_for_host();
    ensure_app_data_dir(&app_data).map_err(|e| e.to_string())?;
    oclive_kernel_host::infrastructure::app_data_migration::ensure_canonical_app_data_ready(&app_data)
        .map_err(|e| e.to_string())?;

    let mut cmd = Command::new(&candidate.binary);
    cmd.args(&candidate.extra_args)
        .arg("--api")
        .arg("--port")
        .arg(port.to_string())
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    for (k, v) in spawn_env(port, roles_dir, &app_data, distro_profile_override) {
        cmd.env(k, v);
    }

    let child = cmd
        .spawn()
        .map_err(|e| format!("spawn {}: {e}", candidate.binary.display()))?;

    conn.set_spawn_metadata(
        candidate.binary.display().to_string(),
        candidate.tier,
        child,
    );

    if wait_for_health(&conn.base_url).await {
        Ok(())
    } else {
        conn.kill_spawned_child();
        Err(format!(
            "spawned {} but /health did not become ready on port {port}",
            candidate.binary.display()
        ))
    }
}

#[must_use]
pub fn tier_label(tier: KernelTier) -> &'static str {
    match tier {
        KernelTier::Shared => "shared",
        KernelTier::DevFull => "dev-full",
        KernelTier::DevHeadless => "dev-headless",
        KernelTier::Bundled => "bundled",
        KernelTier::Settings => "settings",
        KernelTier::Env => "env",
    }
}
