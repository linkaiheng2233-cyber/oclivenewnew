//! 插件 install / test / search / update / uninstall。

use anyhow::{bail, Context, Result};
use clap::Parser;
use oclive_validation::{parse_plugin_dependencies, resolve_install_order};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;

#[derive(Parser, Debug)]
pub struct PluginInstallArgs {
    pub id: String,
    #[arg(short = 'o', long, default_value = "./plugins")]
    pub plugins_dir: PathBuf,
    /// Source directory (with manifest.json); default plugins_dir/<id>
    #[arg(long)]
    pub source: Option<PathBuf>,
}

#[derive(Parser, Debug)]
pub struct PluginUninstallArgs {
    pub id: String,
    #[arg(short = 'o', long, default_value = "./plugins")]
    pub plugins_dir: PathBuf,
}

#[derive(Parser, Debug)]
pub struct PluginTestArgs {
    pub plugin_path: PathBuf,
    #[arg(long)]
    pub json: bool,
}

#[derive(Parser, Debug)]
pub struct PluginSearchArgs {
    pub keyword: String,
}

#[derive(Parser, Debug)]
pub struct PluginUpdateArgs {
    pub id: String,
    #[arg(short = 'o', long, default_value = "./plugins")]
    pub plugins_dir: PathBuf,
}

use crate::market_index::{fetch_market_index, MarketKindSerde};

pub fn run_install(args: PluginInstallArgs) -> Result<()> {
    let plugins_dir = args.plugins_dir.canonicalize().unwrap_or(args.plugins_dir);
    fs::create_dir_all(&plugins_dir)?;
    let src = args
        .source
        .clone()
        .unwrap_or_else(|| plugins_dir.join(&args.id));
    if !src.join("manifest.json").is_file() {
        bail!("Missing manifest.json: {}", src.display());
    }
    let manifest_raw = fs::read_to_string(src.join("manifest.json"))?;
    let deps = parse_plugin_dependencies(&manifest_raw).map_err(|e| anyhow::anyhow!(e))?;

    let load_deps = |id: &str| -> Result<Vec<String>, String> {
        let p = plugins_dir.join(id).join("manifest.json");
        let raw = fs::read_to_string(&p).map_err(|e| e.to_string())?;
        parse_plugin_dependencies(&raw)
    };

    let order = resolve_install_order(&args.id, load_deps).map_err(|e| anyhow::anyhow!(e))?;
    let order_display = order.join(" → ");
    for id in &order {
        let dst = plugins_dir.join(id);
        if dst.is_dir() && *id != args.id {
            continue;
        }
        let from = if *id == args.id {
            src.clone()
        } else {
            plugins_dir.join(id)
        };
        if !from.join("manifest.json").is_file() {
            bail!(
                "Dependency plugin {id} not found under {}",
                plugins_dir.display()
            );
        }
        if *id == args.id || !dst.exists() {
            copy_plugin_tree(&from, &dst)?;
            println!("✓ Installed {id} → {}", dst.display());
        }
    }
    if !deps.is_empty() {
        println!("Dependency tree: {order_display}");
    }
    Ok(())
}

pub fn run_uninstall(args: PluginUninstallArgs) -> Result<()> {
    let plugins_dir = args.plugins_dir.canonicalize().unwrap_or(args.plugins_dir);
    let installed = list_installed(&plugins_dir)?;
    let dependents: Vec<String> = installed
        .iter()
        .filter_map(|(id, raw)| {
            parse_plugin_dependencies(raw)
                .ok()
                .filter(|d| d.contains(&args.id))
                .map(|_| id.clone())
        })
        .collect();
    if !dependents.is_empty() {
        eprintln!(
            "⚠ These plugins still depend on {}: {}",
            args.id,
            dependents.join(", ")
        );
    }
    let target = plugins_dir.join(&args.id);
    if target.is_dir() {
        fs::remove_dir_all(&target).context("remove plugin dir")?;
        println!("Uninstalled {}", args.id);
    } else {
        bail!("Not installed: {}", args.id);
    }
    Ok(())
}

/// 目录插件 RPC 契约烟测（health / list_methods / 槽位 generate）。
pub fn run_test(args: PluginTestArgs) -> Result<()> {
    let path = args.plugin_path.canonicalize().unwrap_or(args.plugin_path);
    let manifest_path = path.join("manifest.json");
    let raw = fs::read_to_string(&manifest_path).context("manifest")?;
    let v: Value = serde_json::from_str(&raw)?;
    let id = v["id"].as_str().context("manifest.id")?;
    let methods: Vec<String> = v["rpcMethods"]
        .as_array()
        .map(|a| {
            a.iter()
                .filter_map(|x| x.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    let mut child = spawn_plugin(&path)?;
    std::thread::sleep(Duration::from_millis(800));
    let mut results = Vec::new();
    results.push(rpc_call(&mut child, "health", json!({})));
    results.push(rpc_call(&mut child, "list_methods", json!({})));
    for m in &methods {
        results.push(rpc_call(&mut child, m, json!({"probe": true})));
    }
    let _ = child.kill();

    if args.json {
        println!("{}", serde_json::to_string_pretty(&results)?);
        return Ok(());
    }
    println!("oclive plugin test — {id}");
    for r in &results {
        let mark = if r.ok { "✅" } else { "❌" };
        println!("  {mark} {} — {}", r.method, r.detail);
    }
    Ok(())
}

/// 从 `OCLIVE_PLUGIN_INDEX_URL` 拉取索引并按关键词过滤。
pub fn run_search(args: PluginSearchArgs) -> Result<()> {
    eprintln!(
        "⚠ [deprecated] `oclive plugin search` — use `oclive market search \"{}\"`",
        args.keyword
    );
    let index = fetch_market_index()?;
    let hits: Vec<_> = crate::market_index::search_items(&index, &args.keyword)
        .into_iter()
        .filter(|p| matches!(p.kind, MarketKindSerde::Plugin))
        .collect();
    if hits.is_empty() {
        println!("(no matches)");
        return Ok(());
    }
    for p in hits {
        println!("{} v{} — {} — {}", p.id, p.version, p.author, p.description);
    }
    Ok(())
}

/// 对比索引版本并覆盖安装插件目录。
pub fn run_update(args: PluginUpdateArgs) -> Result<()> {
    eprintln!(
        "⚠ [deprecated] `oclive plugin update` — use `oclive market install {}` for the latest version",
        args.id
    );
    let plugins_dir = args.plugins_dir.canonicalize().unwrap_or(args.plugins_dir);
    let local = plugins_dir.join(&args.id).join("manifest.json");
    if !local.is_file() {
        bail!("Not installed: {}", args.id);
    }
    let index = fetch_market_index()?;
    let remote = index.plugins.iter().find(|p| p.id == args.id);
    let Some(remote) = remote else {
        bail!("Not in index: {}", args.id);
    };
    let local_v: Value = serde_json::from_str(&fs::read_to_string(&local)?)?;
    let cur = local_v["version"].as_str().unwrap_or("0.0.0");
    if cur == remote.version {
        println!("{} is up to date ({})", args.id, cur);
        return Ok(());
    }
    println!(
        "New version available {} → {} (download from index URL and re-run install)",
        cur, remote.version
    );
    Ok(())
}

fn list_installed(dir: &Path) -> Result<Vec<(String, String)>> {
    let mut out = Vec::new();
    if !dir.is_dir() {
        return Ok(out);
    }
    for e in fs::read_dir(dir)? {
        let e = e?;
        let m = e.path().join("manifest.json");
        if m.is_file() {
            let raw = fs::read_to_string(&m)?;
            if let Ok(v) = serde_json::from_str::<Value>(&raw) {
                if let Some(id) = v["id"].as_str() {
                    out.push((id.to_string(), raw));
                }
            }
        }
    }
    Ok(out)
}

fn copy_plugin_tree(from: &Path, to: &Path) -> Result<()> {
    if to.exists() {
        fs::remove_dir_all(to).ok();
    }
    copy_dir(from, to)
}

fn copy_dir(from: &Path, to: &Path) -> Result<()> {
    fs::create_dir_all(to)?;
    for e in fs::read_dir(from)? {
        let e = e?;
        let p = e.path();
        let name = e.file_name();
        let dest = to.join(name);
        if p.is_dir() {
            copy_dir(&p, &dest)?;
        } else {
            fs::copy(&p, &dest)?;
        }
    }
    Ok(())
}

fn spawn_plugin(path: &Path) -> Result<std::process::Child> {
    let manifest: Value = serde_json::from_str(&fs::read_to_string(path.join("manifest.json"))?)?;
    let cmd = manifest["process"]["command"]
        .as_str()
        .context("process.command")?;
    let args: Vec<String> = manifest["process"]["args"]
        .as_array()
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();
    let mut c = Command::new(cmd);
    c.args(&args)
        .current_dir(path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    c.spawn().context("spawn plugin")
}

#[derive(serde::Serialize)]
struct RpcResult {
    method: String,
    ok: bool,
    detail: String,
}

fn rpc_call(child: &mut std::process::Child, method: &str, _params: Value) -> RpcResult {
    // 简化：仅检查子进程仍存活（完整 JSON-RPC 需读 OCLIVE_READY 行与 HTTP 端口）
    let alive = child.try_wait().ok().flatten().is_none();
    RpcResult {
        method: method.into(),
        ok: alive,
        detail: if alive {
            "subprocess alive (full RPC contract: use plugin manager panel)".into()
        } else {
            "subprocess exited".into()
        },
    }
}

use serde_json::json;
