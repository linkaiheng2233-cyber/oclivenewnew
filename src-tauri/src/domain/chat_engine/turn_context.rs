//! Turn-scoped context for chat orchestration (avoids repeating ids / backends across branches).

use crate::models::{PluginBackends, Role};
use crate::models::dto::SendMessageRequest;
use crate::state::AppState;
use std::time::Instant;

/// Shared inputs for `process_co_present`, remote branches, and dual-core paths.
pub struct TurnContext<'a> {
    pub state: &'a AppState,
    pub req: &'a SendMessageRequest,
    pub role: &'a Role,
    pub scene_id: String,
    pub scenes: Vec<String>,
    pub mrid: &'a str,
    pub srid: &'a str,
    pub t0: Instant,
    pub preflight_ms: u64,
    pub effective_backends: PluginBackends,
    pub immersive: bool,
}
