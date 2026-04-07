use crate::domain::plugin_host::{PluginHost, ResolvedRolePlugins};
use crate::domain::repository::{FavorabilityRepository, MemoryRepository};
use crate::domain::{
    DefaultEmotionPolicy, DefaultEventPolicy, DefaultMemoryPolicy, EmotionPolicy,
    EmotionPolicyConfig, EventEstimator, EventPolicy, MemoryPolicy, MemoryPolicyConfig,
    MemoryRetrieval, PolicyConfig, PromptAssembler, UserEmotionAnalyzer,
};
use crate::error::Result;
use crate::infrastructure::db::DbManager;
use crate::infrastructure::llm::ollama_llm;
use crate::infrastructure::llm::LlmClient;
use crate::infrastructure::ollama_client::OllamaClient;
use crate::infrastructure::repositories::{SqliteFavorabilityRepository, SqliteMemoryRepository};
use crate::infrastructure::storage::RoleStorage;
use crate::models::{PersonalityVector, PluginBackends, Role};
use parking_lot::RwLock;
use serde::Deserialize;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// 开发时进程 cwd 可能是 `src-tauri/`，优先定位到项目根的 `roles/`。
/// 日志 target：`oclive_roles`（与 `lib.rs` 中打包路径日志一致，便于过滤）。
pub fn resolve_roles_dir() -> PathBuf {
    if let Ok(custom) = std::env::var("OCLIVE_ROLES_DIR") {
        let p = PathBuf::from(custom);
        if p.is_dir() {
            log::info!(
                target: "oclive_roles",
                "resolve_roles_dir: OCLIVE_ROLES_DIR -> {}",
                p.display()
            );
            return p;
        }
    }
    if let Ok(cwd) = std::env::current_dir() {
        let a = cwd.join("roles");
        if a.is_dir() {
            log::info!(
                target: "oclive_roles",
                "resolve_roles_dir: cwd/roles -> {}",
                a.display()
            );
            return a;
        }
        let b = cwd.join("..").join("roles");
        if let Ok(canon) = b.canonicalize() {
            if canon.is_dir() {
                log::info!(
                    target: "oclive_roles",
                    "resolve_roles_dir: ../roles -> {}",
                    canon.display()
                );
                return canon;
            }
        }
    }
    let fallback = PathBuf::from("roles");
    log::info!(
        target: "oclive_roles",
        "resolve_roles_dir: relative fallback -> {}",
        fallback.display()
    );
    fallback
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

pub struct AppState {
    pub db_manager: Arc<DbManager>,
    pub memory_repo: Arc<dyn MemoryRepository>,
    pub favorability_repo: Arc<dyn FavorabilityRepository>,
    pub llm: Arc<dyn LlmClient>,
    pub role_cache: Arc<RwLock<HashMap<String, Role>>>,
    pub personality_cache: Arc<RwLock<HashMap<String, PersonalityVector>>>,
    pub storage: RoleStorage,
    policy_runtime: Arc<RwLock<PolicyRuntime>>,
    /// Ollama 模型名（可用环境变量 `OLLAMA_MODEL` 覆盖）
    pub ollama_model: String,
    /// 可替换子系统实现（按 `Role.plugin_backends` 选择）
    pub plugins: PluginHost,
}

impl AppState {
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

    /// `roles_dir_override`：打包应用传入 `resource_dir/roles`；`None` 时用 [`resolve_roles_dir`]。
    pub async fn new(
        db_path: impl AsRef<Path>,
        roles_dir_override: Option<PathBuf>,
    ) -> Result<Self> {
        let path = db_path.as_ref();
        let db = if path == Path::new(":memory:") {
            SqlitePoolOptions::new()
                .max_connections(5)
                .connect("sqlite::memory:")
                .await
                .map_err(|e| crate::error::AppError::DatabaseError(e.to_string()))?
        } else {
            if let Some(parent) = path.parent() {
                if !parent.as_os_str().is_empty() {
                    fs::create_dir_all(parent)?;
                }
            }
            let opts = SqliteConnectOptions::new()
                .filename(path)
                .create_if_missing(true);
            SqlitePoolOptions::new()
                .max_connections(5)
                .connect_with(opts)
                .await
                .map_err(|e| crate::error::AppError::DatabaseError(e.to_string()))?
        };

        sqlx::migrate!("./migrations")
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

        // 默认需与本机 `ollama list` 中已有模型一致；未安装 llama3.2 时会 404
        let ollama_model =
            std::env::var("OLLAMA_MODEL").unwrap_or_else(|_| "qwen2.5:7b".to_string());
        let registry = load_policy_registry();
        let runtime = Self::build_policy_sets_from_registry(registry);

        let storage = RoleStorage::new(roles_dir_override.unwrap_or_else(resolve_roles_dir));
        let plugins = PluginHost::new(llm.clone());

        Ok(Self {
            db_manager,
            memory_repo,
            favorability_repo,
            llm,
            role_cache: Arc::new(RwLock::new(HashMap::new())),
            personality_cache: Arc::new(RwLock::new(HashMap::new())),
            storage,
            policy_runtime: Arc::new(RwLock::new(runtime)),
            ollama_model,
            plugins,
        })
    }

    /// 内存库 + 注入 LLM（集成测试 / 不连 Ollama）
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
        let db = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(5)
            .connect("sqlite::memory:")
            .await
            .map_err(|e| crate::error::AppError::DatabaseError(e.to_string()))?;

        sqlx::migrate!("./migrations")
            .run(&db)
            .await
            .map_err(|e| crate::error::AppError::DatabaseError(e.to_string()))?;

        let db_manager = Arc::new(DbManager::new(db));

        let memory_repo: Arc<dyn MemoryRepository> =
            Arc::new(SqliteMemoryRepository::new(db_manager.clone()));
        let favorability_repo: Arc<dyn FavorabilityRepository> =
            Arc::new(SqliteFavorabilityRepository::new(db_manager.clone()));

        let storage = RoleStorage::new(roles_dir);
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

        let plugins = PluginHost::new(llm.clone());

        Ok(Self {
            db_manager,
            memory_repo,
            favorability_repo,
            llm,
            role_cache: Arc::new(RwLock::new(HashMap::new())),
            personality_cache: Arc::new(RwLock::new(HashMap::new())),
            storage,
            policy_runtime: Arc::new(RwLock::new(runtime)),
            ollama_model: "test-model".to_string(),
            plugins,
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

    /// 当前有效性格：`manifest` 核心 + `role_runtime.delta_personality`，再按 `evolution_bounds` 限幅。
    pub async fn get_current_personality(
        &self,
        role_id: &str,
        role: &Role,
    ) -> Result<PersonalityVector> {
        if let Some(p) = self.personality_cache.read().get(role_id) {
            return Ok(p.clone());
        }
        let (_, delta_s) = self
            .db_manager
            .get_core_delta_personality_json(role_id)
            .await?;
        let delta_v = delta_s
            .and_then(|s| PersonalityVector::from_json_vec(&s).ok())
            .unwrap_or_else(PersonalityVector::zero);
        let effective = PersonalityVector::effective_from_core_delta(
            &role.default_personality,
            &delta_v,
            &role.evolution_bounds,
        );
        self.personality_cache
            .write()
            .insert(role_id.to_string(), effective.clone());
        Ok(effective)
    }

    /// 单次对话内优先调用本方法一次，再复用返回的 `memory` / `emotion` / `event` / `prompt` / `llm`，避免重复解析后端枚举。
    pub fn resolved_plugins_for(&self, role: &Role) -> ResolvedRolePlugins {
        self.plugins.resolve_for_role(role)
    }

    pub fn memory_retrieval_for(&self, role: &Role) -> Arc<dyn MemoryRetrieval> {
        self.plugins.memory_retrieval(role.plugin_backends.memory)
    }

    pub fn user_emotion_analyzer_for(&self, role: &Role) -> Arc<dyn UserEmotionAnalyzer> {
        self.plugins
            .user_emotion_analyzer(role.plugin_backends.emotion)
    }

    pub fn event_estimator_for(&self, role: &Role) -> Arc<dyn EventEstimator> {
        self.plugins.event_estimator(role.plugin_backends.event)
    }

    pub fn prompt_assembler_for(&self, role: &Role) -> Arc<dyn PromptAssembler> {
        self.plugins.prompt_assembler(role.plugin_backends.prompt)
    }

    /// 测试或遥测：当前角色包声明的后端集合
    pub fn plugin_backends_snapshot(&self, role: &Role) -> PluginBackends {
        role.plugin_backends.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_app_state_creation() {
        let state = AppState::new(":memory:", None).await;
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
}
