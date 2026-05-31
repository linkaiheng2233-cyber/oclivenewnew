//! `oclive dev`: watch role pack directories for changes, making it easy to trigger hot reload manually or via scripts during development.

use anyhow::{Context, Result};
use clap::Parser;
use notify::{Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::mpsc::channel;
use std::time::Duration;

#[derive(Parser, Debug, Clone)]
pub struct DevArgs {
    /// Kernel project root (contains Cargo.toml)
    #[arg(short = 'o', long, default_value = ".")]
    pub path: PathBuf,

    /// Roles root relative to project (subdirs per role id)
    #[arg(long, default_value = "roles")]
    pub roles: PathBuf,

    /// Do not watch filesystem (print hint once and exit)
    #[arg(long)]
    pub no_watch: bool,

    /// Shell command on change (if unset, only print a hint)
    #[arg(long)]
    pub reload_cmd: Option<String>,
}

fn resolve_root(path: &Path) -> Result<PathBuf> {
    let root = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir().context("current_dir")?.join(path)
    };
    root.canonicalize()
        .with_context(|| format!("cannot resolve project path: {}", root.display()))
}

fn is_role_pack_hot_file(path: &Path) -> bool {
    path.file_name()
        .and_then(|n| n.to_str())
        .is_some_and(|n| n == "manifest.json" || n == "settings.json")
}

/// Resolve the role pack id from `roles/<id>/manifest.json` or `roles/<id>/settings.json`.
fn role_pack_id_from_hot_file(path: &Path, roles_root: &Path) -> Option<String> {
    let rel = path.strip_prefix(roles_root).ok()?;
    if rel.components().count() < 2 {
        return None;
    }
    rel.parent()
        .and_then(|dir| dir.file_name())
        .map(|s| s.to_string_lossy().into_owned())
}

pub fn run(args: DevArgs) -> Result<()> {
    let root = resolve_root(&args.path)?;
    let watch_dir = root.join(&args.roles);
    if args.no_watch {
        eprintln!(
            "[oclive dev] --no-watch: not watching. Role packs dir: {}",
            watch_dir.display()
        );
        return Ok(());
    }
    if !watch_dir.is_dir() {
        anyhow::bail!(
            "Role packs root does not exist: {} (run `oclive init` or create roles/)",
            watch_dir.display()
        );
    }
    eprintln!(
        "[oclive dev] watching manifest.json / settings.json under {} recursively",
        watch_dir.display()
    );
    let (tx, rx) = channel();
    let mut watcher = RecommendedWatcher::new(tx, Config::default()).context("notify watcher")?;
    watcher
        .watch(&watch_dir, RecursiveMode::Recursive)
        .with_context(|| format!("watch {}", watch_dir.display()))?;

    let mut last_fire = std::time::Instant::now()
        .checked_sub(Duration::from_secs(10))
        .unwrap_or_else(std::time::Instant::now);
    while let Ok(ev) = rx.recv() {
        let Ok(ev) = ev else { continue };
        if !matches!(
            ev.kind,
            EventKind::Modify(_) | EventKind::Create(_) | EventKind::Remove(_)
        ) {
            continue;
        }
        let mut role_id: Option<String> = None;
        for p in ev.paths {
            if is_role_pack_hot_file(&p) {
                role_id = role_pack_id_from_hot_file(&p, &watch_dir).or(role_id);
            }
        }
        let Some(rid) = role_id else {
            continue;
        };
        if last_fire.elapsed() < Duration::from_millis(500) {
            continue;
        }
        last_fire = std::time::Instant::now();
        println!("[oclive dev] role pack '{rid}' changed; reload signaled");
        if let Some(cmd) = args.reload_cmd.as_deref() {
            let st = if cfg!(windows) {
                std::process::Command::new("cmd")
                    .args(["/C", cmd])
                    .current_dir(&root)
                    .status()
            } else {
                std::process::Command::new("sh")
                    .arg("-c")
                    .arg(cmd)
                    .current_dir(&root)
                    .status()
            }
            .with_context(|| format!("reload_cmd: {cmd}"))?;
            if !st.success() {
                eprintln!("[oclive dev] reload_cmd exit code: {:?}", st.code());
            }
        }
    }
    Ok(())
}
