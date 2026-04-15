//! 磁盘 `manifest.json`（新角色包规范），反序列化后与内部 `Role` 映射。
//!
//! 结构体定义见共享 crate [`oclive_validation`]；此处提供 `Role` ↔ 磁盘 的转换函数。

use std::collections::HashMap;

use oclive_validation::IdentityBinding;
pub use oclive_validation::{
    DiskRoleManifest, EvolutionConfigDisk, MemoryConfigDisk, UserRelationDisk,
};

use super::role::{
    EvolutionBounds, EvolutionConfig, LifeTrajectoryDisk, MemoryConfig, PersonalityDefaults, Role,
    UserRelation,
};

/// 从运行时 `Role` 生成可写回磁盘的 manifest（用于测试或导出）
pub fn disk_manifest_from_role(role: &Role) -> DiskRoleManifest {
    let mut default_personality: Vec<f32> = vec![
        role.default_personality.stubbornness,
        role.default_personality.clinginess,
        role.default_personality.sensitivity,
        role.default_personality.assertiveness,
        role.default_personality.forgiveness,
        role.default_personality.talkativeness,
        role.default_personality.warmth,
    ];
    if default_personality.len() != 7 {
        default_personality = vec![0.5; 7];
    }
    let mut user_relations: HashMap<String, UserRelationDisk> = HashMap::new();
    for ur in &role.user_relations {
        let display_name = if ur.name == ur.id {
            String::new()
        } else {
            ur.name.clone()
        };
        user_relations.insert(
            ur.id.clone(),
            UserRelationDisk {
                display_name,
                prompt_hint: ur.prompt_hint.clone(),
                favor_multiplier: ur.favor_multiplier,
                initial_favorability: ur.initial_favorability,
            },
        );
    }
    DiskRoleManifest {
        id: role.id.clone(),
        name: role.name.clone(),
        version: role.version.clone(),
        author: role.author.clone(),
        description: role.description.clone(),
        ollama_model: None,
        default_personality,
        evolution: EvolutionConfigDisk {
            event_impact_factor: role.evolution_config.event_impact_factor,
            ai_analysis_interval: role.evolution_config.ai_analysis_interval,
            max_change_per_event: role.evolution_config.max_change_per_event,
            max_total_change: role.evolution_config.max_total_change,
            personality_source: role.evolution_config.personality_source,
        },
        scenes: vec![],
        user_relations,
        default_relation: role.default_relation.clone(),
        memory_config: MemoryConfigDisk::default(),
        identity_binding: IdentityBinding::default(),
        life_trajectory: life_trajectory_for_manifest_export(role),
        life_schedule: role.life_schedule.clone(),
        dev_only: role.dev_only,
        knowledge: None,
        min_runtime_version: role.min_runtime_version.clone(),
    }
}

pub fn disk_manifest_to_role(d: &DiskRoleManifest) -> Role {
    let def = vec_to_defaults(&d.default_personality);
    let bounds = EvolutionBounds::full_01();
    let user_relations: Vec<UserRelation> = d
        .user_relations
        .iter()
        .map(|(id, r)| {
            let name = r.display_name.trim();
            UserRelation {
                id: id.clone(),
                name: if name.is_empty() {
                    id.clone()
                } else {
                    name.to_string()
                },
                prompt_hint: r.prompt_hint.clone(),
                favor_multiplier: r.favor_multiplier,
                initial_favorability: r.initial_favorability.clamp(0.0, 100.0),
            }
        })
        .collect();

    let evolution_config = EvolutionConfig {
        event_impact_factor: d.evolution.event_impact_factor,
        ai_analysis_interval: d.evolution.ai_analysis_interval,
        max_change_per_event: d.evolution.max_change_per_event,
        max_total_change: d.evolution.max_total_change,
        personality_source: d.evolution.personality_source,
    };

    let memory_config = MemoryConfig {
        scene_weight_multiplier: d.memory_config.scene_weight_multiplier,
        topic_weights: d.memory_config.topic_weights.clone(),
    };

    Role {
        id: d.id.clone(),
        name: d.name.clone(),
        description: d.description.clone(),
        version: d.version.clone(),
        author: d.author.clone(),
        core_personality: String::new(),
        default_personality: def,
        evolution_bounds: bounds,
        user_relations,
        evolution_config,
        memory_config: Some(memory_config),
        default_relation: if d.default_relation.is_empty() {
            "friend".to_string()
        } else {
            d.default_relation.clone()
        },
        ollama_model: d.ollama_model.clone(),
        identity_binding: d.identity_binding,
        life_trajectory: d.life_trajectory.clone(),
        life_schedule: d.life_schedule.clone(),
        remote_presence: None,
        autonomous_scene: None,
        interaction_mode: None,
        min_runtime_version: d.min_runtime_version.clone(),
        dev_only: d.dev_only,
        plugin_backends: super::PluginBackends::default(),
        knowledge_index: None,
        ui_config: super::UiConfig::default(),
    }
}

fn life_trajectory_for_manifest_export(role: &Role) -> Option<LifeTrajectoryDisk> {
    let mut lt = role.life_trajectory.clone().unwrap_or_default();
    if lt.stub_messages.is_empty() {
        if let Some(rp) = role.remote_presence.as_ref() {
            lt.stub_messages.clone_from(&rp.stub_messages);
        }
    }
    let summary_empty = lt.effective_summary().is_none();
    let stubs_empty = lt.stub_messages.is_empty();
    if summary_empty && stubs_empty {
        None
    } else {
        Some(lt)
    }
}

fn vec_to_defaults(v: &[f32]) -> PersonalityDefaults {
    let z = |i: usize| v.get(i).copied().unwrap_or(0.5);
    PersonalityDefaults {
        stubbornness: z(0),
        clinginess: z(1),
        sensitivity: z(2),
        assertiveness: z(3),
        forgiveness: z(4),
        talkativeness: z(5),
        warmth: z(6),
    }
}
