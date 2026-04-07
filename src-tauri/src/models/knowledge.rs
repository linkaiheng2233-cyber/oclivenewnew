//! 角色包「世界观知识」：`roles/{id}/knowledge/**/*.md`，YAML front matter + 正文。
//!
//! 目录名固定为 **`knowledge/`**（不用 `worldview/`，与计划及 manifest 字段一致）。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

use super::EventType;

/// manifest.json / settings.json 可选块：`"knowledge": { "enabled": true, "glob": "knowledge/**/*.md" }`
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KnowledgePackConfigDisk {
    /// 为 `false` 时不加载（即使存在 `knowledge/` 目录）。
    #[serde(default = "default_knowledge_enabled")]
    pub enabled: bool,
    /// 相对角色包根目录的 glob，默认扫描所有子目录下的 `.md`。
    #[serde(default = "default_knowledge_glob")]
    pub glob: String,
}

fn default_knowledge_enabled() -> bool {
    true
}

fn default_knowledge_glob() -> String {
    "knowledge/**/*.md".to_string()
}

/// 单条 front matter 中的 `event_hints` 子表（键为事件类型 snake_name）。
#[derive(Debug, Clone, Deserialize)]
pub struct EventHintEntryDisk {
    #[serde(default)]
    pub keywords: Vec<String>,
    /// 预留：用于后续加权排序；当前仅解析校验。
    #[serde(default)]
    pub weight: Option<f64>,
}

/// 内存中的一条知识块
#[derive(Debug, Clone)]
pub struct KnowledgeChunk {
    pub id: String,
    pub source_path: PathBuf,
    #[allow(dead_code)]
    pub tags: Vec<String>,
    /// `None` 表示不限场景；否则仅在这些 `scene_id` 参与检索。
    pub scenes: Option<Vec<String>>,
    pub weight: f64,
    pub body: String,
    /// 自 front matter 解析；键为 [`EventType`]
    pub event_hints: HashMap<EventType, Vec<String>>,
}

/// 按角色加载后的知识索引（仅内存；随 `load_role` 刷新）。
#[derive(Debug, Clone, Default)]
pub struct KnowledgeIndex {
    pub chunks: Vec<KnowledgeChunk>,
}

impl KnowledgeIndex {
    /// 对用户句做轻量重合打分 + 场景过滤，取 Top-K（确定性排序：分数降序，`id` 升序）。
    pub fn retrieve<'a>(
        &'a self,
        user_message: &str,
        scene_id: Option<&str>,
        top_k: usize,
    ) -> Vec<&'a KnowledgeChunk> {
        let k = top_k.max(1);
        let mut scored: Vec<(f64, &str, &'a KnowledgeChunk)> = self
            .chunks
            .iter()
            .filter(|c| Self::scene_allows(c, scene_id))
            .map(|c| {
                let s = Self::score_chunk(user_message, c);
                (s * c.weight, c.id.as_str(), c)
            })
            .filter(|(s, _, _)| *s > 0.0)
            .collect();
        scored.sort_by(|a, b| {
            b.0.partial_cmp(&a.0)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.1.cmp(b.1))
        });
        scored.into_iter().take(k).map(|(_, _, c)| c).collect()
    }

    fn scene_allows(chunk: &KnowledgeChunk, scene_id: Option<&str>) -> bool {
        let Some(filter) = &chunk.scenes else {
            return true;
        };
        if filter.is_empty() {
            return true;
        }
        let Some(sid) = scene_id else {
            return true;
        };
        filter.iter().any(|s| s == sid)
    }

    fn score_chunk(query: &str, chunk: &KnowledgeChunk) -> f64 {
        let q = query.trim();
        if q.is_empty() {
            return 0.0;
        }
        let ql = q.to_lowercase();
        let hay = format!(
            "{} {}",
            chunk.body.to_lowercase(),
            chunk.tags.join(" ").to_lowercase()
        );
        let mut hits = 0usize;
        for w in ql.split_whitespace() {
            if w.len() >= 2 && hay.contains(w) {
                hits += 1;
            }
        }
        if hits == 0 && ql.chars().count() >= 2 {
            for w in ql.as_str().chars().collect::<Vec<_>>().windows(2) {
                let s: String = w.iter().collect();
                if hay.contains(&s) {
                    hits += 1;
                }
            }
        }
        (hits as f64 * 0.2).min(1.0)
    }

    /// 将检索到的块合并为 Prompt 用纯文本（已截断）。
    pub fn format_for_prompt(chunks: &[&KnowledgeChunk], max_chars: usize) -> String {
        let mut out = String::new();
        for c in chunks {
            if !out.is_empty() {
                out.push_str("\n\n---\n\n");
            }
            let block = format!("（{}）\n{}", c.id, c.body.trim());
            if out.len() + block.len() > max_chars {
                let remain = max_chars.saturating_sub(out.len());
                if remain > 0 {
                    let take = block.chars().take(remain).collect::<String>();
                    out.push_str(&take);
                }
                break;
            }
            out.push_str(&block);
        }
        out
    }

    /// 从检索结果合并事件关键词，供 [`crate::domain::event_detector::EventDetector`] 使用。
    pub fn merge_event_augment(chunks: &[&KnowledgeChunk]) -> KnowledgeEventAugment {
        let mut by_event: HashMap<EventType, Vec<String>> = HashMap::new();
        for ch in chunks {
            for (et, kws) in &ch.event_hints {
                let entry = by_event.entry(*et).or_default();
                for kw in kws {
                    let t = kw.trim();
                    if !t.is_empty() && !entry.iter().any(|x: &String| x == t) {
                        entry.push(t.to_string());
                    }
                }
            }
        }
        KnowledgeEventAugment { by_event }
    }
}

/// 知识驱动的额外事件关键词（B1：作为 `EventDetector` 的补充输入）。
#[derive(Debug, Clone, Default)]
pub struct KnowledgeEventAugment {
    pub by_event: HashMap<EventType, Vec<String>>,
}

impl KnowledgeEventAugment {
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.by_event.values().all(|v| v.is_empty())
    }
}
