//! `oclive plugin search` — scan locally installed directory plugins (manifest `provides` / keywords).

use anyhow::{bail, Result};
use clap::Parser;
use serde::Serialize;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Parser, Debug)]
pub struct PluginSearchArgs {
    /// Keyword (id, manifest fields); optional when --provides is set
    pub keyword: Option<String>,

    /// Filter by manifest `provides` slot (e.g. llm, memory)
    #[arg(long = "provides")]
    pub provides: Option<String>,

    #[arg(short = 'o', long, default_value = "./plugins")]
    pub plugins_dir: PathBuf,

    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Clone, Serialize)]
struct PluginSearchHit {
    id: String,
    version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    provides: Vec<String>,
    path: String,
}

pub fn run_search(args: PluginSearchArgs) -> Result<()> {
    let kw = args.keyword.as_deref().map(str::trim).filter(|s| !s.is_empty());
    let slot = args
        .provides
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| s.to_ascii_lowercase());
    if kw.is_none() && slot.is_none() {
        bail!("Pass a keyword and/or --provides <slot>");
    }
    let plugins_dir = args.plugins_dir.canonicalize().unwrap_or(args.plugins_dir);
    if !plugins_dir.is_dir() {
        bail!("plugins dir not found: {}", plugins_dir.display());
    }
    let hits = collect_hits(&plugins_dir, kw, slot.as_deref())?;
    if args.json {
        println!("{}", serde_json::to_string_pretty(&hits)?);
        return Ok(());
    }
    if hits.is_empty() {
        println!("No matching plugins under {}", plugins_dir.display());
        return Ok(());
    }
    for h in &hits {
        let slots = if h.provides.is_empty() {
            "—".to_string()
        } else {
            h.provides.join(", ")
        };
        let title = h.name.as_deref().unwrap_or(&h.id);
        println!(
            "[plugin] {} v{} — provides: {} — {}",
            title, h.version, slots, h.path
        );
    }
    Ok(())
}

fn parse_provides(v: &Value) -> Vec<String> {
    v.get("provides")
        .and_then(|x| x.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|e| e.as_str().map(str::trim).filter(|s| !s.is_empty()))
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

fn matches_keyword(v: &Value, id: &str, kw: &str) -> bool {
    let k = kw.to_ascii_lowercase();
    if id.to_ascii_lowercase().contains(&k) {
        return true;
    }
    for key in ["name", "description", "author"] {
        if v.get(key)
            .and_then(|x| x.as_str())
            .is_some_and(|s| s.to_ascii_lowercase().contains(&k))
        {
            return true;
        }
    }
    if parse_provides(v)
        .iter()
        .any(|p| p.to_ascii_lowercase().contains(&k))
    {
        return true;
    }
    false
}

fn collect_hits(plugins_dir: &Path, kw: Option<&str>, slot: Option<&str>) -> Result<Vec<PluginSearchHit>> {
    let mut hits = Vec::new();
    for entry in fs::read_dir(plugins_dir)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let root = entry.path();
        let manifest_path = root.join("manifest.json");
        if !manifest_path.is_file() {
            continue;
        }
        let raw = fs::read_to_string(&manifest_path)?;
        let v: Value = serde_json::from_str(&raw)?;
        let id = v
            .get("id")
            .and_then(|x| x.as_str())
            .unwrap_or("")
            .trim()
            .to_string();
        if id.is_empty() {
            continue;
        }
        let provides = parse_provides(&v);
        if let Some(want) = slot {
            if !provides.iter().any(|p| p.eq_ignore_ascii_case(want)) {
                continue;
            }
        }
        if let Some(k) = kw {
            if !matches_keyword(&v, &id, k) {
                continue;
            }
        }
        hits.push(PluginSearchHit {
            id,
            version: v
                .get("version")
                .and_then(|x| x.as_str())
                .unwrap_or("?")
                .to_string(),
            name: v.get("name").and_then(|x| x.as_str()).map(str::to_string),
            provides,
            path: root.display().to_string(),
        });
    }
    hits.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(hits)
}
