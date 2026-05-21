//! `oclive lint` — 内核工程静态健康检查。

use anyhow::Result;
use clap::Parser;
use serde::Serialize;
use std::path::{Path, PathBuf};

#[derive(Parser, Debug)]
pub struct LintArgs {
    #[arg(short = 'o', long, default_value = ".")]
    pub path: PathBuf,
    #[arg(long)]
    pub json: bool,

    /// Dependency audit (cargo-audit) and yanked crate check via cargo metadata
    #[arg(long)]
    pub deps: bool,

    /// Check `.github/workflows/ci.yml` for cargo-audit job configuration
    #[arg(long = "audit-ci")]
    pub audit_ci: bool,
}

#[derive(Serialize, Clone)]
struct LintItem {
    level: String,
    check: String,
    message: String,
}

pub fn run(args: LintArgs) -> Result<()> {
    let root = args.path.canonicalize().unwrap_or(args.path);
    if args.audit_ci {
        return crate::lint_audit_ci::run_audit_ci(&root);
    }
    if args.deps {
        return run_deps_audit(&root, args.json);
    }
    let mut items = Vec::new();
    for (dir, name) in [
        ("src", "src/"),
        ("docs", "docs/"),
        ("roles", "roles/ (optional)"),
    ] {
        let p = root.join(dir);
        if p.is_dir() {
            items.push(pass(&format!("dir_{dir}"), &format!("found {name}")));
        } else if dir == "roles" {
            items.push(warn(&format!("dir_{dir}"), format!("missing {name}")));
        } else {
            items.push(fail(&format!("dir_{dir}"), format!("missing {name}")));
        }
    }
    lint_cargo_toml(&root, &mut items);
    lint_settings(&root, &mut items);
    lint_monolith(&root, &mut items);
    lint_git_dirty(&root, &mut items);

    if args.json {
        println!("{}", serde_json::to_string_pretty(&items)?);
        return Ok(());
    }
    println!("oclive lint — {}", root.display());
    for it in &items {
        let icon = match it.level.as_str() {
            "pass" => "✅",
            "warn" => "⚠️",
            _ => "❌",
        };
        println!("  {icon} [{}] {}", it.check, it.message);
    }
    Ok(())
}

fn lint_cargo_toml(root: &Path, items: &mut Vec<LintItem>) {
    let p = root.join("Cargo.toml");
    let Ok(raw) = std::fs::read_to_string(&p) else {
        items.push(fail("cargo_toml", "cannot read Cargo.toml"));
        return;
    };
    let v: toml::Value = match toml::from_str(&raw) {
        Ok(v) => v,
        Err(e) => {
            items.push(fail("cargo_toml", format!("parse failed: {e}")));
            return;
        }
    };
    let pkg = v.get("package").and_then(|x| x.as_table());
    for key in ["name", "version"] {
        if pkg.and_then(|t| t.get(key)).is_some() {
            items.push(pass(
                &format!("cargo_{key}"),
                &format!("[package].{key} set"),
            ));
        } else {
            items.push(fail("cargo_toml", format!("missing [package].{key}")));
        }
    }
    for key in ["authors", "license"] {
        if pkg.and_then(|t| t.get(key)).is_some() {
            items.push(pass(
                &format!("cargo_{key}"),
                &format!("[package].{key} set"),
            ));
        } else {
            items.push(warn(
                "cargo_meta",
                format!("consider setting [package].{key}"),
            ));
        }
    }
}

fn lint_settings(root: &Path, items: &mut Vec<LintItem>) {
    let roles = root.join("roles");
    if !roles.is_dir() {
        return;
    }
    for entry in walk_role_roots(&roles) {
        let settings = entry.join("settings.json");
        if !settings.is_file() {
            continue;
        }
        let raw = match std::fs::read_to_string(&settings) {
            Ok(r) => r,
            Err(_) => continue,
        };
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&raw) {
            if let Some(map) = v.as_object() {
                match oclive_validation::validate_settings_top_level_keys(map) {
                    Ok(()) => items.push(pass(
                        "settings_keys",
                        &format!("{} settings top-level keys valid", entry.display()),
                    )),
                    Err(e) => items.push(fail("settings_keys", e)),
                }
            }
        }
    }
}

fn lint_monolith(root: &Path, items: &mut Vec<LintItem>) {
    let p = root.join("monolith.toml");
    if !p.is_file() {
        items.push(warn("monolith", "no monolith.toml (standard mode)"));
        return;
    }
    match std::fs::read_to_string(&p) {
        Ok(raw) => match crate::monolith_config::parse_monolith_toml(&raw) {
            Ok(f) => {
                if let Err(e) = crate::monolith_config::validate_monolith_section(&f.monolith) {
                    items.push(fail("monolith", e.to_string()));
                } else {
                    items.push(pass("monolith", "monolith.toml format OK"));
                }
            }
            Err(e) => items.push(fail("monolith", e.to_string())),
        },
        Err(e) => items.push(fail("monolith", e.to_string())),
    }
}

fn lint_git_dirty(root: &Path, items: &mut Vec<LintItem>) {
    if !root.join(".git").exists() {
        items.push(warn("git", "not a Git repository"));
        return;
    }
    let out = std::process::Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(root)
        .output();
    match out {
        Ok(o) if o.status.success() => {
            let s = String::from_utf8_lossy(&o.stdout);
            if s.trim().is_empty() {
                items.push(pass("git", "working tree clean"));
            } else {
                items.push(warn("git", "uncommitted changes"));
            }
        }
        _ => items.push(warn("git", "cannot run git status")),
    }
}

fn walk_role_roots(roles: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    if let Ok(rd) = std::fs::read_dir(roles) {
        for e in rd.flatten() {
            let p = e.path();
            if p.join("manifest.json").is_file() {
                out.push(p);
            }
        }
    }
    out
}

fn pass(check: &str, msg: &str) -> LintItem {
    LintItem {
        level: "pass".into(),
        check: check.into(),
        message: msg.into(),
    }
}

fn warn(check: &str, msg: impl ToString) -> LintItem {
    LintItem {
        level: "warn".into(),
        check: check.into(),
        message: msg.to_string(),
    }
}

fn fail(check: &str, msg: impl ToString) -> LintItem {
    LintItem {
        level: "fail".into(),
        check: check.into(),
        message: msg.to_string(),
    }
}

fn run_deps_audit(root: &Path, json: bool) -> Result<()> {
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
                items.push(pass("cargo_audit", "no vulnerabilities reported"));
            } else {
                let vuln_count = stdout.matches("\"id\":").count();
                if vuln_count == 0 && o.status.success() {
                    items.push(pass("cargo_audit", "clean"));
                } else {
                    items.push(warn(
                        "cargo_audit",
                        "audit findings or non-zero exit (see cargo audit)".to_string(),
                    ));
                }
            }
        }
        Err(e) => items.push(fail("cargo_audit", e.to_string())),
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
                items.push(pass("yanked", "no yanked packages in lockfile metadata"));
            } else {
                items.push(fail("yanked", format!("yanked: {}", yanked.join(", "))));
            }
        }
        _ => items.push(warn("yanked", "cargo metadata failed")),
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
