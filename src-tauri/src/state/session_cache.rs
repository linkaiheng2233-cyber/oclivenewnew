//! 进程内会话级缓存（细粒度锁，与 [`crate::state::AppState`] 解耦）。

use crate::models::{PersonalityVector, PluginBackendsOverride};
use oclive_validation::SlotOverridePatch;
use parking_lot::RwLock;
use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

/// 按用途分锁的会话缓存；各 [`RwLock`] 互不阻塞。
pub struct SessionCache {
    plugin_overrides: RwLock<HashMap<String, PluginBackendsOverride>>,
    slot_overrides: RwLock<HashMap<String, BTreeMap<String, SlotOverridePatch>>>,
    complex_emotion_narrative_hint: RwLock<HashMap<String, String>>,
    personality_snapshots: RwLock<HashMap<String, PersonalityVector>>,
}

impl SessionCache {
    #[must_use]
    pub fn new() -> Self {
        Self {
            plugin_overrides: RwLock::new(HashMap::new()),
            slot_overrides: RwLock::new(HashMap::new()),
            complex_emotion_narrative_hint: RwLock::new(HashMap::new()),
            personality_snapshots: RwLock::new(HashMap::new()),
        }
    }

    #[must_use]
    pub fn shared() -> Arc<Self> {
        Arc::new(Self::new())
    }

    pub fn stored_complex_emotion_narrative_hint(&self, srid: &str) -> String {
        self.complex_emotion_narrative_hint
            .read()
            .get(srid)
            .cloned()
            .unwrap_or_default()
    }

    pub fn set_stored_complex_emotion_narrative_hint(&self, srid: &str, hint: String) {
        let mut w = self.complex_emotion_narrative_hint.write();
        if hint.trim().is_empty() {
            w.remove(srid);
        } else {
            w.insert(srid.to_string(), hint);
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

    pub fn personality_cache(&self) -> &RwLock<HashMap<String, PersonalityVector>> {
        &self.personality_snapshots
    }
}

impl Default for SessionCache {
    fn default() -> Self {
        Self::new()
    }
}
