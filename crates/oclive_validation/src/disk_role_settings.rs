//! Disk `settings.json` engine section: serde aligned with host `DiskRoleSettings`; merged into `DiskRoleManifest` then `validate_disk_manifest`.

use serde::{Deserialize, Serialize};

use crate::manifest::{
    DiskRoleManifest, EvolutionConfigDisk, IdentityBinding, KnowledgePackConfigDisk,
    MemoryConfigDisk,
};
use crate::plugin_backends::PluginBackends;

pub const CURRENT_SETTINGS_SCHEMA_VERSION: u32 = 1;

fn default_schema_version() -> u32 {
    CURRENT_SETTINGS_SCHEMA_VERSION
}

/// Autonomous scene-change rules after virtual time jumps (`settings.json` → `autonomous_scene`).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AutonomousSceneConfig {
    #[serde(default)]
    pub on_virtual_time: Vec<AutonomousSceneRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutonomousSceneRule {
    pub when_scene: String,
    pub hour_start: u8,
    pub hour_end: u8,
    pub to_scene: String,
}

/// Remote-presence inner-voice mode toggle (`settings.json` → `remote_presence`).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RemotePresenceConfig {
    #[serde(default)]
    pub default_enabled: Option<bool>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub stub_messages: Vec<String>,
}

/// Role pack engine settings (may live in a separate file from `manifest.json`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskRoleSettings {
    #[serde(default = "default_schema_version")]
    pub schema_version: u32,
    #[serde(default)]
    pub identity_binding: Option<IdentityBinding>,
    #[serde(default)]
    pub evolution: Option<EvolutionConfigDisk>,
    #[serde(default)]
    pub memory_config: Option<MemoryConfigDisk>,
    #[serde(default, alias = "model")]
    pub ollama_model: Option<String>,
    #[serde(default)]
    pub remote_presence: Option<RemotePresenceConfig>,
    #[serde(default)]
    pub autonomous_scene: Option<AutonomousSceneConfig>,
    #[serde(default)]
    pub interaction_mode: Option<String>,
    #[serde(default)]
    pub plugin_backends: Option<PluginBackends>,
    #[serde(default)]
    pub knowledge: Option<KnowledgePackConfigDisk>,
    #[serde(default)]
    pub reply_quality_anchor: Option<String>,
}

impl DiskRoleSettings {
    /// Overwrites parsed manifest with fields present in this file (`Some` entries only).
    pub fn apply_to_manifest(&self, manifest: &mut DiskRoleManifest) {
        if let Some(ref m) = self.ollama_model {
            manifest.ollama_model = Some(m.clone());
        }
        if let Some(ib) = self.identity_binding {
            manifest.identity_binding = ib;
        }
        if let Some(ref ev) = self.evolution {
            manifest.evolution = ev.clone();
        }
        if let Some(ref mc) = self.memory_config {
            manifest.memory_config = mc.clone();
        }
        if let Some(ref k) = self.knowledge {
            manifest.knowledge = Some(k.clone());
        }
    }
}

impl Default for DiskRoleSettings {
    fn default() -> Self {
        Self {
            schema_version: default_schema_version(),
            identity_binding: None,
            evolution: None,
            memory_config: None,
            ollama_model: None,
            remote_presence: None,
            autonomous_scene: None,
            interaction_mode: None,
            plugin_backends: None,
            knowledge: None,
            reply_quality_anchor: None,
        }
    }
}
