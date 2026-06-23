//! Role pack `portrait_catalog.json` + `config.json` → `portrait_catalog` toggle (A2 SSOT).

use serde::{Deserialize, Serialize};

/// Fixed 7-slot asset ids for simple role packs (B1 export SSOT).
pub const SIMPLE_PORTRAIT_SLOT_IDS: &[&str] = &[
    "happy_default",
    "sad_default",
    "angry_default",
    "neutral_default",
    "excited_default",
    "confused_default",
    "shy_default",
];

/// `config.json` → `portrait_catalog` (enabled toggle; assets live in `portrait_catalog.json`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct PortraitCatalogToggle {
    #[serde(default)]
    pub enabled: bool,
}

/// `portrait_catalog.json` on-disk SSOT.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PortraitCatalogFile {
    #[serde(default = "default_portrait_catalog_schema_version")]
    pub schema_version: u32,
    #[serde(default)]
    pub assets: Vec<PortraitCatalogAsset>,
}

fn default_portrait_catalog_schema_version() -> u32 {
    1
}

impl Default for PortraitCatalogFile {
    fn default() -> Self {
        Self {
            schema_version: default_portrait_catalog_schema_version(),
            assets: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PortraitCatalogAsset {
    pub id: String,
    pub path: String,
    #[serde(default)]
    pub desc: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default = "default_asset_kind")]
    pub kind: PortraitAssetKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cluster: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resources: Option<PortraitAssetResources>,
}

fn default_asset_kind() -> PortraitAssetKind {
    PortraitAssetKind::Image
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum PortraitAssetKind {
    #[default]
    Image,
    Live2d,
    Rig3d,
    Procedural,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct PortraitAssetResources {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub live2d_model: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rig3d_model: Option<String>,
}
