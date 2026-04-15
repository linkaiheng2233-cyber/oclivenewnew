//! 角色包根目录 `ui.json`：创作者推荐的前端布局（整壳、主题、布局与三插槽）。

use serde::{Deserialize, Serialize};
use std::path::Path;

/// 角色包建议的主题变量（供内置界面与插槽 iframe 通过 CSS 变量参考）。
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

/// 主界面布局偏好（内置 Vue 壳）。
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

/// 与磁盘 JSON 对齐（`settings.panel` / `role.detail` 键名）。
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
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct SlotConfig {
    #[serde(default)]
    pub order: Vec<String>,
    #[serde(default)]
    pub visible: Vec<String>,
}

impl UiConfig {
    /// 从角色包目录读取 `ui.json`；不存在或解析失败时返回默认空配置。
    pub fn load_from_path(path: &Path) -> Self {
        let raw = std::fs::read_to_string(path).ok();
        let Some(s) = raw else {
            return Self::default();
        };
        serde_json::from_str(&s).unwrap_or_default()
    }

    /// 无任何推荐（与缺省文件等价）：不触发「从包初始化」逻辑，可走 legacy 迁移或空状态。
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
    }
}
