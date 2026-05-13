//! 用户自定义全局快捷键：`app_data/hotkey_bindings.json`（序列化/校验；系统注册由宿主完成）。
//!
//! 提供 [`HotkeyBindingsFile::load_async`] / [`HotkeyBindingsFile::save_async`]（`tokio::fs`），供 async 宿主路径使用。

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct HotkeyBindingsFile {
    #[serde(default = "hotkey_schema_v1")]
    pub schema_version: u32,
    #[serde(default)]
    pub bindings: Vec<HotkeyBinding>,
}

fn hotkey_schema_v1() -> u32 {
    1
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct HotkeyBinding {
    pub id: String,
    pub accelerator: String,
    /// 为 `false` 时不注册系统全局快捷键（默认关闭，避免误触）。
    #[serde(default)]
    pub enabled: bool,
    pub action: HotkeyAction,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum HotkeyAction {
    #[serde(rename = "openPluginSlot")]
    OpenPluginSlot {
        #[serde(rename = "pluginId")]
        plugin_id: String,
        slot: String,
        #[serde(default, rename = "appearanceId")]
        appearance_id: String,
    },
    #[serde(rename = "openLauncherList")]
    OpenLauncherList,
}

impl HotkeyBindingsFile {
    #[must_use]
    pub fn path(app_data: &Path) -> PathBuf {
        app_data.join("hotkey_bindings.json")
    }

    #[must_use]
    pub fn load(app_data: &Path) -> Self {
        let p = Self::path(app_data);
        std::fs::read_to_string(&p)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn save(&self, app_data: &Path) -> Result<(), String> {
        let p = Self::path(app_data);
        if let Some(parent) = p.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let raw = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        std::fs::write(&p, raw).map_err(|e| e.to_string())
    }

    pub async fn load_async(app_data: &Path) -> Self {
        let p = Self::path(app_data);
        match tokio::fs::read_to_string(&p).await {
            Ok(s) => serde_json::from_str(&s).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    pub async fn save_async(&self, app_data: &Path) -> Result<(), String> {
        let p = Self::path(app_data);
        if let Some(parent) = p.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| e.to_string())?;
        }
        let raw = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        tokio::fs::write(&p, raw).await.map_err(|e| e.to_string())
    }
}

pub fn validate_hotkey_bindings(file: &HotkeyBindingsFile) -> Result<(), String> {
    let mut seen = HashSet::new();
    for b in &file.bindings {
        if !b.enabled {
            continue;
        }
        let acc = b.accelerator.trim();
        if acc.is_empty() {
            continue;
        }
        if !seen.insert(acc.to_string()) {
            return Err(format!("重复的已启用快捷键：{}", acc));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_rejects_duplicate_enabled_accelerators() {
        let file = HotkeyBindingsFile {
            schema_version: 1,
            bindings: vec![
                HotkeyBinding {
                    id: "a".into(),
                    accelerator: "Ctrl+A".into(),
                    enabled: true,
                    action: HotkeyAction::OpenLauncherList,
                },
                HotkeyBinding {
                    id: "b".into(),
                    accelerator: "Ctrl+A".into(),
                    enabled: true,
                    action: HotkeyAction::OpenLauncherList,
                },
            ],
        };
        assert!(validate_hotkey_bindings(&file).is_err());
    }
}
