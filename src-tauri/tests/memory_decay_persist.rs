//! Memory decay helpers.

#![allow(clippy::unwrap_used)]

use chrono::Utc;
use oclive_kernel_host::domain::memory_engine::MemoryEngine;
use oclive_kernel_types::models::{Memory, RolePackMemoryConfig};

#[test]
fn decay_memories_in_place_reduces_weight() {
    let cfg = RolePackMemoryConfig::default();
    let created = Utc::now() - chrono::Duration::days(14);
    let mut memories = vec![Memory {
        id: "1".into(),
        role_id: "r".into(),
        content: "x".into(),
        importance: 0.8,
        weight: 1.0,
        created_at: created,
        scene_id: None,
        mention_count: 1,
        accessed_at: None,
    }];
    MemoryEngine::decay_memories_in_place(&mut memories, |_| 14.0, &cfg);
    assert!(memories[0].weight < 1.0);
}
