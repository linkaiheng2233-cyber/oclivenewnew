//! Canonical DTOs for locally managed llama.cpp LoRA adapters.
//!
//! The kernel currently accepts only LoRA adapters already converted to GGUF.
//! Hugging Face/PEFT adapters intentionally belong to a separate, optional
//! converter/runtime plugin and are not represented as executable inputs here.

use super::content_rating::ContentRating;
use serde::{Deserialize, Serialize};

/// Backward-compatible name for the shared local content classification.
pub type LoraContentRating = ContentRating;

/// One installed llama.cpp LoRA GGUF adapter.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalLoraAdapterDto {
    pub id: String,
    pub name: String,
    pub version: String,
    pub format: String,
    pub content_rating: LoraContentRating,
    pub file_name: String,
    pub size_bytes: u64,
    pub sha256: String,
    pub base_model: Option<String>,
    pub architecture: Option<String>,
    pub description: Option<String>,
    pub license: Option<String>,
    pub source: Option<String>,
    pub installed_at: String,
    pub active: bool,
}

/// Import either a raw llama.cpp LoRA `.gguf` or a packaged `.ocadapter`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportLocalLoraAdapterRequest {
    pub source_path: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub base_model: Option<String>,
    #[serde(default)]
    pub content_rating: LoraContentRating,
    #[serde(default)]
    pub replace_existing: bool,
}

/// Select an installed adapter, or clear the selection with `adapter_id: null`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivateLocalLoraAdapterRequest {
    #[serde(default)]
    pub adapter_id: Option<String>,
    #[serde(default)]
    pub adult_content_acknowledged: bool,
}

/// Delete an inactive installed adapter.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteLocalLoraAdapterRequest {
    pub adapter_id: String,
}
