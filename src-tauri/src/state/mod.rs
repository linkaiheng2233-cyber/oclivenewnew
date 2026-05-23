use crate::domain::plugin_host::{PluginHost, ResolvedRolePlugins};
use crate::domain::ports::PluginHostPort;
use crate::domain::repository::{FavorabilityRepository, MemoryRepository};
use crate::domain::{
    EventEstimator, FileManifestLocalPluginBridge, LocalPluginBridge, LocalPluginCapability,
    LocalPluginProviderDescriptor, MemoryRetrieval, PromptAssembler, UserEmotionAnalyzer,
};
use crate::error::Result;
use crate::infrastructure::db::DbManager;
use crate::infrastructure::directory_plugins::DirectoryPluginRuntime;
use crate::infrastructure::high_risk_grants::HighRiskGrantStore;
use crate::infrastructure::llm::LlmClient;
use crate::infrastructure::policy_registry::{
    build_policy_sets_from_registry, load_policy_registry_from_path, PolicyRuntime, PolicySet,
};
use crate::infrastructure::remote_fallback_policy::{
    remote_fallback_env_override, remote_fallback_from_db_value,
};
use crate::infrastructure::storage::RoleStorage;
use crate::models::{
    PersonalitySource, PersonalityVector, PluginBackends, Role,
};
use arc_swap::ArcSwap;
use dashmap::DashMap;
use parking_lot::{Mutex, RwLock};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

mod app_state_builder;
mod roles_dir;
mod session_backends;
mod session_cache;
pub use roles_dir::resolve_roles_dir;
pub use session_cache::SessionCache;

use app_state_builder::AppStateBuilder;

pub struct AppState {
    pub db_manager: Arc<DbManager>,
    pub memory_repo: Arc<dyn MemoryRepository>,
    pub favorability_repo: Arc<dyn FavorabilityRepository>,
    pub llm: Arc<dyn LlmClient>,
    pub role_cache: Arc<RwLock<HashMap<String, Arc<Role>>>>,
    /// 同一 `role_id` 冷加载串行化；表项在无人再持有对应 `Arc` 时移除（见 [`AppState::load_role_cached`]）。
    role_load_inflight: DashMap<String, Arc<Mutex<()>>>,
    pub session_cache: Arc<SessionCache>,
    pub storage: RoleStorage,
    policy_runtime: Arc<ArcSwap<PolicyRuntime>>,
    /// Ollama 模型名（可用环境变量 `OLLAMA_MODEL` 覆盖）
    pub ollama_model: String,
    /// 可替换子系统实现（按 `Role.plugin_backends` 选择）
    pub plugins: PluginHost,
    /// 目录式插件（`plugins/*/manifest.json`）扫描与懒启动。
    pub directory_plugins: Arc<DirectoryPluginRuntime>,
    /// MCP 传输 / 目录插件子进程等高风险能力授权（`high_risk_grants.json`）。
    pub high_risk_grants: Arc<HighRiskGrantStore>,
    /// 首轮 `process_message` 启动自检结果（致命错误缓存，后续请求直接短路）。
    pub(crate) startup_health: Mutex<Option<std::result::Result<(), String>>>,
    /// 远端 HTTP 插件失败时是否允许静默降级内置（与 `app_settings.remote_fallback_to_builtin` 及 `OCLIVE_REMOTE_FALLBACK_TO_BUILTIN` 对齐）。
    pub remote_fallback_allowed: Arc<AtomicBool>,
    policy_file_applied: AtomicBool,
}

impl AppState {
    /// # Errors
    ///
    /// Returns [`Err`] with a human-readable message when the operation fails.
    /// `roles_dir_override`：打包应用传入 `resource_dir/roles`；`None` 时用 [`resolve_roles_dir`]。
    /// `app_data_dir`：应用数据目录（与 SQLite 同级），用于 `oclive_host_plugins.json` 与 `plugins/` 扫描根之一。
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
    /// 内存库 + 注入 LLM（集成测试 / 不连 Ollama）
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
            .build()
            .await
    }

    /// 与 `app_settings.remote_fallback_to_builtin` 及 `OCLIVE_REMOTE_FALLBACK_TO_BUILTIN` 对齐进程内开关。
    pub fn sync_remote_fallback_from_db_value(&self, raw: Option<String>) {
        let mut allowed = remote_fallback_from_db_value(raw);
        if let Some(v) = remote_fallback_env_override() {
            allowed = v;
        }
        self.remote_fallback_allowed
            .store(allowed, Ordering::Relaxed);
    }

    pub fn policies_for_scene(&self, scene_id: Option<&str>) -> Arc<PolicySet> {
        let runtime = self.policy_runtime.load_full();
        scene_id
            .and_then(|s| runtime.scene_policy_sets.get(s).cloned())
            .unwrap_or_else(|| runtime.default_policy_set.clone())
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
    /// 优先使用 [`Self::role_cache`]（与 [`crate::domain::chat_engine`] 一致）；未命中时从磁盘加载并写入缓存。
    ///
    /// 同一 `role_id` 在 [`Self::role_load_inflight`] 下串行冷加载；写缓存前再查一次。本线程退出时若已无其它 waiter，从 inflight 表摘掉该键，避免无限增长。
    pub fn load_role_cached(&self, role_id: &str) -> Result<Arc<Role>> {
        if let Some(r) = self.role_cache.read().get(role_id) {
            return Ok(Arc::clone(r));
        }
        let key = role_id.to_string();
        let gate = self
            .role_load_inflight
            .entry(key.clone())
            .or_insert_with(|| Arc::new(Mutex::new(())))
            .clone();
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
        loaded
    }

    /// 异步路径加载角色：磁盘 I/O 在 `spawn_blocking` 中执行，避免阻塞 tokio reactor。
    ///
    /// # Errors
    ///
    /// 与 [`Self::load_role_cached`] 相同（角色不存在、磁盘 I/O 失败等）。
    pub async fn load_role_cached_async(&self, role_id: &str) -> Result<Arc<Role>> {
        if let Some(r) = self.role_cache.read().get(role_id) {
            return Ok(Arc::clone(r));
        }
        let key = role_id.to_string();
        let gate = self
            .role_load_inflight
            .entry(key.clone())
            .or_insert_with(|| Arc::new(Mutex::new(())))
            .clone();

        {
            let _serial = gate.lock();
            if let Some(r) = self.role_cache.read().get(role_id) {
                return Ok(Arc::clone(r));
            }
        }

        let storage = self.storage.clone();
        let role_id_blocking = role_id.to_string();
        let role = tokio::task::spawn_blocking(move || storage.load_role(&role_id_blocking))
            .await
            .map_err(|e| crate::error::AppError::Unknown(format!("load_role task failed: {e}")))?
            ?;

        let _serial = gate.lock();
        if let Some(r) = self.role_cache.read().get(role_id) {
            return Ok(Arc::clone(r));
        }
        let candidate = Arc::new(role);
        let mut map = self.role_cache.write();
        if let Some(r) = map.get(role_id) {
            return Ok(Arc::clone(r));
        }
        map.insert(role_id.to_string(), Arc::clone(&candidate));
        Ok(candidate)
    }

    /// 丢弃该 manifest 角色及其试聊会话命名空间下的有效性格缓存（磁盘包重载、`default_personality` / 边界等已变时必须调用）。
    pub fn invalidate_personality_cache_for_role(&self, manifest_role_id: &str) {
        let cache = self.session_cache.personality_cache();
        cache.remove(manifest_role_id);
        let prefix = format!("{}__sess__", manifest_role_id);
        cache.retain(|k, _| !k.starts_with(&prefix));
    }

    /// 丢弃内存中的 `Role` 缓存（`pipeline.ocblueprint` 写盘后须调用）。
    pub fn invalidate_role_cache(&self, role_id: &str) {
        self.role_cache.write().remove(role_id);
    }
    /// # Errors
    ///
    /// Returns [`Err`] with a human-readable message when the operation fails.
    /// 当前有效性格：`vector` 模式为 `default_personality` + `delta`；`profile` 模式由核心性格档案 + DB「可变性格档案」归纳七维。
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
            .insert(role_id.to_string(), effective.clone());
        Ok(effective)
    }

    #[must_use]
    pub fn plugin_host_port(&self) -> &dyn PluginHostPort<Resolved = ResolvedRolePlugins> {
        &self.plugins
    }

    /// 单次对话内优先调用本方法一次，再复用返回的 `memory` / `emotion` / `event` / `prompt` / `llm`，避免重复解析后端枚举。
    pub fn resolved_plugins_for(&self, role: &Role) -> ResolvedRolePlugins {
        self.plugin_host_port().resolve_for_role(role)
    }

    /// 会话级后端解析：effective `slot_registry` 折叠六槽 + 实例键/六槽覆盖后再绑定实现。
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

    /// 测试或遥测：当前角色包声明的后端集合
    pub fn plugin_backends_snapshot(&self, role: &Role) -> PluginBackends {
        role.plugin_backends.clone()
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
        state
            .session_cache
            .personality_cache()
            .insert("r1".to_string(), PersonalityVector::zero());
        state
            .session_cache
            .personality_cache()
            .insert("r1__sess__abc".to_string(), PersonalityVector::zero());
        state
            .session_cache
            .personality_cache()
            .insert("r2".to_string(), PersonalityVector::zero());
        state.invalidate_personality_cache_for_role("r1");
        let c = state.session_cache.personality_cache();
        assert!(!c.contains_key("r1"));
        assert!(!c.contains_key("r1__sess__abc"));
        assert!(c.contains_key("r2"));
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
}
