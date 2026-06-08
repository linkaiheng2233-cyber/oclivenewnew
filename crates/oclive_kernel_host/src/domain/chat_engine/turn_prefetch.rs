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

/// Load recent dialogue and active user identity in parallel.
///
/// # Errors
///
/// Database or identity resolution failures.
pub async fn build_turn_prefetch(
    state: &AppState,
    role: &Role,
    srid: &str,
    scene_id: &str,
) -> Result<TurnPrefetch> {
    let (context, resolved_identity) = tokio::try_join!(
        load_recent_context(state, srid),
        resolve_active_user_identity(state, role, srid, Some(scene_id)),
    )?;
    let (recent_turns, recent_turns_for_event, recent_events) = context;
    Ok(TurnPrefetch {
        recent_turns,
        recent_turns_for_event,
        recent_events,
        resolved_identity,
    })
}
