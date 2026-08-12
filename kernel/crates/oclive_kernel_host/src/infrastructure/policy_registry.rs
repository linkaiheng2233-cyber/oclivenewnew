//! Policy registry file loading and runtime policy set construction.

use crate::domain::{
    DefaultEmotionPolicy, DefaultEventPolicy, DefaultMemoryPolicy, EmotionPolicy, EventPolicy,
    MemoryPolicy, MemoryPolicyConfig, PolicyConfig,
};
use crate::error::Result;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Arc;

pub struct PolicySet {
    pub emotion: Arc<dyn EmotionPolicy>,
    pub event: Arc<dyn EventPolicy>,
    pub memory: Arc<dyn MemoryPolicy>,
}

pub(crate) struct PolicyRuntime {
    pub default_policy_set: Arc<PolicySet>,
    pub scene_policy_sets: HashMap<String, Arc<PolicySet>>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(default, deny_unknown_fields)]
pub struct PolicyRegistryFile {
    pub default: PolicyConfig,
    pub default_profile: String,
    pub profiles: HashMap<String, PolicyConfig>,
    pub scene_bindings: HashMap<String, String>,
}

impl PolicyRegistryFile {
    #[must_use]
    pub fn with_defaults() -> Self {
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
    // B M1 slice 2: emotion hold env overrides removed with the policy hold
    // fields (POLICY_EMOTION_NEUTRAL_HOLD_ENABLED / _LOW_CONFIDENCE_HOLD_THRESHOLD).
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

/// # Errors
///
/// IO/parse failures when `strict` is true; otherwise falls back to defaults.
pub fn load_policy_registry_from_path(path: &Path, strict: bool) -> Result<PolicyRegistryFile> {
    let mut registry = if path.exists() {
        let content = fs::read_to_string(path).map_err(crate::error::AppError::IoError)?;
        match toml::from_str::<PolicyFileSchema>(&content) {
            Ok(PolicyFileSchema::Registry(parsed)) => {
                tracing::info!("policy config loaded source=file path={}", path.display());
                parsed
            }
            Ok(PolicyFileSchema::Legacy(legacy)) => {
                tracing::info!(
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
                tracing::warn!(
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

#[must_use]
pub fn load_policy_registry() -> PolicyRegistryFile {
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

pub(crate) fn build_policy_sets_from_registry(registry: PolicyRegistryFile) -> PolicyRuntime {
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
