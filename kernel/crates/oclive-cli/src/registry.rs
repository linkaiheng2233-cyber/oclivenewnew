//! Local kernel project registry (`~/.oclive/registry.json`).

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use crate::init::{InitTemplateArg, ProjectConfig};

const REGISTRY_SCHEMA: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryFile {
    pub schema: u32,
    pub projects: Vec<RegistryEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryEntry {
    pub name: String,
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template: Option<String>,
    pub created_at: u64,
}

pub fn oclive_home() -> PathBuf {
    if let Ok(p) = std::env::var("OCLIVE_HOME") {
        if !p.trim().is_empty() {
            return PathBuf::from(p.trim());
        }
    }
    dirs_home().join(".oclive")
}

fn dirs_home() -> PathBuf {
    std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
}

pub fn registry_path() -> PathBuf {
    oclive_home().join("registry.json")
}

pub fn load_registry() -> Result<RegistryFile> {
    let path = registry_path();
    if !path.is_file() {
        return Ok(RegistryFile {
            schema: REGISTRY_SCHEMA,
            projects: vec![],
        });
    }
    let raw = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let mut file: RegistryFile = serde_json::from_str(&raw).context("parse registry.json")?;
    if file.schema == 0 {
        file.schema = REGISTRY_SCHEMA;
    }
    Ok(file)
}

pub fn save_registry(file: &RegistryFile) -> Result<()> {
    let path = registry_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    let mut out = file.clone();
    out.schema = REGISTRY_SCHEMA;
    fs::write(&path, serde_json::to_string_pretty(&out)?)
        .with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

pub fn register_project(name: &str, path: &Path, template: Option<InitTemplateArg>) -> Result<()> {
    let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let mut file = load_registry()?;
    let template_s = template.map(template_id_str).map(str::to_string);
    let entry = RegistryEntry {
        name: name.to_string(),
        path: canonical.display().to_string(),
        template: template_s,
        created_at: now_ts(),
    };
    if let Some(i) = file.projects.iter().position(|p| p.name == entry.name) {
        file.projects[i] = entry;
    } else {
        file.projects.push(entry);
    }
    save_registry(&file)
}

fn template_id_str(t: InitTemplateArg) -> &'static str {
    match t {
        InitTemplateArg::RobotSoul => "robot-soul",
        InitTemplateArg::RobotGateway => "robot-gateway",
        InitTemplateArg::DialogueOnly => "dialogue-only",
        InitTemplateArg::HeadlessApi => "headless-api",
        InitTemplateArg::LibraryEmbed => "library-embed",
    }
}

pub fn register_after_init(cfg: &ProjectConfig, output: &Path) -> Result<()> {
    register_project(cfg.project_name.as_str(), output, cfg.factory_template)
}

fn now_ts() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

pub fn remove_entry(name: &str) -> Result<bool> {
    let mut file = load_registry()?;
    let before = file.projects.len();
    file.projects.retain(|p| p.name != name);
    if file.projects.len() == before {
        return Ok(false);
    }
    save_registry(&file)?;
    Ok(true)
}

pub fn find_entry(name: &str) -> Result<Option<RegistryEntry>> {
    let file = load_registry()?;
    Ok(file.projects.into_iter().find(|p| p.name == name))
}
