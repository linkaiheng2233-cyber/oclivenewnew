//! Dependency audit helpers for `oclive lint --deps`.

use super::lint::{fail, pass, warn};
use anyhow::Result;
use std::path::Path;

pub(super) fn run_deps_audit(root: &Path, json: bool) -> Result<()> {
    use std::process::Command;

    let mut items = Vec::new();
    let audit_bin = Command::new("cargo-audit").arg("--version").output();
    if audit_bin.is_err()
        || !audit_bin
            .as_ref()
            .map(|o| o.status.success())
            .unwrap_or(false)
    {
        let msg = "cargo-audit not installed. Install: cargo install cargo-audit";
        if json {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!([{
                    "level": "warn", "check": "cargo_audit", "message": msg
                }]))?
            );
        } else {
            println!("oclive lint --deps — {}", root.display());
            println!("  [WARN] {msg}");
        }
        return Ok(());
    }

    let out = Command::new("cargo")
        .args(["audit", "--json"])
        .current_dir(root)
        .output();
    match out {
        Ok(o) => {
            let stdout = String::from_utf8_lossy(&o.stdout);
            if o.status.success() && stdout.trim().is_empty() {
                items.push(pass("cargo_audit", "no vulnerabilities reported", None));
            } else {
                let vuln_count = stdout.matches("\"id\":").count();
                if vuln_count == 0 && o.status.success() {
                    items.push(pass("cargo_audit", "clean", None));
                } else {
                    items.push(warn(
                        "cargo_audit",
                        "audit findings or non-zero exit (see cargo audit)".to_string(),
                        Some("cargo audit  # or upgrade deps per KNOWN_VULNERABILITIES.md".into()),
                    ));
                }
            }
        }
        Err(e) => items.push(fail(
            "cargo_audit",
            e.to_string(),
            Some("cargo install cargo-audit --version 0.22.1 --locked".into()),
        )),
    }

    let meta = Command::new("cargo")
        .args(["metadata", "--format-version", "1", "--locked"])
        .current_dir(root)
        .output();
    match meta {
        Ok(o) if o.status.success() => {
            let body = String::from_utf8_lossy(&o.stdout);
            let v: serde_json::Value = serde_json::from_str(&body).unwrap_or(serde_json::json!({}));
            let mut yanked = Vec::new();
            if let Some(pkgs) = v.get("packages").and_then(|p| p.as_array()) {
                for pkg in pkgs {
                    if pkg.get("yanked").and_then(|y| y.as_bool()) == Some(true) {
                        let name = pkg.get("name").and_then(|n| n.as_str()).unwrap_or("?");
                        let ver = pkg.get("version").and_then(|n| n.as_str()).unwrap_or("?");
                        yanked.push(format!("{name}@{ver}"));
                    }
                }
            }
            if yanked.is_empty() {
                items.push(pass(
                    "yanked",
                    "no yanked packages in lockfile metadata",
                    None,
                ));
            } else {
                items.push(fail(
                    "yanked",
                    format!("yanked: {}", yanked.join(", ")),
                    Some("cargo update -p <crate>  # pin non-yanked version in Cargo.lock".into()),
                ));
            }
        }
        _ => items.push(warn(
            "yanked",
            "cargo metadata failed",
            Some("cargo metadata --format-version 1 --locked".into()),
        )),
    }

    if json {
        println!("{}", serde_json::to_string_pretty(&items)?);
        return Ok(());
    }
    println!("oclive lint --deps — {}", root.display());
    for it in &items {
        let icon = match it.level.as_str() {
            "pass" => "PASS",
            "warn" => "WARN",
            _ => "FAIL",
        };
        println!("  [{icon}] {} — {}", it.check, it.message);
    }
    let failed = items.iter().any(|i| i.level == "fail");
    if failed {
        anyhow::bail!("dependency health check failed");
    }
    Ok(())
}
