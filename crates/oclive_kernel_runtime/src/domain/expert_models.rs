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

/// 双轨编译产物：本地侧车配置，和/或「本会话走宿主 OpenAI 兼容云端 LLM」标志。
#[derive(Debug, Clone)]
pub struct ExpertCompilePlan {
    pub use_remote_llm: bool,
    /// 非空时覆盖云端请求体 `model`；空串表示仅用宿主 `host_cloud_llm_json` 默认模型。
    pub remote_model_override: Option<String>,
    pub llama: LlamaLocalPluginConfig,
}

fn is_path_under(child: &Path, parent: &Path) -> bool {
    let cn = child.canonicalize().unwrap_or_else(|_| child.to_path_buf());
    let pn = parent
        .canonicalize()
        .unwrap_or_else(|_| parent.to_path_buf());
    cn.starts_with(pn)
}

fn reachable_from_base<'a>(
    start: &'a str,
    adj: &HashMap<&'a str, Vec<&'a str>>,
) -> HashSet<&'a str> {
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

/// 编译专家图：若存在可达且启用的 `CloudModel`（`host`），则 `use_remote_llm` 为真且不再要求本地 GGUF。
pub fn compile_expert_graph_plan(
    graph: &ExpertGraph,
    models_gguf_dir: &Path,
    loras_dir: &Path,
) -> Result<ExpertCompilePlan> {
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
            ExpertNode::CloudModel { id, .. } => id.as_str(),
            ExpertNode::EventTrigger { id, .. } => id.as_str(),
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

    let active_base_id: Option<&str> = if base_ids.is_empty() {
        None
    } else if base_ids.len() == 1 || !has_edges {
        base_ids.first().copied()
    } else {
        let mut best: Option<(&str, usize)> = None;
        for &bid in &base_ids {
            let r = reachable_from_base(bid, &adj);
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
        reachable_from_base(bid, &adj)
    } else {
        HashSet::new()
    };

    let mut remote_model_override: Option<String> = None;
    let mut use_remote_llm = false;
    for n in nodes {
        let ExpertNode::CloudModel {
            id,
            host_source,
            model,
            enabled,
            ..
        } = n
        else {
            continue;
        };
        if !*enabled {
            continue;
        }
        let nid = id.as_str().trim();
        if nid.is_empty() {
            continue;
        }
        let src = host_source.trim();
        if !src.is_empty() && !src.eq_ignore_ascii_case("host") {
            return Err(AppError::InvalidParameter(format!(
                "CloudModel {id}: unsupported host_source={host_source} (only \"host\")"
            )));
        }
        if has_edges && active_base_id.is_some() && !reachable_set.contains(nid) {
            continue;
        }
        if !use_remote_llm {
            use_remote_llm = true;
            remote_model_override = model
                .as_deref()
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string());
        }
    }

    if use_remote_llm {
        return Ok(ExpertCompilePlan {
            use_remote_llm: true,
            remote_model_override,
            llama: LlamaLocalPluginConfig::default(),
        });
    }

    let llama = compile_llama_local_only(
        graph,
        models_gguf_dir,
        loras_dir,
        has_edges,
        &adj,
        active_base_id,
        &reachable_set,
    )?;

    Ok(ExpertCompilePlan {
        use_remote_llm: false,
        remote_model_override: None,
        llama,
    })
}

fn compile_llama_local_only(
    graph: &ExpertGraph,
    models_gguf_dir: &Path,
    loras_dir: &Path,
    has_edges: bool,
    _adj: &HashMap<&str, Vec<&str>>,
    active_base_id: Option<&str>,
    reachable_set: &HashSet<&str>,
) -> Result<LlamaLocalPluginConfig> {
    let base = graph.nodes.iter().find_map(|n| match (n, active_base_id) {
        (ExpertNode::BaseModel { id, gguf_path, .. }, Some(bid)) if id == bid => {
            Some(gguf_path.as_str())
        }
        (ExpertNode::BaseModel { gguf_path, .. }, None) => Some(gguf_path.as_str()),
        _ => None,
    });

    let model_path = base
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(PathBuf::from);

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

pub fn compile_graph_to_llama_local_config(
    graph: &ExpertGraph,
    models_gguf_dir: &Path,
    loras_dir: &Path,
) -> Result<LlamaLocalPluginConfig> {
    let plan = compile_expert_graph_plan(graph, models_gguf_dir, loras_dir)?;
    if plan.use_remote_llm {
        return Err(AppError::InvalidParameter(
            "expert graph activates a cloud model; local llama compile is not applicable".into(),
        ));
    }
    Ok(plan.llama)
}
