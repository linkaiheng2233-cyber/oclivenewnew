//! `oclive collab` — 基于 Git 的角色包协作。

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Parser, Debug)]
pub struct CollabCli {
    #[command(subcommand)]
    pub command: CollabCommands,
}

#[derive(Subcommand, Debug)]
pub enum CollabCommands {
    /// Initialize `.oclive-collab.yml` (optional git init)
    Init(CollabInitArgs),
    /// Collaboration status (unpushed local / remote ahead)
    Status(CollabStatusArgs),
    /// Pull remote edits
    Pull(CollabPullArgs),
    /// Push local edits
    Push(CollabPushArgs),
    /// Diff against remote branch
    Diff(CollabDiffArgs),
}

#[derive(Parser, Debug)]
pub struct CollabInitArgs {
    /// Role pack root (with manifest.json; default: current directory)
    #[arg(short = 'o', long, default_value = ".")]
    pub path: PathBuf,
    /// Git remote URL
    #[arg(long)]
    pub remote: String,
    #[arg(long, default_value = "main")]
    pub branch: String,
    #[arg(long)]
    pub auto_sync: bool,
}

#[derive(Parser, Debug)]
pub struct CollabStatusArgs {
    #[arg(short = 'o', long, default_value = ".")]
    pub path: PathBuf,
}

#[derive(Parser, Debug)]
pub struct CollabPullArgs {
    #[arg(short = 'o', long, default_value = ".")]
    pub path: PathBuf,
}

#[derive(Parser, Debug)]
pub struct CollabPushArgs {
    #[arg(short = 'o', long, default_value = ".")]
    pub path: PathBuf,
}

#[derive(Parser, Debug)]
pub struct CollabDiffArgs {
    #[arg(short = 'o', long, default_value = ".")]
    pub path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollabConfig {
    pub remote: String,
    pub branch: String,
    #[serde(default)]
    pub auto_sync: bool,
}

pub fn run(cli: CollabCli) -> Result<()> {
    match cli.command {
        CollabCommands::Init(a) => run_init(a),
        CollabCommands::Status(a) => run_status(a),
        CollabCommands::Pull(a) => run_pull(a),
        CollabCommands::Push(a) => run_push(a),
        CollabCommands::Diff(a) => run_diff(a),
    }
}

fn collab_file(root: &Path) -> PathBuf {
    root.join(".oclive-collab.yml")
}

fn load_config(root: &Path) -> Result<CollabConfig> {
    let p = collab_file(root);
    if !p.is_file() {
        bail!(
            "Not found: {}; run oclive collab init first",
            p.display()
        );
    }
    let raw = fs::read_to_string(&p)?;
    serde_yaml::from_str(&raw).context("parse .oclive-collab.yml")
}

fn run_init(args: CollabInitArgs) -> Result<()> {
    let root = resolve_role_pack_root(&args.path)?;
    let cfg = CollabConfig {
        remote: args.remote.clone(),
        branch: args.branch.clone(),
        auto_sync: args.auto_sync,
    };
    fs::write(
        collab_file(&root),
        serde_yaml::to_string(&cfg).context("serialize collab yml")?,
    )?;
    if !root.join(".git").is_dir() {
        git_in(&root, &["init"])?;
        println!("git init: {}", root.display());
    }
    let _ = git_in(&root, &["remote", "remove", "origin"]).ok();
    git_in(&root, &["remote", "add", "origin", &args.remote])?;
    println!("Wrote {} ", collab_file(&root).display());
    println!("  remote: {}", args.remote);
    println!("  branch: {}", args.branch);
    Ok(())
}

fn run_status(args: CollabStatusArgs) -> Result<()> {
    let root = resolve_role_pack_root(&args.path)?;
    let cfg = load_config(&root)?;
    println!("oclive collab status — {}", root.display());
    println!("  remote: {}", cfg.remote);
    println!("  branch: {}", cfg.branch);
    git_in(&root, &["fetch", "origin"])?;
    let porcelain = git_output(&root, &["status", "--porcelain"])?;
    if porcelain.trim().is_empty() {
        println!("  Local workspace: ✅ clean");
    } else {
        println!("  Local workspace: ⚠️ uncommitted changes");
    }
    let ahead = git_output(&root, &["rev-list", "--count", &format!("origin/{}..HEAD", cfg.branch)])?
        .trim()
        .parse::<u32>()
        .unwrap_or(0);
    let behind = git_output(&root, &["rev-list", "--count", &format!("HEAD..origin/{}", cfg.branch)])?
        .trim()
        .parse::<u32>()
        .unwrap_or(0);
    if ahead > 0 {
        println!("  Local ahead of remote by {ahead} commit(s) (collab push)");
    }
    if behind > 0 {
        println!("  Remote ahead of local by {behind} commit(s) (collab pull)");
    }
    if ahead == 0 && behind == 0 {
        println!("  In sync with origin/{}", cfg.branch);
    }
    Ok(())
}

fn run_pull(args: CollabPullArgs) -> Result<()> {
    let root = resolve_role_pack_root(&args.path)?;
    let cfg = load_config(&root)?;
    pre_pull_checks(&root, &cfg)?;
    git_in(&root, &["pull", "origin", &cfg.branch])?;
    println!("✓ Pulled origin/{}", cfg.branch);
    Ok(())
}

fn run_push(args: CollabPushArgs) -> Result<()> {
    let root = resolve_role_pack_root(&args.path)?;
    let cfg = load_config(&root)?;
    pre_push_checks(&root, &cfg)?;
    git_in(&root, &["push", "origin", &cfg.branch])?;
    println!("✓ Pushed to origin/{}", cfg.branch);
    Ok(())
}

fn run_diff(args: CollabDiffArgs) -> Result<()> {
    let root = resolve_role_pack_root(&args.path)?;
    let cfg = load_config(&root)?;
    git_in(&root, &["fetch", "origin"])?;
    let refname = format!("origin/{}", cfg.branch);
    git_in(&root, &["diff", &refname])?;
    Ok(())
}

fn pre_pull_checks(root: &Path, cfg: &CollabConfig) -> Result<()> {
    git_in(root, &["fetch", "origin"])?;
    let ahead = git_output(root, &["rev-list", "--count", &format!("origin/{}..HEAD", cfg.branch)])?
        .trim()
        .parse::<u32>()
        .unwrap_or(0);
    if ahead > 0 {
        eprintln!("⚠ Local has {ahead} unpushed commit(s); pull may create a merge commit.");
        eprintln!("  Consider `oclive collab push` or `git stash` before pull.");
    }
    Ok(())
}

fn pre_push_checks(root: &Path, cfg: &CollabConfig) -> Result<()> {
    let porcelain = git_output(root, &["status", "--porcelain"])?;
    if !porcelain.trim().is_empty() {
        bail!(
            "Uncommitted changes; run git add / git commit before collab push\n{porcelain}"
        );
    }
    git_in(root, &["fetch", "origin"])?;
    let behind = git_output(root, &["rev-list", "--count", &format!("HEAD..origin/{}", cfg.branch)])?
        .trim()
        .parse::<u32>()
        .unwrap_or(0);
    if behind > 0 {
        bail!(
            "Remote is {behind} commit(s) ahead; run `oclive collab pull`, resolve conflicts, then push.\n\
             On conflict: edit files → git add → git commit → collab push"
        );
    }
    Ok(())
}

fn resolve_role_pack_root(path: &Path) -> Result<PathBuf> {
    let p = path
        .canonicalize()
        .unwrap_or_else(|_| path.to_path_buf());
    if p.join("manifest.json").is_file() {
        return Ok(p);
    }
    if p.join("roles").is_dir() {
        bail!(
            "Point to a single role pack directory (with manifest.json), not kernel project root {}",
            p.display()
        );
    }
    bail!("{} is not a role pack root (missing manifest.json)", p.display())
}

fn git_in(root: &Path, args: &[&str]) -> Result<()> {
    let out = Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .with_context(|| format!("git {}", args.join(" ")))?;
    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        bail!("git {} failed: {stderr}", args.join(" "));
    }
    Ok(())
}

fn git_output(root: &Path, args: &[&str]) -> Result<String> {
    let out = Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .with_context(|| format!("git {}", args.join(" ")))?;
    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        bail!("git {} failed: {stderr}", args.join(" "));
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}
