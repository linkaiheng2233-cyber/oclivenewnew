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
    /// Scene id at write time; legacy rows may be `None`.
    #[serde(default)]
    pub scene_id: Option<String>,
    /// Times reinforced by similar-topic mentions; defaults to 1 on new writes.
    #[serde(default = "default_mention_count")]
    pub mention_count: i32,
}

fn default_mention_count() -> i32 {
    1
}

impl Memory {
    /// Effective strength for sorting and forgetting curves (`importance × weight`).
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
