//! `memory.rank` JSON-RPC — 见 REMOTE_PLUGIN_PROTOCOL.md

use crate::domain::memory_engine::MemoryEngine;
use crate::domain::memory_retrieval::{MemoryRetrieval, MemoryRetrievalInput};
use crate::domain::BuiltinMemoryRetrieval;
use crate::infrastructure::remote_plugin::config::RemotePluginHttpConfig;
use crate::infrastructure::remote_plugin::jsonrpc::{self, RemoteRpcChannel};
use crate::models::{Memory, MemoryContext};
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};

const METHOD_MEMORY_RANK: &str = "memory.rank";

pub struct RemoteMemoryRetrievalHttp {
    client: reqwest::blocking::Client,
    cfg: RemotePluginHttpConfig,
    fallback: BuiltinMemoryRetrieval,
}

impl RemoteMemoryRetrievalHttp {
    pub fn new(cfg: RemotePluginHttpConfig) -> Self {
        let client = reqwest::blocking::Client::builder()
            .connect_timeout(cfg.connect_timeout())
            .timeout(cfg.timeout)
            .build()
            .expect("reqwest blocking client");
        Self {
            client,
            cfg,
            fallback: BuiltinMemoryRetrieval,
        }
    }

    fn rank_remote(&self, input: &MemoryRetrievalInput<'_>) -> Option<Vec<Memory>> {
        let memories_json: Value = serde_json::to_value(input.memories).ok()?;
        let params = json!({
            "memories": memories_json,
            "user_query": input.user_query,
            "scene_id": input.scene_id,
            "limit": input.limit,
        });
        let result = jsonrpc::call_blocking(
            RemoteRpcChannel::Plugin,
            &self.client,
            &self.cfg.endpoint,
            METHOD_MEMORY_RANK,
            params,
            self.cfg.bearer_token.as_deref(),
        )
        .ok()?;
        let ordered_ids: Vec<String> = result
            .get("ordered_ids")?
            .as_array()?
            .iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect();
        Some(apply_ordered_ids(input.memories, &ordered_ids, input.limit))
    }
}

fn apply_ordered_ids(memories: &[Memory], ordered_ids: &[String], limit: usize) -> Vec<Memory> {
    let limit = limit.max(1);
    let by_id: HashMap<&str, Memory> = memories
        .iter()
        .map(|m| (m.id.as_str(), m.clone()))
        .collect();
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for id in ordered_ids {
        if out.len() >= limit {
            break;
        }
        if let Some(m) = by_id.get(id.as_str()) {
            seen.insert(m.id.clone());
            out.push(m.clone());
        }
    }
    for m in memories {
        if out.len() >= limit {
            break;
        }
        if !seen.contains(&m.id) {
            seen.insert(m.id.clone());
            out.push(m.clone());
        }
    }
    out
}

impl MemoryRetrieval for RemoteMemoryRetrievalHttp {
    fn rank_memories(&self, input: MemoryRetrievalInput<'_>) -> Vec<Memory> {
        match self.rank_remote(&input) {
            Some(v) if !v.is_empty() => v,
            _ => {
                log::warn!(
                    target: "oclive_plugin",
                    "memory.rank remote failed or empty endpoint={} fallback=builtin",
                    self.cfg.endpoint
                );
                self.fallback.rank_memories(input)
            }
        }
    }

    fn build_context(&self, memories: &[Memory], max_tokens: usize) -> MemoryContext {
        MemoryEngine::build_context(memories, max_tokens)
    }

    fn search_memories(&self, keyword: &str, memories: &[Memory]) -> Vec<Memory> {
        MemoryEngine::search_memories(keyword, memories)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn apply_ordered_ids_respects_limit_and_tail() {
        let m = |id: &str| Memory {
            id: id.to_string(),
            role_id: "r".into(),
            content: "x".into(),
            importance: 1.0,
            weight: 1.0,
            created_at: Utc::now(),
            scene_id: None,
        };
        let memories = vec![m("a"), m("b"), m("c")];
        let out = apply_ordered_ids(&memories, &["c".into(), "a".into()], 2);
        assert_eq!(out.len(), 2);
        assert_eq!(out[0].id, "c");
        assert_eq!(out[1].id, "a");
    }

    #[test]
    fn method_name_matches_remote_protocol() {
        assert_eq!(METHOD_MEMORY_RANK, "memory.rank");
    }
}
