//! 角色市场索引 `roles.json`（与 `validate_role_market_index_v1` 对齐）。

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoleIndexFile {
    #[serde(default)]
    pub generated_at: Option<String>,
    #[serde(default)]
    pub roles: Vec<RoleIndexEntry>,
    #[serde(default)]
    pub warning: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoleIndexEntry {
    #[serde(rename = "type")]
    pub entry_type: String,
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub author: String,
    pub version: String,
    #[serde(default)]
    pub min_runtime_version: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub downloads: Vec<RoleIndexDownload>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoleIndexDownload {
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub kind: String,
    pub url: String,
    pub sha256: String,
    #[serde(default)]
    pub note: Option<String>,
    #[serde(default)]
    pub trust: Option<String>,
}
