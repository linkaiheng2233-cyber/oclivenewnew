//! Role pack worldview knowledge: `roles/{id}/knowledge/**/*.md`, YAML front matter + body.
//!
//! Directory name is fixed as **`knowledge/`** (not `worldview/`; aligned with the plan and manifest fields).

use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;

use super::EventType;

pub use oclive_validation::KnowledgePackConfigDisk;

/// `event_hints` sub-table in a single front matter entry (keys are event-type snake_names).
#[derive(Debug, Clone, Deserialize)]
pub struct EventHintEntryDisk {
    #[serde(default)]
    pub keywords: Vec<String>,
    /// Reserved for future weighted ranking; currently parsed and validated only.
    #[serde(default)]
    pub weight: Option<f64>,
}

/// A single in-memory knowledge chunk.
#[derive(Debug, Clone)]
pub struct KnowledgeChunk {
    pub id: String,
    pub source_path: PathBuf,
    #[allow(dead_code)]
    pub tags: Vec<String>,
    /// `None` means all scenes; otherwise only these `scene_id` values participate in retrieval.
    pub scenes: Option<Vec<String>>,
    pub weight: f64,
    pub body: String,
    /// Parsed from front matter; keys are [`EventType`].
    pub event_hints: HashMap<EventType, Vec<String>>,
}

/// Knowledge index loaded per role (in-memory only; refreshed on `load_role`).
#[derive(Debug, Clone, Default)]
pub struct KnowledgeIndex {
    pub chunks: Vec<KnowledgeChunk>,
}

impl KnowledgeIndex {
    /// Lightweight overlap scoring on the user sentence + scene filter; returns Top-K (deterministic sort: score desc, `id` asc).
    #[must_use]
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

    /// Merge retrieved chunks into plain text for the prompt (truncated).
    #[must_use]
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

    /// Merge event keywords from retrieval results for [`crate::domain::event_detector::EventDetector`].
    #[must_use]
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

/// Knowledge-driven extra event keywords (B1: supplementary input to `EventDetector`).
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
