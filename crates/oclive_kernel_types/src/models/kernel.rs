//! Kernel scheduling / health DTOs (SSOT for HTTP, CLI, and policy).

use serde::{Deserialize, Serialize};

/// Caller-side requirements from `distro.oclive.toml` (scheduling subset).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DistroProfileRequirements {
    pub distro_id: String,
    #[serde(default)]
    pub required_modules: Vec<String>,
    #[serde(default)]
    pub forbidden_modules: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post_process_profile: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_profile: Option<String>,
}

/// Running kernel effective profile summary (`GET /health`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ActiveProfileSummary {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub distro_id: Option<String>,
    #[serde(default)]
    pub enabled_modules: Vec<String>,
    #[serde(default)]
    pub disabled_modules: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post_process_profile: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_profile: Option<String>,
}

/// Why attach was chosen (healthy kernel path).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttachReason {
    ProfileCompatible,
    RunningKernelOk,
    KernelPinned,
    KernelPinnedProfileMismatch,
    ProfileMismatchNoReplace,
    LegacyFallback,
}

/// Why replace was chosen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReplaceReason {
    BinaryUpgrade,
    ProfileMismatch,
}

/// Profile compatibility outcome for diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProfileCompat {
    Compatible,
    Incompatible,
    Unknown,
}

/// `GET /health` JSON body (shared fields; `kernel_manifest` supplied by host/runtime).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KernelHealthJson {
    pub ok: bool,
    #[serde(default)]
    pub runtime_api_version: Option<String>,
    #[serde(default)]
    pub schema_migration_version: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub distro_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub distro_profile_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active_profile_summary: Option<ActiveProfileSummary>,
}
