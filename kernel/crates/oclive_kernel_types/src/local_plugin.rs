//! Local-plugin discovery descriptors (pure data structures).

use serde::{Deserialize, Serialize};

/// Currently supported value of the local-plugin spec version (`schema_version`).
pub const LOCAL_PLUGIN_SCHEMA_VERSION: u32 = 1;

/// Capability slot a local plugin provider can implement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LocalPluginCapability {
    Memory,
    Emotion,
    Event,
    Prompt,
    Llm,
}

/// Discovered local plugin metadata (id, schema version, capabilities).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocalPluginProviderDescriptor {
    pub provider_id: String,
    /// `schema_version` / `min_runtime_version` stay consistent with the documented spec, used for host gating.
    pub schema_version: u32,
    #[serde(default)]
    pub min_runtime_version: Option<String>,
    #[serde(default)]
    pub capabilities: Vec<LocalPluginCapability>,
}
