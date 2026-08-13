//! Spawn `oclive-kernel-server --api` with canonical cross-host env.

use crate::kernel_lifecycle::connection::KernelConnection;
use oclive_kernel_host::domain::host_profile::{
    load_host_profile_from_env, HostProfile, ENV_DISTRO_ID, ENV_DISTRO_PROFILE,
};
use oclive_kernel_runtime::{
    ensure_app_data_dir, find_app_data_dir_for_host, parse_distro_requirements_file,
    KernelCandidate, KernelTier, ENV_HTTP_API_MOCK_LLM, ENV_ROLES_DIR,
};
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Duration;
use uuid::Uuid;

const HEALTH_POLL_MS: u64 = 500;
// Cold Windows starts can spend more than 15 seconds in SQLite verification
// and DLL initialization before the loopback listener is available.
const HEALTH_POLL_MAX: u32 = 60;
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

async fn wait_for_spawn_health(conn: &KernelConnection) -> bool {
    for _ in 0..HEALTH_POLL_MAX {
        if crate::kernel_attach::KernelHttpClient::probe_health(&conn.base_url).await {
            return true;
        }
        if conn.try_wait_spawned_child() {
            return false;
        }
        tokio::time::sleep(Duration::from_millis(HEALTH_POLL_MS)).await;
    }
    false
}

fn spawn_env(
    port: u16,
    roles_dir: &Path,
    app_data: &Path,
    kernel_binary: &Path,
    profile_override: Option<&Path>,
    api_token: Option<&str>,
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
    if let Some(parent) = kernel_binary.parent() {
        let migrations = parent.join("migrations");
        if oclive_kernel_host::infrastructure::sql_migrate::is_migrations_dir(&migrations) {
            pairs.push((
                oclive_kernel_host::infrastructure::sql_migrate::ENV_MIGRATIONS_DIR.into(),
                migrations.to_string_lossy().into_owned(),
            ));
        }
    }
    if let Some(token) = api_token.filter(|token| !token.trim().is_empty()) {
        pairs.push((
            oclive_kernel_host::http_api::ENV_API_TOKEN.into(),
            token.to_string(),
        ));
    }
    if std::env::var(ENV_HTTP_API_MOCK_LLM)
        .ok()
        .is_some_and(|v| v == "1" || v.eq_ignore_ascii_case("true"))
    {
        pairs.push((ENV_HTTP_API_MOCK_LLM.into(), "1".into()));
    }
    append_distro_env(&mut pairs, &load_host_profile_from_env(), profile_override);
    pairs
}

/// Best-effort persistence of the loopback API token next to the shared kernel binary.
/// A later app session that attaches to a leftover kernel reads it back instead of
/// failing every protected call with 401.
pub(super) fn persist_api_token(token: &str) {
    let dir = oclive_kernel_runtime::shared_runtime_dir();
    if let Err(e) = std::fs::create_dir_all(&dir) {
        tracing::warn!(target: "oclive_desktop", error = %e, "cannot create shared runtime dir for api token");
        return;
    }
    if let Err(e) = std::fs::write(dir.join("api-token"), token) {
        tracing::warn!(target: "oclive_desktop", error = %e, "failed to persist kernel api token");
    }
}

fn append_distro_env(
    pairs: &mut Vec<(String, String)>,
    host: &HostProfile,
    profile_override: Option<&Path>,
) {
    let distro_id = profile_override
        .filter(|p| p.is_file())
        .and_then(|p| parse_distro_requirements_file(p).ok())
        .map(|req| req.distro_id)
        .or_else(|| {
            if host.distro_id != "default" {
                Some(host.distro_id.clone())
            } else {
                None
            }
        })
        .unwrap_or_else(|| "desktop".into());
    pairs.push((ENV_DISTRO_ID.into(), distro_id));
    if let Some(p) = profile_override.filter(|p| p.is_file()) {
        pairs.push((ENV_DISTRO_PROFILE.into(), p.to_string_lossy().into_owned()));
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
    let app_data = find_app_data_dir_for_host();
    ensure_app_data_dir(&app_data).map_err(|e| e.to_string())?;
    oclive_kernel_host::infrastructure::app_data_migration::ensure_canonical_app_data_ready(
        &app_data,
    )
    .map_err(|e| e.to_string())?;

    let api_token = conn.api_token_snapshot().unwrap_or_else(|| {
        let token = Uuid::new_v4().to_string();
        conn.set_api_token(token.clone());
        token
    });
    persist_api_token(&api_token);

    let mut cmd = Command::new(&candidate.binary);
    cmd.args(&candidate.extra_args)
        .arg("--api")
        .arg("--port")
        .arg(port.to_string())
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    for (k, v) in spawn_env(
        port,
        roles_dir,
        &app_data,
        &candidate.binary,
        distro_profile_override,
        Some(&api_token),
    ) {
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

    if wait_for_spawn_health(conn).await {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spawn_env_includes_migrations_next_to_kernel() {
        let temp = tempfile::tempdir().expect("tempdir");
        let migrations = temp.path().join("migrations");
        std::fs::create_dir_all(&migrations).expect("create migrations");
        std::fs::write(migrations.join("001_init.sql"), "CREATE TABLE t (id INT);")
            .expect("write migration");
        let binary = temp.path().join(if cfg!(windows) {
            "oclive-kernel-server.exe"
        } else {
            "oclive-kernel-server"
        });

        let env = spawn_env(
            8420,
            Path::new("roles"),
            Path::new("app-data"),
            &binary,
            None,
            None,
        );
        let configured = env
            .iter()
            .find(|(key, _)| {
                key == oclive_kernel_host::infrastructure::sql_migrate::ENV_MIGRATIONS_DIR
            })
            .map(|(_, value)| value.as_str());
        assert_eq!(configured, Some(migrations.to_string_lossy().as_ref()));
    }
}
