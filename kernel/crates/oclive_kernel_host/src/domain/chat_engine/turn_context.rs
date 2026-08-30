//! Turn-scoped context for chat orchestration (avoids repeating ids / backends across branches).

use crate::domain::chat_engine::turn_prefetch::TurnPrefetch;
use crate::domain::plugin_host::ResolvedRolePlugins;
use crate::domain::role_runtime_snapshot::RoleRuntimeSnapshot;
use crate::models::dto::{SendMessageRequest, TurnOrigin};
use crate::models::{PluginBackends, Role};
use crate::state::{AppState, EffectiveSessionConfig};
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
    /// Opaque identifier shared by every Event Ring emission produced by this turn.
    pub correlation_id: String,
    pub role: &'a Role,
    pub scene_id: &'a str,
    pub scenes: Arc<[String]>,
    pub mrid: &'a str,
    pub srid: &'a str,
    pub t0: Instant,
    pub preflight_ms: u64,
    pub session_config: Arc<EffectiveSessionConfig>,
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
    /// One-row `role_runtime` snapshot loaded once per turn.
    pub runtime_snapshot: RoleRuntimeSnapshot,
    /// Shared ownership for background tasks (avoids `Role` clone per spawn).
    pub role_arc: Arc<Role>,
    /// Recent context + user identity shared by agent (when enabled) and `pre_llm`.
    pub prefetch: TurnPrefetch,
    /// Semantic input source. This is kept outside `SendMessageRequest` so universal HTTP/Tauri
    /// clients cannot select a lower-persistence execution policy.
    pub origin: TurnOrigin,
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

    /// Background adult beats are generated speculatively and must remain
    /// side-effect free until the foreground explicitly commits them.
    #[must_use]
    pub fn is_staged(&self) -> bool {
        self.req
            .adult
            .as_ref()
            .and_then(|adult| adult.stage.as_ref())
            .is_some()
    }

    /// Whether this turn may commit user-chat state.
    #[must_use]
    pub fn persists_user_state(&self) -> bool {
        !self.is_staged() && self.origin.persists_user_state()
    }
}
