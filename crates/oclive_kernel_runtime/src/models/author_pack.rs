//! 角色包 `author.json`：创作者说明、推荐插件与 UI/后端建议（不替代 `settings.json`）。

use super::plugin_backends::PluginBackends;
use super::ui_config::UiConfig;
use serde::{Deserialize, Serialize};

/// 推荐安装的目录插件（市场 / 版本语义由前端与后续校验消费）。
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct AuthorRecommendedPlugin {
    pub id: String,
    /// 如 `^1.0.0`；可选
    #[serde(default)]
    pub version_range: Option<String>,
    #[serde(default)]
    pub slots: Vec<String>,
    /// 约束建议适用的后端模块（如 `memory`）；可选
    #[serde(default)]
    pub for_backends: Vec<String>,
    #[serde(default)]
    pub optional: bool,
    #[serde(default)]
    pub note: Option<String>,
}

/// 磁盘 `author.json` 顶层（与角色包其余文件并列于 `roles/{id}/`）。
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct AuthorPackFile {
    #[serde(default = "author_pack_schema_v1")]
    pub schema_version: u32,
    #[serde(default)]
    pub summary: String,
    #[serde(default)]
    pub detail_markdown: String,
    #[serde(default)]
    pub recommended_plugins: Vec<AuthorRecommendedPlugin>,
    /// 与 `ui.json` 相同形状；非空时优先于 `ui.json` 作为插件 UI 状态种子/重置基线。
    #[serde(default)]
    pub suggested_ui: Option<UiConfig>,
    /// 与 `settings.json` → `plugin_backends` 相同形状；仅建议，经 UI 确认后写入会话覆盖等。
    #[serde(default)]
    pub suggested_plugin_backends: Option<PluginBackends>,
}

fn author_pack_schema_v1() -> u32 {
    1
}

impl AuthorPackFile {
    /// 从 UTF-8 JSON 解析；失败时返回 `None`（调用方可记日志）。
    pub fn from_json_str(raw: &str) -> Option<Self> {
        serde_json::from_str(raw.trim()).ok()
    }
}
