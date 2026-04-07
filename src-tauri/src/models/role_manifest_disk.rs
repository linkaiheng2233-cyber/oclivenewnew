//! 磁盘 `manifest.json`（新角色包规范），反序列化后与内部 `Role` 映射。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::knowledge::KnowledgePackConfigDisk;
use super::role::{
    EvolutionBounds, EvolutionConfig, IdentityBinding, LifeScheduleDisk, LifeTrajectoryDisk,
    MemoryConfig, PersonalityDefaults, Role, UserRelation,
};

/// 顶层 manifest（与 `handoff/DECISIONS_2026-04-02.md` 一致）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskRoleManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    /// Ollama 模型名（可与 JSON 字段 `model` 互换）
    #[serde(default, alias = "model")]
    pub ollama_model: Option<String>,
    /// 旧七维顺序：倔强、黏人、敏感、强势、宽容、话多、温暖
    #[serde(default)]
    pub default_personality: Vec<f32>,
    #[serde(default)]
    pub evolution: EvolutionConfigDisk,
    #[serde(default)]
    pub scenes: Vec<String>,
    /// 关系键 -> 提示与好感倍率
    #[serde(default)]
    pub user_relations: HashMap<String, UserRelationDisk>,
    #[serde(default)]
    pub default_relation: String,
    #[serde(default)]
    pub memory_config: MemoryConfigDisk,
    /// `global`：全剧一个身份；`per_scene`：可按场景覆盖（默认，与旧版一致）。
    #[serde(default)]
    pub identity_binding: IdentityBinding,
    /// 异地时生活轨迹总述与占位句（创作者）；开关见 `settings.json` → `remote_presence.default_enabled`
    #[serde(default)]
    pub life_trajectory: Option<LifeTrajectoryDisk>,
    /// 可选：按虚拟时间推断「此刻在做什么」（与 `life_trajectory` 异地文案气质并存）
    #[serde(default)]
    pub life_schedule: Option<LifeScheduleDisk>,
    /// 为 true 时默认不出现在角色列表（`list_roles`）；仍可通过 `load_role` 按 id 加载。调试/示例包使用；见 `OCLIVE_LIST_DEV_ROLES`。
    #[serde(default, skip_serializing_if = "is_false")]
    pub dev_only: bool,
    /// 可选：世界观知识目录 `knowledge/` 与 glob；省略时若存在 `knowledge/` 则自动加载。
    #[serde(default)]
    pub knowledge: Option<KnowledgePackConfigDisk>,
}

fn is_false(b: &bool) -> bool {
    !*b
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionConfigDisk {
    #[serde(default = "default_one")]
    pub event_impact_factor: f64,
    #[serde(default = "default_ai_interval")]
    pub ai_analysis_interval: i32,
    #[serde(default = "default_max_change")]
    pub max_change_per_event: f64,
    #[serde(default = "default_max_total")]
    pub max_total_change: f64,
}

fn default_one() -> f64 {
    1.0
}
fn default_ai_interval() -> i32 {
    15
}
fn default_max_change() -> f64 {
    0.05
}
fn default_max_total() -> f64 {
    0.5
}

impl Default for EvolutionConfigDisk {
    fn default() -> Self {
        Self {
            event_impact_factor: default_one(),
            ai_analysis_interval: default_ai_interval(),
            max_change_per_event: default_max_change(),
            max_total_change: default_max_total(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRelationDisk {
    /// 界面与提示中展示的名称；省略或空则使用关系键 `id`（英文）。
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub display_name: String,
    #[serde(default)]
    pub prompt_hint: String,
    #[serde(default = "default_favor_mult")]
    pub favor_multiplier: f32,
    #[serde(default = "default_initial_favor_disk")]
    pub initial_favorability: f64,
}

impl Default for UserRelationDisk {
    fn default() -> Self {
        Self {
            display_name: String::new(),
            prompt_hint: String::new(),
            favor_multiplier: default_favor_mult(),
            initial_favorability: default_initial_favor_disk(),
        }
    }
}

fn default_favor_mult() -> f32 {
    1.0
}

fn default_initial_favor_disk() -> f64 {
    50.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConfigDisk {
    #[serde(default = "default_scene_w")]
    pub scene_weight_multiplier: f64,
    #[serde(default)]
    pub topic_weights: HashMap<String, HashMap<String, f64>>,
}

fn default_scene_w() -> f64 {
    1.2
}

impl Default for MemoryConfigDisk {
    fn default() -> Self {
        Self {
            scene_weight_multiplier: default_scene_w(),
            topic_weights: HashMap::new(),
        }
    }
}

/// 导出 manifest 时写入 `life_trajectory`：优先内存中 `role.life_trajectory`；若占位句仅在 `settings` 遗留字段中，一并带上便于迁移。
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

impl DiskRoleManifest {
    /// 从运行时 `Role` 生成可写回磁盘的 manifest（用于测试或导出）
    pub fn from_role(role: &Role) -> Self {
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
        Self {
            id: role.id.clone(),
            name: role.name.clone(),
            version: role.version.clone(),
            author: role.author.clone(),
            description: role.description.clone(),
            ollama_model: None,
            default_personality,
            evolution: EvolutionConfigDisk::default(),
            scenes: vec![],
            user_relations,
            default_relation: role.default_relation.clone(),
            memory_config: MemoryConfigDisk::default(),
            identity_binding: IdentityBinding::default(),
            life_trajectory: life_trajectory_for_manifest_export(role),
            life_schedule: role.life_schedule.clone(),
            dev_only: role.dev_only,
            knowledge: None,
        }
    }

    pub fn to_role(&self) -> Role {
        let def = Self::vec_to_defaults(&self.default_personality);
        let bounds = EvolutionBounds::full_01();
        let user_relations: Vec<UserRelation> = self
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
            event_impact_factor: self.evolution.event_impact_factor,
            ai_analysis_interval: self.evolution.ai_analysis_interval,
            max_change_per_event: self.evolution.max_change_per_event,
            max_total_change: self.evolution.max_total_change,
        };

        let memory_config = MemoryConfig {
            scene_weight_multiplier: self.memory_config.scene_weight_multiplier,
            topic_weights: self.memory_config.topic_weights.clone(),
        };

        Role {
            id: self.id.clone(),
            name: self.name.clone(),
            description: self.description.clone(),
            version: self.version.clone(),
            author: self.author.clone(),
            core_personality: String::new(),
            default_personality: def,
            evolution_bounds: bounds,
            user_relations,
            evolution_config,
            memory_config: Some(memory_config),
            default_relation: if self.default_relation.is_empty() {
                "friend".to_string()
            } else {
                self.default_relation.clone()
            },
            ollama_model: self.ollama_model.clone(),
            identity_binding: self.identity_binding,
            life_trajectory: self.life_trajectory.clone(),
            life_schedule: self.life_schedule.clone(),
            remote_presence: None,
            autonomous_scene: None,
            interaction_mode: None,
            dev_only: self.dev_only,
            plugin_backends: super::PluginBackends::default(),
            knowledge_index: None,
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
}
