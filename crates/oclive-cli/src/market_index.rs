//! 插件 / 模板 / 角色包市场索引（在线 + `~/.oclive/plugin_index_cache.json`）。

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::registry::oclive_home;
use crate::template_catalog::CATALOG;

pub const DEFAULT_MARKET_INDEX_URL: &str =
    "https://raw.githubusercontent.com/linkaiheng2233-cyber/oclivenewnew/main/examples/market-index.json";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarketKind {
    Plugin,
    Template,
    RolePack,
}

impl MarketKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Plugin => "插件",
            Self::Template => "模板",
            Self::RolePack => "角色包",
        }
    }

    pub fn id(self) -> &'static str {
        match self {
            Self::Plugin => "plugins",
            Self::Template => "templates",
            Self::RolePack => "role_packs",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketItem {
    pub id: String,
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub install_count: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub download_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub git: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub template_id: Option<String>,
    #[serde(default = "default_kind_plugin")]
    pub kind: MarketKindSerde,
}

fn default_kind_plugin() -> MarketKindSerde {
    MarketKindSerde::Plugin
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MarketKindSerde {
    Plugin,
    Template,
    RolePack,
}

impl From<MarketKindSerde> for MarketKind {
    fn from(k: MarketKindSerde) -> Self {
        match k {
            MarketKindSerde::Plugin => MarketKind::Plugin,
            MarketKindSerde::Template => MarketKind::Template,
            MarketKindSerde::RolePack => MarketKind::RolePack,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MarketIndexFile {
    #[serde(default)]
    pub schema_version: u32,
    #[serde(default)]
    pub plugins: Vec<MarketItem>,
    #[serde(default)]
    pub templates: Vec<MarketItem>,
    #[serde(default)]
    pub role_packs: Vec<MarketItem>,
}

impl MarketIndexFile {
    pub fn all_items(&self) -> Vec<MarketItem> {
        let mut out = Vec::new();
        out.extend(self.plugins.iter().cloned());
        out.extend(self.templates.iter().cloned());
        out.extend(self.role_packs.iter().cloned());
        out
    }

    pub fn items_for_kind(&self, kind: MarketKind) -> Vec<MarketItem> {
        match kind {
            MarketKind::Plugin => self.plugins.clone(),
            MarketKind::Template => {
                let mut t = self.templates.clone();
                if t.is_empty() {
                    t = builtin_templates();
                }
                t
            }
            MarketKind::RolePack => self.role_packs.clone(),
        }
    }
}

pub fn cache_path() -> PathBuf {
    oclive_home().join("plugin_index_cache.json")
}

pub fn index_url() -> String {
    std::env::var("OCLIVE_PLUGIN_INDEX_URL")
        .or_else(|_| std::env::var("OCLIVE_MARKET_INDEX_URL"))
        .unwrap_or_else(|_| DEFAULT_MARKET_INDEX_URL.to_string())
}

/// 拉取市场索引；失败时回退缓存。
pub fn fetch_market_index() -> Result<MarketIndexFile> {
    let url = index_url();
    match fetch_online(&url) {
        Ok(file) => {
            let _ = save_cache(&file);
            Ok(file)
        }
        Err(e) => {
            if let Ok(cached) = load_cache() {
                eprintln!("⚠ 在线索引失败 ({e})，使用缓存 {}", cache_path().display());
                return Ok(cached);
            }
            Err(e)
        }
    }
}

pub fn fetch_online(url: &str) -> Result<MarketIndexFile> {
    let body = ureq::get(url)
        .call()
        .map_err(|e| anyhow::anyhow!("拉取市场索引失败: {e}"))?
        .into_string()?;
    parse_index_json(&body)
}

pub fn parse_index_json(body: &str) -> Result<MarketIndexFile> {
    let v: serde_json::Value = serde_json::from_str(body).context("parse index JSON")?;
    if v.get("plugins").is_some() || v.get("templates").is_some() {
        let mut file: MarketIndexFile = serde_json::from_value(v)?;
        tag_kind_on_sections(&mut file);
        if file.templates.is_empty() {
            file.templates = builtin_templates();
        }
        return Ok(file);
    }
    // 兼容仅含 plugins 的旧 plugin-index.json
    #[derive(Deserialize)]
    struct Legacy {
        plugins: Vec<LegacyPlugin>,
    }
    #[derive(Deserialize)]
    struct LegacyPlugin {
        id: String,
        name: String,
        version: String,
        #[serde(default)]
        author: String,
        #[serde(default)]
        description: String,
        #[serde(default)]
        tags: Vec<String>,
    }
    let leg: Legacy = serde_json::from_str(body).context("parse legacy plugin index")?;
    Ok(MarketIndexFile {
        schema_version: 1,
        plugins: leg
            .plugins
            .into_iter()
            .map(|p| MarketItem {
                id: p.id,
                name: p.name,
                version: p.version,
                author: p.author,
                description: p.description,
                tags: p.tags,
                install_count: 0,
                download_url: None,
                git: None,
                template_id: None,
                kind: MarketKindSerde::Plugin,
            })
            .collect(),
        templates: builtin_templates(),
        role_packs: vec![],
    })
}

fn builtin_templates() -> Vec<MarketItem> {
    CATALOG
        .iter()
        .map(|e| MarketItem {
            id: format!("template:{}", e.id),
            name: e.id.to_string(),
            version: "1.0.0".into(),
            author: "oclive".into(),
            description: e.description.to_string(),
            tags: vec![e.scene.to_string(), e.preset.to_string()],
            install_count: 0,
            download_url: None,
            git: None,
            template_id: Some(e.id.to_string()),
            kind: MarketKindSerde::Template,
        })
        .collect()
}

pub fn save_cache(file: &MarketIndexFile) -> Result<()> {
    let path = cache_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&path, serde_json::to_string_pretty(file)?).context("write cache")?;
    Ok(())
}

pub fn load_cache() -> Result<MarketIndexFile> {
    let raw = fs::read_to_string(cache_path()).context("read plugin_index_cache.json")?;
    parse_index_json(&raw)
}

pub fn search_items(index: &MarketIndexFile, keyword: &str) -> Vec<MarketItem> {
    let kw = keyword.to_ascii_lowercase();
    index
        .all_items()
        .into_iter()
        .filter(|p| matches_keyword(p, &kw))
        .collect()
}

pub fn find_item(index: &MarketIndexFile, id: &str) -> Option<MarketItem> {
    if let Some(tid) = id.strip_prefix("template:") {
        return index
            .items_for_kind(MarketKind::Template)
            .into_iter()
            .find(|i| i.template_id.as_deref() == Some(tid) || i.id == id);
    }
    index
        .all_items()
        .into_iter()
        .find(|i| i.id == id || i.template_id.as_deref() == Some(id))
}

fn tag_kind_on_sections(file: &mut MarketIndexFile) {
    for p in &mut file.plugins {
        p.kind = MarketKindSerde::Plugin;
    }
    for p in &mut file.templates {
        p.kind = MarketKindSerde::Template;
    }
    for p in &mut file.role_packs {
        p.kind = MarketKindSerde::RolePack;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_plugin_index_parses() {
        let raw = include_str!("../../../examples/plugin-index.json");
        let f = parse_index_json(raw).unwrap();
        assert!(!f.plugins.is_empty());
        assert!(!f.templates.is_empty());
    }
}

fn matches_keyword(p: &MarketItem, kw: &str) -> bool {
    p.id.to_ascii_lowercase().contains(kw)
        || p.name.to_ascii_lowercase().contains(kw)
        || p.description.to_ascii_lowercase().contains(kw)
        || p.author.to_ascii_lowercase().contains(kw)
        || p.tags.iter().any(|t| t.to_ascii_lowercase().contains(kw))
}
