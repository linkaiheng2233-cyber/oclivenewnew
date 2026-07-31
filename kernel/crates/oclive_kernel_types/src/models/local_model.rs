//! Canonical DTOs and sidecar metadata for local GGUF base models.

use super::content_rating::ContentRating;
use serde::{Deserialize, Serialize};

/// Sidecar schema version understood by the local model scanner.
pub const LOCAL_MODEL_MANIFEST_SCHEMA_VERSION: u32 = 1;
/// Sidecar kind used to distinguish base-model metadata from other JSON files.
pub const LOCAL_MODEL_MANIFEST_KIND: &str = "oclive.local-base-model";
/// Suffix appended to a model file name for its metadata sidecar.
pub const LOCAL_MODEL_MANIFEST_SUFFIX: &str = ".ocmodel.json";

/// One selectable local base model.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalModelFileDto {
    pub name: String,
    pub path: String,
    pub size_bytes: u64,
    #[serde(default)]
    pub content_rating: ContentRating,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub license: Option<String>,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub sha256: Option<String>,
}

/// Metadata stored beside a GGUF file as `<file>.ocmodel.json`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalModelManifest {
    pub schema_version: u32,
    pub kind: String,
    pub file_name: String,
    pub name: String,
    #[serde(default)]
    pub content_rating: ContentRating,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub license: Option<String>,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub sha256: Option<String>,
}
