//! 本地插件发现描述符（纯数据结构）。

use serde::{Deserialize, Serialize};

/// 本地插件规范版本（`schema_version`）当前支持值。
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
    /// `schema_version` / `min_runtime_version` 与文档规范保持一致，用于宿主门禁。
    pub schema_version: u32,
    #[serde(default)]
    pub min_runtime_version: Option<String>,
    #[serde(default)]
    pub capabilities: Vec<LocalPluginCapability>,
}
