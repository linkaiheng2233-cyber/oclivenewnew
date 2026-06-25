//! Shared `distro.oclive.toml` serde model — single parse entry (K-PROFILE-01).

use oclive_kernel_types::DistroProfileRequirements;
use serde::Deserialize;
use std::path::Path;

/// Parsed `distro.oclive.toml` (scheduling + runtime fields).
#[derive(Debug, Default, Clone, Deserialize)]
pub struct DistroOcliveFile {
    #[serde(default)]
    pub schema_version: Option<u32>,
    pub distro_id: Option<String>,
    #[serde(default)]
    pub display_name: Option<String>,
    pub host_flags: Option<HostFlagsToml>,
    pub slots: Option<SlotsToml>,
    pub prompt: Option<PromptToml>,
    pub user_identity: Option<UserIdentityToml>,
    pub post_process: Option<PostProcessToml>,
    pub state_expression: Option<StateExpressionToml>,
    pub memory: Option<MemoryToml>,
    pub plugin_backends: Option<PluginBackendsToml>,
    pub interaction: Option<InteractionToml>,
    pub visual_presentation: Option<VisualPresentationToml>,
    pub theater: Option<TheaterToml>,
    pub turn_thinking: Option<TurnThinkingToml>,
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct TheaterToml {
    /// Directory plugin id with `provides: theater_director`.
    pub director_plugin: Option<String>,
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct VisualPresentationToml {
    /// `off` | `image_only` | `stage_full` — see DISTRO_CAPABILITY_PROFILE.md §3.3
    pub mode: Option<String>,
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct InteractionToml {
    /// `pure_chat` | `immersive` — seeded when `role_runtime.interaction_mode` is unset.
    pub default_mode: Option<String>,
    #[serde(default)]
    pub allow_mode_switch: Option<bool>,
    /// Frontend hint: suggest story mode after N successful turns (default 10 when omitted).
    pub immersive_unlock_hint_after_turns: Option<u32>,
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct HostFlagsToml {
    pub skip_agent: Option<bool>,
    pub skip_complex_emotion: Option<bool>,
    /// When `false`, co-present event impact uses rule-based `EventDetector` only (no pre-LLM `generate_tag`).
    pub event_impact_llm: Option<bool>,
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct TurnThinkingToml {
    /// `auto` | `fast` | `deep`
    pub default: Option<String>,
    pub fast_skip_complex_emotion: Option<bool>,
    pub auto_deep_min_chars: Option<usize>,
    #[serde(default)]
    pub auto_deep_keywords: Option<Vec<String>>,
    pub fast_knowledge_limit: Option<usize>,
    pub fast_memory_cap: Option<usize>,
    pub deep_capsule: Option<bool>,
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct SlotsToml {
    pub complex_emotion: Option<String>,
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct PromptToml {
    pub profile: Option<String>,
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct UserIdentityToml {
    pub default_id: Option<String>,
    pub allowed_ids: Option<Vec<String>>,
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct PostProcessToml {
    pub chain: Option<String>,
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct StateExpressionToml {
    pub favor_high: Option<String>,
    pub favor_mid: Option<String>,
    pub favor_low: Option<String>,
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct MemoryToml {
    pub retrieval: Option<String>,
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct PluginBackendsToml {
    pub memory: Option<String>,
    pub emotion: Option<String>,
    pub event: Option<String>,
    pub prompt: Option<String>,
    pub llm: Option<String>,
    pub agent: Option<String>,
}

/// Parse TOML text into [`DistroOcliveFile`].
///
/// # Errors
///
/// Returns TOML deserialization errors as strings.
pub fn parse_distro_oclive_toml(raw: &str) -> Result<DistroOcliveFile, String> {
    toml::from_str(raw).map_err(|e| e.to_string())
}

/// Read and parse a distro profile file.
///
/// # Errors
///
/// Returns I/O or TOML parse errors as strings.
pub fn parse_distro_oclive_file(path: &Path) -> Result<DistroOcliveFile, String> {
    let raw = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    parse_distro_oclive_toml(&raw)
}

impl DistroOcliveFile {
    /// Resolve `distro_id` from file body or path stem.
    #[must_use]
    pub fn effective_distro_id(&self, path_hint: &Path) -> String {
        self.distro_id.clone().unwrap_or_else(|| {
            path_hint
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("default")
                .to_string()
        })
    }

    /// `(skip_agent, skip_complex_emotion)` from host_flags and slots.
    #[must_use]
    pub fn parse_distro_skip_flags(&self) -> (bool, bool) {
        let mut skip_agent = false;
        let mut skip_complex_emotion = false;
        if let Some(ref hf) = self.host_flags {
            skip_agent = hf.skip_agent.unwrap_or(false);
            skip_complex_emotion = hf.skip_complex_emotion.unwrap_or(false);
        }
        if let Some(ref slots) = self.slots {
            if slots
                .complex_emotion
                .as_deref()
                .is_some_and(|s| s.eq_ignore_ascii_case("off"))
            {
                skip_complex_emotion = true;
            }
        }
        (skip_agent, skip_complex_emotion)
    }

    /// Scheduling subset for attach/replace policy (`kernel_strategy`).
    #[must_use]
    pub fn into_requirements(self, path_hint: &Path) -> DistroProfileRequirements {
        let distro_id = self.effective_distro_id(path_hint);
        let (skip_agent, skip_complex_emotion) = self.parse_distro_skip_flags();
        requirements_from_flags(
            &distro_id,
            skip_agent,
            skip_complex_emotion,
            self.prompt.and_then(|p| p.profile),
            self.post_process.and_then(|p| p.chain),
        )
    }
}

#[must_use]
pub fn requirements_from_flags(
    distro_id: &str,
    skip_agent: bool,
    skip_complex_emotion: bool,
    prompt: Option<String>,
    post_process: Option<String>,
) -> DistroProfileRequirements {
    let mut forbidden_modules = Vec::new();
    let mut required_modules = vec![
        "memory".into(),
        "emotion".into(),
        "event".into(),
        "prompt".into(),
        "llm".into(),
    ];
    if skip_agent {
        forbidden_modules.push("agent".into());
    } else {
        required_modules.push("agent".into());
    }
    if skip_complex_emotion {
        forbidden_modules.push("complex_emotion".into());
    } else {
        required_modules.push("complex_emotion".into());
    }
    DistroProfileRequirements {
        distro_id: distro_id.to_string(),
        required_modules,
        forbidden_modules,
        post_process_profile: post_process,
        prompt_profile: prompt,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn parse_desktop_chat_interaction_defaults() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../examples/distro-profiles");
        let file = parse_distro_oclive_file(&root.join("desktop-chat.oclive.toml")).unwrap();
        let ix = file.interaction.as_ref().expect("interaction");
        assert_eq!(ix.default_mode.as_deref(), Some("pure_chat"));
        assert_eq!(ix.allow_mode_switch, Some(true));
        assert_eq!(ix.immersive_unlock_hint_after_turns, Some(10));
    }

    #[test]
    fn parse_vscode_example_matches_requirements() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../examples/distro-profiles");
        let path = root.join("vscode.oclive.toml");
        let file = parse_distro_oclive_file(&path).unwrap();
        let req = file.clone().into_requirements(&path);
        assert_eq!(req.distro_id, "vscode");
        assert!(req.forbidden_modules.contains(&"agent".to_string()));
        assert_eq!(req.prompt_profile.as_deref(), Some("concise"));
        assert!(file.memory.as_ref().is_some());
        assert_eq!(
            file.memory.as_ref().unwrap().retrieval.as_deref(),
            Some("light")
        );
    }
}
