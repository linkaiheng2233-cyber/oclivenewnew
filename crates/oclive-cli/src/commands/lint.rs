//! `oclive lint`: static project health checks.

use super::lint_deps::run_deps_audit;
use super::lint_deny::run_deny_check;
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

    /// License compliance and duplicate deps (`cargo deny check licenses` / `bans`)
    #[arg(long)]
    pub deny: bool,
}

#[derive(Serialize, Clone)]
pub(super) struct LintItem {
    pub(super) level: String,
    pub(super) check: String,
    pub(super) message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) fix: Option<String>,
}

/// Run static project health checks (or `--deps` / `--audit-ci` modes).
///
/// # Errors
///
/// Returns an error when any check fails or subprocess tools cannot run.
pub fn run(args: LintArgs) -> Result<()> {
    let root = args.path.canonicalize().unwrap_or(args.path);
    if args.audit_ci {
        return crate::lint_audit_ci::run_audit_ci(&root);
    }
    if args.deps {
        return run_deps_audit(&root, args.json);
    }
    if args.deny {
        return run_deny_check(&root, args.json);
    }
    let mut items = Vec::new();
    for (dir, name) in [
        ("src", "src/"),
        ("docs", "docs/"),
        ("roles", "roles/ (optional)"),
    ] {
        let p = root.join(dir);
        if p.is_dir() {
            items.push(pass(
                &format!("dir_{dir}"),
                &format!("found {name}"),
                None,
            ));
        } else if dir == "roles" {
            items.push(warn(
                &format!("dir_{dir}"),
                format!("missing {name}"),
                Some(format!(
                    "mkdir -p {}",
                    p.display()
                )),
            ));
        } else {
            items.push(fail(
                &format!("dir_{dir}"),
                format!("missing {name}"),
                Some(format!(
                    "re-run oclive init or mkdir -p {}",
                    p.display()
                )),
            ));
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
    let mut fail_n = 0u32;
    for it in &items {
        let icon = match it.level.as_str() {
            "pass" => "✅",
            "warn" => "⚠️",
            _ => {
                fail_n += 1;
                "❌"
            }
        };
        println!("  {icon} [{}] {}", it.check, it.message);
        if let Some(ref fix) = it.fix {
            println!("      → {fix}");
        }
    }
    if fail_n > 0 {
        anyhow::bail!("lint: {fail_n} failed check(s)");
    }
    Ok(())
}

fn lint_cargo_toml(root: &Path, items: &mut Vec<LintItem>) {
    let p = root.join("Cargo.toml");
    let Ok(raw) = std::fs::read_to_string(&p) else {
        items.push(fail(
            "cargo_toml",
            "cannot read Cargo.toml",
            Some("ensure you are in the kernel project root".into()),
        ));
        return;
    };
    let v: toml::Value = match toml::from_str(&raw) {
        Ok(v) => v,
        Err(e) => {
            items.push(fail(
                "cargo_toml",
                format!("parse failed: {e}"),
                Some("fix Cargo.toml syntax (invalid TOML)".into()),
            ));
            return;
        }
    };
    let pkg = v.get("package").and_then(|x| x.as_table());
    for key in ["name", "version"] {
        if pkg.and_then(|t| t.get(key)).is_some() {
            items.push(pass(
                &format!("cargo_{key}"),
                &format!("[package].{key} set"),
                None,
            ));
        } else {
            items.push(fail(
                "cargo_toml",
                format!("missing [package].{key}"),
                Some(format!("add [package].{key} = \"…\" to Cargo.toml")),
            ));
        }
    }
    for key in ["authors", "license"] {
        if pkg.and_then(|t| t.get(key)).is_some() {
            items.push(pass(
                &format!("cargo_{key}"),
                &format!("[package].{key} set"),
                None,
            ));
        } else {
            items.push(warn(
                "cargo_meta",
                format!("consider setting [package].{key}"),
                Some(
                    "cargo run -p oclive-cli -- init --author \"…\" --license MIT -o .  # or edit Cargo.toml"
                        .to_string(),
                ),
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
                        None,
                    )),
                    Err(e) => items.push(fail(
                        "settings_keys",
                        e,
                        Some(format!(
                            "cargo run -p oclive-cli -- pack migrate-to-blueprint {}",
                            entry.display()
                        )),
                    )),
                }
            }
        }
    }
}

fn lint_monolith(root: &Path, items: &mut Vec<LintItem>) {
    let p = root.join("monolith.toml");
    if !p.is_file() {
        items.push(warn(
            "monolith",
            "no monolith.toml (standard mode)",
            Some(
                "cargo run -p oclive-cli -- init --monolith --monolith-preset latency -o ."
                    .into(),
            ),
        ));
        return;
    }
    match std::fs::read_to_string(&p) {
        Ok(raw) => match crate::monolith_config::parse_monolith_toml(&raw) {
            Ok(f) => {
                if let Err(e) = crate::monolith_config::validate_monolith_section(&f.monolith) {
                    items.push(fail(
                        "monolith",
                        e.to_string(),
                        Some("edit monolith.toml weld_modules / preset per RFC_OCLIVE_MONOLITH_MODE".into()),
                    ));
                } else {
                    items.push(pass("monolith", "monolith.toml format OK", None));
                }
            }
            Err(e) => items.push(fail(
                "monolith",
                e.to_string(),
                Some("fix monolith.toml TOML syntax".into()),
            )),
        },
        Err(e) => items.push(fail(
            "monolith",
            e.to_string(),
            None,
        )),
    }
}

fn lint_git_dirty(root: &Path, items: &mut Vec<LintItem>) {
    if !root.join(".git").exists() {
        items.push(warn(
            "git",
            "not a Git repository",
            Some("git init && git add .".into()),
        ));
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
                items.push(pass("git", "working tree clean", None));
            } else {
                items.push(warn(
                    "git",
                    "uncommitted changes",
                    Some("git add -A && git commit -m \"…\"".into()),
                ));
            }
        }
        _ => items.push(warn("git", "cannot run git status", None)),
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

pub(super) fn pass(check: &str, msg: &str, fix: Option<String>) -> LintItem {
    LintItem {
        level: "pass".into(),
        check: check.into(),
        message: msg.into(),
        fix,
    }
}

pub(super) fn warn(check: &str, msg: impl ToString, fix: Option<String>) -> LintItem {
    LintItem {
        level: "warn".into(),
        check: check.into(),
        message: msg.to_string(),
        fix,
    }
}

pub(super) fn fail(check: &str, msg: impl ToString, fix: Option<String>) -> LintItem {
    LintItem {
        level: "fail".into(),
        check: check.into(),
        message: msg.to_string(),
        fix,
    }
}

