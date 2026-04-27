use crate::domain::event_estimator::EventEstimator;
use crate::domain::local_plugin_bridge::{
    FileManifestLocalPluginBridge, LocalPluginBridge, LocalPluginCapability,
    LocalPluginProviderDescriptor,
};
use crate::domain::memory_retrieval::MemoryRetrieval;
use crate::domain::plugin_host::{PluginHost, ResolvedRolePlugins};
use crate::domain::policy::{
    DefaultEmotionPolicy, DefaultEventPolicy, DefaultMemoryPolicy, EmotionPolicy,
    EmotionPolicyConfig, EventPolicy, MemoryPolicy, MemoryPolicyConfig, PolicyConfig,
};
use crate::domain::prompt_assembler::PromptAssembler;
use crate::domain::repository::{FavorabilityRepository, MemoryRepository};
use crate::domain::user_emotion_analyzer::UserEmotionAnalyzer;
use crate::error::Result;
use crate::infrastructure::db::DbManager;
use crate::infrastructure::directory_plugins::DirectoryPluginRuntime;
use crate::infrastructure::llm::ollama_llm;
use crate::infrastructure::llm::LlmClient;
use crate::infrastructure::ollama_client::OllamaClient;
use crate::infrastructure::repositories_runtime::{
    SqliteFavorabilityRepository, SqliteMemoryRepository,
};
use crate::infrastructure::storage::RoleStorage;
use crate::models::{
    LlmBackend, PersonalitySource, PersonalityVector, PluginBackendSource, PluginBackends,
    PluginBackendsOverride, PluginBackendsSourceMap, Role,
};
use crate::state::resolve_roles_dir;
use parking_lot::{Mutex, RwLock};
use serde::Deserialize;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tempfile::TempDir;

/// 与 oclive-launcher 注入的取值一致：`ollama` / `remote` / `directory`（大小写不敏感）。
fn resolve_llm_backend_env_override() -> Option<LlmBackend> {
    let Ok(v) = std::env::var("OCLIVE_LLM_BACKEND") else {
        return None;
    };
    let t = v.trim();
    if t.is_empty() {
        return None;
    }
    if t.eq_ignore_ascii_case("ollama") {
        Some(LlmBackend::Ollama)
    } else if t.eq_ignore_ascii_case("remote") {
        Some(LlmBackend::Remote)
    } else if t.eq_ignore_ascii_case("directory") {
        Some(LlmBackend::Directory)
    } else {
        None
    }
}

pub struct PolicySet {
    pub emotion: Arc<dyn EmotionPolicy>,
    pub event: Arc<dyn EventPolicy>,
    pub memory: Arc<dyn MemoryPolicy>,
}

struct PolicyRuntime {
    default_policy_set: Arc<PolicySet>,
    scene_policy_sets: HashMap<String, Arc<PolicySet>>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(default, deny_unknown_fields)]
struct PolicyRegistryFile {
    default: PolicyConfig,
    default_profile: String,
    profiles: HashMap<String, PolicyConfig>,
    scene_bindings: HashMap<String, String>,
}

impl PolicyRegistryFile {
    fn with_defaults() -> Self {
        let mut profiles = HashMap::new();
        profiles.insert("default".to_string(), PolicyConfig::default());
        Self {
            default: PolicyConfig::default(),
            default_profile: "default".to_string(),
            profiles,
            scene_bindings: HashMap::new(),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum PolicyFileSchema {
    Registry(PolicyRegistryFile),
    Legacy(PolicyConfig),
}

fn env_bool(key: &str, default: bool) -> bool {
    std::env::var(key)
        .ok()
        .and_then(|v| match v.trim().to_ascii_lowercase().as_str() {
            "1" | "true" | "yes" | "on" => Some(true),
            "0" | "false" | "no" | "off" => Some(false),
            _ => None,
        })
        .unwrap_or(default)
}

fn env_f64(key: &str, default: f64) -> f64 {
    std::env::var(key)
        .ok()
        .and_then(|v| v.trim().parse::<f64>().ok())
        .unwrap_or(default)
}

fn env_i32(key: &str, default: i32) -> i32 {
    std::env::var(key)
        .ok()
        .and_then(|v| v.trim().parse::<i32>().ok())
        .unwrap_or(default)
}

fn apply_policy_config_env_overrides(config: &mut PolicyConfig) {
    config.emotion = EmotionPolicyConfig {
        neutral_hold_enabled: env_bool(
            "POLICY_EMOTION_NEUTRAL_HOLD_ENABLED",
            config.emotion.neutral_hold_enabled,
        ),
        low_confidence_hold_threshold: env_f64(
            "POLICY_EMOTION_LOW_CONFIDENCE_HOLD_THRESHOLD",
            config.emotion.low_confidence_hold_threshold,
        ),
    };
    config.memory = MemoryPolicyConfig {
        ignore_single_char_filter: env_bool(
            "POLICY_MEMORY_IGNORE_SINGLE_CHAR_FILTER",
            config.memory.ignore_single_char_filter,
        ),
        default_importance: env_f64(
            "POLICY_MEMORY_DEFAULT_IMPORTANCE",
            config.memory.default_importance,
        ),
        fifo_limit: env_i32("POLICY_MEMORY_FIFO_LIMIT", config.memory.fifo_limit),
    };
}

fn load_policy_registry_from_path(path: &Path, strict: bool) -> Result<PolicyRegistryFile> {
    let mut registry = if path.exists() {
        let content = fs::read_to_string(path).map_err(crate::error::AppError::IoError)?;
        match toml::from_str::<PolicyFileSchema>(&content) {
            Ok(PolicyFileSchema::Registry(parsed)) => {
                log::info!("policy config loaded source=file path={}", path.display());
                parsed
            }
            Ok(PolicyFileSchema::Legacy(legacy)) => {
                log::info!(
                    "policy config loaded as legacy source=file path={}",
                    path.display()
                );
                let mut r = PolicyRegistryFile::with_defaults();
                r.profiles.insert("default".to_string(), legacy);
                r
            }
            Err(err) => {
                if strict {
                    return Err(crate::error::AppError::InvalidParameter(format!(
                        "invalid policy.toml: {}",
                        err
                    )));
                }
                log::warn!(
                    "policy config parse failed source=file path={} err={}",
                    path.display(),
                    err
                );
                PolicyRegistryFile::with_defaults()
            }
        }
    } else if strict {
        return Err(crate::error::AppError::InvalidParameter(format!(
            "policy file not found: {}",
            path.display()
        )));
    } else {
        PolicyRegistryFile::with_defaults()
    };
    if let Some(default_cfg) = registry.profiles.get_mut(&registry.default_profile) {
        apply_policy_config_env_overrides(default_cfg);
    } else {
        let mut fallback = registry.default.clone();
        apply_policy_config_env_overrides(&mut fallback);
        registry
            .profiles
            .insert(registry.default_profile.clone(), fallback);
    }
    Ok(registry)
}

fn load_policy_registry() -> PolicyRegistryFile {
    let path = Path::new("./config/policy.toml");
    load_policy_registry_from_path(path, false)
        .unwrap_or_else(|_| PolicyRegistryFile::with_defaults())
}

fn build_policy_set(config: &PolicyConfig) -> Arc<PolicySet> {
    Arc::new(PolicySet {
        emotion: Arc::new(DefaultEmotionPolicy::new(config.emotion.clone())),
        event: Arc::new(DefaultEventPolicy),
        memory: Arc::new(DefaultMemoryPolicy::new(config.memory.clone())),
    })
}

pub struct KernelAppState {
    pub db_manager: Arc<DbManager>,
    pub memory_repo: Arc<dyn MemoryRepository>,
    pub favorability_repo: Arc<dyn FavorabilityRepository>,
    pub llm: Arc<dyn LlmClient>,
    pub role_cache: Arc<RwLock<HashMap<String, Arc<Role>>>>,
    role_load_inflight: Mutex<HashMap<String, Arc<Mutex<()>>>>,
    pub personality_cache: Arc<RwLock<HashMap<String, PersonalityVector>>>,
    pub storage: RoleStorage,
    policy_runtime: Arc<RwLock<PolicyRuntime>>,
    pub ollama_model: String,
    pub plugins: PluginHost,
    pub directory_plugins: Arc<DirectoryPluginRuntime>,
    session_plugin_overrides: Arc<RwLock<HashMap<String, PluginBackendsOverride>>>,
    // Keep temp dir alive when using ephemeral DBs.
    _temp_db_dir: Option<TempDir>,
}

impl KernelAppState {
    fn build_policy_sets_from_registry(registry: PolicyRegistryFile) -> PolicyRuntime {
        let default_cfg = registry
            .profiles
            .get(&registry.default_profile)
            .cloned()
            .unwrap_or_default();
        let default_policy_set = build_policy_set(&default_cfg);
        let mut scene_policy_sets: HashMap<String, Arc<PolicySet>> = HashMap::new();
        for (scene, profile) in &registry.scene_bindings {
            let cfg = registry
                .profiles
                .get(profile)
                .cloned()
                .unwrap_or_else(|| default_cfg.clone());
            scene_policy_sets.insert(scene.clone(), build_policy_set(&cfg));
        }
        PolicyRuntime {
            default_policy_set,
            scene_policy_sets,
        }
    }

    pub async fn new(
        db_path: impl AsRef<Path>,
        roles_dir_override: Option<PathBuf>,
        app_data_dir: impl AsRef<Path>,
    ) -> Result<Self> {
        let path = db_path.as_ref();
        let (db, temp_db_dir) = if path == Path::new(":memory:") {
            // Use a per-instance temp file DB to avoid concurrency races on `_sqlx_migrations`
            // across async tests and pooled connections.
            let dir = TempDir::new().map_err(|e| crate::error::AppError::DatabaseError(e.to_string()))?;
            let db_file = dir.path().join("kernel_runtime.sqlite");
            let opts = SqliteConnectOptions::new()
                .filename(&db_file)
                .create_if_missing(true);
            let pool = SqlitePoolOptions::new()
                .max_connections(5)
                .connect_with(opts)
                .await
                .map_err(|e| crate::error::AppError::DatabaseError(e.to_string()))?;
            (pool, Some(dir))
        } else {
            if let Some(parent) = path.parent() {
                if !parent.as_os_str().is_empty() {
                    fs::create_dir_all(parent)?;
                }
            }
            let opts = SqliteConnectOptions::new()
                .filename(path)
                .create_if_missing(true);
            let pool = SqlitePoolOptions::new()
                .max_connections(5)
                .connect_with(opts)
                .await
                .map_err(|e| crate::error::AppError::DatabaseError(e.to_string()))?;
            (pool, None)
        };

        // NOTE: temporary: reuse existing SQL migrations under `src-tauri/migrations`.
        sqlx::migrate!("../../src-tauri/migrations")
            .run(&db)
            .await
            .map_err(|e| crate::error::AppError::DatabaseError(e.to_string()))?;

        let db_manager = Arc::new(DbManager::new(db.clone()));

        let memory_repo: Arc<dyn MemoryRepository> =
            Arc::new(SqliteMemoryRepository::new(db_manager.clone()));
        let favorability_repo: Arc<dyn FavorabilityRepository> =
            Arc::new(SqliteFavorabilityRepository::new(db_manager.clone()));

        let ollama = OllamaClient::new(
            std::env::var("OLLAMA_BASE_URL")
                .unwrap_or_else(|_| "http://localhost:11434".to_string()),
        );
        let llm = ollama_llm(ollama);

        let ollama_model =
            std::env::var("OLLAMA_MODEL").unwrap_or_else(|_| "qwen2.5:7b".to_string());
        let registry = load_policy_registry();
        let runtime = Self::build_policy_sets_from_registry(registry);

        let storage = RoleStorage::new(roles_dir_override.unwrap_or_else(resolve_roles_dir));
        let directory_plugins =
            DirectoryPluginRuntime::bootstrap(storage.roles_dir(), app_data_dir.as_ref());
        let plugins = PluginHost::new(
            db_manager.clone(),
            llm.clone(),
            Some(directory_plugins.clone()),
            app_data_dir.as_ref().to_path_buf(),
        );
        Self::bootstrap_local_plugin_providers(&plugins, storage.roles_dir());

        Ok(Self {
            db_manager,
            memory_repo,
            favorability_repo,
            llm,
            role_cache: Arc::new(RwLock::new(HashMap::new())),
            role_load_inflight: Mutex::new(HashMap::new()),
            personality_cache: Arc::new(RwLock::new(HashMap::new())),
            storage,
            policy_runtime: Arc::new(RwLock::new(runtime)),
            ollama_model,
            plugins,
            directory_plugins,
            session_plugin_overrides: Arc::new(RwLock::new(HashMap::new())),
            _temp_db_dir: temp_db_dir,
        })
    }

    pub async fn new_in_memory_with_llm(
        llm: Arc<dyn LlmClient>,
        roles_dir: impl AsRef<Path>,
    ) -> Result<Self> {
        Self::new_in_memory_with_llm_and_policy_file(llm, roles_dir, None).await
    }

    pub async fn new_in_memory_with_llm_and_policy_file(
        llm: Arc<dyn LlmClient>,
        roles_dir: impl AsRef<Path>,
        policy_file: Option<&Path>,
    ) -> Result<Self> {
        let dir = TempDir::new().map_err(|e| crate::error::AppError::DatabaseError(e.to_string()))?;
        let db_file = dir.path().join("kernel_runtime.sqlite");
        let opts = SqliteConnectOptions::new()
            .filename(&db_file)
            .create_if_missing(true);
        let db = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(opts)
            .await
            .map_err(|e| crate::error::AppError::DatabaseError(e.to_string()))?;

        sqlx::migrate!("../../src-tauri/migrations")
            .run(&db)
            .await
            .map_err(|e| crate::error::AppError::DatabaseError(e.to_string()))?;

        let db_manager = Arc::new(DbManager::new(db));

        let memory_repo: Arc<dyn MemoryRepository> =
            Arc::new(SqliteMemoryRepository::new(db_manager.clone()));
        let favorability_repo: Arc<dyn FavorabilityRepository> =
            Arc::new(SqliteFavorabilityRepository::new(db_manager.clone()));

        let storage = RoleStorage::new(roles_dir);
        let app_data_dir = storage.roles_dir().join(".oclive_directory_plugin_data");
        let _ = fs::create_dir_all(&app_data_dir);
        let directory_plugins =
            DirectoryPluginRuntime::bootstrap(storage.roles_dir(), &app_data_dir);
        let runtime = if let Some(path) = policy_file {
            let registry = load_policy_registry_from_path(path, false)
                .unwrap_or_else(|_| PolicyRegistryFile::with_defaults());
            Self::build_policy_sets_from_registry(registry)
        } else {
            PolicyRuntime {
                default_policy_set: build_policy_set(&PolicyConfig::default()),
                scene_policy_sets: HashMap::new(),
            }
        };

        let plugins = PluginHost::new(
            db_manager.clone(),
            llm.clone(),
            Some(directory_plugins.clone()),
            app_data_dir.clone(),
        );
        Self::bootstrap_local_plugin_providers(&plugins, storage.roles_dir());

        Ok(Self {
            db_manager,
            memory_repo,
            favorability_repo,
            llm,
            role_cache: Arc::new(RwLock::new(HashMap::new())),
            role_load_inflight: Mutex::new(HashMap::new()),
            personality_cache: Arc::new(RwLock::new(HashMap::new())),
            storage,
            policy_runtime: Arc::new(RwLock::new(runtime)),
            ollama_model: "test-model".to_string(),
            plugins,
            directory_plugins,
            session_plugin_overrides: Arc::new(RwLock::new(HashMap::new())),
            _temp_db_dir: Some(dir),
        })
    }

    pub fn policies_for_scene(&self, scene_id: Option<&str>) -> Arc<PolicySet> {
        let runtime = self.policy_runtime.read();
        scene_id
            .and_then(|s| runtime.scene_policy_sets.get(s).cloned())
            .unwrap_or_else(|| runtime.default_policy_set.clone())
    }

    pub fn scene_policy_count(&self) -> usize {
        self.policy_runtime.read().scene_policy_sets.len()
    }

    pub fn reload_policy_plugins(&self) -> Result<usize> {
        let path = Path::new("./config/policy.toml");
        let registry = load_policy_registry_from_path(path, true)?;
        let runtime = Self::build_policy_sets_from_registry(registry);
        let count = runtime.scene_policy_sets.len();
        *self.policy_runtime.write() = runtime;
        log::info!(
            "policy plugins reloaded path={} scene_count={}",
            path.display(),
            count
        );
        Ok(count)
    }

    pub fn load_role_cached(&self, role_id: &str) -> Result<Arc<Role>> {
        if let Some(r) = self.role_cache.read().get(role_id) {
            return Ok(Arc::clone(r));
        }
        let key = role_id.to_string();
        let gate = {
            let mut inflight = self.role_load_inflight.lock();
            inflight
                .entry(key.clone())
                .or_insert_with(|| Arc::new(Mutex::new(())))
                .clone()
        };
        let _serial = gate.lock();

        let loaded = (|| -> Result<Arc<Role>> {
            if let Some(r) = self.role_cache.read().get(role_id) {
                return Ok(Arc::clone(r));
            }
            let role = self.storage.load_role(role_id)?;
            let candidate = Arc::new(role);
            let mut map = self.role_cache.write();
            if let Some(r) = map.get(role_id) {
                return Ok(Arc::clone(r));
            }
            map.insert(role_id.to_string(), Arc::clone(&candidate));
            Ok(candidate)
        })();

        drop(_serial);
        drop(gate);
        let mut inflight = self.role_load_inflight.lock();
        if let Some(e) = inflight.get(&key) {
            if Arc::strong_count(e) == 1 {
                inflight.remove(&key);
            }
        }

        loaded
    }

    pub fn invalidate_personality_cache_for_role(&self, manifest_role_id: &str) {
        let mut cache = self.personality_cache.write();
        cache.remove(manifest_role_id);
        let prefix = format!("{}__sess__", manifest_role_id);
        cache.retain(|k, _| !k.starts_with(&prefix));
    }

    pub async fn get_current_personality(
        &self,
        role_id: &str,
        role: &Role,
    ) -> Result<PersonalityVector> {
        if let Some(p) = self.personality_cache.read().get(role_id) {
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
        self.personality_cache
            .write()
            .insert(role_id.to_string(), effective.clone());
        Ok(effective)
    }

    pub fn resolved_plugins_for(&self, role: &Role) -> ResolvedRolePlugins {
        self.plugins.resolve_for_role(role)
    }

    pub fn resolved_plugins_for_session(
        &self,
        role: &Role,
        session_namespace: Option<&str>,
    ) -> ResolvedRolePlugins {
        let ov = session_namespace.and_then(|ns| self.session_backend_override(ns));
        self.plugins
            .resolve_for_role_with_override(role, ov.as_ref())
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

    pub fn plugin_backends_snapshot(&self, role: &Role) -> PluginBackends {
        role.plugin_backends.clone()
    }

    #[must_use]
    pub fn session_backend_override(
        &self,
        session_namespace: &str,
    ) -> Option<PluginBackendsOverride> {
        self.session_plugin_overrides
            .read()
            .get(session_namespace)
            .cloned()
    }

    pub fn set_session_backend_override(
        &self,
        session_namespace: &str,
        override_backends: PluginBackendsOverride,
    ) {
        if override_backends.is_empty() {
            self.session_plugin_overrides
                .write()
                .remove(session_namespace);
            return;
        }
        self.session_plugin_overrides
            .write()
            .insert(session_namespace.to_string(), override_backends);
    }

    pub fn clear_session_backend_override(&self, session_namespace: &str) {
        self.session_plugin_overrides
            .write()
            .remove(session_namespace);
    }

    #[must_use]
    pub fn effective_plugin_backends_for_session(
        &self,
        role: &Role,
        session_namespace: &str,
    ) -> PluginBackends {
        self.session_backend_override(session_namespace)
            .map(|ov| ov.apply_to(&role.plugin_backends))
            .unwrap_or_else(|| role.plugin_backends.clone())
    }

    #[must_use]
    pub fn effective_plugin_backend_sources_for_session(
        &self,
        session_namespace: &str,
    ) -> PluginBackendsSourceMap {
        let session_ov = self.session_backend_override(session_namespace);
        let mut out = PluginBackendsSourceMap::default();
        if let Some(ov) = session_ov {
            if ov.memory.is_some() || ov.local_memory_provider_id.is_some() {
                out.memory = PluginBackendSource::SessionOverride;
            }
            if ov.emotion.is_some() {
                out.emotion = PluginBackendSource::SessionOverride;
            }
            if ov.event.is_some() {
                out.event = PluginBackendSource::SessionOverride;
            }
            if ov.prompt.is_some() {
                out.prompt = PluginBackendSource::SessionOverride;
            }
            if ov.llm.is_some() {
                out.llm = PluginBackendSource::SessionOverride;
            }
            if ov.agent.is_some() {
                out.agent = PluginBackendSource::SessionOverride;
            }
        }
        if out.llm == PluginBackendSource::PackDefault
            && resolve_llm_backend_env_override().is_some()
        {
            out.llm = PluginBackendSource::EnvOverride;
        }
        out
    }

    pub fn register_local_plugin_provider(
        &self,
        descriptor: LocalPluginProviderDescriptor,
    ) -> std::result::Result<(), String> {
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

    fn bootstrap_local_plugin_providers(plugins: &PluginHost, roles_dir: &Path) {
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
                Err(e) => log::warn!(
                    target: "oclive_plugin",
                    "local plugin register failed provider_id={} bridge={} err={}",
                    desc.provider_id,
                    bridge.bridge_name(),
                    e
                ),
            }
        }
        log::info!(
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
        let state = KernelAppState::new_in_memory_with_llm(
            Arc::new(crate::infrastructure::llm::MockLlmClient {
                reply: "ok".to_string(),
            }),
            "./roles",
        )
        .await
        .expect("state");
        state
            .personality_cache
            .write()
            .insert("r1".to_string(), PersonalityVector::zero());
        state
            .personality_cache
            .write()
            .insert("r1__sess__abc".to_string(), PersonalityVector::zero());
        state
            .personality_cache
            .write()
            .insert("r2".to_string(), PersonalityVector::zero());
        state.invalidate_personality_cache_for_role("r1");
        let c = state.personality_cache.read();
        assert!(!c.contains_key("r1"));
        assert!(!c.contains_key("r1__sess__abc"));
        assert!(c.contains_key("r2"));
    }

    #[tokio::test]
    async fn test_app_state_creation() {
        let tmp = TempDir::new().expect("temp");
        let state = KernelAppState::new(":memory:", None, tmp.path()).await;
        assert!(state.is_ok());
    }

    #[tokio::test]
    async fn test_scene_policy_fallback_works() {
        let state = KernelAppState::new_in_memory_with_llm(
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

        let state = KernelAppState::new_in_memory_with_llm(
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
}
