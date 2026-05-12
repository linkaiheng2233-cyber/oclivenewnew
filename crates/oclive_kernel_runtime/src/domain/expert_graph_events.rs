//! Module 9: EventTrigger nodes — after a chat turn, write structured memories via [`MemoryRepository`].

use crate::error::Result;
use crate::models::expert_models::{EventTriggerMatchScope, ExpertGraph, ExpertNode};
use crate::state::KernelAppState;

/// Replace `{match}` / `{keyword}` with the matched substring (trimmed needle).
pub(crate) fn apply_event_memory_template(template: &str, needle: &str) -> String {
    template
        .replace("{match}", needle)
        .replace("{keyword}", needle)
}

/// Returns `(fires, hit_user, hit_bot)` for substring `needle` (non-empty).
pub(crate) fn event_trigger_fires(
    scope: EventTriggerMatchScope,
    needle: &str,
    user_message: &str,
    bot_reply: &str,
) -> (bool, bool, bool) {
    let hit_user = user_message.contains(needle);
    let hit_bot = bot_reply.contains(needle);
    let fires = match scope {
        EventTriggerMatchScope::Any => hit_user || hit_bot,
        EventTriggerMatchScope::UserOnly => hit_user,
        EventTriggerMatchScope::BotOnly => hit_bot,
    };
    (fires, hit_user, hit_bot)
}

/// After `apply_chat_turn_atomic`, evaluate `EventTrigger` nodes on the **effective** expert graph
/// (session override JSON → role default) and persist matching memories.
pub async fn apply_expert_graph_event_triggers_after_turn(
    state: &KernelAppState,
    manifest_role_id: &str,
    session_namespace: &str,
    user_message: &str,
    bot_reply: &str,
) -> Result<()> {
    let raw_sess = state
        .expert_models_repo
        .get_expert_models_session_override_json(session_namespace)
        .await?;
    let graph = if let Some(s) = raw_sess.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        serde_json::from_str::<ExpertGraph>(s).ok()
    } else {
        None
    };
    let graph = if let Some(g) = graph {
        g
    } else {
        let raw_role = state
            .expert_models_repo
            .get_expert_models_role_default_json(manifest_role_id)
            .await?;
        let Some(s) = raw_role.as_deref().map(str::trim).filter(|s| !s.is_empty()) else {
            return Ok(());
        };
        serde_json::from_str::<ExpertGraph>(s).unwrap_or_default()
    };

    for n in &graph.nodes {
        let ExpertNode::EventTrigger {
            match_substring,
            memory_content,
            importance,
            enabled,
            match_scope,
            ..
        } = n
        else {
            continue;
        };
        if !*enabled {
            continue;
        }
        let needle = match_substring.trim();
        if needle.is_empty() || memory_content.trim().is_empty() {
            continue;
        }
        let (fires, _, _) = event_trigger_fires(*match_scope, needle, user_message, bot_reply);
        if !fires {
            continue;
        }
        let body = apply_event_memory_template(memory_content, needle);
        let trimmed = body.trim();
        if trimmed.is_empty() {
            continue;
        }
        let imp = if importance.is_finite() && *importance >= 0.0 {
            *importance as f64
        } else {
            0.5
        };
        let _ = state
            .memory_repo
            .save_memory(session_namespace, trimmed, imp)
            .await?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn template_replaces_placeholders() {
        assert_eq!(
            apply_event_memory_template("回忆{keyword}与{match}", "猫"),
            "回忆猫与猫"
        );
    }

    #[test]
    fn scope_user_only_ignores_bot_hit() {
        let (fires, hit_u, hit_b) = event_trigger_fires(
            EventTriggerMatchScope::UserOnly,
            "hi",
            "hi there",
            "hi bot",
        );
        assert!(fires && hit_u && hit_b);
        let (fires2, _, _) =
            event_trigger_fires(EventTriggerMatchScope::UserOnly, "hi", "no match", "hi bot");
        assert!(!fires2);
    }

    #[test]
    fn scope_bot_only() {
        let (fires, _, _) =
            event_trigger_fires(EventTriggerMatchScope::BotOnly, "x", "no", "has x");
        assert!(fires);
        let (fires2, _, _) =
            event_trigger_fires(EventTriggerMatchScope::BotOnly, "x", "has x", "no");
        assert!(!fires2);
    }
}
