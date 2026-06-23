//! Shared relation / favor / portrait emotion snapshot loading.

use crate::domain::role_runtime_snapshot::RoleRuntimeSnapshot;
use crate::error::Result;
use crate::state::AppState;

pub(crate) struct RelationSnapshot {
    pub relation_state: String,
    pub favorability: f64,
    pub portrait_emotion: String,
}

pub(crate) async fn load_relation_snapshot(
    state: &AppState,
    srid: &str,
    user_relation_key: &str,
    runtime_snapshot: Option<&RoleRuntimeSnapshot>,
) -> Result<RelationSnapshot> {
    let portrait_emotion = runtime_snapshot
        .and_then(|s| s.emotion.clone())
        .unwrap_or_else(|| "neutral".to_string());
    let (rel_id, rel_global, favorability) = tokio::try_join!(
        state
            .db_manager
            .get_relation_state_for_identity(srid, user_relation_key),
        state.db_manager.get_relation_state(srid),
        state
            .db_manager
            .favorability_for_identity_with_runtime_fallback(srid, user_relation_key),
    )?;
    Ok(RelationSnapshot {
        relation_state: rel_id
            .or(rel_global)
            .or_else(|| runtime_snapshot.and_then(|s| s.relation_state.clone()))
            .unwrap_or_else(|| "Stranger".to_string()),
        favorability,
        portrait_emotion,
    })
}
