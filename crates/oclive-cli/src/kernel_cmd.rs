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
    /// Shared runtime kernel path, manifest, backups
    Status(KernelStatusArgs),
    /// Promote a kernel binary into %LOCALAPPDATA%/OCLive/runtime/ (with backup)
    Promote(KernelPromoteArgs),
    /// Roll back shared runtime to the latest backup
    Rollback(KernelRollbackArgs),
}

#[derive(Parser, Debug)]
pub struct KernelStatusArgs {
    #[arg(long)]
    pub json: bool,
    /// Probe loopback /health (default port 8420)
    #[arg(long, default_value_t = 8420)]
    pub port: u16,
}

#[derive(Parser, Debug)]
pub struct KernelPromoteArgs {
    /// Kernel binary to copy (default: best local dev build under cwd)
    #[arg(short = 'b', long)]
    pub binary: Option<PathBuf>,
}

#[derive(Parser, Debug)]
pub struct KernelRollbackArgs {
    #[arg(long)]
    pub json: bool,
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
        KernelCommands::Status(a) => run_status(a),
        KernelCommands::Promote(a) => run_promote(a),
        KernelCommands::Rollback(a) => run_rollback(a),
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
                        compatibility_note:
                            "Stub scaffold — use init --kernel-source <oclivenewnew root>.".into(),
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
                let commit = git_head_short(abs.parent().unwrap_or(&abs));
                ("path".into(), None, Some(abs.display().to_string()), commit)
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
        Some(
            "Run `cargo update -p oclive_kernel_runtime` or check crates.io for newer releases."
                .into(),
        )
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
    let p = path?;
    let lib_rs = Path::new(p).join("src/lib.rs");
    let raw = fs::read_to_string(lib_rs).ok()?;
    for line in raw.lines() {
        if line.contains("RUNTIME_API_VERSION") {
            return Some(line.trim().to_string());
        }
    }
    None
}

#[derive(Serialize)]
struct KernelRuntimeStatus {
    shared_binary: String,
    shared_exists: bool,
    manifest: Option<oclive_kernel_runtime::KernelBinaryManifest>,
    backups: Vec<String>,
    health_ok: Option<bool>,
}

fn run_status(args: KernelStatusArgs) -> Result<()> {
    use oclive_kernel_runtime::{
        list_runtime_backups, shared_kernel_binary_path, KernelBinaryManifest,
    };

    let shared = shared_kernel_binary_path();
    let manifest = KernelBinaryManifest::read_sidecar(&shared);
    let backups: Vec<String> = list_runtime_backups()
        .into_iter()
        .map(|p| p.display().to_string())
        .collect();
    let health_ok = probe_health(args.port);

    let report = KernelRuntimeStatus {
        shared_binary: shared.display().to_string(),
        shared_exists: shared.is_file(),
        manifest,
        backups,
        health_ok,
    };

    if args.json {
        println!("{}", serde_json::to_string_pretty(&report)?);
        return Ok(());
    }

    println!("oclive kernel status");
    println!("  shared:   {} ({})", report.shared_binary, if report.shared_exists { "present" } else { "missing" });
    if let Some(ref m) = report.manifest {
        println!("  version:  {} profile={}", m.version, m.build_profile);
        if let Some(ref g) = m.git_commit {
            println!("  git:      {g}");
        }
    }
    println!("  backups:  {}", report.backups.len());
    for b in &report.backups {
        println!("    - {b}");
    }
    if let Some(ok) = report.health_ok {
        println!("  health:   {}", if ok { "ok" } else { "unreachable" });
    }
    Ok(())
}

fn probe_health(port: u16) -> Option<bool> {
    let url = format!("http://127.0.0.1:{port}/health");
    let agent = ureq::AgentBuilder::new().timeout(std::time::Duration::from_secs(3)).build();
    agent.get(&url).call().ok().map(|r| r.status() == 200)
}

fn run_promote(args: KernelPromoteArgs) -> Result<()> {
    use oclive_kernel_runtime::{
        discover_spawn_kernel_candidates, pick_best_kernel, promote_with_backup,
        KernelBinaryManifest,
    };

    let binary = if let Some(b) = args.binary {
        b
    } else {
        let cwd = std::env::current_dir().context("cwd")?;
        let candidates = discover_spawn_kernel_candidates(&[cwd], None, None);
        let Some(best) = pick_best_kernel(&candidates) else {
            anyhow::bail!("no kernel candidate found; pass --binary PATH");
        };
        best.binary.clone()
    };

    let manifest = KernelBinaryManifest::read_sidecar(&binary);
    let report = promote_with_backup(&binary, manifest.as_ref())
        .map_err(|e| anyhow::anyhow!(e))?;
    println!("promoted to {}", report.dest.display());
    if let Some(ref b) = report.backup_dir {
        println!("backup at {}", b.display());
    }
    Ok(())
}

fn run_rollback(args: KernelRollbackArgs) -> Result<()> {
    use oclive_kernel_runtime::rollback_shared_kernel;

    let dest = rollback_shared_kernel().map_err(|e| anyhow::anyhow!(e))?;
    if args.json {
        println!(
            "{}",
            serde_json::json!({ "restored": dest.display().to_string() })
        );
    } else {
        println!("rolled back shared runtime to {}", dest.display());
    }
    Ok(())
}

fn compatibility_blurb() -> String {
    "oclivenewnew desktop host 0.2.x aligns with oclive_kernel_runtime 0.2.x path deps. \
     See creator-docs/COMPATIBILITY.md for editor vs host matrices. \
     Scaffold CLI (oclive-cli) uses independent semver 0.1.x."
        .into()
}
