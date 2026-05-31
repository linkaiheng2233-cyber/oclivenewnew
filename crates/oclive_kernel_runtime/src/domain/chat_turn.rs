//! Reusable per-turn steps (pure functions or small structs) to slim `chat_engine::process_message`,
//! ease unit tests, and allow extensions (e.g. swapping memory weighting).

use crate::models::{Memory, Role};

/// Weights same-scene memories by manifest `scene_weight_multiplier` (call before relevant-memory retrieval).
pub fn weight_memories_for_scene(memories: &mut [Memory], scene_id: &str, multiplier: f64) {
    if (multiplier - 1.0).abs() < f64::EPSILON {
        return;
    }
    for m in memories.iter_mut() {
        if m.scene_id.as_deref() == Some(scene_id) {
            m.importance *= multiplier;
        }
    }
}

/// Prompt hint and favor multiplier for the current user relation key (from role pack `user_relations`).
pub struct RelationFavorContext<'a> {
    pub relation_hint: &'a str,
    pub favor_mult: f64,
}

#[must_use]
pub fn relation_favor_for_key<'a>(
    role: &'a Role,
    user_relation_key: &str,
) -> RelationFavorContext<'a> {
    let ur = role
        .user_relations
        .iter()
        .find(|r| r.id == user_relation_key);
    RelationFavorContext {
        relation_hint: ur.map(|r| r.prompt_hint.as_str()).unwrap_or(""),
        favor_mult: ur.map(|r| r.favor_multiplier as f64).unwrap_or(1.0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn weight_doubles_same_scene_only() {
        let mut memories = vec![
            Memory {
                id: "1".into(),
                role_id: "r".into(),
                content: "a".into(),
                importance: 1.0,
                weight: 1.0,
                created_at: Utc::now(),
                scene_id: Some("home".into()),
                mention_count: 1,
            },
            Memory {
                id: "2".into(),
                role_id: "r".into(),
                content: "b".into(),
                importance: 1.0,
                weight: 1.0,
                created_at: Utc::now(),
                scene_id: Some("school".into()),
                mention_count: 1,
            },
        ];
        weight_memories_for_scene(&mut memories, "home", 2.0);
        assert!((memories[0].importance - 2.0).abs() < 1e-9);
        assert!((memories[1].importance - 1.0).abs() < 1e-9);
    }
}
