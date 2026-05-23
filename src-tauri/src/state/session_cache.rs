//! 进程内会话级缓存（细粒度锁，与 [`crate::state::AppState`] 解耦）。

use crate::infrastructure::cache::Cache;
use crate::models::{PersonalityVector, PluginBackendsOverride};
use dashmap::DashMap;
use oclive_validation::SlotOverridePatch;
use parking_lot::RwLock;
use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, Once};
use std::time::Duration;

const PERSONALITY_CACHE_CAPACITY: usize = 1000;
const CLEANUP_INTERVAL: Duration = Duration::from_secs(300);

/// 按用途分锁的会话缓存；各 [`RwLock`] / [`DashMap`] / [`Cache`] 互不阻塞。
pub struct SessionCache {
    plugin_overrides: RwLock<HashMap<String, PluginBackendsOverride>>,
    slot_overrides: RwLock<HashMap<String, BTreeMap<String, SlotOverridePatch>>>,
    complex_emotion_narrative_hint: DashMap<String, String>,
    personality_snapshots: Cache<PersonalityVector>,
}

impl SessionCache {
    #[must_use]
    pub fn new() -> Self {
        Self {
            plugin_overrides: RwLock::new(HashMap::new()),
            slot_overrides: RwLock::new(HashMap::new()),
            complex_emotion_narrative_hint: DashMap::new(),
            personality_snapshots: Cache::with_capacity(PERSONALITY_CACHE_CAPACITY),
        }
    }

    #[must_use]
    pub fn shared() -> Arc<Self> {
        let cache = Arc::new(Self::new());
        static CLEANUP_STARTED: Once = Once::new();
        let weak = Arc::downgrade(&cache);
        CLEANUP_STARTED.call_once(|| {
            tokio::spawn(async move {
                let mut interval = tokio::time::interval(CLEANUP_INTERVAL);
                interval.tick().await;
                loop {
                    interval.tick().await;
                    let Some(cache) = weak.upgrade() else {
                        break;
                    };
                    cache.personality_snapshots.cleanup_expired();
                }
            });
        });
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
}

impl Default for SessionCache {
    fn default() -> Self {
        Self::new()
    }
}
