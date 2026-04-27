//! 从角色包目录加载 [`KnowledgeIndex`](crate::models::knowledge::KnowledgeIndex)：Markdown + YAML front matter。

use crate::error::{AppError, Result};
use crate::models::knowledge::{EventHintEntryDisk, KnowledgeChunk, KnowledgeIndex};
use crate::models::role_manifest_disk::DiskRoleManifest;
use crate::models::EventType;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Deserialize)]
struct KnowledgeFrontMatter {
    id: String,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    scenes: Option<Vec<String>>,
    #[serde(default = "default_chunk_weight")]
    weight: f64,
    #[serde(default)]
    event_hints: HashMap<String, EventHintEntryDisk>,
}

fn default_chunk_weight() -> f64 {
    1.0
}

fn parse_event_type_key(key: &str) -> Option<EventType> {
    match key.trim().to_ascii_lowercase().as_str() {
        "quarrel" => Some(EventType::Quarrel),
        "apology" => Some(EventType::Apology),
        "praise" => Some(EventType::Praise),
        "complaint" => Some(EventType::Complaint),
        "confession" => Some(EventType::Confession),
        "joke" => Some(EventType::Joke),
        "ignore" => Some(EventType::Ignore),
        _ => None,
    }
}

fn split_front_matter(raw: &str) -> Result<(Option<String>, &str)> {
    let t = raw.trim_start();
    if !t.starts_with("---") {
        return Ok((None, raw));
    }
    let after = t[3..].trim_start();
    let Some(end_rel) = after.find("\n---") else {
        return Ok((None, raw));
    };
    let fm_text = &after[..end_rel];
    let rest_start = end_rel + "\n---".len();
    let body = after[rest_start..].trim_start();
    Ok((Some(fm_text.to_string()), body))
}

fn parse_markdown_chunk(path: &Path, raw: &str) -> Result<KnowledgeChunk> {
    let (fm_opt, body) = split_front_matter(raw)?;
    let fm_str = fm_opt.ok_or_else(|| {
        AppError::InvalidParameter(format!(
            "知识文件缺少 YAML front matter（应以 --- 开头）: {}",
            path.display()
        ))
    })?;
    let fm: KnowledgeFrontMatter = serde_yaml::from_str(&fm_str).map_err(|e| {
        AppError::InvalidParameter(format!(
            "知识文件 front matter 解析失败 {}: {}",
            path.display(),
            e
        ))
    })?;
    let id = fm.id.trim().to_string();
    if id.is_empty() {
        return Err(AppError::InvalidParameter(format!(
            "知识块 id 不能为空: {}",
            path.display()
        )));
    }
    let mut event_hints: HashMap<EventType, Vec<String>> = HashMap::new();
    for (key, hint) in fm.event_hints {
        let Some(et) = parse_event_type_key(&key) else {
            return Err(AppError::InvalidParameter(format!(
                "知识文件 {} 中 event_hints 含未知事件键「{}」（须为 quarrel|apology|praise|complaint|confession|joke|ignore）",
                path.display(),
                key
            )));
        };
        let kws: Vec<String> = hint
            .keywords
            .into_iter()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        if !kws.is_empty() {
            event_hints.insert(et, kws);
        }
        if let Some(w) = hint.weight {
            if !w.is_finite() || w < 0.0 {
                return Err(AppError::InvalidParameter(format!(
                    "知识文件 {} 中 event_hints.{}.weight 须为非负有限数",
                    path.display(),
                    key
                )));
            }
        }
    }
    Ok(KnowledgeChunk {
        id,
        source_path: path.to_path_buf(),
        tags: fm.tags,
        scenes: fm.scenes,
        weight: if fm.weight.is_finite() && fm.weight > 0.0 {
            fm.weight
        } else {
            1.0
        },
        body: body.trim().to_string(),
        event_hints,
    })
}

/// 是否应在本次加载中读取知识（manifest 省略时仅当存在 `knowledge/` 目录）。
pub fn should_load_knowledge(disk: &DiskRoleManifest, role_dir: &Path) -> bool {
    match &disk.knowledge {
        Some(k) => k.enabled,
        None => role_dir.join("knowledge").is_dir(),
    }
}

/// 解析 manifest 中的知识配置；`None` 表示不加载（未启用或自动模式下无目录）。
pub fn resolved_knowledge_glob(disk: &DiskRoleManifest, role_dir: &Path) -> Option<String> {
    if !should_load_knowledge(disk, role_dir) {
        return None;
    }
    let g = disk
        .knowledge
        .as_ref()
        .map(|k| k.glob.as_str())
        .unwrap_or("knowledge/**/*.md");
    Some(g.to_string())
}

/// 枚举 `roles/{id}/knowledge/` 下（递归）所有 `.md` 文件。
/// `glob` 仅支持约定形式 `knowledge/**/*.md` 或 `knowledge/**`（忽略末尾片段，行为一致）。
pub fn collect_knowledge_files(role_dir: &Path, glob_pat: &str) -> Result<Vec<PathBuf>> {
    let g = glob_pat.trim();
    if !g.starts_with("knowledge/") {
        return Err(AppError::InvalidParameter(
            "knowledge.glob 须以 knowledge/ 开头（仓库约定目录名为 knowledge）".into(),
        ));
    }
    let base = role_dir.join("knowledge");
    if !base.is_dir() {
        return Ok(vec![]);
    }
    let mut out = Vec::new();
    for e in WalkDir::new(&base)
        .into_iter()
        .filter_map(std::result::Result::ok)
    {
        let p = e.path();
        if p.is_file() && p.extension().and_then(|x| x.to_str()) == Some("md") {
            out.push(p.to_path_buf());
        }
    }
    out.sort();
    Ok(out)
}

/// 加载并校验整包知识；任一文件失败则返回错误（避免半包静默）。
pub fn load_knowledge_index(role_dir: &Path, disk: &DiskRoleManifest) -> Result<KnowledgeIndex> {
    let Some(glob_pat) = resolved_knowledge_glob(disk, role_dir) else {
        return Ok(KnowledgeIndex::default());
    };
    let explicit = disk.knowledge.is_some();
    let paths = collect_knowledge_files(role_dir, &glob_pat)?;
    if paths.is_empty() {
        if explicit {
            return Err(AppError::InvalidParameter(format!(
                "角色包知识已启用（glob「{}」），但未找到任何 .md 文件；请添加文件或关闭 knowledge.enabled",
                glob_pat
            )));
        }
        return Ok(KnowledgeIndex::default());
    }
    let mut chunks = Vec::new();
    let mut seen_ids = HashSet::<String>::new();
    for p in paths {
        let raw = fs::read_to_string(&p).map_err(AppError::IoError)?;
        let chunk = parse_markdown_chunk(&p, &raw)?;
        if seen_ids.contains(&chunk.id) {
            return Err(AppError::InvalidParameter(format!(
                "知识块 id 重复: {}（文件 {}）",
                chunk.id,
                p.display()
            )));
        }
        seen_ids.insert(chunk.id.clone());
        chunks.push(chunk);
    }
    Ok(KnowledgeIndex { chunks })
}

pub use oclive_validation::validate_knowledge_manifest_disk;
