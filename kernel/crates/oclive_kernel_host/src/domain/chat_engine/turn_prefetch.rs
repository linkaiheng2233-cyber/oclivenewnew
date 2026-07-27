//! Shared turn prefetch: recent context + user identity (agent + pre_llm).

use crate::domain::chat_engine::context::load_recent_context;
use crate::domain::user_identity_loader::{resolve_active_user_identity, ResolvedUserIdentity};
use crate::error::Result;
use crate::models::{Event, Role};
use crate::state::AppState;

/// Context loaded once per turn and reused by agent (when enabled) and `pre_llm`.
#[derive(Debug, Clone)]
pub struct TurnPrefetch {
    pub recent_turns: Vec<(String, String)>,
    pub recent_turns_for_event: Vec<(String, String)>,
    pub recent_events: Vec<Event>,
    pub resolved_identity: ResolvedUserIdentity,
}

/// Load the active identity before dialogue so an explicitly ineligible
/// identity can never cause adult-scoped memories to enter the turn.
///
/// # Errors
///
/// Database or identity resolution failures.
pub async fn build_turn_prefetch(
    state: &AppState,
    role: &Role,
    srid: &str,
    scene_id: &str,
    include_adult: bool,
) -> Result<TurnPrefetch> {
    let resolved_identity = resolve_active_user_identity(state, role, srid, Some(scene_id)).await?;
    let context = load_recent_context(
        state,
        srid,
        include_adult && resolved_identity.adult_eligible,
    )
    .await?;
    let (recent_turns, recent_turns_for_event, recent_events) = context;
    Ok(TurnPrefetch {
        recent_turns,
        recent_turns_for_event,
        recent_events,
        resolved_identity,
    })
}
