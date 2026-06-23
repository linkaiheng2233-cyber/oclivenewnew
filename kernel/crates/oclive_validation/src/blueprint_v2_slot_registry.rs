//! Slot-registry folding and override helpers for blueprint v2.

use std::collections::{BTreeMap, HashMap};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::blueprint_v2::SlotRegistryEntry;
use crate::plugin_backends::{
    AgentBackend, DirectoryPluginSlots, EmotionBackend, EventBackend, LlmBackend, MemoryBackend,
    PluginBackends, PromptBackend,
};

/// Folds `slot_registry` into production six-slot `PluginBackends` (same type **last-wins**, by `position`).
#[must_use]
pub fn slot_registry_to_plugin_backends(
    registry: &BTreeMap<String, SlotRegistryEntry>,
) -> PluginBackends {
    let mut winners: HashMap<&str, (&str, &SlotRegistryEntry)> = HashMap::new();
    for (key, entry) in registry {
        let t = entry.slot_type.trim();
        let keep = winners
            .get(t)
            .map(|(_, e)| entry.position >= e.position)
            .unwrap_or(true);
        if keep {
            winners.insert(t, (key.as_str(), entry));
        }
    }

    let mut pb = PluginBackends::default();
    let mut dir = DirectoryPluginSlots::default();

    if let Some((_, e)) = winners.get("memory") {
        if let Ok(b) = parse_backend_wire::<MemoryBackend>(&e.backend) {
            pb.memory = b;
        }
        if b_is_local(&e.backend) {
            pb.local_memory_provider_id = e.local_memory_provider_id.clone();
        }
        if b_is_directory(&e.backend) {
            dir.memory = single_plugin_id(e);
        }
    }
    if let Some((_, e)) = winners.get("emotion") {
        if let Ok(b) = parse_backend_wire::<EmotionBackend>(&e.backend) {
            pb.emotion = b;
        }
        if b_is_directory(&e.backend) {
            dir.emotion = single_plugin_id(e);
        }
    }
    if let Some((_, e)) = winners.get("event") {
        if let Ok(b) = parse_backend_wire::<EventBackend>(&e.backend) {
            pb.event = b;
        }
        if b_is_directory(&e.backend) {
            dir.event = single_plugin_id(e);
        }
    }
    if let Some((_, e)) = winners.get("prompt") {
        if let Ok(b) = parse_backend_wire::<PromptBackend>(&e.backend) {
            pb.prompt = b;
        }
        if b_is_directory(&e.backend) {
            dir.prompt = single_plugin_id(e);
        }
    }
    if let Some((_, e)) = winners.get("llm") {
        if let Ok(b) = parse_backend_wire::<LlmBackend>(&e.backend) {
            pb.llm = b;
        }
        if b_is_directory(&e.backend) {
            dir.llm = single_plugin_id(e);
        }
    }
    if let Some((_, e)) = winners.get("agent") {
        if let Ok(b) = parse_backend_wire::<AgentBackend>(&e.backend) {
            pb.agent = b;
        }
        if b_is_directory(&e.backend) {
            dir.agent = single_plugin_id(e).or_else(|| {
                e.plugins
                    .as_ref()
                    .and_then(|ps| ps.first())
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
            });
        }
    }

    pb.directory_plugins = dir;
    crate::agent_backend::sanitize_unimplemented_agent_backend(pb).backends
}

/// Session-level override for a single instance (not persisted).
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct SlotOverridePatch {
    #[serde(default)]
    pub backend: Option<String>,
    #[serde(default)]
    pub plugin: Option<String>,
    #[serde(default)]
    pub plugins: Option<Vec<String>>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub local_memory_provider_id: Option<String>,
}

impl SlotOverridePatch {
    /// Merges multiple C1/slot API calls for the same `slot_key` in a session into one override (later non-empty fields win).
    pub fn merge_into(&self, base: &mut SlotOverridePatch) {
        if let Some(ref b) = self.backend {
            base.backend = Some(b.clone());
        }
        if self.plugin.is_some() {
            base.plugin = self.plugin.clone();
        }
        if self.plugins.is_some() {
            base.plugins = self.plugins.clone();
        }
        if self.model.is_some() {
            base.model = self.model.clone();
        }
        if self.local_memory_provider_id.is_some() {
            base.local_memory_provider_id = self.local_memory_provider_id.clone();
        }
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.backend.is_none()
            && self.plugin.is_none()
            && self.plugins.is_none()
            && self.model.is_none()
            && self.local_memory_provider_id.is_none()
    }
}

/// Merges package-default `slot_registry` with namespace overrides into an effective view.
#[must_use]
pub fn effective_slot_registry(
    pack: &BTreeMap<String, SlotRegistryEntry>,
    overrides: &BTreeMap<String, SlotOverridePatch>,
) -> BTreeMap<String, SlotRegistryEntry> {
    let mut out = pack.clone();
    for (key, patch) in overrides {
        if patch.is_empty() {
            continue;
        }
        if let Some(entry) = out.get_mut(key) {
            apply_slot_override(entry, patch);
        }
    }
    out
}

/// Default six-slot module name → `slot_registry` key (C1 thin wrapper).
#[must_use]
pub fn default_slot_key_for_module(module: &str) -> Option<&'static str> {
    match module.trim().to_ascii_lowercase().as_str() {
        "memory" => Some("memory"),
        "emotion" => Some("emotion"),
        "event" => Some("event"),
        "prompt" => Some("prompt"),
        "llm" => Some("llm"),
        "agent" => Some("agent"),
        "complex_emotion" => Some("complex_emotion"),
        _ => None,
    }
}

pub fn apply_slot_override(entry: &mut SlotRegistryEntry, patch: &SlotOverridePatch) {
    if let Some(ref b) = patch.backend {
        let t = b.trim();
        if !t.is_empty() {
            entry.backend = t.to_string();
        }
    }
    if patch.plugin.is_some() {
        entry.plugin = patch.plugin.clone();
    }
    if patch.plugins.is_some() {
        entry.plugins = patch.plugins.clone();
    }
    if patch.model.is_some() {
        entry.model = patch.model.clone();
    }
    if patch.local_memory_provider_id.is_some() {
        entry.local_memory_provider_id = patch.local_memory_provider_id.clone();
    }
}

/// Instances of the same `type` sorted ascending by `position` (P3 multi-instance resolution).
#[must_use]
pub fn slot_registry_instances_sorted(
    registry: &BTreeMap<String, SlotRegistryEntry>,
    slot_type: &str,
) -> Vec<(String, SlotRegistryEntry)> {
    let want = slot_type.trim();
    let mut v: Vec<_> = registry
        .iter()
        .filter(|(_, e)| e.slot_type.trim() == want)
        .map(|(k, e)| (k.clone(), e.clone()))
        .collect();
    v.sort_by_key(|(_, e)| e.position);
    v
}

/// Single instance → folded six-slot `PluginBackends` (only the slot matching this instance's `type` is non-default).
#[must_use]
pub fn plugin_backends_for_slot_entry(entry: &SlotRegistryEntry) -> PluginBackends {
    let mut one = BTreeMap::new();
    one.insert("_".to_string(), entry.clone());
    slot_registry_to_plugin_backends(&one)
}

/// Merges all `type: agent` with `backend: directory` `plugin` / `plugins[]` (deduped, lexicographic order).
#[must_use]
pub fn merged_agent_directory_plugin_ids(
    registry: &BTreeMap<String, SlotRegistryEntry>,
) -> Vec<String> {
    let mut ids = Vec::new();
    for (_, entry) in slot_registry_instances_sorted(registry, "agent") {
        if entry.backend.trim() != "directory" {
            continue;
        }
        if let Some(p) = single_plugin_id(&entry) {
            ids.push(p);
        }
        if let Some(ps) = &entry.plugins {
            for p in ps {
                let t = p.trim();
                if !t.is_empty() {
                    ids.push(t.to_string());
                }
            }
        }
    }
    ids.sort();
    ids.dedup();
    ids
}

fn parse_backend_wire<T: serde::de::DeserializeOwned>(backend: &str) -> Result<T, ()> {
    serde_json::from_value(Value::String(backend.trim().to_string())).map_err(|_| ())
}

fn b_is_directory(backend: &str) -> bool {
    backend.trim() == "directory"
}

fn b_is_local(backend: &str) -> bool {
    backend.trim() == "local"
}

fn single_plugin_id(entry: &SlotRegistryEntry) -> Option<String> {
    entry
        .plugin
        .as_ref()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}
