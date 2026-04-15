//! 持久化：`app_data/plugin_state.json`（按角色隔离：整壳、禁用插件、插槽顺序、按插槽隐藏某插件贡献）。

use crate::models::ui_config::{SlotConfig, UiConfig};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct PluginStateFile {
    #[serde(default)]
    pub disabled_plugins: Vec<String>,
    /// 如 `chat_toolbar` → 插件 id 顺序（未列出的 id 按字典序排在后面）。
    #[serde(default)]
    pub slot_order: HashMap<String, Vec<String>>,
    /// 某插槽内不渲染的插件 id（整插件仍可在其他插槽或 RPC 中使用，除非同时列入 `disabled_plugins`）。
    #[serde(default)]
    pub disabled_slot_contributions: HashMap<String, Vec<String>>,
    /// 为真时忽略 `vueComponent`，全部插槽仅用 iframe（用户可在设置中开启）。
    #[serde(default)]
    pub force_iframe_mode: bool,
}

/// 单角色下的插件 UI 状态（含整壳选择）。
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct RolePluginState {
    /// 整壳插件 id；空字符串表示使用内置主界面。
    #[serde(default)]
    pub shell_plugin_id: String,
    #[serde(flatten)]
    pub slots: PluginStateFile,
}

impl RolePluginState {
    /// 由角色包 `ui.json` 生成初始状态（`visible` 必须为 `order` 子集；此处再过滤一遍）。
    pub fn from_ui_config(cfg: &UiConfig) -> Self {
        let mut slots = PluginStateFile::default();
        apply_slot("chat_toolbar", &cfg.slots.chat_toolbar, &mut slots);
        apply_slot("settings.panel", &cfg.slots.settings_panel, &mut slots);
        apply_slot("role.detail", &cfg.slots.role_detail, &mut slots);
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
        out
            .disabled_slot_contributions
            .insert(slot.to_string(), hidden);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginStateStore {
    #[serde(default = "schema_v2")]
    pub schema_version: u32,
    /// 旧版全局 `plugin_state.json` 迁移用；首次按角色落库后可清空。
    #[serde(default)]
    pub legacy_v1: Option<PluginStateFile>,
    #[serde(default)]
    pub roles: HashMap<String, RolePluginState>,
}

fn schema_v2() -> u32 {
    2
}

impl Default for PluginStateStore {
    fn default() -> Self {
        Self {
            schema_version: 2,
            legacy_v1: None,
            roles: HashMap::new(),
        }
    }
}

impl PluginStateStore {
    pub fn load(path: &Path) -> Self {
        let Ok(s) = std::fs::read_to_string(path) else {
            return Self::default();
        };
        let Ok(val) = serde_json::from_str::<serde_json::Value>(&s) else {
            return Self::default();
        };
        if val.get("schema_version").and_then(|v| v.as_u64()) == Some(2) {
            return serde_json::from_value(val).unwrap_or_default();
        }
        if let Ok(v1) = serde_json::from_value::<PluginStateFile>(val) {
            return Self {
                schema_version: 2,
                legacy_v1: Some(v1),
                roles: HashMap::new(),
            };
        }
        Self::default()
    }

    pub fn save(&self, path: &Path) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let raw = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        std::fs::write(path, raw).map_err(|e| e.to_string())
    }
}

impl PluginStateFile {
    pub fn load(path: &Path) -> Self {
        std::fs::read_to_string(path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

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
    #[must_use]
    pub fn is_plugin_disabled(&self, id: &str) -> bool {
        self.slots.is_plugin_disabled(id)
    }

    #[must_use]
    pub fn is_slot_contribution_disabled(&self, slot: &str, plugin_id: &str) -> bool {
        self.slots
            .is_slot_contribution_disabled(slot, plugin_id)
    }
}
