//! 磁盘 `settings.json`：引擎相关字段，加载后合并进 `DiskRoleManifest` 再校验与转 `Role`。
//!
//! 结构体定义见 [`oclive_validation::DiskRoleSettings`]。

use super::role::{RemotePresenceConfig, Role};
pub use oclive_validation::{DiskRoleSettings, CURRENT_SETTINGS_SCHEMA_VERSION};

/// 从运行时 `Role` 生成完整 `settings.json` 内容（与旧版单文件 manifest 中引擎段一致）。
pub fn disk_role_settings_from_role(role: &Role) -> DiskRoleSettings {
    use super::role_manifest_disk::EvolutionConfigDisk;
    let memory_config = role.memory_config.clone().unwrap_or_default();
    DiskRoleSettings {
        schema_version: CURRENT_SETTINGS_SCHEMA_VERSION,
        ollama_model: role.ollama_model.clone(),
        identity_binding: Some(role.identity_binding),
        evolution: Some(EvolutionConfigDisk {
            event_impact_factor: role.evolution_config.event_impact_factor,
            ai_analysis_interval: role.evolution_config.ai_analysis_interval,
            max_change_per_event: role.evolution_config.max_change_per_event,
            max_total_change: role.evolution_config.max_total_change,
            personality_source: role.evolution_config.personality_source,
        }),
        memory_config: Some(super::role_manifest_disk::MemoryConfigDisk {
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
        reply_quality_anchor: role.reply_quality_anchor.clone(),
    }
}
