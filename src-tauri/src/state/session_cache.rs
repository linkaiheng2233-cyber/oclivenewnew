//! 进程内会话级缓存（细粒度锁，与 [`crate::state::AppState`] 解耦）。

use crate::infrastructure::cache::Cache;
use crate::models::{PersonalityVector, PluginBackendsOverride};
use dashmap::DashMap;
use oclive_validation::SlotOverridePatch;
use parking_lot::RwLock;
use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, Weak};
use std::time::Duration;

const PERSONALITY_CACHE_CAPACITY: usize = 1000;
#[cfg(test)]
const CLEANUP_INTERVAL: Duration = Duration::from_millis(10);
#[cfg(not(test))]
const CLEANUP_INTERVAL: Duration = Duration::from_secs(300);

/// 按用途分锁的会话缓存；各 [`RwLock`] / [`DashMap`] / [`Cache`] 互不阻塞。
pub struct SessionCache {
    plugin_overrides: RwLock<HashMap<String, PluginBackendsOverride>>,
    slot_overrides: RwLock<HashMap<String, BTreeMap<String, SlotOverridePatch>>>,
    complex_emotion_narrative_hint: DashMap<String, String>,
    /// 专家 `slot.prompt_enhance.apply` 注入的 Prompt 片段（本回合 assemble 时追加）。
    expert_prompt_enhance: DashMap<String, String>,
    /// 专家 `slot.memory.inject` 写入的临时记忆 id（失败回滚时删除）。
    expert_injected_memory_ids: DashMap<String, Vec<String>>,
    /// 专家 `slot.lora.apply` 已应用的 directory 插件 id（失败时清除会话标记）。
    expert_lora_plugin_id: DashMap<String, String>,
    personality_snapshots: Cache<PersonalityVector>,
}

async fn run_personality_cleanup(weak: Weak<SessionCache>) {
    let mut ticker = tokio::time::interval(CLEANUP_INTERVAL);
    ticker.tick().await;
    loop {
        ticker.tick().await;
        let Some(cache) = weak.upgrade() else {
            break;
        };
        cache.personality_snapshots.cleanup_expired();
    }
}

fn spawn_personality_cleanup(weak: Weak<SessionCache>) {
    tokio::spawn(run_personality_cleanup(weak));
}

impl SessionCache {
    #[must_use]
    pub fn new() -> Self {
        Self {
            plugin_overrides: RwLock::new(HashMap::new()),
            slot_overrides: RwLock::new(HashMap::new()),
            complex_emotion_narrative_hint: DashMap::new(),
            expert_prompt_enhance: DashMap::new(),
            expert_injected_memory_ids: DashMap::new(),
            expert_lora_plugin_id: DashMap::new(),
            personality_snapshots: Cache::with_capacity(PERSONALITY_CACHE_CAPACITY),
        }
    }

    #[must_use]
    pub fn shared() -> Arc<Self> {
        let cache = Arc::new(Self::new());
        spawn_personality_cleanup(Arc::downgrade(&cache));
        cache
    }

    pub fn stored_complex_emotion_narrative_hint(&self, srid: &str) -> String {
        self.complex_emotion_narrative_hint
            .get(srid)
            .map(|v| v.clone())
            .unwrap_or_default()
    }

    pub fn set_stored_complex_emotion_narrative_hint(&self, srid: &str, hint: String) {
        if hint.trim().is_empty() {
            self.complex_emotion_narrative_hint.remove(srid);
        } else {
            self.complex_emotion_narrative_hint
                .insert(srid.to_string(), hint);
        }
    }

    pub fn session_plugin_overrides(&self) -> &RwLock<HashMap<String, PluginBackendsOverride>> {
        &self.plugin_overrides
    }

    pub fn session_slot_overrides(
        &self,
    ) -> &RwLock<HashMap<String, BTreeMap<String, SlotOverridePatch>>> {
        &self.slot_overrides
    }

    pub fn personality_cache(&self) -> &Cache<PersonalityVector> {
        &self.personality_snapshots
    }

    pub fn expert_prompt_enhance(&self, srid: &str) -> String {
        self.expert_prompt_enhance
            .get(srid)
            .map(|v| v.clone())
            .unwrap_or_default()
    }

    pub fn set_expert_prompt_enhance(&self, srid: &str, fragment: String) {
        if fragment.trim().is_empty() {
            self.expert_prompt_enhance.remove(srid);
        } else {
            self.expert_prompt_enhance
                .insert(srid.to_string(), fragment);
        }
    }

    pub fn push_expert_injected_memory(&self, srid: &str, memory_id: String) {
        self.expert_injected_memory_ids
            .entry(srid.to_string())
            .or_default()
            .push(memory_id);
    }

    pub fn expert_injected_memory_ids(&self, srid: &str) -> Vec<String> {
        self.expert_injected_memory_ids
            .get(srid)
            .map(|v| v.clone())
            .unwrap_or_default()
    }

    pub fn clear_expert_injected_memories(&self, srid: &str) {
        self.expert_injected_memory_ids.remove(srid);
    }

    pub fn set_expert_lora_plugin(&self, srid: &str, plugin_id: Option<String>) {
        match plugin_id {
            None => {
                self.expert_lora_plugin_id.remove(srid);
            }
            Some(id) if id.trim().is_empty() => {
                self.expert_lora_plugin_id.remove(srid);
            }
            Some(id) => {
                self.expert_lora_plugin_id
                    .insert(srid.to_string(), id.trim().to_string());
            }
        }
    }

    pub fn expert_lora_plugin_id(&self, srid: &str) -> Option<String> {
        self.expert_lora_plugin_id.get(srid).map(|v| v.clone())
    }
}

impl Default for SessionCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};

    #[tokio::test]
    async fn personality_cleanup_exits_after_cache_dropped() {
        let cache = Arc::new(SessionCache::new());
        let weak = Arc::downgrade(&cache);
        let exited_flag = Arc::new(AtomicBool::new(false));
        let done = exited_flag.clone();
        let handle = tokio::spawn(async move {
            run_personality_cleanup(weak).await;
            done.store(true, Ordering::SeqCst);
        });

        drop(cache);
        tokio::time::sleep(CLEANUP_INTERVAL * 3).await;
        handle.await.expect("cleanup task join");

        assert!(
            exited_flag.load(Ordering::SeqCst),
            "cleanup loop should finish when SessionCache Arc is dropped"
        );
    }

    #[tokio::test]
    async fn shared_creates_distinct_instances() {
        let a = SessionCache::shared();
        let b = SessionCache::shared();
        assert!(Arc::strong_count(&a) >= 1);
        assert!(Arc::strong_count(&b) >= 1);
        assert!(!Arc::ptr_eq(&a, &b));
    }
}