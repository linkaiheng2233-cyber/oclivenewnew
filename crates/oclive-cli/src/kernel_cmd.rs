//! `oclive kernel` — runtime dependency introspection.

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

use crate::project_introspect::{analyze_project, git_head_short};

#[derive(Parser, Debug)]
pub struct KernelCli {
    #[command(subcommand)]
    pub command: KernelCommands,
}

#[derive(Subcommand, Debug)]
pub enum KernelCommands {
    /// Show oclive_kernel_runtime dependency version, path, and compatibility notes
    Info(KernelInfoArgs),
}

#[derive(Parser, Debug)]
pub struct KernelInfoArgs {
    #[arg(short = 'o', long, default_value = ".")]
    pub path: PathBuf,
    #[arg(long)]
    pub json: bool,
}

#[derive(Serialize)]
struct KernelInfoReport {
    schema_version: u32,
    dependency_name: String,
    dep_kind: String,
    version_req: Option<String>,
    path: Option<String>,
    git_commit: Option<String>,
    runtime_api_version: Option<String>,
    compatibility_note: String,
    crates_io_update_hint: Option<String>,
}

pub fn run(cli: KernelCli) -> Result<()> {
    match cli.command {
        KernelCommands::Info(a) => run_info(a),
    }
}

fn run_info(args: KernelInfoArgs) -> Result<()> {
    let root = args.path.canonicalize().context("path")?;
    let snap = analyze_project(&root)?;
    let cargo = fs::read_to_string(root.join("Cargo.toml"))?;
    let v: toml::Value = toml::from_str(&cargo)?;
    let deps = v.get("dependencies").and_then(|d| d.as_table());
    let rt = deps.and_then(|d| d.get("oclive_kernel_runtime"));

    let (dep_kind, version_req, path, git_commit): (
        String,
        Option<String>,
        Option<String>,
        Option<String>,
    ) = match rt {
        None => {
            if args.json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&KernelInfoReport {
                        schema_version: 1,
                        dependency_name: "oclive_kernel_runtime".into(),
                        dep_kind: "none".into(),
                        version_req: None,
                        path: None,
                        git_commit: None,
                        runtime_api_version: None,
                        compatibility_note: "Stub scaffold — use init --kernel-source <oclivenewnew root>.".into(),
                        crates_io_update_hint: None,
                    })?
                );
            } else {
                println!("oclive kernel info — {}", root.display());
                println!("  oclive_kernel_runtime: not linked (stub Cargo.toml)");
                println!("  hint: oclive init --kernel-source <path-to-oclivenewnew>");
            }
            return Ok(());
        }
        Some(t) => {
            if let Some(p) = t.get("path").and_then(|x| x.as_str()) {
                let abs = root.join(p).canonicalize().unwrap_or_else(|_| root.join(p));
                let commit = git_head_short(&abs.parent().unwrap_or(&abs));
                (
                    "path".into(),
                    None,
                    Some(abs.display().to_string()),
                    commit,
                )
            } else if let Some(s) = t.as_str() {
                ("version".into(), Some(s.into()), None, None)
            } else if let Some(ver) = t.get("version").and_then(|x| x.as_str()) {
                ("version".into(), Some(ver.into()), None, None)
            } else {
                ("unknown".into(), None, None, None)
            }
        }
    };

    let api_ver = read_runtime_api_version(path.as_deref());
    let compat = compatibility_blurb();
    let update_hint = if dep_kind == "version" {
        Some("Run `cargo update -p oclive_kernel_runtime` or check crates.io for newer releases.".into())
    } else {
        None
    };

    let report = KernelInfoReport {
        schema_version: 1,
        dependency_name: "oclive_kernel_runtime".into(),
        dep_kind: dep_kind.clone(),
        version_req: version_req.clone(),
        path: path.clone(),
        git_commit: git_commit.clone(),
        runtime_api_version: api_ver.clone(),
        compatibility_note: compat.clone(),
        crates_io_update_hint: update_hint.clone(),
    };

    if args.json {
        println!("{}", serde_json::to_string_pretty(&report)?);
        return Ok(());
    }

    println!("oclive kernel info — {}", root.display());
    println!("  dependency:     oclive_kernel_runtime ({dep_kind})");
    if let Some(ref v) = version_req {
        println!("  version req:    {v}");
    }
    if let Some(ref p) = path {
        println!("  path:           {p}");
    }
    if let Some(ref c) = git_commit {
        println!("  git commit:     {c}");
    }
    if let Some(ref a) = api_ver {
        println!("  RUNTIME_API:    {a}");
    }
    println!("  project preset: {} (inferred)", snap.preset);
    println!();
    println!("Compatibility (summary):");
    for line in compat.lines() {
        println!("  {line}");
    }
    if let Some(h) = update_hint {
        println!("\n{h}");
    }
    Ok(())
}

fn read_runtime_api_version(path: Option<&str>) -> Option<String> {
    let Some(p) = path else {
        return None;
    };
    let lib_rs = Path::new(p).join("src/lib.rs");
    let raw = fs::read_to_string(lib_rs).ok()?;
    for line in raw.lines() {
        if line.contains("RUNTIME_API_VERSION") {
            return Some(line.trim().to_string());
        }
    }
    None
}

fn compatibility_blurb() -> String {
    "oclivenewnew desktop host 0.2.x aligns with oclive_kernel_runtime 0.2.x path deps. \
     See creator-docs/COMPATIBILITY.md for editor vs host matrices. \
     Scaffold CLI (oclive-cli) uses independent semver 0.1.x."
        .into()
}
