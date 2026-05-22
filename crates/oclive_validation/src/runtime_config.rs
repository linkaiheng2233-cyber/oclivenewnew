//! 蓝图 `runtime_config` 段（v3 目标 SSOT；v2 出现时不报错、宿主仍读 `meta` 过渡期字段）。

use serde::Deserialize;

use crate::disk_role_settings::{AutonomousSceneConfig, RemotePresenceConfig};
use crate::manifest::{EvolutionConfigDisk, MemoryConfigDisk};
use crate::validate::validate_interaction_mode_pack_setting;

/// 双核开关（仅蓝图；默认关）。
#[derive(Debug, Clone, Default, Deserialize, PartialEq, Eq)]
pub struct DualCoreConfig {
    #[serde(default)]
    pub enabled: bool,
}

/// 系统运行时配置（蓝图专属，非角色包创作者视图）。
#[derive(Debug, Clone, Default, Deserialize)]
pub struct RuntimeConfig {
    #[serde(default)]
    pub interaction_mode: Option<String>,
    #[serde(default)]
    pub memory_config: Option<MemoryConfigDisk>,
    #[serde(default)]
    pub reply_quality_anchor: Option<String>,
    /// 与宿主 `app_settings.remote_fallback_to_builtin` 对齐的**包级建议**（可选）。
    #[serde(default)]
    pub remote_fallback_to_builtin: Option<bool>,
    #[serde(default)]
    pub dual_core: Option<DualCoreConfig>,
    #[serde(default)]
    pub identity_binding: Option<crate::manifest::IdentityBinding>,
    #[serde(default)]
    pub evolution: Option<EvolutionConfigDisk>,
    #[serde(default, alias = "model")]
    pub ollama_model: Option<String>,
    #[serde(default)]
    pub remote_presence: Option<RemotePresenceConfig>,
    #[serde(default)]
    pub autonomous_scene: Option<AutonomousSceneConfig>,
}

/// 校验 `runtime_config` 子字段（v3 蓝图加载前）。
///
/// # Errors
///
/// 子字段契约不符时返回 `Err(Vec<String>)`。
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
