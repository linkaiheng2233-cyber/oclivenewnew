//! Blueprint `runtime_config` section (Stable v4 SSOT; frozen v3 keeps the
//! dual-core Beta field; v2 presence is non-fatal and ignored by its load path).

use serde::{Deserialize, Serialize};

use crate::disk_role_settings::{AutonomousSceneConfig, RemotePresenceConfig};
use crate::manifest::{EvolutionConfigDisk, MemoryConfigDisk};
use crate::validate::validate_interaction_mode_pack_setting;

const MAX_CONTEXT_TOKENS: u32 = 262_144;
const MAX_OUTPUT_TOKENS: u32 = 32_768;

/// Dual-core toggle (blueprint only; off by default).
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct DualCoreConfig {
    #[serde(default)]
    pub enabled: bool,
}

/// Portable, backend-agnostic generation preferences authored for one role.
///
/// These values describe the role's ideal inference behavior. The host remains
/// responsible for selecting the installed model/runtime and may clamp the
/// preferences to model, device, user-setting, and kernel safety limits.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct InferenceProfileConfig {
    #[serde(default)]
    pub generation: Option<InferenceGenerationConfig>,
    #[serde(default)]
    pub context: Option<InferenceContextConfig>,
    #[serde(default)]
    pub reasoning: Option<InferenceReasoningConfig>,
    #[serde(default)]
    pub performance_intent: Option<InferencePerformanceIntentConfig>,
}

/// Portable sampling and response-budget preferences.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct InferenceGenerationConfig {
    #[serde(default)]
    pub temperature: Option<f32>,
    #[serde(default)]
    pub top_p: Option<f32>,
    #[serde(default)]
    pub preferred_output_tokens: Option<u32>,
    #[serde(default)]
    pub maximum_output_tokens: Option<u32>,
}

/// Portable context-window intent. It does not select or install a model.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct InferenceContextConfig {
    #[serde(default)]
    pub preferred_tokens: Option<u32>,
    #[serde(default)]
    pub minimum_tokens: Option<u32>,
}

/// Model-independent reasoning intent.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct InferenceReasoningConfig {
    #[serde(default)]
    pub mode: Option<InferenceReasoningMode>,
    #[serde(default)]
    pub effort: Option<f32>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum InferenceReasoningMode {
    Instant,
    Adaptive,
    Deep,
}

/// Resource behavior requested by a role without embedding machine-specific
/// GPU layers, thread counts, runtime paths, or model identifiers.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct InferencePerformanceIntentConfig {
    #[serde(default)]
    pub priority: Option<InferencePerformancePriority>,
    #[serde(default)]
    pub prefer_prefix_cache: Option<bool>,
    #[serde(default)]
    pub prefer_model_residency: Option<bool>,
    #[serde(default)]
    pub allow_context_reduction: Option<bool>,
    #[serde(default)]
    pub allow_output_reduction: Option<bool>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum InferencePerformancePriority {
    Latency,
    Balanced,
    Quality,
}

/// System runtime configuration (blueprint-only; not the role-pack creator view).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeConfig {
    #[serde(default)]
    pub interaction_mode: Option<String>,
    #[serde(default)]
    pub memory_config: Option<MemoryConfigDisk>,
    #[serde(default)]
    pub reply_quality_anchor: Option<String>,
    /// Optional pack-level hint aligned with host `app_settings.remote_fallback_to_builtin`.
    #[serde(default)]
    pub remote_fallback_to_builtin: Option<bool>,
    #[serde(default)]
    pub dual_core: Option<DualCoreConfig>,
    #[serde(default)]
    pub identity_binding: Option<crate::manifest::IdentityBinding>,
    #[serde(default)]
    pub evolution: Option<EvolutionConfigDisk>,
    #[serde(default)]
    pub ollama_model: Option<String>,
    #[serde(default)]
    pub remote_presence: Option<RemotePresenceConfig>,
    #[serde(default)]
    pub autonomous_scene: Option<AutonomousSceneConfig>,
    /// Stable v4 portable inference intent; never contains local model/runtime
    /// selection or machine-specific resource values.
    #[serde(default)]
    pub inference_profile: Option<InferenceProfileConfig>,
}

/// Validate `runtime_config` sub-fields (before v3 blueprint load).
///
/// # Errors
///
/// Returns `Err(Vec<String>)` when sub-field contracts are violated.
pub fn validate_runtime_config(rc: &RuntimeConfig) -> Result<(), Vec<String>> {
    let mut errs = Vec::new();
    if let Some(ref m) = rc.interaction_mode {
        if let Err(e) = validate_interaction_mode_pack_setting(Some(m.as_str())) {
            errs.push(e);
        }
    }
    if let Some(ref profile) = rc.inference_profile {
        validate_inference_profile(profile, &mut errs);
    }
    if errs.is_empty() {
        Ok(())
    } else {
        Err(errs)
    }
}

fn validate_inference_profile(profile: &InferenceProfileConfig, errs: &mut Vec<String>) {
    if let Some(ref generation) = profile.generation {
        validate_optional_f32(
            generation.temperature,
            0.0,
            2.0,
            "runtime_config.inference_profile.generation.temperature",
            errs,
        );
        if generation
            .top_p
            .is_some_and(|value| !value.is_finite() || value <= 0.0 || value > 1.0)
        {
            errs.push(
                "runtime_config.inference_profile.generation.top_p 须为大于 0 且不大于 1 的有限数值"
                    .into(),
            );
        }
        validate_optional_u32(
            generation.preferred_output_tokens,
            MAX_OUTPUT_TOKENS,
            "runtime_config.inference_profile.generation.preferred_output_tokens",
            errs,
        );
        validate_optional_u32(
            generation.maximum_output_tokens,
            MAX_OUTPUT_TOKENS,
            "runtime_config.inference_profile.generation.maximum_output_tokens",
            errs,
        );
        if let (Some(preferred), Some(maximum)) = (
            generation.preferred_output_tokens,
            generation.maximum_output_tokens,
        ) {
            if preferred > maximum {
                errs.push("runtime_config.inference_profile.generation.preferred_output_tokens 不得大于 maximum_output_tokens".into());
            }
        }
    }

    if let Some(ref context) = profile.context {
        validate_optional_u32(
            context.preferred_tokens,
            MAX_CONTEXT_TOKENS,
            "runtime_config.inference_profile.context.preferred_tokens",
            errs,
        );
        validate_optional_u32(
            context.minimum_tokens,
            MAX_CONTEXT_TOKENS,
            "runtime_config.inference_profile.context.minimum_tokens",
            errs,
        );
        if let (Some(minimum), Some(preferred)) = (context.minimum_tokens, context.preferred_tokens)
        {
            if minimum > preferred {
                errs.push("runtime_config.inference_profile.context.minimum_tokens 不得大于 preferred_tokens".into());
            }
        }
    }

    if let Some(ref reasoning) = profile.reasoning {
        validate_optional_f32(
            reasoning.effort,
            0.0,
            1.0,
            "runtime_config.inference_profile.reasoning.effort",
            errs,
        );
    }
}

fn validate_optional_f32(
    value: Option<f32>,
    minimum: f32,
    maximum: f32,
    path: &str,
    errs: &mut Vec<String>,
) {
    if value.is_some_and(|value| !value.is_finite() || value < minimum || value > maximum) {
        errs.push(format!("{path} 须为 {minimum}～{maximum} 的有限数值"));
    }
}

fn validate_optional_u32(value: Option<u32>, maximum: u32, path: &str, errs: &mut Vec<String>) {
    if value.is_some_and(|value| value == 0 || value > maximum) {
        errs.push(format!("{path} 须为 1～{maximum} 的整数"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inference_profile_accepts_portable_preferences() {
        let config: RuntimeConfig = serde_json::from_value(serde_json::json!({
            "inference_profile": {
                "generation": {
                    "temperature": 0.8,
                    "top_p": 0.9,
                    "preferred_output_tokens": 768,
                    "maximum_output_tokens": 1536
                },
                "context": { "minimum_tokens": 8192, "preferred_tokens": 16384 },
                "reasoning": { "mode": "adaptive", "effort": 0.65 },
                "performance_intent": {
                    "priority": "balanced",
                    "prefer_prefix_cache": true,
                    "allow_context_reduction": true
                }
            }
        }))
        .expect("valid runtime config");

        assert!(validate_runtime_config(&config).is_ok());
        let profile = config.inference_profile.expect("inference profile");
        assert_eq!(
            profile.reasoning.and_then(|reasoning| reasoning.mode),
            Some(InferenceReasoningMode::Adaptive)
        );
    }

    #[test]
    fn inference_profile_rejects_invalid_ranges_and_budget_order() {
        let config: RuntimeConfig = serde_json::from_value(serde_json::json!({
            "inference_profile": {
                "generation": {
                    "temperature": 2.5,
                    "top_p": 0.0,
                    "preferred_output_tokens": 2048,
                    "maximum_output_tokens": 1024
                },
                "context": { "minimum_tokens": 32768, "preferred_tokens": 8192 },
                "reasoning": { "effort": 1.5 }
            }
        }))
        .expect("shape remains parseable");

        let errors = validate_runtime_config(&config).expect_err("invalid profile");
        assert!(errors.iter().any(|error| error.contains("temperature")));
        assert!(errors.iter().any(|error| error.contains("top_p")));
        assert!(errors
            .iter()
            .any(|error| error.contains("maximum_output_tokens")));
        assert!(errors.iter().any(|error| error.contains("minimum_tokens")));
        assert!(errors
            .iter()
            .any(|error| error.contains("reasoning.effort")));
    }
}
