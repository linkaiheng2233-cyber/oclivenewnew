//! `oclive test --oocp`: automatically launch the kernel, run the OOCP black-box suite, and clean up processes.

use anyhow::{bail, Context, Result};
use oclive_kernel_runtime::{resolve_project_roles_dir, DEFAULT_API_PORT};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

const HEALTH_TIMEOUT_SECS: u64 = 30;
const HEALTH_POLL_MS: u64 = 500;

pub fn run_oocp_integration(repo_root: &Path) -> Result<()> {
    let script = find_oocp_runner(repo_root).ok_or_else(|| {
        anyhow::anyhow!(
            "examples/oocp-test-suite/run.mjs not found under {}",
            repo_root.display()
        )
    })?;
    let mut child = spawn_kernel_api(repo_root)?;
    let base_url = kernel_base_url();
    let health = format!("{base_url}/health");
    eprintln!("oclive test --oocp: waiting for {health} (timeout {HEALTH_TIMEOUT_SECS}s)…");
    match wait_for_health(&health) {
        Ok(()) => {
            let st = Command::new("node")
                .arg(&script)
                .current_dir(repo_root)
                .env("OCLIVE_API_BASE", &base_url)
                .env("OCLIVE_HTTP_API_MOCK_LLM", "1")
                .env("OCLIVE_API_TOKEN", crate::http_client::api_token())
                .status()
                .context("run OOCP suite")?;
            terminate_child(&mut child);
            if st.success() {
                eprintln!("oclive test --oocp: all OOCP scenarios passed");
                Ok(())
            } else {
                bail!("OOCP suite failed (exit {:?})", st.code());
            }
        }
        Err(e) => {
            terminate_child(&mut child);
            Err(e)
        }
    }
}

fn spawn_kernel_api(repo_root: &Path) -> Result<Child> {
    let manifest = repo_root.join("distros/desktop-tauri/Cargo.toml");
    if !manifest.is_file() {
        bail!(
            "distros/desktop-tauri/Cargo.toml not found under {}; run from oclivenewnew root or set -o",
            repo_root.display()
        );
    }
    Command::new("cargo")
        .args([
            "run",
            "--release",
            "--manifest-path",
            manifest
                .to_str()
                .unwrap_or("distros/desktop-tauri/Cargo.toml"),
            "--",
            "--api",
        ])
        .current_dir(repo_root)
        .env("OCLIVE_HTTP_API_MOCK_LLM", "1")
        .env("OCLIVE_API_TOKEN", crate::http_client::api_token())
        .env(
            "OCLIVE_ROLES_DIR",
            resolve_project_roles_dir(repo_root)
                .to_string_lossy()
                .into_owned(),
        )
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .context("spawn oclivenewnew-tauri --api")
}

fn kernel_base_url() -> String {
    std::env::var("OCLIVE_API_BASE")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| format!("http://127.0.0.1:{DEFAULT_API_PORT}"))
}

fn wait_for_health(url: &str) -> Result<()> {
    let deadline = Instant::now() + Duration::from_secs(HEALTH_TIMEOUT_SECS);
    while Instant::now() < deadline {
        if health_ok(url) {
            return Ok(());
        }
        std::thread::sleep(Duration::from_millis(HEALTH_POLL_MS));
    }
    bail!("kernel did not become healthy within {HEALTH_TIMEOUT_SECS}s ({url})");
}

fn health_ok(url: &str) -> bool {
    let Ok(resp) = crate::http_client::get(url).call() else {
        return false;
    };
    resp.status() == 200
}

fn terminate_child(child: &mut Child) {
    let _ = child.kill();
    let _ = child.wait();
}

fn find_oocp_runner(root: &Path) -> Option<PathBuf> {
    let p = root.join("examples/oocp-test-suite/run.mjs");
    if p.is_file() {
        return Some(p);
    }
    if let Ok(env) = std::env::var("OCLIVE_ROOT") {
        let p = PathBuf::from(env).join("examples/oocp-test-suite/run.mjs");
        if p.is_file() {
            return Some(p);
        }
    }
    None
}
