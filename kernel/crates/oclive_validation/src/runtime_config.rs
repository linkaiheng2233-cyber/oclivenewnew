//! Blueprint `runtime_config` section (v3 target SSOT; v2 presence is non-fatal—host still reads transitional `meta` fields).

use serde::{Deserialize, Serialize};

use crate::disk_role_settings::{AutonomousSceneConfig, RemotePresenceConfig};
use crate::manifest::{EvolutionConfigDisk, MemoryConfigDisk};
use crate::validate::validate_interaction_mode_pack_setting;

/// Dual-core toggle (blueprint only; off by default).
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct DualCoreConfig {
    #[serde(default)]
    pub enabled: bool,
}

/// System runtime configuration (blueprint-only; not the role-pack creator view).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeConfig {
    #[serde(default)]
    pub interaction_mode: Option<String>,
    #[serde(default)]
    pub memory_config: Option<MemoryConfigDisk>,
    #[serde(default)]
    pub reply_quality_anchor: Option<String>,
    /// Optional pack-level hint aligned with host `app_settings.remote_fallback_to_builtin`.
    #[serde(default)]
    pub remote_fallback_to_builtin: Option<bool>,
    #[serde(default)]
    pub dual_core: Option<DualCoreConfig>,
    #[serde(default)]
    pub identity_binding: Option<crate::manifest::IdentityBinding>,
    #[serde(default)]
    pub evolution: Option<EvolutionConfigDisk>,
    #[serde(default)]
    pub ollama_model: Option<String>,
    #[serde(default)]
    pub remote_presence: Option<RemotePresenceConfig>,
    #[serde(default)]
    pub autonomous_scene: Option<AutonomousSceneConfig>,
}

/// Validate `runtime_config` sub-fields (before v3 blueprint load).
///
/// # Errors
///
/// Returns `Err(Vec<String>)` when sub-field contracts are violated.
pub fn validate_runtime_config(rc: &RuntimeConfig) -> Result<(), Vec<String>> {
    let mut errs = Vec::new();
    if let Some(ref m) = rc.interaction_mode {
        if let Err(e) = validate_interaction_mode_pack_setting(Some(m.as_str())) {
            errs.push(e);
        }
    }
    if errs.is_empty() {
        Ok(())
    } else {
        Err(errs)
    }
}
