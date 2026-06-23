//! Global / project-level config (`~/.oclive/config.toml`, `.oclive.toml`).

use anyhow::{Context, Result};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::registry::oclive_home;

/// Keys manageable in the config file (also supports `oclive config set` for any `OCLIVE_*` key).
pub const KNOWN_KEYS: &[&str] = &[
    "OCLIVE_REGISTRY_URL",
    "OCLIVE_REGISTRY_TOKEN",
    "OCLIVE_MARKET_INDEX_URL",
    "OCLIVE_PLUGIN_INDEX_URL",
    "OCLIVE_LLAMACPP_SERVER_URL",
    "OCLIVE_REMOTE_LLM_URL",
    "OCLIVE_REMOTE_PLUGIN_URL",
    "OCLIVE_HOME",
    "OCLIVE_ROOT",
    "OCLIVE_HTTP_API_MOCK_LLM",
];

pub fn global_config_path() -> PathBuf {
    oclive_home().join("config.toml")
}

pub fn local_config_path(project_root: &Path) -> PathBuf {
    project_root.join(".oclive.toml")
}

fn load_toml_map(path: &Path) -> Result<BTreeMap<String, String>> {
    if !path.is_file() {
        return Ok(BTreeMap::new());
    }
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let v: toml::Value = toml::from_str(&raw).context("parse config toml")?;
    let Some(table) = v.as_table() else {
        return Ok(BTreeMap::new());
    };
    let mut out = BTreeMap::new();
    for (k, val) in table {
        let s = match val {
            toml::Value::String(s) => s.clone(),
            _ => val.to_string(),
        };
        if !s.is_empty() {
            out.insert(k.clone(), s);
        }
    }
    Ok(out)
}

fn save_toml_map(path: &Path, map: &BTreeMap<String, String>) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut table = toml::map::Map::new();
    for (k, v) in map {
        table.insert(k.clone(), toml::Value::String(v.clone()));
    }
    let doc = toml::Value::Table(table);
    fs::write(path, toml::to_string_pretty(&doc)?)
        .with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

/// Resolve config: process env vars > project `.oclive.toml` > user `config.toml`.
pub fn resolve(key: &str, project_root: Option<&Path>) -> Option<String> {
    if let Ok(v) = std::env::var(key) {
        if !v.trim().is_empty() {
            return Some(v);
        }
    }
    if let Some(root) = project_root {
        if let Ok(local) = load_toml_map(&local_config_path(root)) {
            if let Some(v) = local.get(key) {
                return Some(v.clone());
            }
        }
    }
    load_toml_map(&global_config_path())
        .ok()
        .and_then(|g| g.get(key).cloned())
}

pub fn set_key(
    key: &str,
    value: &str,
    global: bool,
    project_root: Option<&Path>,
) -> Result<PathBuf> {
    let path = if global {
        global_config_path()
    } else {
        let root =
            project_root.context("project-level config requires a project directory (cwd)")?;
        local_config_path(root)
    };
    let mut map = load_toml_map(&path)?;
    map.insert(key.to_string(), value.to_string());
    save_toml_map(&path, &map)?;
    Ok(path)
}

pub fn unset_key(key: &str, global: bool, project_root: Option<&Path>) -> Result<PathBuf> {
    let path = if global {
        global_config_path()
    } else {
        let root = project_root.context("project-level config requires a project directory")?;
        local_config_path(root)
    };
    let mut map = load_toml_map(&path)?;
    map.remove(key);
    save_toml_map(&path, &map)?;
    Ok(path)
}

pub fn get_key(key: &str, global: bool, project_root: Option<&Path>) -> Result<Option<String>> {
    if global {
        Ok(load_toml_map(&global_config_path())?.get(key).cloned())
    } else {
        let root = project_root.context("project-level config requires a project directory")?;
        Ok(load_toml_map(&local_config_path(root))?.get(key).cloned())
    }
}

pub fn list_keys(global: bool, project_root: Option<&Path>) -> Result<BTreeMap<String, String>> {
    let file_map = if global {
        load_toml_map(&global_config_path())?
    } else {
        let root = project_root.context("project-level config requires a project directory")?;
        load_toml_map(&local_config_path(root))?
    };
    let mut merged = BTreeMap::new();
    for k in KNOWN_KEYS {
        if let Some(v) = resolve(k, project_root) {
            merged.insert((*k).to_string(), v);
        }
    }
    for (k, v) in file_map {
        merged.entry(k).or_insert(v);
    }
    Ok(merged)
}

/// Import `OCLIVE_*` env vars set in the current process into the global config (without overwriting existing file keys).
pub fn import_env_to_global() -> Result<usize> {
    let path = global_config_path();
    let mut map = load_toml_map(&path)?;
    let mut n = 0usize;
    for (k, v) in std::env::vars() {
        if k.starts_with("OCLIVE_") && !v.trim().is_empty() && !map.contains_key(&k) {
            map.insert(k, v);
            n += 1;
        }
    }
    if n > 0 {
        save_toml_map(&path, &map)?;
    }
    Ok(n)
}
