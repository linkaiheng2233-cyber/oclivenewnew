//! Bounded, desktop-owned runtime log preview. Never accepts an arbitrary file path.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SupportLogFile {
    pub name: String,
    pub size_bytes: u64,
    pub modified_at_epoch_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SupportLogSnapshot {
    pub directory: String,
    pub files: Vec<SupportLogFile>,
    pub selected_file: Option<String>,
    pub text: String,
    pub truncated: bool,
}
