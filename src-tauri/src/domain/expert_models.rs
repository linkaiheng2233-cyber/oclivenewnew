//! Module 9: Expert Models compiler + safe apply to llama local sidecar config.

use crate::error::{AppError, Result};
use crate::models::expert_models::{
    ExpertGraph, ExpertNode, LlamaLocalPluginConfig, PromptStyleOverride,
};
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::{Path, PathBuf};

pub const LLAMA_LOCAL_PLUGIN_ID: &str = "com.oclive.llama.local";

#[derive(Debug, Clone)]
pub struct ExpertEffectiveConfig {
    pub graph: ExpertGraph,
    pub prompt_style: Option<PromptStyleOverride>,
}

fn is_path_under(child: &Path, parent: &Path) -> bool {
    let cn = child.canonicalize().unwrap_or_else(|_| child.to_path_buf());
    let pn = parent
        .canonicalize()
        .unwrap_or_else(|_| parent.to_path_buf());
    cn.starts_with(pn)
}

pub fn compile_graph_to_llama_local_config(
    graph: &ExpertGraph,
    models_gguf_dir: &Path,
    loras_dir: &Path,
) -> Result<LlamaLocalPluginConfig> {
    let nodes = graph.nodes.as_slice();
    let edges = graph.edges.as_slice();
    let has_edges = edges
        .iter()
        .any(|e| !e.from.trim().is_empty() && !e.to.trim().is_empty());

    let mut by_id: HashMap<&str, &ExpertNode> = HashMap::new();
    for n in nodes {
        let id = match n {
            ExpertNode::BaseModel { id, .. } => id.as_str(),
            ExpertNode::LoraAdapter { id, .. } => id.as_str(),
            ExpertNode::PromptStyle { id, .. } => id.as_str(),
        };
        if !id.trim().is_empty() {
            by_id.insert(id, n);
        }
    }

    let mut adj: HashMap<&str, Vec<&str>> = HashMap::new();
    if has_edges {
        for e in edges {
            let from = e.from.trim();
            let to = e.to.trim();
            if from.is_empty() || to.is_empty() {
                continue;
            }
            if !by_id.contains_key(from) || !by_id.contains_key(to) {
                continue;
            }
            adj.entry(from).or_default().push(to);
        }
        for v in adj.values_mut() {
            v.sort();
            v.dedup();
        }
    }

    let base_ids: Vec<&str> = nodes
        .iter()
        .filter_map(|n| match n {
            ExpertNode::BaseModel { id, .. } => Some(id.as_str()),
            _ => None,
        })
        .filter(|s| !s.trim().is_empty())
        .collect();

    fn reachable<'a>(start: &'a str, adj: &HashMap<&'a str, Vec<&'a str>>) -> HashSet<&'a str> {
        let mut seen: HashSet<&'a str> = HashSet::new();
        let mut q: VecDeque<&'a str> = VecDeque::new();
        seen.insert(start);
        q.push_back(start);
        while let Some(cur) = q.pop_front() {
            if let Some(nexts) = adj.get(cur) {
                for &n in nexts {
                    if seen.insert(n) {
                        q.push_back(n);
                    }
                }
            }
        }
        seen
    }

    let active_base_id: Option<&str> = if base_ids.is_empty() {
        None
    } else if base_ids.len() == 1 || !has_edges {
        base_ids.first().copied()
    } else {
        // Choose base with the largest reachable set; tie-break by id.
        let mut best: Option<(&str, usize)> = None;
        for &bid in &base_ids {
            let r = reachable(bid, &adj);
            let score = r.len();
            match best {
                None => best = Some((bid, score)),
                Some((cur, cur_score)) => {
                    if score > cur_score || (score == cur_score && bid < cur) {
                        best = Some((bid, score));
                    }
                }
            }
        }
        best.map(|(id, _)| id)
    };

    let reachable_set: HashSet<&str> = if let (true, Some(bid)) = (has_edges, active_base_id) {
        reachable(bid, &adj)
    } else {
        HashSet::new()
    };

    let base = nodes.iter().find_map(|n| match (n, active_base_id) {
        (ExpertNode::BaseModel { id, gguf_path, .. }, Some(bid)) if id == bid => {
            Some(gguf_path.as_str())
        }
        (ExpertNode::BaseModel { gguf_path, .. }, None) => Some(gguf_path.as_str()),
        _ => None,
    });

    let model_path = base
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| PathBuf::from(s));

    if let Some(ref p) = model_path {
        if !p.is_file() {
            return Err(AppError::InvalidParameter(format!(
                "base model path not found: {}",
                p.display()
            )));
        }
        if !is_path_under(p, models_gguf_dir) {
            return Err(AppError::InvalidParameter(format!(
                "base model must be under models/gguf (got {})",
                p.display()
            )));
        }
    }

    // loras: enabled LoraAdapter nodes. If graph has edges, require reachability from active base.
    let mut loras: Vec<(String, f32, i32, String)> = vec![];
    for n in &graph.nodes {
        if let ExpertNode::LoraAdapter {
            id,
            gguf_path,
            strength,
            enabled,
            order,
            ..
        } = n
        {
            if !*enabled {
                continue;
            }
            if has_edges {
                let nid = id.as_str();
                if active_base_id.is_some() && !reachable_set.contains(nid) {
                    continue;
                }
            }
            let path = gguf_path.trim();
            if path.is_empty() {
                continue;
            }
            let p = PathBuf::from(path);
            if !p.is_file() {
                return Err(AppError::InvalidParameter(format!(
                    "LoRA not found: {}",
                    p.display()
                )));
            }
            if !(is_path_under(&p, loras_dir) || is_path_under(&p, models_gguf_dir)) {
                return Err(AppError::InvalidParameter(format!(
                    "LoRA must be under models/loras or models/gguf (got {})",
                    p.display()
                )));
            }
            if !strength.is_finite() {
                return Err(AppError::InvalidParameter(format!(
                    "LoRA strength must be finite (id={})",
                    id
                )));
            }
            loras.push((path.to_string(), *strength, *order, id.clone()));
        }
    }
    loras.sort_by(|a, b| (a.2, &a.3).cmp(&(b.2, &b.3)));

    let llama_args = if loras.is_empty() {
        None
    } else {
        // Conservative mapping: repeat `--lora <path> --lora-scale <strength>` pairs.
        // This keeps compatibility with sidecar's whitespace-splitting arg forwarding.
        let mut parts: Vec<String> = Vec::new();
        for (p, s, _, _id) in loras.iter() {
            parts.push("--lora".to_string());
            parts.push(p.to_string());
            parts.push("--lora-scale".to_string());
            parts.push(format!("{:.6}", *s));
        }
        Some(parts.join(" "))
    };

    Ok(LlamaLocalPluginConfig {
        model_path: model_path.map(|p| p.to_string_lossy().to_string()),
        llama_args,
    })
}
