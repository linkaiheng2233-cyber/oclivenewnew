//! `memory.rank` JSON-RPC — see REMOTE_PLUGIN_PROTOCOL.md

use crate::domain::error_helpers::serde_to_unknown;
use crate::domain::memory_engine::MemoryEngine;
use crate::domain::memory_retrieval::{MemoryRetrieval, MemoryRetrievalInput};
use crate::domain::BuiltinMemoryRetrieval;
use crate::error::{AppError, Result};
use crate::infrastructure::high_risk_grants::HighRiskGrantStore;
use crate::infrastructure::remote_fallback_policy::remote_fallback_load;
use crate::infrastructure::remote_plugin::adapter::RemotePluginAdapterBlocking;
use crate::infrastructure::remote_plugin::config::RemotePluginHttpConfig;
use crate::models::{Memory, MemoryContext};
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

const METHOD_MEMORY_RANK: &str = "memory.rank";

pub struct RemoteMemoryRetrievalHttp {
    adapter: RemotePluginAdapterBlocking,
    fallback: BuiltinMemoryRetrieval,
    remote_fallback_allowed: Arc<AtomicBool>,
}

impl RemoteMemoryRetrievalHttp {
    #[must_use]
    pub fn new(
        http_client: Arc<reqwest::Client>,
        cfg: RemotePluginHttpConfig,
        remote_fallback_allowed: Arc<AtomicBool>,
        high_risk_grants: Arc<HighRiskGrantStore>,
        network_grant_id: Option<String>,
    ) -> Self {
        let fb = remote_fallback_allowed.clone();
        Self {
            adapter: RemotePluginAdapterBlocking::new(
                http_client,
                cfg,
                fb.clone(),
                high_risk_grants,
                network_grant_id,
            ),
            fallback: BuiltinMemoryRetrieval,
            remote_fallback_allowed: fb,
        }
    }

    fn rank_remote(&self, input: &MemoryRetrievalInput<'_>) -> Result<Option<Vec<Memory>>> {
        let memories_json: Value = serde_json::to_value(input.memories)
            .map_err(|e| serde_to_unknown("memory.rank encode memories", e))?;
        let params = json!({
            "memories": memories_json,
            "user_query": input.user_query,
            "scene_id": input.scene_id,
            "limit": input.limit,
        });
        let result = match self
            .adapter
            .http
            .call_plugin_soft(METHOD_MEMORY_RANK, params)?
        {
            Some(v) => v,
            None => return Ok(None),
        };
        let Some(arr) = result.get("ordered_ids").and_then(|x| x.as_array()) else {
            return Ok(None);
        };
        let ordered_ids: Vec<String> = arr
            .iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect();
        Ok(Some(apply_ordered_ids(
            input.memories,
            &ordered_ids,
            input.limit,
        )))
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
    fn rank_memories(&self, input: MemoryRetrievalInput<'_>) -> Result<Vec<Memory>> {
        match self.rank_remote(&input)? {
            Some(v) if !v.is_empty() => Ok(v),
            _ => {
                if remote_fallback_load(&self.remote_fallback_allowed) {
                    tracing::warn!(
                        target: "oclive_plugin",
                        "memory.rank remote failed or empty endpoint={} fallback=builtin",
                        self.adapter.http.endpoint()
                    );
                    self.fallback.rank_memories(input)
                } else {
                    Err(AppError::RemoteServiceUnavailable(format!(
                        "memory.rank remote failed or empty endpoint={}",
                        self.adapter.http.endpoint()
                    )))
                }
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
            mention_count: 1,
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
