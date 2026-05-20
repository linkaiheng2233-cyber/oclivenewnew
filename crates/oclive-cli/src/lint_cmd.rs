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
}

#[derive(Serialize, Clone)]
struct LintItem {
    level: String,
    check: String,
    message: String,
}

pub fn run(args: LintArgs) -> Result<()> {
    let root = args.path.canonicalize().unwrap_or(args.path);
    let mut items = Vec::new();
    for (dir, name) in [
        ("src", "src/"),
        ("docs", "docs/"),
        ("roles", "roles/（可选）"),
    ] {
        let p = root.join(dir);
        if p.is_dir() {
            items.push(pass(&format!("dir_{dir}"), &format!("存在 {name}")));
        } else if dir == "roles" {
            items.push(warn(&format!("dir_{dir}"), &format!("缺少 {name}")));
        } else {
            items.push(fail(&format!("dir_{dir}"), &format!("缺少 {name}")));
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
        items.push(fail("cargo_toml", "无法读取 Cargo.toml"));
        return;
    };
    let v: toml::Value = match toml::from_str(&raw) {
        Ok(v) => v,
        Err(e) => {
            items.push(fail("cargo_toml", format!("解析失败: {e}")));
            return;
        }
    };
    let pkg = v.get("package").and_then(|x| x.as_table());
    for key in ["name", "version"] {
        if pkg.and_then(|t| t.get(key)).is_some() {
            items.push(pass(&format!("cargo_{key}"), &format!("[package].{key} 已填写")));
        } else {
            items.push(fail("cargo_toml", format!("缺少 [package].{key}")));
        }
    }
    for key in ["authors", "license"] {
        if pkg.and_then(|t| t.get(key)).is_some() {
            items.push(pass(&format!("cargo_{key}"), &format!("[package].{key} 已填写")));
        } else {
            items.push(warn("cargo_meta", format!("建议填写 [package].{key}")));
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
                        &format!("{} settings 顶层键合法", entry.display()),
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
        items.push(warn("monolith", "无 monolith.toml（标准模式）"));
        return;
    }
    match std::fs::read_to_string(&p) {
        Ok(raw) => match crate::monolith_config::parse_monolith_toml(&raw) {
            Ok(f) => {
                if let Err(e) = crate::monolith_config::validate_monolith_section(&f.monolith) {
                    items.push(fail("monolith", e.to_string()));
                } else {
                    items.push(pass("monolith", "monolith.toml 格式正确"));
                }
            }
            Err(e) => items.push(fail("monolith", e.to_string())),
        },
        Err(e) => items.push(fail("monolith", e.to_string())),
    }
}

fn lint_git_dirty(root: &Path, items: &mut Vec<LintItem>) {
    if !root.join(".git").exists() {
        items.push(warn("git", "非 Git 仓库"));
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
                items.push(pass("git", "工作区干净"));
            } else {
                items.push(warn("git", "存在未提交变更"));
            }
        }
        _ => items.push(warn("git", "无法运行 git status")),
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
