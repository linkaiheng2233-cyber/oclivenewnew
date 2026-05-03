//! 插件评价索引 `reviews.json`。

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginReviewsIndexFile {
    pub schema_version: i32,
    #[serde(default)]
    pub generated_at: Option<String>,
    #[serde(default)]
    pub reviews: Vec<PluginReviewEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginReviewEntry {
    pub id: String,
    pub plugin_id: String,
    #[serde(default)]
    pub pubkey_id: Option<String>,
    #[serde(default)]
    pub version: Option<String>,
    pub rating: i32,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub body: Option<String>,
    pub created_at: String,
    #[serde(default)]
    pub author_github: Option<String>,
}
