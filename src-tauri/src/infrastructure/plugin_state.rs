//! Persistence: `app_data/plugin_state.json` (isolated per role: shell, disabled plugins, slot order, per-slot hiding of a plugin's contribution).

use crate::models::ui_config::{SlotConfig, UiConfig};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, HashMap};
use std::path::Path;

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct PluginStateFile {
    #[serde(default)]
    pub disabled_plugins: Vec<String>,
    /// E.g. `chat_toolbar` → plugin id order (ids not listed are sorted lexicographically afterward).
    #[serde(default)]
    pub slot_order: HashMap<String, Vec<String>>,
    /// Plugin ids not rendered within a given slot (the whole plugin can still be used in other slots or via RPC, unless also listed in `disabled_plugins`).
    #[serde(default)]
    pub disabled_slot_contributions: HashMap<String, Vec<String>>,
    /// Appearance selected per plugin and slot: `plugin_id` → `slot` → `appearance_id` (consistent with `appearance_id` in the manifest).
    #[serde(default)]
    pub slot_appearance: HashMap<String, HashMap<String, String>>,
    /// When true, ignore `vueComponent` and use iframe only for all slots (users can enable this in settings).
    #[serde(default)]
    pub force_iframe_mode: bool,
}

/// Plugin UI state for a single role (including shell selection).
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct RolePluginState {
    /// Shell plugin id; an empty string means use the builtin main UI.
    #[serde(default)]
    pub shell_plugin_id: String,
    #[serde(flatten)]
    pub slots: PluginStateFile,
}

impl RolePluginState {
    /// Generate the initial state from the role pack's `ui.json` (`visible` must be a subset of `order`; filtered again here).
    #[must_use]
    pub fn from_ui_config(cfg: &UiConfig) -> Self {
        let mut slots = PluginStateFile::default();
        apply_slot("chat_toolbar", &cfg.slots.chat_toolbar, &mut slots);
        apply_slot("settings.panel", &cfg.slots.settings_panel, &mut slots);
        apply_slot("role.detail", &cfg.slots.role_detail, &mut slots);
        apply_slot("sidebar", &cfg.slots.sidebar, &mut slots);
        apply_slot("chat.header", &cfg.slots.chat_header, &mut slots);
        apply_slot("settings.plugins", &cfg.slots.settings_plugins, &mut slots);
        apply_slot(
            "settings.advanced",
            &cfg.slots.settings_advanced,
            &mut slots,
        );
        apply_slot("overlay.floating", &cfg.slots.overlay_floating, &mut slots);
        apply_slot("launcher.palette", &cfg.slots.launcher_palette, &mut slots);
        apply_slot("debug.dock", &cfg.slots.debug_dock, &mut slots);
        Self {
            shell_plugin_id: cfg.shell.trim().to_string(),
            slots,
        }
    }
}

fn apply_slot(slot: &str, sc: &SlotConfig, out: &mut PluginStateFile) {
    let order: Vec<String> = sc
        .order
        .iter()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    let order_set: HashMap<String, ()> = order.iter().map(|s| (s.clone(), ())).collect();
    let visible: Vec<String> = sc
        .visible
        .iter()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty() && order_set.contains_key(s.as_str()))
        .collect();
    let visible_set: HashMap<String, ()> = visible.iter().map(|s| (s.clone(), ())).collect();
    let hidden: Vec<String> = order
        .iter()
        .filter(|id| !visible_set.contains_key(*id))
        .cloned()
        .collect();
    if !order.is_empty() {
        out.slot_order.insert(slot.to_string(), order);
    }
    if !hidden.is_empty() {
        out.disabled_slot_contributions
            .insert(slot.to_string(), hidden);
    }
    for (pid, aid) in &sc.appearance {
        let pid = pid.trim();
        if pid.is_empty() {
            continue;
        }
        let aid = aid.trim();
        if aid.is_empty() {
            continue;
        }
        out.slot_appearance
            .entry(pid.to_string())
            .or_default()
            .insert(slot.to_string(), aid.to_string());
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginStateStore {
    #[serde(default = "schema_v3")]
    pub schema_version: u32,
    /// For migrating the legacy global `plugin_state.json`; can be cleared once data is first persisted per role.
    #[serde(default)]
    pub legacy_v1: Option<PluginStateFile>,
    /// Cross-role defaults (the "global default" in plugin management); when merged with `roles`, the role layer wins field by field (see [`RolePluginState::merge_global_defaults`]).
    #[serde(default)]
    pub global: Option<RolePluginState>,
    #[serde(default)]
    pub roles: HashMap<String, RolePluginState>,
}

fn schema_v3() -> u32 {
    3
}

impl Default for PluginStateStore {
    fn default() -> Self {
        Self {
            schema_version: 3,
            legacy_v1: None,
            global: None,
            roles: HashMap::new(),
        }
    }
}

impl PluginStateStore {
    #[must_use]
    pub fn load(path: &Path) -> Self {
        let Ok(s) = std::fs::read_to_string(path) else {
            return Self::default();
        };
        let Ok(val) = serde_json::from_str::<serde_json::Value>(&s) else {
            return Self::default();
        };
        if matches!(
            val.get("schema_version").and_then(|v| v.as_u64()),
            Some(2) | Some(3)
        ) {
            return serde_json::from_value(val).unwrap_or_default();
        }
        if let Ok(v1) = serde_json::from_value::<PluginStateFile>(val) {
            return Self {
                schema_version: 3,
                legacy_v1: Some(v1),
                global: None,
                roles: HashMap::new(),
            };
        }
        Self::default()
    }
    /// # Errors
    ///
    /// Returns [`Err`] with a human-readable message when the operation fails.
    pub fn save(&self, path: &Path) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let raw = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        std::fs::write(path, raw).map_err(|e| e.to_string())
    }
}

impl PluginStateFile {
    /// Merge global defaults with per-role storage: **per slot** and the disabled list are overridden by the role layer over global; `disabled_plugins` is the union.
    #[must_use]
    pub fn merge_global_and_role(global: &PluginStateFile, role: &PluginStateFile) -> Self {
        let mut disabled_plugins: BTreeSet<String> = BTreeSet::new();
        for s in &global.disabled_plugins {
            let t = s.trim();
            if !t.is_empty() {
                disabled_plugins.insert(t.to_string());
            }
        }
        for s in &role.disabled_plugins {
            let t = s.trim();
            if !t.is_empty() {
                disabled_plugins.insert(t.to_string());
            }
        }
        let mut slot_order: HashMap<String, Vec<String>> = HashMap::new();
        let mut keys: BTreeSet<String> = BTreeSet::new();
        for k in global.slot_order.keys().chain(role.slot_order.keys()) {
            keys.insert(k.clone());
        }
        for k in keys {
            let pick = role
                .slot_order
                .get(&k)
                .filter(|v| !v.is_empty())
                .cloned()
                .or_else(|| global.slot_order.get(&k).cloned());
            if let Some(v) = pick {
                if !v.is_empty() {
                    slot_order.insert(k, v);
                }
            }
        }
        let mut disabled_slot_contributions: HashMap<String, Vec<String>> = HashMap::new();
        let mut dkeys: BTreeSet<String> = BTreeSet::new();
        for k in global
            .disabled_slot_contributions
            .keys()
            .chain(role.disabled_slot_contributions.keys())
        {
            dkeys.insert(k.clone());
        }
        for k in dkeys {
            let v = if role.disabled_slot_contributions.contains_key(&k) {
                role.disabled_slot_contributions.get(&k)
            } else {
                global.disabled_slot_contributions.get(&k)
            };
            if let Some(list) = v {
                if !list.is_empty() {
                    disabled_slot_contributions.insert(k, list.clone());
                }
            }
        }
        let mut slot_appearance: HashMap<String, HashMap<String, String>> = HashMap::new();
        let mut appearance_plugin_ids: BTreeSet<String> = BTreeSet::new();
        for k in global
            .slot_appearance
            .keys()
            .chain(role.slot_appearance.keys())
        {
            appearance_plugin_ids.insert(k.clone());
        }
        for pid in appearance_plugin_ids {
            let mut slot_keys: BTreeSet<String> = BTreeSet::new();
            if let Some(g) = global.slot_appearance.get(&pid) {
                for s in g.keys() {
                    slot_keys.insert(s.clone());
                }
            }
            if let Some(r) = role.slot_appearance.get(&pid) {
                for s in r.keys() {
                    slot_keys.insert(s.clone());
                }
            }
            let mut inner = HashMap::new();
            for sk in slot_keys {
                let pick = role
                    .slot_appearance
                    .get(&pid)
                    .and_then(|m| m.get(&sk))
                    .or_else(|| global.slot_appearance.get(&pid).and_then(|m| m.get(&sk)));
                if let Some(v) = pick {
                    let t = v.trim();
                    if !t.is_empty() {
                        inner.insert(sk, t.to_string());
                    }
                }
            }
            if !inner.is_empty() {
                slot_appearance.insert(pid, inner);
            }
        }
        Self {
            disabled_plugins: disabled_plugins.into_iter().collect(),
            slot_order,
            disabled_slot_contributions,
            slot_appearance,
            force_iframe_mode: role.force_iframe_mode || global.force_iframe_mode,
        }
    }

    #[must_use]
    pub fn load(path: &Path) -> Self {
        std::fs::read_to_string(path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }
    /// # Errors
    ///
    /// Returns [`Err`] with a human-readable message when the operation fails.
    pub fn save(&self, path: &Path) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let raw = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        std::fs::write(path, raw).map_err(|e| e.to_string())
    }

    #[must_use]
    pub fn is_plugin_disabled(&self, id: &str) -> bool {
        self.disabled_plugins.iter().any(|d| d.trim() == id)
    }

    #[must_use]
    pub fn is_slot_contribution_disabled(&self, slot: &str, plugin_id: &str) -> bool {
        self.disabled_slot_contributions
            .get(slot)
            .map(|v| v.iter().any(|d| d.trim() == plugin_id))
            .unwrap_or(false)
    }
}

impl RolePluginState {
    /// Use `store.global` as defaults and merge with the saved per-role state in `role` (the shell id uses the role's value when non-empty).
    #[must_use]
    pub fn merge_global_defaults(global: Option<&RolePluginState>, role: &RolePluginState) -> Self {
        let empty_global = RolePluginState::default();
        let g = global.unwrap_or(&empty_global);
        let shell_plugin_id = if !role.shell_plugin_id.trim().is_empty() {
            role.shell_plugin_id.clone()
        } else {
            g.shell_plugin_id.clone()
        };
        Self {
            shell_plugin_id,
            slots: PluginStateFile::merge_global_and_role(&g.slots, &role.slots),
        }
    }

    #[must_use]
    pub fn is_plugin_disabled(&self, id: &str) -> bool {
        self.slots.is_plugin_disabled(id)
    }

    #[must_use]
    pub fn is_slot_contribution_disabled(&self, slot: &str, plugin_id: &str) -> bool {
        self.slots.is_slot_contribution_disabled(slot, plugin_id)
    }
}

#[cfg(test)]
mod merge_tests {
    use super::*;

    #[test]
    fn merge_prefers_role_shell_order_and_unions_disabled_plugins() {
        let global = RolePluginState {
            shell_plugin_id: "shell_a".into(),
            slots: PluginStateFile {
                disabled_plugins: vec!["p1".into()],
                slot_order: [("chat_toolbar".into(), vec!["a".into(), "b".into()])]
                    .into_iter()
                    .collect(),
                disabled_slot_contributions: HashMap::new(),
                slot_appearance: HashMap::new(),
                force_iframe_mode: true,
            },
        };
        let role = RolePluginState {
            shell_plugin_id: "shell_b".into(),
            slots: PluginStateFile {
                disabled_plugins: vec!["p2".into()],
                slot_order: [("chat_toolbar".into(), vec!["b".into(), "a".into()])]
                    .into_iter()
                    .collect(),
                disabled_slot_contributions: HashMap::new(),
                slot_appearance: HashMap::new(),
                force_iframe_mode: false,
            },
        };
        let m = RolePluginState::merge_global_defaults(Some(&global), &role);
        assert_eq!(m.shell_plugin_id, "shell_b");
        assert_eq!(m.slots.disabled_plugins, vec!["p1", "p2"]);
        assert_eq!(m.slots.slot_order["chat_toolbar"], vec!["b", "a"]);
        assert!(m.slots.force_iframe_mode);
    }

    #[test]
    fn merge_falls_back_to_global_when_role_empty() {
        let global = RolePluginState {
            shell_plugin_id: "gsh".into(),
            slots: PluginStateFile {
                slot_order: [("sidebar".into(), vec!["x".into()])].into_iter().collect(),
                ..Default::default()
            },
        };
        let role = RolePluginState::default();
        let m = RolePluginState::merge_global_defaults(Some(&global), &role);
        assert_eq!(m.shell_plugin_id, "gsh");
        assert_eq!(m.slots.slot_order["sidebar"], vec!["x"]);
    }

    #[test]
    fn merge_slot_appearance_prefers_role_per_key() {
        let global = RolePluginState {
            shell_plugin_id: String::new(),
            slots: PluginStateFile {
                slot_appearance: [(
                    "p1".into(),
                    [("chat_toolbar".into(), "a".into())].into_iter().collect(),
                )]
                .into_iter()
                .collect(),
                ..Default::default()
            },
        };
        let role = RolePluginState {
            shell_plugin_id: String::new(),
            slots: PluginStateFile {
                slot_appearance: [(
                    "p1".into(),
                    [("chat_toolbar".into(), "b".into())].into_iter().collect(),
                )]
                .into_iter()
                .collect(),
                ..Default::default()
            },
        };
        let m = RolePluginState::merge_global_defaults(Some(&global), &role);
        assert_eq!(m.slots.slot_appearance["p1"]["chat_toolbar"], "b");
    }
}
