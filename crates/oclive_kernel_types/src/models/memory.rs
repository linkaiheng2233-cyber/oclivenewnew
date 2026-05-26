use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Persisted long-term memory row for a role.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memory {
    pub id: String,
    pub role_id: String,
    pub content: String,
    pub importance: f64,
    pub weight: f64,
    pub created_at: DateTime<Utc>,
    /// 写入时的场景 id；旧数据为 `None`
    #[serde(default)]
    pub scene_id: Option<String>,
    /// 被相似话题强化（提及）次数；新写入默认为 1。
    #[serde(default = "default_mention_count")]
    pub mention_count: i32,
}

fn default_mention_count() -> i32 {
    1
}

impl Memory {
    /// 用于排序与遗忘曲线的有效强度（importance × weight）。
    #[must_use]
    pub fn effective_strength(&self) -> f64 {
        (self.importance * self.weight).clamp(0.0, 1.0)
    }
}

/// Ranked memories plus token budget metadata for prompt assembly.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryContext {
    pub memories: Vec<Memory>,
    pub total_tokens: usize,
}
