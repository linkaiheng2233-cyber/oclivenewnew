//! `oclive profile` — 内核工程性能画像。

use anyhow::{Context, Result};
use clap::Parser;
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::SystemTime;

#[derive(Parser, Debug)]
pub struct ProfileArgs {
    #[arg(short = 'o', long, default_value = ".")]
    pub path: PathBuf,
    #[arg(long)]
    pub json: bool,
}

#[derive(Serialize)]
struct ProfileReport {
    project: String,
    binary_mib: Option<f64>,
    dep_lines: u32,
    max_depth: u32,
    build_secs_hint: Option<f64>,
    top_crates: Vec<CrateShare>,
}

#[derive(Serialize)]
struct CrateShare {
    name: String,
    mib: f64,
}

pub fn run(args: ProfileArgs) -> Result<()> {
    let root = args.path.canonicalize().context("path")?;
    let name = read_package_name(&root)?;
    let binary_mib = release_binary_mib(&root, &name);
    let (dep_lines, max_depth) = cargo_tree_stats(&root)?;
    let build_secs_hint = target_build_age_secs(&root);
    let top_crates = cargo_bloat_top(&root).unwrap_or_default();

    let report = ProfileReport {
        project: name.clone(),
        binary_mib,
        dep_lines,
        max_depth,
        build_secs_hint,
        top_crates,
    };

    if args.json {
        println!("{}", serde_json::to_string_pretty(&report)?);
        return Ok(());
    }

    println!("📊 Kernel Profile: {name}");
    if let Some(b) = report.binary_mib {
        println!("├── 二进制大小: {b:.1} MiB");
    }
    println!("├── 依赖行数: {}", report.dep_lines);
    println!("├── 最大依赖深度: {}", report.max_depth);
    if let Some(s) = report.build_secs_hint {
        println!("├── target 最近写入: 约 {s:.0} 秒前");
    }
    println!("└── Top crate (cargo bloat，若可用):");
    for c in &report.top_crates {
        println!("    ├── {}: {:.1} MiB", c.name, c.mib);
    }
    if report.top_crates.is_empty() {
        println!("    （未安装 cargo-bloat 或构建失败；可: cargo install cargo-bloat）");
    }
    Ok(())
}

fn read_package_name(root: &Path) -> Result<String> {
    let raw = std::fs::read_to_string(root.join("Cargo.toml"))?;
    let v: toml::Value = toml::from_str(&raw)?;
    Ok(v["package"]["name"]
        .as_str()
        .context("package.name")?
        .to_string())
}

fn release_binary_mib(root: &Path, name: &str) -> Option<f64> {
    let mut p = root.join("target/release").join(name);
    if cfg!(windows) {
        p.set_extension("exe");
    }
    let meta = std::fs::metadata(&p).ok()?;
    Some(meta.len() as f64 / (1024.0 * 1024.0))
}

fn cargo_tree_stats(root: &Path) -> Result<(u32, u32)> {
    let out = Command::new("cargo")
        .args(["tree", "--depth", "3", "--manifest-path"])
        .arg(root.join("Cargo.toml"))
        .output()
        .context("cargo tree")?;
    let text = String::from_utf8_lossy(&out.stdout);
    let lines = text.lines().count() as u32;
    let max_depth = text
        .lines()
        .map(|l| l.chars().take_while(|c| c.is_ascii_whitespace()).count() as u32 / 2)
        .max()
        .unwrap_or(0);
    Ok((lines, max_depth))
}

fn target_build_age_secs(root: &Path) -> Option<f64> {
    let t = root.join("target/release");
    let meta = std::fs::metadata(t).ok()?;
    let modified = meta.modified().ok()?;
    let now = SystemTime::now();
    now.duration_since(modified)
        .ok()
        .map(|d| d.as_secs_f64())
}

fn cargo_bloat_top(root: &Path) -> Result<Vec<CrateShare>> {
    let out = Command::new("cargo")
        .args([
            "bloat",
            "--release",
            "--manifest-path",
            root.join("Cargo.toml").to_str().unwrap_or("Cargo.toml"),
            "-n",
            "5",
        ])
        .output();
    let Ok(out) = out else {
        return Ok(vec![]);
    };
    if !out.status.success() {
        return Ok(vec![]);
    }
    let text = String::from_utf8_lossy(&out.stdout);
    let mut shares = Vec::new();
    for line in text.lines().skip(1) {
        let parts: Vec<_> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            if let Ok(kb) = parts[parts.len() - 1].replace(',', "").parse::<f64>() {
                shares.push(CrateShare {
                    name: parts[0].to_string(),
                    mib: kb / 1024.0,
                });
            }
        }
    }
    Ok(shares)
}
