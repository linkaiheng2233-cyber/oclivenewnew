//! 磁盘 `manifest.json` 结构体：与 `oclivenewnew` 运行时 serde 形状一致（单一事实来源在本 crate）。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// manifest 中 `knowledge` 块（与 `models/knowledge.rs` 对齐）
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KnowledgePackConfigDisk {
    #[serde(default = "default_knowledge_enabled")]
    pub enabled: bool,
    #[serde(default = "default_knowledge_glob")]
    pub glob: String,
}

fn default_knowledge_enabled() -> bool {
    true
}

fn default_knowledge_glob() -> String {
    "knowledge/**/*.md".to_string()
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum IdentityBinding {
    Global,
    #[default]
    PerScene,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum LifeAvailability {
    #[default]
    Free,
    Distracted,
    Busy,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LifeTrajectoryDisk {
    #[serde(default)]
    pub summary: Option<String>,
    #[serde(default)]
    pub summary_lines: Vec<String>,
    #[serde(default)]
    pub stub_ooc: Option<String>,
    #[serde(default)]
    pub stub_messages: Vec<String>,
}

impl LifeTrajectoryDisk {
    #[must_use]
    pub fn effective_summary(&self) -> Option<String> {
        if !self.summary_lines.is_empty() {
            let joined = self
                .summary_lines
                .iter()
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>()
                .join("\n\n");
            if !joined.is_empty() {
                return Some(joined);
            }
        }
        self.summary
            .as_ref()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifeScheduleEntryDisk {
    pub weekday: u8,
    pub time_start: String,
    pub time_end: String,
    pub activity_id: String,
    pub label: String,
    #[serde(default)]
    pub preferred_scene_id: Option<String>,
    #[serde(default)]
    pub availability: Option<LifeAvailability>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LifeScheduleDisk {
    #[serde(default)]
    pub timezone_offset_minutes: Option<i32>,
    #[serde(default)]
    pub entries: Vec<LifeScheduleEntryDisk>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskRoleManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    #[serde(default, alias = "model")]
    pub ollama_model: Option<String>,
    #[serde(default)]
    pub default_personality: Vec<f32>,
    #[serde(default)]
    pub evolution: EvolutionConfigDisk,
    #[serde(default)]
    pub scenes: Vec<String>,
    #[serde(default)]
    pub user_relations: HashMap<String, UserRelationDisk>,
    #[serde(default)]
    pub default_relation: String,
    #[serde(default)]
    pub memory_config: MemoryConfigDisk,
    #[serde(default)]
    pub identity_binding: IdentityBinding,
    #[serde(default)]
    pub life_trajectory: Option<LifeTrajectoryDisk>,
    #[serde(default)]
    pub life_schedule: Option<LifeScheduleDisk>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub dev_only: bool,
    #[serde(default)]
    pub knowledge: Option<KnowledgePackConfigDisk>,
    /// 最低 oclive 宿主版本（semver，如 `"0.2.0"`）；省略则不检查。
    #[serde(default)]
    pub min_runtime_version: Option<String>,
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
