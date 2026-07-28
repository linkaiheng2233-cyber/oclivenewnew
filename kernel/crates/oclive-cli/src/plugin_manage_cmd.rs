//! `oclive plugin manage` — advanced slot / blueprint management (CLI).

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use oclive_validation::{
    load_blueprint_slot_registry_for_role_dir, write_role_pack_blueprint_slot_registry,
    SlotOverridePatch, SlotRegistryEntry, PIPELINE_BLUEPRINT_FILENAME,
};
use serde::Serialize;
use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Parser, Debug)]
pub struct PluginManageCli {
    /// Role pack directory (contains pipeline.ocblueprint)
    #[arg(long)]
    pub role: Option<PathBuf>,

    /// Machine-readable JSON output
    #[arg(long)]
    pub json: bool,

    /// Interactive TUI (slot list & link overview)
    #[arg(long)]
    pub tui: bool,

    #[command(subcommand)]
    pub command: Option<ManageSubcommand>,
}

#[derive(Subcommand, Debug)]
pub enum ManageSubcommand {
    /// List slot_registry entries and linked plugins
    List,
    /// Add a new slot instance
    AddSlot {
        #[arg(value_name = "TYPE")]
        slot_type: String,
        label: String,
    },
    /// Remove a slot instance by registry key
    RemoveSlot {
        #[arg(value_name = "KEY")]
        key: String,
    },
    /// Set backend for a slot key
    SetBackend { key: String, backend: String },
    /// Link directory plugin id to a slot key
    Link { key: String, plugin_id: String },
    /// Clear plugin link on a slot key
    Unlink { key: String },
}

const HOST_VERSION: &str = "999.0.0";

pub fn run_manage(cli: PluginManageCli) -> Result<()> {
    if cli.tui && cli.command.is_none() {
        return crate::plugin_manage_tui::run_plugin_manage_tui(cli.role.as_deref());
    }
    let role_dir = find_role_dir(cli.role.as_deref())?;
    let sub = cli.command.unwrap_or(ManageSubcommand::List);
    match sub {
        ManageSubcommand::List => cmd_list(&role_dir, cli.json),
        ManageSubcommand::AddSlot { slot_type, label } => {
            cmd_add_slot(&role_dir, &slot_type, &label, cli.json)
        }
        ManageSubcommand::RemoveSlot { key } => cmd_remove_slot(&role_dir, &key, cli.json),
        ManageSubcommand::SetBackend { key, backend } => {
            cmd_set_backend(&role_dir, &key, &backend, cli.json)
        }
        ManageSubcommand::Link { key, plugin_id } => {
            cmd_link(&role_dir, &key, &plugin_id, cli.json)
        }
        ManageSubcommand::Unlink { key } => cmd_unlink(&role_dir, &key, cli.json),
    }
}

pub fn find_role_dir(role: Option<&std::path::Path>) -> Result<PathBuf> {
    if let Some(r) = role {
        let p = r.canonicalize().unwrap_or_else(|_| r.to_path_buf());
        if !p.join(PIPELINE_BLUEPRINT_FILENAME).is_file() {
            bail!(
                "role dir missing {}: {}",
                PIPELINE_BLUEPRINT_FILENAME,
                p.display()
            );
        }
        return Ok(p);
    }
    let roles = PathBuf::from("roles");
    if !roles.is_dir() {
        bail!("pass --role <path> or run from a project with ./roles/");
    }
    let mut candidates: Vec<PathBuf> = std::fs::read_dir(&roles)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.is_dir() && p.join(PIPELINE_BLUEPRINT_FILENAME).is_file())
        .collect();
    candidates.sort();
    match candidates.len() {
        0 => bail!("no role pack with pipeline.ocblueprint under ./roles/"),
        1 => Ok(candidates.remove(0)),
        n => bail!(
            "multiple role packs ({n}); use --role <path> (e.g. {})",
            candidates[0].display()
        ),
    }
}

pub fn load_registry(role_dir: &std::path::Path) -> Result<BTreeMap<String, SlotRegistryEntry>> {
    load_blueprint_slot_registry_for_role_dir(role_dir, HOST_VERSION)
        .map_err(|e| anyhow::anyhow!(e.join("; ")))
}

fn save_registry(
    role_dir: &std::path::Path,
    reg: &BTreeMap<String, SlotRegistryEntry>,
) -> Result<()> {
    write_role_pack_blueprint_slot_registry(role_dir, reg, HOST_VERSION)
        .map_err(|e| anyhow::anyhow!(e.join("; ")))
}

#[derive(Serialize)]
struct SlotRowOut {
    key: String,
    #[serde(rename = "type")]
    slot_type: String,
    label: String,
    backend: String,
    position: i64,
    plugin: Option<String>,
}

fn cmd_list(role_dir: &std::path::Path, json: bool) -> Result<()> {
    let reg = load_registry(role_dir)?;
    let rows: Vec<SlotRowOut> = reg
        .iter()
        .map(|(k, e)| SlotRowOut {
            key: k.clone(),
            slot_type: e.slot_type.clone(),
            label: e.label.clone(),
            backend: e.backend.clone(),
            position: e.position,
            plugin: e.plugin.clone(),
        })
        .collect();
    if json {
        println!("{}", serde_json::to_string_pretty(&rows)?);
        return Ok(());
    }
    println!("Role: {}", role_dir.display());
    println!("slot_registry ({} entries):", rows.len());
    for r in &rows {
        let plug = r.plugin.as_deref().unwrap_or("—");
        println!(
            "  {:<20} type={:<16} backend={:<10} pos={} plugin={}",
            r.key, r.slot_type, r.backend, r.position, plug
        );
    }
    Ok(())
}

fn cmd_add_slot(
    role_dir: &std::path::Path,
    slot_type: &str,
    label: &str,
    json: bool,
) -> Result<()> {
    let mut reg = load_registry(role_dir)?;
    let st = slot_type.trim();
    let key = if !reg.contains_key(st) {
        st.to_string()
    } else {
        let mut n = 2u32;
        loop {
            let cand = format!("{st}_{n}");
            if !reg.contains_key(&cand) {
                break cand;
            }
            n += 1;
        }
    };
    let max_pos = reg.values().map(|e| e.position).max().unwrap_or(0);
    reg.insert(
        key.clone(),
        SlotRegistryEntry {
            slot_type: st.to_string(),
            label: label.trim().to_string(),
            backend: "builtin".into(),
            position: max_pos + 1,
            plugin: None,
            plugins: None,
            model: None,
            url: None,
            local_memory_provider_id: None,
            zone: None,
            policy: None,
        },
    );
    save_registry(role_dir, &reg)?;
    if json {
        println!(r#"{{"ok":true,"key":"{key}"}}"#);
    } else {
        println!("✓ Added slot {key}");
    }
    Ok(())
}

fn cmd_remove_slot(role_dir: &std::path::Path, key: &str, json: bool) -> Result<()> {
    let mut reg = load_registry(role_dir)?;
    if reg.remove(key.trim()).is_none() {
        bail!("slot key not found: {}", key);
    }
    if !reg.values().any(|e| e.slot_type == "llm") {
        bail!("cannot remove last llm slot");
    }
    save_registry(role_dir, &reg)?;
    if json {
        println!(r#"{{"ok":true,"removed":"{key}"}}"#);
    } else {
        println!("✓ Removed slot {key}");
    }
    Ok(())
}

fn cmd_set_backend(role_dir: &std::path::Path, key: &str, backend: &str, json: bool) -> Result<()> {
    let mut reg = load_registry(role_dir)?;
    let entry = reg
        .get_mut(key.trim())
        .context(format!("slot key not found: {key}"))?;
    oclive_validation::apply_slot_override(
        entry,
        &SlotOverridePatch {
            backend: Some(backend.trim().to_string()),
            plugin: None,
            plugins: None,
            model: None,
            local_memory_provider_id: None,
        },
    );
    save_registry(role_dir, &reg)?;
    if json {
        println!(r#"{{"ok":true,"key":"{key}","backend":"{backend}"}}"#);
    } else {
        println!("✓ {key} backend → {backend}");
    }
    Ok(())
}

fn cmd_link(role_dir: &std::path::Path, key: &str, plugin_id: &str, json: bool) -> Result<()> {
    let mut reg = load_registry(role_dir)?;
    let entry = reg
        .get_mut(key.trim())
        .context(format!("slot key not found: {key}"))?;
    entry.backend = "directory".into();
    entry.plugin = Some(plugin_id.trim().to_string());
    save_registry(role_dir, &reg)?;
    if json {
        println!(r#"{{"ok":true,"key":"{key}","plugin":"{plugin_id}"}}"#);
    } else {
        println!("✓ {key} → directory plugin {plugin_id}");
    }
    Ok(())
}

fn cmd_unlink(role_dir: &std::path::Path, key: &str, json: bool) -> Result<()> {
    let mut reg = load_registry(role_dir)?;
    let entry = reg
        .get_mut(key.trim())
        .context(format!("slot key not found: {key}"))?;
    entry.plugin = None;
    save_registry(role_dir, &reg)?;
    if json {
        println!(r#"{{"ok":true,"key":"{key}"}}"#);
    } else {
        println!("✓ Cleared plugin on {key}");
    }
    Ok(())
}
