//! `oclive lint --deny` — cargo-deny license and duplicate-dependency checks.

use super::lint::{fail, pass, warn};
use anyhow::{bail, Result};
use std::path::Path;
use std::process::Command;

pub(super) fn run_deny_check(root: &Path, json: bool) -> Result<()> {
    let mut items = Vec::new();
    let deny_bin = Command::new("cargo-deny").arg("--version").output();
    if deny_bin.is_err()
        || !deny_bin
            .as_ref()
            .map(|o| o.status.success())
            .unwrap_or(false)
    {
        let msg = "cargo-deny not installed. Install: cargo install cargo-deny";
        if json {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!([{
                    "level": "warn", "check": "cargo_deny", "message": msg
                }]))?
            );
        } else {
            println!("oclive lint --deny — {}", root.display());
            println!("  [WARN] {msg}");
        }
        return Ok(());
    }

    let deny_toml = root.join("deny.toml");
    if !deny_toml.is_file() {
        items.push(warn(
            "deny_toml",
            "no deny.toml at project root".to_string(),
            Some("copy deny.toml from oclivenewnew root or run: cargo deny init".into()),
        ));
    }

    for (check, label) in [("licenses", "license compliance"), ("bans", "duplicate deps / bans")] {
        let st = Command::new("cargo")
            .args(["deny", "check", check])
            .current_dir(root)
            .status();
        match st {
            Ok(s) if s.success() => {
                items.push(pass(
                    &format!("deny_{check}"),
                    &format!("{label}: passed"),
                    None,
                ));
            }
            Ok(s) => items.push(fail(
                &format!("deny_{check}"),
                format!("{label}: exit {:?}", s.code()),
                Some(format!("cargo deny check {check}  # fix in deny.toml")),
            )),
            Err(e) => items.push(fail(
                &format!("deny_{check}"),
                format!("{label}: {e}"),
                Some("cargo install cargo-deny".into()),
            )),
        }
    }

    if json {
        println!("{}", serde_json::to_string_pretty(&items)?);
        return Ok(());
    }
    println!("oclive lint --deny — {}", root.display());
    for it in &items {
        let icon = match it.level.as_str() {
            "pass" => "PASS",
            "warn" => "WARN",
            _ => "FAIL",
        };
        println!("  [{icon}] {} — {}", it.check, it.message);
        if let Some(fix) = &it.fix {
            println!("         → {fix}");
        }
    }
    if items.iter().any(|i| i.level == "fail") {
        bail!("cargo-deny check failed");
    }
    Ok(())
}
