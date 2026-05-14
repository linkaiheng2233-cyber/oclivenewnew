//! `oclive dev`：监听角色包目录变更，便于开发时手动或脚本触发热重载。

use anyhow::{Context, Result};
use clap::Parser;
use notify::{Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::mpsc::channel;
use std::time::Duration;

#[derive(Parser, Debug, Clone)]
pub struct DevArgs {
    /// 内核项目根（含 Cargo.toml）
    #[arg(short = 'o', long, default_value = ".")]
    pub path: PathBuf,

    /// 相对于项目根的 roles 根目录（其下为各角色 id 子目录）
    #[arg(long, default_value = "roles")]
    pub roles: PathBuf,

    /// 不监听文件系统（仅打印一次提示后退出）
    #[arg(long)]
    pub no_watch: bool,

    /// 检测到变更后执行的 shell 命令（未设置则仅打印提示）
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
        .with_context(|| format!("无法解析项目路径: {}", root.display()))
}

fn is_role_pack_hot_file(path: &Path) -> bool {
    path.file_name()
        .and_then(|n| n.to_str())
        .is_some_and(|n| n == "manifest.json" || n == "settings.json")
}

pub fn run(args: DevArgs) -> Result<()> {
    let root = resolve_root(&args.path)?;
    let watch_dir = root.join(&args.roles);
    if args.no_watch {
        eprintln!(
            "[oclive dev] --no-watch：未监听。角色包目录：{}",
            watch_dir.display()
        );
        return Ok(());
    }
    if !watch_dir.is_dir() {
        anyhow::bail!(
            "角色包根目录不存在：{}（可先 `oclive init` 或创建 roles/）",
            watch_dir.display()
        );
    }
    eprintln!(
        "[oclive dev] 监听 {}（manifest.json / settings.json 变更将提示重载）",
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
        let mut hit = false;
        for p in ev.paths {
            if is_role_pack_hot_file(&p) {
                hit = true;
                break;
            }
        }
        if !hit {
            continue;
        }
        if last_fire.elapsed() < Duration::from_millis(400) {
            continue;
        }
        last_fire = std::time::Instant::now();
        println!("[oclive dev] 检测到角色包配置变更，请重载内核（或配置 --reload-cmd 自动执行）");
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
                eprintln!("[oclive dev] reload_cmd 退出码: {:?}", st.code());
            }
        }
    }
    Ok(())
}
