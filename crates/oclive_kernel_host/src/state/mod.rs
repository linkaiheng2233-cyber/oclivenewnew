use crate::domain::plugin_host::{PluginHost, ResolvedRolePlugins};
use crate::domain::ports::PluginHostPort;
use crate::domain::repository::{FavorabilityRepository, MemoryRepository};
use crate::domain::{
    EventEstimator, FileManifestLocalPluginBridge, LocalPluginBridge, LocalPluginCapability,
    LocalPluginProviderDescriptor, MemoryRetrieval, PromptAssembler, UserEmotionAnalyzer,
};
use crate::error::Result;
use crate::infrastructure::chat_storage::{ConversationStore, ReplayTaskRegistry};
use crate::infrastructure::db::DbManager;
use crate::infrastructure::directory_plugins::DirectoryPluginRuntime;
use crate::infrastructure::high_risk_grants::HighRiskGrantStore;
use crate::infrastructure::llm::LlmClient;
use crate::infrastructure::policy_registry::{
    build_policy_sets_from_registry, load_policy_registry_from_path, PolicyRuntime, PolicySet,
};
use crate::infrastructure::remote_fallback_policy::remote_fallback_from_db_value;
use crate::infrastructure::storage::RoleStorage;
use crate::models::{PersonalitySource, PersonalityVector, PluginBackends, Role};
use arc_swap::ArcSwap;
use dashmap::DashMap;
use parking_lot::RwLock;
use indexmap::IndexMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::{Mutex, OnceCell};

mod app_state_builder;
mod effective_session_config;
pub(crate) mod host_backends;
pub(crate) mod profile_evolution;
mod models_dir;
mod roles_dir;
mod session_backends;
mod session_cache;
pub use effective_session_config::EffectiveSessionConfig;
pub use models_dir::{
    ensure_models_dir, ensure_models_dir_for_roles, is_managed_legacy_models_path,
    legacy_models_dir_candidates, migrate_and_cleanup_models, paths_equal,
    reconcile_legacy_models_layout, resolve_models_dir, resource_dir_from_roles, ENV_MODELS_DIR,
};
pub use roles_dir::resolve_roles_dir;
pub use session_cache::SessionCache;

/// Per-`srid` turn mutex with last-touch time for eviction of idle entries.
struct TurnLockEntry {
    lock: Arc<Mutex<()>>,
    last_touch_ms: AtomicU64,
}

const TURN_LOCK_SOFT_CAP: usize = 512;
const TURN_LOCK_TARGET: usize = 256;
const ROLE_CACHE_CAPACITY: usize = 32;

fn turn_lock_now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_millis() as u64)
}

/// Tauri-managed application state (shared with in-process HTTP API).
pub type SharedAppState = Arc<AppState>;

use crate::domain::startup_health::StartupHealthCache;
use app_state_builder::AppStateBuilder;

pub struct AppState {
    pub db_manager: Arc<DbManager>,
    /// SQLite + JSON mirror chat history (decoupled from memory tables).
    pub conversation_store: Arc<dyn ConversationStore>,
    /// Background memory replay from chat history (progress keyed by task id).
    pub replay_tasks: Arc<ReplayTaskRegistry>,
    pub memory_repo: Arc<dyn MemoryRepository>,
    pub favorability_repo: Arc<dyn FavorabilityRepository>,
    pub user_llm_secrets: Arc<dyn oclive_kernel_contracts::UserLlmSecretsPort>,
    pub llm: Arc<dyn LlmClient>,
    /// Hot-path lock layering:
    /// - `role_cache` / `role_load_inflight`: dedupe role reads;
    /// - `session_cache`: session overrides;
    /// - `user_llm_*`: in-process LLM config mirror;
    ///
    ///   none nested—avoids lock-order cycles.
    pub role_cache: Arc<RwLock<IndexMap<String, Arc<Role>>>>,
    /// Dedupe cold loads for the same `role_id` ([`OnceCell`]); after load, write [`Self::role_cache`] and remove this entry.
    role_load_inflight: DashMap<String, Arc<OnceCell<Arc<Role>>>>,
    /// Roles loaded from arbitrary `role_path` for HTTP `--api` trial chat; not written to [`Self::role_cache`].
    pub(crate) http_api_roles: DashMap<String, Arc<Role>>,
    /// FIFO order for [`Self::http_api_roles`] eviction (cap [`ROLE_CACHE_CAPACITY`]).
    http_api_roles_order: RwLock<IndexMap<String, ()>>,
    pub session_cache: Arc<SessionCache>,
    pub storage: RoleStorage,
    policy_runtime: Arc<ArcSwap<PolicyRuntime>>,
    /// Ollama model name (overridable via `OLLAMA_MODEL`).
    pub ollama_model: String,
    /// Swappable subsystem implementations (selected by `Role.plugin_backends`).
    pub plugins: PluginHost,
    /// Directory plugins (`plugins/*/manifest.json`) scan and lazy start.
    pub directory_plugins: Arc<DirectoryPluginRuntime>,
    /// MCP transport / directory plugin subprocess etc. high-risk capability grants (`high_risk_grants.json`).
    pub high_risk_grants: Arc<HighRiskGrantStore>,
    /// Per-session (`srid`) mutex: serializes concurrent turns on the same namespace (`--api` / parallel invoke).
    turn_locks: DashMap<String, TurnLockEntry>,
    /// Startup self-check cache (success is permanent; failures retry per role with TTL).
    pub(crate) startup_health: parking_lot::RwLock<StartupHealthCache>,
    /// Read `OCLIVE_REMOTE_FALLBACK_TO_BUILTIN` once at startup; `sync_remote_fallback_from_db_value` does not re-read env.
    remote_fallback_env_override: Option<bool>,
    /// Whether remote HTTP plugin failure may silently fall back to builtin (aligned with `app_settings.remote_fallback_to_builtin` and `OCLIVE_REMOTE_FALLBACK_TO_BUILTIN`).
    pub remote_fallback_allowed: Arc<AtomicBool>,
    policy_file_applied: AtomicBool,
    /// In-process cache of `user_llm_provider` app_setting (`cloud` / `local` / empty).
    pub(crate) user_llm_provider: parking_lot::RwLock<String>,
    /// LLM env change version; incremented on each settings write.
    pub(crate) user_llm_env_version: AtomicU64,
    /// Version last successfully applied to process env.
    pub(crate) user_llm_env_applied_version: AtomicU64,
    /// Fast dirty flag: skip redundant `apply_user_llm_env` when unchanged.
    pub(crate) user_llm_env_dirty: AtomicBool,
    /// Distro capability profile (P4); loaded once at kernel startup from env.
    pub host_profile: Arc<crate::domain::host_profile::HostProfile>,
    /// Remote/directory reply post-processor wiring (infrastructure only).
    pub(crate) reply_post_processor_resolver:
        Arc<dyn oclive_kernel_contracts::ReplyPostProcessorResolver>,
}

impl AppState {
    /// # Errors
    ///
    /// Returns [`Err`] with a human-readable message when the operation fails.
    /// `roles_dir_override`: bundled app passes `resource_dir/roles`; when `None`, use [`resolve_roles_dir`].
    /// `app_data_dir`: app data root (same level as SQLite), used for `oclive_host_plugins.json` and `plugins/` scan root.
    pub async fn new(
        db_path: impl AsRef<Path>,
        roles_dir_override: Option<PathBuf>,
        app_data_dir: impl AsRef<Path>,
    ) -> Result<Self> {
        AppStateBuilder::production(
            db_path,
            roles_dir_override.unwrap_or_else(|| resolve_roles_dir(None)),
            app_data_dir,
        )
        .build()
        .await
    }
    /// # Errors
    ///
    /// Returns [`Err`] with a human-readable message when the operation fails.
    /// In-memory DB + injected LLM (integration tests / no Ollama).
    pub async fn new_in_memory_with_llm(
        llm: Arc<dyn LlmClient>,
        roles_dir: impl AsRef<Path>,
    ) -> Result<Self> {
        Self::new_in_memory_with_llm_and_policy_file(llm, roles_dir, None).await
    }
    /// # Errors
    ///
    /// Returns [`Err`] with a human-readable message when the operation fails.
    pub async fn new_in_memory_with_llm_and_policy_file(
        llm: Arc<dyn LlmClient>,
        roles_dir: impl AsRef<Path>,
        policy_file: Option<&Path>,
    ) -> Result<Self> {
        AppStateBuilder::in_memory_test(llm, roles_dir, policy_file)
            .with_host_profile(crate::domain::host_profile::load_host_profile_from_env())
            .build()
            .await
    }

    /// Align in-process switch with `app_settings.remote_fallback_to_builtin` and `OCLIVE_REMOTE_FALLBACK_TO_BUILTIN`.
    pub fn sync_remote_fallback_from_db_value(&self, raw: Option<String>) {
        let mut allowed = remote_fallback_from_db_value(raw);
        if let Some(v) = self.remote_fallback_env_override {
            allowed = v;
        }
        self.remote_fallback_allowed
            .store(allowed, Ordering::Relaxed);
    }

    pub fn mark_user_llm_env_dirty(&self) {
        self.user_llm_env_version.fetch_add(1, Ordering::AcqRel);
        self.user_llm_env_dirty.store(true, Ordering::Release);
    }

    pub fn policies_for_scene(&self, scene_id: Option<&str>) -> Arc<PolicySet> {
        let runtime = self.policy_runtime.load_full();
        scene_id
            .and_then(|s| runtime.scene_policy_sets.get(s).cloned())
            .unwrap_or_else(|| runtime.default_policy_set.clone())
    }

    /// Scene memory policy for the current turn (domain port; impl in infrastructure).
    #[must_use]
    pub fn turn_policies_for_scene(
        &self,
        scene_id: Option<&str>,
    ) -> crate::domain::ports::turn_policies::TurnPolicies {
        use crate::domain::ports::turn_policies::TurnPoliciesPort;
        crate::infrastructure::turn_ports::AppTurnPoliciesPort::new(self)
            .policies_for_scene(scene_id)
    }

    #[must_use]
    pub fn chat_turn_persistence_port(
        &self,
    ) -> Arc<dyn crate::domain::ports::turn_persistence::ChatTurnPersistencePort> {
        Arc::new(crate::infrastructure::turn_ports::DbChatTurnPersistencePort::new(
            Arc::clone(&self.db_manager),
        ))
    }

    #[must_use]
    pub fn conversation_persist_port(
        &self,
    ) -> Arc<dyn crate::domain::ports::conversation_persist::ConversationPersistPort> {
        Arc::new(
            crate::infrastructure::turn_ports::StoreConversationPersistPort::new(Arc::clone(
                &self.conversation_store,
            )),
        )
    }

    pub fn scene_policy_count(&self) -> usize {
        self.policy_runtime.load_full().scene_policy_sets.len()
    }

    /// # Errors
    ///
    /// Returns [`Err`] with a human-readable message when the operation fails.
    pub fn reload_policy_plugins(&self) -> Result<usize> {
        let path = Path::new("./config/policy.toml");
        let registry = load_policy_registry_from_path(path, true)?;
        let runtime = build_policy_sets_from_registry(registry);
        let count = runtime.scene_policy_sets.len();
        self.policy_runtime.store(Arc::new(runtime));
        self.policy_file_applied
            .store(true, std::sync::atomic::Ordering::Release);
        tracing::info!(
            "policy plugins reloaded path={} scene_count={}",
            path.display(),
            count
        );
        Ok(count)
    }
    /// # Errors
    ///
    /// Returns [`Err`] with a human-readable message when the operation fails.
    /// Prefer [`Self::role_cache`] (same as [`crate::domain::chat_engine`]); on miss load from disk and cache.
    ///
    /// Async role load: disk I/O runs in `spawn_blocking` to avoid blocking the tokio reactor.
    ///
    /// # Errors
    ///
    /// Returns [`Err`] when role is missing, disk I/O fails, etc.
    pub async fn load_role_cached_async(&self, role_id: &str) -> Result<Arc<Role>> {
        {
            let mut map = self.role_cache.write();
            if let Some(r) = map.swap_remove(role_id) {
                let loaded = Arc::clone(&r);
                map.insert(role_id.to_string(), r);
                return Ok(loaded);
            }
        }
        if let Some(r) = self.http_api_roles.get(role_id) {
            let loaded = Arc::clone(r.value());
            self.insert_role_cache(role_id, &loaded);
            return Ok(loaded);
        }

        let cell = self
            .role_load_inflight
            .entry(role_id.to_string())
            .or_insert_with(|| Arc::new(OnceCell::new()))
            .clone();

        let storage = self.storage.clone();
        let role_id_owned = role_id.to_string();
        let loaded = match cell
            .get_or_try_init(|| async move {
                let role = tokio::task::spawn_blocking(move || storage.load_role(&role_id_owned))
                    .await
                    .map_err(|e| {
                        crate::error::AppError::Unknown(format!("load_role task failed: {e}"))
                    })??;
                Ok::<Arc<Role>, crate::error::AppError>(Arc::new(role))
            })
            .await
        {
            Ok(v) => v,
            Err(e) => {
                self.role_load_inflight.remove(role_id);
                return Err(e);
            }
        };

        self.insert_role_cache(role_id, loaded);
        self.role_load_inflight.remove(role_id);
        Ok(Arc::clone(loaded))
    }

    /// Insert or refresh an HTTP `--api` trial-chat role; evicts oldest when at [`ROLE_CACHE_CAPACITY`].
    pub(crate) fn insert_http_api_role(&self, role_id: String, role: Arc<Role>) {
        let mut order = self.http_api_roles_order.write();
        if order.contains_key(&role_id) {
            order.swap_remove(&role_id);
            order.insert(role_id.clone(), ());
        } else {
            while order.len() >= ROLE_CACHE_CAPACITY {
                if let Some((oldest, _)) = order.swap_remove_index(0) {
                    self.http_api_roles.remove(&oldest);
                } else {
                    break;
                }
            }
            order.insert(role_id.clone(), ());
        }
        drop(order);
        self.http_api_roles.insert(role_id, role);
    }

    fn insert_role_cache(&self, role_id: &str, role: &Arc<Role>) {
        let mut map = self.role_cache.write();
        let key = role_id.to_string();
        if let Some(existing) = map.swap_remove(&key) {
            map.insert(key, existing);
            return;
        }
        while map.len() >= ROLE_CACHE_CAPACITY {
            if map.is_empty() {
                break;
            }
            map.shift_remove_index(0);
        }
        map.insert(key, Arc::clone(role));
    }

    /// Drop effective personality cache for this manifest role and its trial-chat session namespaces (required after disk pack reload or `default_personality` / bounds change).
    pub fn invalidate_personality_cache_for_role(&self, manifest_role_id: &str) {
        let cache = self.session_cache.personality_cache();
        cache.remove(manifest_role_id);
        let prefix = format!("{}__sess__", manifest_role_id);
        cache.retain(|k| !k.starts_with(&prefix));
    }

    /// Drop in-memory `Role` cache (call after `pipeline.ocblueprint` is written to disk).
    pub fn invalidate_role_cache(&self, role_id: &str) {
        if let Some(role) = self.role_cache.read().get(role_id) {
            role.scene_config_cache.write().clear();
            role.scene_text_cache.write().clear();
        }
        self.role_cache.write().remove(role_id);
        self.role_load_inflight.remove(role_id);
        self.invalidate_personality_cache_for_role(role_id);
    }
    /// # Errors
    ///
    /// Returns [`Err`] with a human-readable message when the operation fails.
    /// Current effective personality: `vector` mode is `default_personality` + `delta`; `profile` mode derives seven dims from core profile + DB mutable profile.
    pub async fn get_current_personality(
        &self,
        role_id: &str,
        role: &Role,
    ) -> Result<PersonalityVector> {
        if let Some(p) = self.session_cache.personality_cache().get(role_id) {
            return Ok(p.clone());
        }
        let effective = if role.evolution_config.personality_source == PersonalitySource::Profile {
            let mutable = self.db_manager.get_mutable_personality(role_id).await?;
            crate::domain::profile_personality::effective_vector_from_profile(role, &mutable)
        } else {
            let (_, delta_s) = self
                .db_manager
                .get_core_delta_personality_json(role_id)
                .await?;
            let delta_v = delta_s
                .and_then(|s| PersonalityVector::from_json_vec(&s).ok())
                .unwrap_or_else(PersonalityVector::zero);
            PersonalityVector::effective_from_core_delta(
                &role.default_personality,
                &delta_v,
                &role.evolution_bounds,
            )
        };
        self.session_cache
            .personality_cache()
            .set(role_id.to_string(), effective.clone());
        Ok(effective)
    }

    #[must_use]
    pub fn plugin_host_port(&self) -> &dyn PluginHostPort<Resolved = ResolvedRolePlugins> {
        &self.plugins
    }

    /// Call once per conversation, then reuse returned `memory` / `emotion` / `event` / `prompt` / `llm` to avoid re-parsing backend enums.
    pub fn resolved_plugins_for(&self, role: &Role) -> ResolvedRolePlugins {
        self.plugin_host_port().resolve_for_role(role)
    }

    /// Session-level backend resolution: fold six slots from effective `slot_registry` + instance key / six-slot overrides, then bind implementations.
    pub fn resolved_plugins_for_session(
        &self,
        role: &Role,
        session_namespace: Option<&str>,
    ) -> ResolvedRolePlugins {
        let host = self.plugin_host_port();
        let Some(ns) = session_namespace.map(str::trim).filter(|s| !s.is_empty()) else {
            return host.resolve_for_role(role);
        };
        let effective = self.effective_plugin_backends_for_session(role, ns);
        let slot_reg = self.effective_slot_registry_for_session(role, ns);
        crate::domain::chat_engine::plugin_resolve::resolve_plugins_for_session(
            host,
            role,
            Some(ns),
            &effective,
            slot_reg.as_ref(),
        )
    }

    pub fn memory_retrieval_for(&self, role: &Role) -> Arc<dyn MemoryRetrieval> {
        self.plugins
            .memory_retrieval_for_plugin_backends(&role.plugin_backends)
    }

    pub fn user_emotion_analyzer_for(&self, role: &Role) -> Arc<dyn UserEmotionAnalyzer> {
        self.plugins
            .user_emotion_analyzer_for_backends(&role.plugin_backends)
    }

    pub fn event_estimator_for(&self, role: &Role) -> Arc<dyn EventEstimator> {
        self.plugins
            .event_estimator_for_backends(&role.plugin_backends)
    }

    pub fn prompt_assembler_for(&self, role: &Role) -> Arc<dyn PromptAssembler> {
        self.plugins
            .prompt_assembler_for_backends(&role.plugin_backends)
    }

    /// Tests or telemetry: backend set declared by current role pack.
    pub fn plugin_backends_snapshot(&self, role: &Role) -> PluginBackends {
        role.plugin_backends.as_ref().clone()
    }

    /// Serialize all turns for one session namespace (`srid`).
    #[must_use]
    pub fn turn_lock_for(&self, srid: &str) -> Arc<Mutex<()>> {
        let now = turn_lock_now_ms();
        let lock = {
            let entry = self
                .turn_locks
                .entry(srid.to_string())
                .or_insert_with(|| TurnLockEntry {
                    lock: Arc::new(Mutex::new(())),
                    last_touch_ms: AtomicU64::new(now),
                });
            entry.last_touch_ms.store(now, Ordering::Relaxed);
            entry.lock.clone()
        };
        if self.turn_locks.len() > TURN_LOCK_SOFT_CAP {
            self.prune_idle_turn_locks();
        }
        lock
    }

    /// Drop idle `srid` locks with no external `Arc` holders when the map grows too large.
    fn prune_idle_turn_locks(&self) {
        if self.turn_locks.len() <= TURN_LOCK_SOFT_CAP {
            return;
        }
        let mut idle: Vec<(String, u64)> = Vec::new();
        for item in self.turn_locks.iter() {
            if Arc::strong_count(&item.value().lock) == 1 {
                idle.push((
                    item.key().clone(),
                    item.value().last_touch_ms.load(Ordering::Relaxed),
                ));
            }
        }
        if idle.is_empty() {
            return;
        }
        idle.sort_by_key(|(_, touched)| *touched);
        let remove_n = self.turn_locks.len().saturating_sub(TURN_LOCK_TARGET);
        for (key, _) in idle.into_iter().take(remove_n) {
            self.turn_locks.remove(&key);
        }
        let active: std::collections::HashSet<String> =
            self.turn_locks.iter().map(|e| e.key().clone()).collect();
        self.session_cache
            .prune_sessions_without_active_turns(&active);
    }

    /// # Errors
    ///
    /// Returns [`Err`] with a human-readable message when the operation fails.
    pub fn register_local_plugin_provider(
        &self,
        descriptor: LocalPluginProviderDescriptor,
    ) -> std::result::Result<(), crate::domain::plugin_host::PluginHostError> {
        self.plugins.register_local_provider(descriptor)
    }

    #[must_use]
    pub fn local_plugin_providers(
        &self,
        capability: LocalPluginCapability,
    ) -> Vec<Arc<LocalPluginProviderDescriptor>> {
        self.plugins.local_providers_for(capability)
    }

    #[must_use]
    pub fn local_plugin_all_providers(&self) -> Vec<Arc<LocalPluginProviderDescriptor>> {
        self.plugins.local_all_providers()
    }

    pub(crate) fn bootstrap_local_plugin_providers(plugins: &PluginHost, roles_dir: &Path) {
        let manifest_dir = roles_dir.join("_local_plugins");
        let bridge = FileManifestLocalPluginBridge::new(&manifest_dir);
        let discovered = bridge.discover_providers();
        if discovered.is_empty() {
            return;
        }
        let mut registered = 0usize;
        for desc in discovered {
            match plugins.register_local_provider(desc.clone()) {
                Ok(()) => registered += 1,
                Err(e) => tracing::warn!(
                    target: "oclive_plugin",
                    "local plugin register failed provider_id={} bridge={} err={}",
                    desc.provider_id,
                    bridge.bridge_name(),
                    e
                ),
            }
        }
        tracing::info!(
            target: "oclive_plugin",
            "local plugin bootstrap bridge={} dir={} registered={}",
            bridge.bridge_name(),
            bridge.manifest_dir().display(),
            registered
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::PersonalityVector;
    use std::fs;
    use tempfile::TempDir;

    #[tokio::test]
    async fn invalidate_personality_cache_for_role_clears_manifest_and_sess_keys() {
        let state = AppState::new_in_memory_with_llm(
            Arc::new(crate::infrastructure::llm::MockLlmClient {
                reply: "ok".to_string(),
            }),
            "./roles",
        )
        .await
        .expect("state");
        let c = state.session_cache.personality_cache();
        c.set("r1".to_string(), PersonalityVector::zero());
        c.set("r1__sess__abc".to_string(), PersonalityVector::zero());
        c.set("r2".to_string(), PersonalityVector::zero());
        state.invalidate_personality_cache_for_role("r1");
        assert!(c.get("r1").is_none());
        assert!(c.get("r1__sess__abc").is_none());
        assert!(c.get("r2").is_some());
    }

    #[tokio::test]
    async fn test_app_state_creation() {
        let tmp = TempDir::new().expect("temp");
        let state = AppState::new(":memory:", None, tmp.path()).await;
        assert!(state.is_ok());
    }

    #[tokio::test]
    async fn test_scene_policy_fallback_works() {
        let state = AppState::new_in_memory_with_llm(
            Arc::new(crate::infrastructure::llm::MockLlmClient {
                reply: "ok".to_string(),
            }),
            "./roles",
        )
        .await
        .expect("state should build");
        let default_ptr = Arc::as_ptr(&state.policies_for_scene(None)) as usize;
        let scene_ptr = Arc::as_ptr(&state.policies_for_scene(Some("unknown_scene"))) as usize;
        assert_eq!(default_ptr, scene_ptr);
    }

    #[tokio::test]
    async fn bootstrap_local_plugins_from_manifest_dir() {
        let dir = TempDir::new().expect("temp");
        let local_dir = dir.path().join("_local_plugins");
        fs::create_dir_all(&local_dir).expect("mkdir");
        fs::write(
            local_dir.join("demo.json"),
            r#"{
  "provider_id": "demo.local",
  "schema_version": 1,
  "capabilities": ["memory"]
}"#,
        )
        .expect("write");

        let state = AppState::new_in_memory_with_llm(
            Arc::new(crate::infrastructure::llm::MockLlmClient {
                reply: "ok".to_string(),
            }),
            dir.path(),
        )
        .await
        .expect("state should build");

        let providers = state.local_plugin_providers(LocalPluginCapability::Memory);
        assert_eq!(providers.len(), 1);
        assert_eq!(providers[0].provider_id, "demo.local");
    }

    #[tokio::test]
    async fn turn_lock_prune_preserves_arc_with_external_holder() {
        let state = AppState::new_in_memory_with_llm(
            Arc::new(crate::infrastructure::llm::MockLlmClient {
                reply: "ok".to_string(),
            }),
            "./roles",
        )
        .await
        .expect("state should build");

        let held_srid = "held-session";
        let held_lock = state.turn_lock_for(held_srid);

        for i in 0..=TURN_LOCK_SOFT_CAP {
            let srid = format!("session-{i}");
            if srid != held_srid {
                let _ = state.turn_lock_for(&srid);
            }
        }

        let lock_after = state.turn_lock_for(held_srid);
        assert!(
            Arc::ptr_eq(&held_lock, &lock_after),
            "prune must not drop a turn lock while an external Arc holder exists"
        );
    }
}
