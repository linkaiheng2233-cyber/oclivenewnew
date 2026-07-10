//! [`AppState`] construction (production DB, in-memory tests).

use super::session_cache::SessionCache;
use super::{AffectSinkHandle, AppState};
use crate::domain::host_profile::{self, HostProfile};
use crate::domain::repository::{FavorabilityRepository, MemoryRepository};
use crate::error::Result;
use crate::infrastructure::chat_storage::{
    build_conversation_store, pick_chat_storage_backend_kind, set_persisted_storage_root,
    ReplayTaskRegistry, APP_SETTING_CHAT_STORAGE_ROOT,
};
use crate::infrastructure::db::DbManager;
use crate::infrastructure::directory_plugins::DirectoryPluginRuntime;
use crate::infrastructure::high_risk_grants::HighRiskGrantStore;
use crate::infrastructure::llm::{LlmClient, SharedOllamaClient};
use crate::infrastructure::ollama_client::OllamaClient;
use crate::infrastructure::policy_registry::{
    build_policy_sets_from_registry, load_policy_registry_from_path, PolicyRegistryFile,
};
use crate::infrastructure::remote_fallback_policy::{
    new_remote_fallback_switch, remote_fallback_env_override, remote_fallback_from_db_value,
};
use crate::infrastructure::repositories::{SqliteFavorabilityRepository, SqliteMemoryRepository};
use crate::infrastructure::sqlite_pool;
use crate::infrastructure::storage::RoleStorage;
use arc_swap::ArcSwap;
use dashmap::DashMap;
use parking_lot::RwLock;
use sqlx::SqlitePool;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64};
use std::sync::Arc;

pub struct AppStateBuilder {
    db_path: PathBuf,
    roles_dir: PathBuf,
    app_data_dir: PathBuf,
    llm: Option<Arc<dyn LlmClient>>,
    #[allow(dead_code)]
    policy_file: Option<PathBuf>,
    ollama_model: Option<String>,
    high_risk_strict: bool,
    use_test_policy_default: bool,
    host_profile: Option<HostProfile>,
}

impl AppStateBuilder {
    pub fn production(
        db_path: impl AsRef<Path>,
        roles_dir: PathBuf,
        app_data_dir: impl AsRef<Path>,
    ) -> Self {
        Self {
            db_path: db_path.as_ref().to_path_buf(),
            roles_dir,
            app_data_dir: app_data_dir.as_ref().to_path_buf(),
            llm: None,
            policy_file: None,
            ollama_model: None,
            high_risk_strict: true,
            use_test_policy_default: false,
            host_profile: None,
        }
    }

    pub fn in_memory_test(
        llm: Arc<dyn LlmClient>,
        roles_dir: impl AsRef<Path>,
        policy_file: Option<&Path>,
    ) -> Self {
        let roles = roles_dir.as_ref().to_path_buf();
        let app_data_dir = roles.join(".oclive_directory_plugin_data");
        Self {
            db_path: PathBuf::from(":memory:"),
            roles_dir: roles,
            app_data_dir,
            llm: Some(llm),
            policy_file: policy_file.map(|p| p.to_path_buf()),
            ollama_model: Some("test-model".to_string()),
            high_risk_strict: false,
            use_test_policy_default: policy_file.is_none(),
            host_profile: None,
        }
    }

    #[must_use]
    pub fn with_host_profile(mut self, profile: HostProfile) -> Self {
        self.host_profile = Some(profile);
        self
    }

    #[must_use]
    pub fn with_app_data_dir(mut self, dir: impl AsRef<Path>) -> Self {
        self.app_data_dir = dir.as_ref().to_path_buf();
        self
    }

    /// # Errors
    ///
    /// Database connect/migrate, policy load, or plugin bootstrap failures.
    pub async fn build(self) -> Result<AppState> {
        let db = connect_db(&self.db_path).await?;
        let backup = if self.db_path != Path::new(":memory:") && self.db_path.is_file() {
            crate::infrastructure::sql_migrate::backup_db_file(&self.db_path, &self.app_data_dir)
                .ok()
        } else {
            None
        };
        if let Err(e) = run_migrations(&db).await {
            if let Some(ref bak) = backup {
                if bak.is_file() {
                    let _ = crate::infrastructure::sql_migrate::restore_db_from_backup(
                        &self.db_path,
                        bak,
                    );
                }
            }
            let _ = crate::infrastructure::sql_migrate::write_migration_failed_marker(
                &self.app_data_dir,
                &e.to_string(),
            );
            return Err(e);
        }

        let db_manager = Arc::new(DbManager::new(db));
        let user_llm_provider = parking_lot::RwLock::new(String::new());
        let remote_fallback_allowed = remote_fallback_switch(&db_manager).await?;

        let memory_repo: Arc<dyn MemoryRepository> =
            Arc::new(SqliteMemoryRepository::new(db_manager.clone()));
        let favorability_repo: Arc<dyn FavorabilityRepository> =
            Arc::new(SqliteFavorabilityRepository::new(db_manager.clone()));

        let (llm, ollama) = match self.llm {
            Some(l) => (l, None),
            None => {
                let client = Arc::new(OllamaClient::new(
                    std::env::var("OLLAMA_BASE_URL")
                        .unwrap_or_else(|_| "http://localhost:11434".to_string()),
                ));
                let llm: Arc<dyn LlmClient> = Arc::new(SharedOllamaClient(Arc::clone(&client)));
                (llm, Some(client))
            }
        };

        let ollama_model = match self.ollama_model {
            Some(m) => m,
            None => {
                let settings = crate::infrastructure::db_ports::DbSettingsPort(db_manager.as_ref());
                crate::domain::user_llm_env::global_ollama_model_from_db_or_env(&settings).await
            }
        };

        let policy_runtime = Arc::new(ArcSwap::from_pointee(build_policy_sets_from_registry(
            PolicyRegistryFile::with_defaults(),
        )));
        let mut policy_file_applied = self.use_test_policy_default;
        if !self.use_test_policy_default {
            let path = self
                .policy_file
                .as_deref()
                .unwrap_or_else(|| Path::new("./config/policy.toml"));
            if path.is_file() {
                let registry = load_policy_registry_from_path(path, true)?;
                policy_runtime.store(Arc::new(build_policy_sets_from_registry(registry)));
                policy_file_applied = true;
            }
        }
        let storage = RoleStorage::new(self.roles_dir);
        let _ = fs::create_dir_all(&self.app_data_dir);
        let host_profile = Arc::new(
            self.host_profile
                .unwrap_or_else(host_profile::load_host_profile_from_env),
        );
        if let Some(parent) = storage.roles_dir().parent() {
            if host_profile::theater_director_enabled(host_profile.as_ref()) {
                crate::infrastructure::theater_director_plugin_seed::seed_official_theater_director_plugin(
                    &self.app_data_dir,
                    Some(&parent.join("plugins")),
                );
            }
        }
        let high_risk_grants =
            HighRiskGrantStore::load(self.app_data_dir.clone(), self.high_risk_strict);
        let directory_plugins = if self.high_risk_strict {
            DirectoryPluginRuntime::bootstrap_deferred_scan(
                storage.roles_dir(),
                &self.app_data_dir,
                high_risk_grants.clone(),
            )
        } else {
            DirectoryPluginRuntime::bootstrap(
                storage.roles_dir(),
                &self.app_data_dir,
                high_risk_grants.clone(),
            )
        };
        let plugins = crate::infrastructure::plugin_wiring::build_plugin_host(
            llm.clone(),
            Some(directory_plugins.clone()),
            self.app_data_dir.clone(),
            high_risk_grants.clone(),
            remote_fallback_allowed.clone(),
        );
        AppState::bootstrap_local_plugin_providers(&plugins, storage.roles_dir());

        let replay_tasks = Arc::new(ReplayTaskRegistry::new());
        if let Ok(Some(raw)) = db_manager
            .get_app_setting(APP_SETTING_CHAT_STORAGE_ROOT)
            .await
        {
            let trimmed = raw.trim();
            if !trimmed.is_empty() {
                set_persisted_storage_root(Some(PathBuf::from(trimmed)));
            }
        }
        let backend_kind = pick_chat_storage_backend_kind(None);
        let default_chat_cfg = crate::models::RolePackChatStorageConfig::default();
        let conversation_store = build_conversation_store(
            backend_kind,
            db_manager.clone(),
            self.app_data_dir.clone(),
            storage.roles_dir().to_path_buf(),
            replay_tasks.clone(),
            &default_chat_cfg,
            None,
        );

        let reply_post_processor_resolver: Arc<
            dyn oclive_kernel_contracts::ReplyPostProcessorResolver,
        > = Arc::new(
            crate::infrastructure::reply_post_processor_wiring::HostReplyPostProcessorResolver::new(
                directory_plugins.clone(),
                remote_fallback_allowed.clone(),
                high_risk_grants.clone(),
            ),
        );

        let theater_director_resolver: Arc<dyn oclive_kernel_contracts::TheaterDirectorResolver> =
            Arc::new(
                crate::infrastructure::theater_director_wiring::HostTheaterDirectorResolver::new(
                    directory_plugins.clone(),
                    remote_fallback_allowed.clone(),
                    high_risk_grants.clone(),
                ),
            );

        let user_llm_secrets: Arc<dyn oclive_kernel_contracts::UserLlmSecretsPort> =
            Arc::new(crate::infrastructure::user_llm_secrets::BuiltinUserLlmSecrets);

        let state = AppState {
            db_manager,
            conversation_store,
            replay_tasks,
            memory_repo,
            favorability_repo,
            user_llm_secrets,
            llm,
            ollama,
            role_cache: Arc::new(RwLock::new(indexmap::IndexMap::new())),
            role_load_inflight: DashMap::new(),
            http_api_roles: DashMap::new(),
            http_api_roles_order: parking_lot::RwLock::new(indexmap::IndexMap::new()),
            session_cache: SessionCache::shared(),
            storage,
            policy_runtime,
            ollama_model: parking_lot::RwLock::new(ollama_model),
            plugins,
            directory_plugins,
            high_risk_grants,
            turn_locks: dashmap::DashMap::new(),
            startup_health: parking_lot::RwLock::new(
                crate::domain::startup_health::StartupHealthCache::default(),
            ),
            remote_fallback_env_override: remote_fallback_env_override(),
            remote_fallback_allowed,
            policy_file_applied: AtomicBool::new(policy_file_applied),
            user_llm_provider,
            user_llm_env_version: AtomicU64::new(1),
            user_llm_env_applied_version: AtomicU64::new(0),
            user_llm_env_dirty: AtomicBool::new(true),
            host_profile,
            affect_metrics_sink: AffectSinkHandle::new(),
            reply_post_processor_resolver,
            theater_director_resolver,
        };
        if let Err(e) = crate::domain::user_llm_env::apply_user_llm_env(&state).await {
            tracing::warn!(target: "oclive_llm", "apply user llm settings: {e}");
        }
        Ok(state)
    }
}

async fn connect_db(db_path: &Path) -> Result<SqlitePool> {
    if db_path == Path::new(":memory:") {
        sqlite_pool::connect_memory()
            .await
            .map_err(|e| crate::error::AppError::DatabaseError(e.to_string()))
    } else {
        if let Some(parent) = db_path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)?;
            }
        }
        sqlite_pool::connect_file(db_path)
            .await
            .map_err(|e| crate::error::AppError::DatabaseError(e.to_string()))
    }
}

async fn run_migrations(db: &SqlitePool) -> Result<()> {
    let migrations_dir = crate::infrastructure::sql_migrate::find_migrations_dir()
        .map_err(crate::error::AppError::DatabaseError)?;
    crate::infrastructure::sql_migrate::run_sql_migrations(db, &migrations_dir)
        .await
        .map_err(crate::error::AppError::DatabaseError)
}

async fn remote_fallback_switch(db_manager: &DbManager) -> Result<Arc<AtomicBool>> {
    let remote_raw = db_manager
        .get_app_setting("remote_fallback_to_builtin")
        .await?;
    let mut remote_allowed = remote_fallback_from_db_value(remote_raw);
    if let Some(v) = remote_fallback_env_override() {
        remote_allowed = v;
    }
    Ok(new_remote_fallback_switch(remote_allowed))
}
