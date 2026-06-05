//! Shared relation / favor / portrait emotion snapshot loading.

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
) -> Result<RelationSnapshot> {
    let (rel_id, rel_global, favorability, portrait_emotion) = tokio::try_join!(
        state
            .db_manager
            .get_relation_state_for_identity(srid, user_relation_key),
        state.db_manager.get_relation_state(srid),
        state
            .db_manager
            .favorability_for_identity_with_runtime_fallback(srid, user_relation_key),
        async {
            state
                .db_manager
                .get_current_emotion(srid)
                .await
                .map(|e| e.unwrap_or_else(|| "neutral".to_string()))
        },
    )?;
    Ok(RelationSnapshot {
        relation_state: rel_id
            .or(rel_global)
            .unwrap_or_else(|| "Stranger".to_string()),
        favorability,
        portrait_emotion,
    })
}
