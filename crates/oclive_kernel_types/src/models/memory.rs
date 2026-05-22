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
}

/// Ranked memories plus token budget metadata for prompt assembly.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryContext {
    pub memories: Vec<Memory>,
    pub total_tokens: usize,
}
