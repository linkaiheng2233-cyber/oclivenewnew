//! Module 9: Expert Models compiler + safe apply to llama local sidecar config.

use crate::error::{AppError, Result};
use crate::models::expert_models::{ExpertGraph, ExpertNode, LlamaLocalPluginConfig, PromptStyleOverride};
use std::path::{Path, PathBuf};

pub const LLAMA_LOCAL_PLUGIN_ID: &str = "com.oclive.llama.local";

#[derive(Debug, Clone)]
pub struct ExpertEffectiveConfig {
    pub graph: ExpertGraph,
    pub prompt_style: Option<PromptStyleOverride>,
}

fn is_path_under(child: &Path, parent: &Path) -> bool {
    let cn = child.canonicalize().unwrap_or_else(|_| child.to_path_buf());
    let pn = parent.canonicalize().unwrap_or_else(|_| parent.to_path_buf());
    cn.starts_with(pn)
}

pub fn compile_graph_to_llama_local_config(
    graph: &ExpertGraph,
    models_gguf_dir: &Path,
    loras_dir: &Path,
) -> Result<LlamaLocalPluginConfig> {
    // base model: pick the first BaseModel node (M1).
    let base = graph.nodes.iter().find_map(|n| match n {
        ExpertNode::BaseModel { gguf_path, .. } => Some(gguf_path.as_str()),
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

    // loras: all enabled LoraAdapter nodes sorted by (order, id)
    let mut loras: Vec<(String, f32, i32, String)> = vec![];
    for n in &graph.nodes {
        if let ExpertNode::LoraAdapter {
            id,
            gguf_path,
            strength,
            enabled,
            order,
        } = n
        {
            if !*enabled {
                continue;
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
        // llama.cpp server supports comma-separated `--lora-scaled FNAME:SCALE,...`
        let mut spec = String::new();
        for (i, (p, s, _, _id)) in loras.iter().enumerate() {
            if i > 0 {
                spec.push(',');
            }
            // Keep original path string for llama-server; it accepts absolute paths.
            spec.push_str(p);
            spec.push(':');
            // Avoid scientific notation for common ranges; keep reasonably compact.
            spec.push_str(&format!("{:.6}", *s));
        }
        Some(format!("--lora-scaled {}", spec))
    };

    Ok(LlamaLocalPluginConfig {
        model_path: model_path.map(|p| p.to_string_lossy().to_string()),
        llama_args,
    })
}

