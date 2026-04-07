//! 磁盘 `settings.json`：引擎相关字段，加载后合并进 `DiskRoleManifest` 再校验与转 `Role`。

use serde::{Deserialize, Serialize};

use super::knowledge::KnowledgePackConfigDisk;
use super::plugin_backends::PluginBackends;
use super::role::{AutonomousSceneConfig, IdentityBinding, RemotePresenceConfig, Role};
use super::role_manifest_disk::{DiskRoleManifest, EvolutionConfigDisk, MemoryConfigDisk};

fn default_schema_version() -> u32 {
    1
}

/// 角色包引擎设置（可与 `manifest.json` 分文件存放）。
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
    /// `immersive` | `pure_chat`；省略表示沉浸
    #[serde(default)]
    pub interaction_mode: Option<String>,
    /// 可替换子系统后端（见 `creator-docs/plugin-and-architecture/PLUGIN_V1.md`）
    #[serde(default)]
    pub plugin_backends: Option<PluginBackends>,
    /// 覆盖 manifest 中的世界观知识开关与 glob（可选）
    #[serde(default)]
    pub knowledge: Option<KnowledgePackConfigDisk>,
}

impl DiskRoleSettings {
    /// 将本文件中出现的字段覆盖到已解析的 manifest（仅 `Some` 项）。
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
        // `remote_presence` 中占位句已归 manifest `life_trajectory`，见 `DiskRoleSettings::from_role`
    }

    /// 从运行时 `Role` 生成完整 `settings.json` 内容（与旧版单文件 manifest 中引擎段一致）。
    pub fn from_role(role: &Role) -> Self {
        let memory_config = role.memory_config.clone().unwrap_or_default();
        Self {
            schema_version: default_schema_version(),
            ollama_model: role.ollama_model.clone(),
            identity_binding: Some(role.identity_binding),
            evolution: Some(EvolutionConfigDisk {
                event_impact_factor: role.evolution_config.event_impact_factor,
                ai_analysis_interval: role.evolution_config.ai_analysis_interval,
                max_change_per_event: role.evolution_config.max_change_per_event,
                max_total_change: role.evolution_config.max_total_change,
            }),
            memory_config: Some(MemoryConfigDisk {
                scene_weight_multiplier: memory_config.scene_weight_multiplier,
                topic_weights: memory_config.topic_weights.clone(),
            }),
            remote_presence: role.remote_presence.as_ref().and_then(|r| {
                r.default_enabled.map(|_| RemotePresenceConfig {
                    default_enabled: r.default_enabled,
                    stub_messages: Vec::new(),
                })
            }),
            autonomous_scene: role.autonomous_scene.clone(),
            interaction_mode: role.interaction_mode.clone(),
            plugin_backends: Some(role.plugin_backends.clone()),
            knowledge: None,
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
        }
    }
}
