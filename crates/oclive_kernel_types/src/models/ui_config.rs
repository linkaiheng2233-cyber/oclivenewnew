//! Role pack root `ui.json`: creator-recommended frontend layout (shell, theme, layout, and embedded slots).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Role pack suggested theme variables (for the built-in UI and slot iframes via CSS variables).
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ThemeConfig {
    #[serde(default)]
    pub primary_color: String,
    #[serde(default)]
    pub background_color: String,
    #[serde(default)]
    pub font_family: String,
}

/// Main UI layout preferences (built-in Vue shell).
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LayoutConfig {
    /// `left` | `right`
    #[serde(default)]
    pub sidebar: String,
    /// `bottom` | `top`
    #[serde(default)]
    pub chat_input: String,
}

/// Aligned with on-disk JSON keys (`settings.panel` / `role.detail`).
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct UiConfig {
    #[serde(default)]
    pub shell: String,
    #[serde(default)]
    pub theme: ThemeConfig,
    #[serde(default)]
    pub layout: LayoutConfig,
    #[serde(default)]
    pub slots: UiSlots,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct UiSlots {
    #[serde(default)]
    pub chat_toolbar: SlotConfig,
    #[serde(rename = "settings.panel", default)]
    pub settings_panel: SlotConfig,
    #[serde(rename = "role.detail", default)]
    pub role_detail: SlotConfig,
    #[serde(default)]
    pub sidebar: SlotConfig,
    #[serde(rename = "chat.header", default)]
    pub chat_header: SlotConfig,
    #[serde(rename = "settings.plugins", default)]
    pub settings_plugins: SlotConfig,
    #[serde(rename = "settings.advanced", default)]
    pub settings_advanced: SlotConfig,
    #[serde(rename = "overlay.floating", default)]
    pub overlay_floating: SlotConfig,
    #[serde(rename = "launcher.palette", default)]
    pub launcher_palette: SlotConfig,
    #[serde(rename = "debug.dock", default)]
    pub debug_dock: SlotConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct SlotConfig {
    #[serde(default)]
    pub order: Vec<String>,
    #[serde(default)]
    pub visible: Vec<String>,
    /// Default appearance per plugin id in this slot (`appearance_id`; matches manifest `ui_slots[].appearance_id`).
    #[serde(default)]
    pub appearance: HashMap<String, String>,
}

impl UiConfig {
    /// Read `ui.json` from a role pack directory; returns default empty config if missing or parse fails.
    #[must_use]
    pub fn load_from_path(path: &Path) -> Self {
        let raw = std::fs::read_to_string(path).ok();
        let Some(s) = raw else {
            return Self::default();
        };
        serde_json::from_str(&s).unwrap_or_default()
    }

    /// No recommendations (equivalent to a missing file): skip pack-init logic; legacy migration or empty state may apply.
    #[must_use]
    pub fn is_effectively_empty(&self) -> bool {
        self.shell.trim().is_empty()
            && self.theme.primary_color.trim().is_empty()
            && self.theme.background_color.trim().is_empty()
            && self.theme.font_family.trim().is_empty()
            && self.layout.sidebar.trim().is_empty()
            && self.layout.chat_input.trim().is_empty()
            && self.slots.chat_toolbar.order.is_empty()
            && self.slots.settings_panel.order.is_empty()
            && self.slots.role_detail.order.is_empty()
            && self.slots.sidebar.order.is_empty()
            && self.slots.chat_header.order.is_empty()
            && self.slots.settings_plugins.order.is_empty()
            && self.slots.settings_advanced.order.is_empty()
            && self.slots.overlay_floating.order.is_empty()
            && self.slots.launcher_palette.order.is_empty()
            && self.slots.debug_dock.order.is_empty()
    }
}
