//! Turn-scoped context for chat orchestration (avoids repeating ids / backends across branches).

use crate::domain::plugin_host::ResolvedRolePlugins;
use crate::models::dto::SendMessageRequest;
use crate::models::{PluginBackends, Role};
use crate::state::AppState;
use std::sync::Arc;
use std::time::Instant;

/// Manifest role id, session namespace, and scene — passed together to avoid `&str` parameter swaps.
#[derive(Clone, Copy)]
pub struct TurnIds<'a> {
    pub mrid: &'a str,
    pub srid: &'a str,
    pub scene_id: &'a str,
}

/// Shared inputs for `process_co_present`, remote branches, and dual-core paths.
pub struct TurnContext<'a> {
    pub state: &'a AppState,
    pub req: &'a SendMessageRequest,
    pub role: &'a Role,
    pub scene_id: &'a str,
    pub scenes: Arc<[String]>,
    pub mrid: &'a str,
    pub srid: &'a str,
    pub t0: Instant,
    pub preflight_ms: u64,
    pub effective_backends: Arc<PluginBackends>,
    /// Session-resolved plugin handles; parsed once per turn in `process_message`.
    pub pl: ResolvedRolePlugins,
    pub immersive: bool,
    /// Character-side scene when user is remote (remote-life prompt / knowledge).
    pub character_scene_id: Option<String>,
    /// Virtual time (ms) prefetched once per turn for prompt / life schedule; 0 if unset.
    pub virtual_time_ms: i64,
    /// `true` when blueprint requests dual-core but the host was built without `dual_core` feature.
    pub dual_core_degraded: bool,
}

impl<'a> TurnContext<'a> {
    #[must_use]
    pub fn ids(&self) -> TurnIds<'a> {
        TurnIds {
            mrid: self.mrid,
            srid: self.srid,
            scene_id: self.scene_id,
        }
    }
}
