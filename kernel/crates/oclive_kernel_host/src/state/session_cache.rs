//! In-process session cache (fine-grained locks, decoupled from [`crate::state::AppState`]).

use crate::infrastructure::cache::Cache;
use crate::models::{PersonalityVector, PluginBackendsOverride};
use dashmap::DashMap;
use oclive_validation::SlotOverridePatch;
use parking_lot::RwLock;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::{Arc, Weak};
use std::time::{Duration, Instant};

const PERSONALITY_CACHE_CAPACITY: usize = 1000;
const SESSION_MAP_CAPACITY: usize = 512;
const SESSION_ENTRY_TTL: Duration = Duration::from_secs(300);
#[cfg(test)]
const CLEANUP_INTERVAL: Duration = Duration::from_millis(10);
#[cfg(not(test))]
const CLEANUP_INTERVAL: Duration = Duration::from_secs(300);

#[derive(Clone, Debug)]
struct TtlSlot<T> {
    value: T,
    touched_at: Instant,
}

/// Bounded TTL map for per-`srid` DashMap entries (cap + idle eviction).
#[derive(Debug)]
struct SessionScopedMap<T: Clone> {
    inner: DashMap<String, TtlSlot<T>>,
}

impl<T: Clone> SessionScopedMap<T> {
    fn new() -> Self {
        Self {
            inner: DashMap::new(),
        }
    }

    fn contains_key(&self, key: &str) -> bool {
        self.get(key).is_some()
    }

    fn get(&self, key: &str) -> Option<T> {
        let entry = self.inner.get(key)?;
        if entry.touched_at.elapsed() > SESSION_ENTRY_TTL {
            drop(entry);
            self.inner.remove(key);
            return None;
        }
        Some(entry.value.clone())
    }

    fn insert(&self, key: String, value: T) {
        self.inner.insert(
            key,
            TtlSlot {
                value,
                touched_at: Instant::now(),
            },
        );
        self.evict_if_needed();
    }

    fn remove(&self, key: &str) {
        self.inner.remove(key);
    }

    fn evict_if_needed(&self) {
        let expired: Vec<String> = self
            .inner
            .iter()
            .filter(|e| e.touched_at.elapsed() > SESSION_ENTRY_TTL)
            .map(|e| e.key().clone())
            .collect();
        for key in expired {
            self.inner.remove(&key);
        }
        if self.inner.len() <= SESSION_MAP_CAPACITY {
            return;
        }
        let mut idle: Vec<(String, Instant)> = self
            .inner
            .iter()
            .map(|e| (e.key().clone(), e.touched_at))
            .collect();
        idle.sort_by_key(|(_, touched)| *touched);
        let remove_n = self.inner.len().saturating_sub(SESSION_MAP_CAPACITY / 2);
        for (key, _) in idle.into_iter().take(remove_n) {
            self.inner.remove(&key);
        }
    }

    fn prune_not_in(&self, active: &HashSet<String>) {
        self.inner.retain(|k, _| active.contains(k.as_str()));
    }

    #[cfg(test)]
    fn len(&self) -> usize {
        self.inner.len()
    }
}

/// Session cache with per-purpose locks; [`RwLock`] / [`DashMap`] / [`Cache`] do not block each other.
pub struct SessionCache {
    plugin_overrides: RwLock<HashMap<String, PluginBackendsOverride>>,
    slot_overrides: RwLock<HashMap<String, BTreeMap<String, SlotOverridePatch>>>,
    complex_emotion_narrative_hint: SessionScopedMap<String>,
    expert_prompt_enhance: SessionScopedMap<String>,
    expert_injected_memory_ids: SessionScopedMap<Vec<String>>,
    expert_lora_plugin_id: SessionScopedMap<String>,
    relation_transitions: SessionScopedMap<RelationTransition>,
    interaction_mode_seeded: SessionScopedMap<()>,
    personality_snapshots: Cache<PersonalityVector>,
    session_touch: DashMap<String, Instant>,
}

/// In-process relation transition frame (consumed each turn until `remaining_turns` reaches zero).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RelationTransition {
    pub hint: String,
    pub remaining_turns: u32,
}

/// Result of consuming one transition turn.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RelationTransitionConsumed {
    pub hint: String,
    pub expired: bool,
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
        cache.evict_idle_session_maps();
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
            complex_emotion_narrative_hint: SessionScopedMap::new(),
            expert_prompt_enhance: SessionScopedMap::new(),
            expert_injected_memory_ids: SessionScopedMap::new(),
            expert_lora_plugin_id: SessionScopedMap::new(),
            relation_transitions: SessionScopedMap::new(),
            interaction_mode_seeded: SessionScopedMap::new(),
            personality_snapshots: Cache::with_capacity(PERSONALITY_CACHE_CAPACITY),
            session_touch: DashMap::new(),
        }
    }

    #[must_use]
    pub fn shared() -> Arc<Self> {
        let cache = Arc::new(Self::new());
        spawn_personality_cleanup(Arc::downgrade(&cache));
        cache
    }

    /// Mark `srid` active for TTL eviction (call on session override writes).
    pub fn touch_session(&self, srid: &str) {
        self.session_touch.insert(srid.to_string(), Instant::now());
        while self.session_touch.len() > SESSION_MAP_CAPACITY {
            let mut oldest: Option<(String, Instant)> = None;
            for item in self.session_touch.iter() {
                let key = item.key().clone();
                let touched = *item.value();
                if oldest.as_ref().is_none_or(|(_, t)| touched < *t) {
                    oldest = Some((key, touched));
                }
            }
            if let Some((key, _)) = oldest {
                self.session_touch.remove(&key);
            } else {
                break;
            }
        }
    }

    fn evict_idle_session_maps(&self) {
        self.complex_emotion_narrative_hint.evict_if_needed();
        self.expert_prompt_enhance.evict_if_needed();
        self.expert_injected_memory_ids.evict_if_needed();
        self.expert_lora_plugin_id.evict_if_needed();
        self.relation_transitions.evict_if_needed();
    }

    /// Drop session-scoped cache rows that no longer have an active turn lock.
    pub fn prune_sessions_without_active_turns(&self, active_srids: &HashSet<String>) {
        {
            let mut map = self.plugin_overrides.write();
            map.retain(|k, _| active_srids.contains(k.as_str()));
        }
        {
            let mut map = self.slot_overrides.write();
            map.retain(|k, _| active_srids.contains(k.as_str()));
        }
        self.session_touch
            .retain(|k, _| active_srids.contains(k.as_str()));
        self.complex_emotion_narrative_hint
            .prune_not_in(active_srids);
        self.expert_prompt_enhance.prune_not_in(active_srids);
        self.expert_injected_memory_ids.prune_not_in(active_srids);
        self.expert_lora_plugin_id.prune_not_in(active_srids);
        self.relation_transitions.prune_not_in(active_srids);
    }

    #[must_use]
    pub fn has_stored_complex_emotion_narrative_hint(&self, srid: &str) -> bool {
        self.complex_emotion_narrative_hint.contains_key(srid)
    }

    pub fn stored_complex_emotion_narrative_hint(&self, srid: &str) -> String {
        self.complex_emotion_narrative_hint
            .get(srid)
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

    /// Clears in-process cache only (for tests simulating restart + DB restore).
    pub fn clear_complex_emotion_narrative_hint_cache(&self, srid: &str) {
        self.complex_emotion_narrative_hint.remove(srid);
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
        self.expert_prompt_enhance.get(srid).unwrap_or_default()
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
        let mut ids = self
            .expert_injected_memory_ids
            .get(srid)
            .unwrap_or_default();
        ids.push(memory_id);
        self.expert_injected_memory_ids
            .insert(srid.to_string(), ids);
    }

    pub fn expert_injected_memory_ids(&self, srid: &str) -> Vec<String> {
        self.expert_injected_memory_ids
            .get(srid)
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
        self.expert_lora_plugin_id.get(srid)
    }

    #[must_use]
    pub fn has_relation_transition(&self, srid: &str) -> bool {
        self.relation_transitions.contains_key(srid)
    }

    #[must_use]
    pub fn is_interaction_mode_seeded(&self, srid: &str) -> bool {
        self.interaction_mode_seeded.contains_key(srid)
    }

    pub fn mark_interaction_mode_seeded(&self, srid: &str) {
        self.interaction_mode_seeded.insert(srid.to_string(), ());
    }

    pub fn set_relation_transition(&self, srid: &str, hint: String, remaining_turns: u32) {
        if hint.trim().is_empty() || remaining_turns == 0 {
            self.relation_transitions.remove(srid);
            return;
        }
        self.relation_transitions.insert(
            srid.to_string(),
            RelationTransition {
                hint,
                remaining_turns,
            },
        );
    }

    pub fn consume_relation_transition(&self, srid: &str) -> Option<RelationTransitionConsumed> {
        let current = self.relation_transitions.get(srid)?;
        let hint = current.hint.clone();
        let expired = if current.remaining_turns > 0 {
            current.remaining_turns - 1 == 0
        } else {
            true
        };
        if expired {
            self.relation_transitions.remove(srid);
        } else {
            self.relation_transitions.insert(
                srid.to_string(),
                RelationTransition {
                    hint: hint.clone(),
                    remaining_turns: current.remaining_turns - 1,
                },
            );
        }
        Some(RelationTransitionConsumed { hint, expired })
    }

    pub fn clear_relation_transition(&self, srid: &str) {
        self.relation_transitions.remove(srid);
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

    #[test]
    fn prune_sessions_without_active_turns_drops_idle_keys() {
        let cache = SessionCache::new();
        cache
            .complex_emotion_narrative_hint
            .insert("idle".into(), "hint".into());
        cache
            .complex_emotion_narrative_hint
            .insert("active".into(), "keep".into());
        cache
            .session_plugin_overrides()
            .write()
            .insert("idle".into(), PluginBackendsOverride::default());
        cache
            .session_plugin_overrides()
            .write()
            .insert("active".into(), PluginBackendsOverride::default());

        let mut active = HashSet::new();
        active.insert("active".to_string());
        cache.prune_sessions_without_active_turns(&active);

        assert!(cache.complex_emotion_narrative_hint.get("idle").is_none());
        assert_eq!(
            cache
                .complex_emotion_narrative_hint
                .get("active")
                .as_deref(),
            Some("keep")
        );
        assert!(!cache.session_plugin_overrides().read().contains_key("idle"));
        assert!(cache
            .session_plugin_overrides()
            .read()
            .contains_key("active"));
    }

    #[test]
    fn session_scoped_map_evicts_when_over_capacity() {
        let map = SessionScopedMap::<String>::new();
        for i in 0..=SESSION_MAP_CAPACITY {
            map.insert(format!("s-{i}"), format!("v-{i}"));
        }
        assert!(map.len() <= SESSION_MAP_CAPACITY);
    }
}
