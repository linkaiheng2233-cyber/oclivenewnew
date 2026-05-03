//! 插件市场索引 `plugins.json` 契约类型（与 `oclive_validation::validate_plugin_market_index_v1` 对齐）。

use super::plugin_backends::PluginBackendsOverride;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PublisherPublicKey {
    pub pubkey_id: String,
    /// base64 编码的 Ed25519 public key（32 bytes）
    pub public_key: String,
    /// active|revoked|rotated（由索引侧约定）
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub rotated_to: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginIndexVersionEntry {
    pub version: String,
    #[serde(default)]
    pub download_url: Option<String>,
    #[serde(default)]
    pub signature_url: Option<String>,
    /// git tag；省略时默认使用 `version`
    #[serde(default)]
    pub git_tag: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginIndexEntry {
    /// 条目类型：`plugin`（默认）| `module` | `profile`
    #[serde(rename = "type", default = "default_index_entry_type")]
    pub entry_type: String,
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub author: String,
    pub version: String,
    /// 仅 `type=plugin` 必填；`module`/`profile` 可为空字符串。
    pub git: String,
    #[serde(default)]
    pub permissions: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub changelog: Option<String>,
    #[serde(default)]
    pub dependencies: HashMap<String, String>,
    #[serde(default)]
    pub publisher: Option<String>,
    #[serde(default)]
    pub public_keys: Vec<PublisherPublicKey>,
    #[serde(default)]
    pub versions: Vec<PluginIndexVersionEntry>,
    #[serde(default)]
    pub module: Option<PluginIndexModuleSpec>,
    #[serde(default)]
    pub profile: Option<PluginIndexProfileSpec>,
}

fn default_index_entry_type() -> String {
    "plugin".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginIndexModuleSpec {
    #[serde(default)]
    pub plugins: Vec<PluginIndexModulePluginSpec>,
    #[serde(default)]
    pub backends: Option<PluginBackendsOverride>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginIndexModulePluginSpec {
    pub id: String,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub source: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginIndexProfileSpec {
    #[serde(default)]
    pub plugins: Vec<PluginIndexModulePluginSpec>,
    #[serde(default)]
    pub backends: Option<PluginBackendsOverride>,
    #[serde(default)]
    pub predeclared_permissions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginIndexFile {
    #[serde(default, alias = "generated_at")]
    pub generated_at: Option<String>,
    #[serde(default)]
    pub plugins: Vec<PluginIndexEntry>,
}
