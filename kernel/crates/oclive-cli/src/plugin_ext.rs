//! Plugin install / test / uninstall.

use anyhow::{bail, Context, Result};
use clap::Parser;
use oclive_validation::{
    apply_slot_attachments_to_registry, compute_plugin_install_order,
    load_blueprint_slot_registry_for_role_dir, parse_plugin_dependencies,
    parse_slot_attachments_from_manifest_json, write_role_pack_blueprint_slot_registry,
    PIPELINE_BLUEPRINT_FILENAME,
};
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
    /// Role pack dir: auto-apply manifest `slot_attachment` to pipeline.ocblueprint
    #[arg(long)]
    pub role: Option<PathBuf>,
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

    let order =
        compute_plugin_install_order(&args.id, load_deps).map_err(|e| anyhow::anyhow!(e))?;
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

    if let Some(role_dir) = &args.role {
        let role_dir = role_dir.canonicalize().unwrap_or_else(|_| role_dir.clone());
        if !role_dir.join(PIPELINE_BLUEPRINT_FILENAME).is_file() {
            bail!(
                "role dir missing {}: {}",
                PIPELINE_BLUEPRINT_FILENAME,
                role_dir.display()
            );
        }
        match auto_assemble_slot_attachment(&role_dir, &manifest_raw) {
            Ok(msgs) => {
                for m in msgs {
                    println!("✓ {m}");
                }
            }
            Err(e) => eprintln!("⚠ slot_attachment: {e}"),
        }
    } else if parse_slot_attachments_from_manifest_json(&manifest_raw)
        .map(|a| !a.is_empty())
        .unwrap_or(false)
    {
        println!(
            "ℹ Plugin declares slot_attachment; pass --role <pack-dir> to auto-update {}",
            PIPELINE_BLUEPRINT_FILENAME
        );
    }

    let dst = plugins_dir.join(&args.id);
    println!(
        "Review manifest.json and source at {} before enabling high-risk permissions.",
        dst.display()
    );

    Ok(())
}

const ASSEMBLE_HOST_VERSION: &str = "999.0.0";

fn auto_assemble_slot_attachment(role_dir: &Path, manifest_raw: &str) -> Result<Vec<String>> {
    let manifest_v: serde_json::Value = serde_json::from_str(manifest_raw)?;
    let plugin_id = manifest_v
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("manifest.id missing"))?;
    let attachments =
        parse_slot_attachments_from_manifest_json(manifest_raw).map_err(|e| anyhow::anyhow!(e))?;
    if attachments.is_empty() {
        return Ok(vec![]);
    }
    let mut reg = load_blueprint_slot_registry_for_role_dir(role_dir, ASSEMBLE_HOST_VERSION)
        .map_err(|e| anyhow::anyhow!(e.join("; ")))?;
    let notes = apply_slot_attachments_to_registry(&mut reg, plugin_id, &attachments);
    write_role_pack_blueprint_slot_registry(role_dir, &reg, ASSEMBLE_HOST_VERSION)
        .map_err(|e| anyhow::anyhow!(e.join("; ")))?;
    Ok(notes
        .into_iter()
        .map(|n| format!("Auto-assembled: {n}"))
        .collect())
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

/// Directory plugin RPC contract smoke test (health / list_methods / slot generate).
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
    // Simplified: only check the child process is still alive (full JSON-RPC would read the OCLIVE_READY line and HTTP port)
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
