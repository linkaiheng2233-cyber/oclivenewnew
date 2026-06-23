//! Blueprint meta shapes (serde-only; validation lives in `oclive_validation`).

use serde::{Deserialize, Serialize};

/// Shared `meta` block for blueprint v2/v3 on disk.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlueprintMetaSchema {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub relations: Vec<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub personality: Option<serde_json::Value>,
}
