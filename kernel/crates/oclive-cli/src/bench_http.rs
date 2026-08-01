//! Shared HTTP contract helpers for CLI benchmark modes.

use anyhow::{bail, Context, Result};
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Resolve a real role-pack directory for `/chat` benchmark requests.
///
/// # Errors
///
/// Returns an error when the generated/project role roots contain no role directory.
pub fn resolve_bench_role_path(root: &Path) -> Result<PathBuf> {
    let role_roots = [root.join("roles"), root.join("distros/chat-pro/roles")];
    for role_root in role_roots {
        let preferred = role_root.join("default");
        if preferred.is_dir() {
            return Ok(preferred);
        }
        if !role_root.is_dir() {
            continue;
        }
        let mut roles = std::fs::read_dir(&role_root)
            .with_context(|| format!("read benchmark roles at {}", role_root.display()))?
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| path.is_dir())
            .collect::<Vec<_>>();
        roles.sort();
        if let Some(role) = roles.into_iter().next() {
            return Ok(role);
        }
    }
    bail!(
        "benchmark requires a role directory under {}/roles or {}/distros/chat-pro/roles",
        root.display(),
        root.display()
    )
}

/// POST one message using the current headless `/chat` contract and return `reply`.
///
/// # Errors
///
/// Returns transport, HTTP, response-body, or response-contract failures with context.
pub fn post_chat(port: u16, role_path: &Path, message: &str, timeout: Duration) -> Result<String> {
    let url = format!("http://127.0.0.1:{port}/chat");
    let body = serde_json::json!({
        "role_path": role_path.to_string_lossy(),
        "message": message,
        "scene_id": "default"
    });
    let response = crate::http_client::post(&url)
        .set("Content-Type", "application/json")
        .timeout(timeout)
        .send_string(&body.to_string())
        .with_context(|| format!("POST {url}"))?;
    let status = response.status();
    let text = response
        .into_string()
        .context("decode benchmark /chat response")?;
    if status >= 400 {
        let body: String = text.chars().take(500).collect();
        bail!("chat HTTP {status}: {body}");
    }
    let value: serde_json::Value =
        serde_json::from_str(&text).context("parse benchmark /chat JSON response")?;
    value
        .get("reply")
        .and_then(serde_json::Value::as_str)
        .map(str::to_string)
        .context("benchmark /chat response missing string field `reply`")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn role_resolution_prefers_default_then_sorted_role() {
        let dir = tempfile::tempdir().expect("temp root");
        let roles = dir.path().join("roles");
        std::fs::create_dir_all(roles.join("zeta")).expect("zeta role");
        std::fs::create_dir_all(roles.join("alpha")).expect("alpha role");
        assert_eq!(
            resolve_bench_role_path(dir.path()).expect("fallback role"),
            roles.join("alpha")
        );
        std::fs::create_dir_all(roles.join("default")).expect("default role");
        assert_eq!(
            resolve_bench_role_path(dir.path()).expect("default role"),
            roles.join("default")
        );
    }
}
