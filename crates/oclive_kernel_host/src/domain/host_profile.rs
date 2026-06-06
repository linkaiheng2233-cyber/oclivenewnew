//! Distro capability profile — loaded from `distro.oclive.toml` (P1 contract, P4 runtime).

use crate::models::plugin_backends::{
    AgentBackend, EmotionBackend, EventBackend, LlmBackend, MemoryBackend, PluginBackends,
    PromptBackend,
};
use serde::Deserialize;
use std::path::{Path, PathBuf};

pub const ENV_DISTRO_ID: &str = "OCLIVE_DISTRO_ID";
pub const ENV_DISTRO_PROFILE: &str = "OCLIVE_DISTRO_PROFILE";

/// Distro default User Identity Prompt Template id (when session has no explicit choice).
#[derive(Debug, Clone, Default)]
pub struct UserIdentityProfile {
    pub default_id: Option<String>,
    pub allowed_ids: Option<Vec<String>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PostProcessChain {
    #[default]
    Standard,
    Minimal,
}

impl PostProcessChain {
    #[must_use]
    pub fn parse(s: &str) -> Self {
        if s.eq_ignore_ascii_case("minimal") {
            Self::Minimal
        } else {
            Self::Standard
        }
    }
}

/// Distro reply post-processor chain policy (`standard` respects role pack; `minimal` forces builtin minimal profile).
#[derive(Debug, Clone, Default)]
pub struct PostProcessChainProfile {
    pub chain: PostProcessChain,
}

/// Effective host policy for kernel runtime (process-level, v1).
#[derive(Debug, Clone)]
pub struct HostProfile {
    pub distro_id: String,
    pub skip_agent: bool,
    pub skip_complex_emotion: bool,
    pub prompt_profile: PromptProfile,
    pub backends_ceiling: Option<PluginBackends>,
    pub user_identity: UserIdentityProfile,
    pub post_process: PostProcessChainProfile,
    /// Path passed to child kernel via `OCLIVE_DISTRO_PROFILE` when spawned.
    pub profile_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PromptProfile {
    #[default]
    Full,
    Concise,
}

impl PromptProfile {
    #[must_use]
    pub fn parse(s: &str) -> Self {
        if s.eq_ignore_ascii_case("concise") {
            Self::Concise
        } else {
            Self::Full
        }
    }
}

#[derive(Debug, Default, Deserialize)]
struct DistroProfileFile {
    #[serde(default)]
    #[allow(dead_code)]
    schema_version: Option<u32>,
    distro_id: Option<String>,
    host_flags: Option<HostFlagsToml>,
    slots: Option<SlotsToml>,
    prompt: Option<PromptToml>,
    user_identity: Option<UserIdentityToml>,
    post_process: Option<PostProcessToml>,
    plugin_backends: Option<PluginBackendsToml>,
}

#[derive(Debug, Default, Deserialize)]
struct HostFlagsToml {
    skip_agent: Option<bool>,
    skip_complex_emotion: Option<bool>,
}

#[derive(Debug, Default, Deserialize)]
struct SlotsToml {
    complex_emotion: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
struct PromptToml {
    profile: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
struct UserIdentityToml {
    default_id: Option<String>,
    allowed_ids: Option<Vec<String>>,
}

#[derive(Debug, Default, Deserialize)]
struct PostProcessToml {
    chain: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
struct PluginBackendsToml {
    memory: Option<String>,
    emotion: Option<String>,
    event: Option<String>,
    prompt: Option<String>,
    llm: Option<String>,
    agent: Option<String>,
}

impl Default for HostProfile {
    fn default() -> Self {
        Self {
            distro_id: "default".into(),
            skip_agent: false,
            skip_complex_emotion: false,
            prompt_profile: PromptProfile::Full,
            backends_ceiling: None,
            user_identity: UserIdentityProfile::default(),
            post_process: PostProcessChainProfile::default(),
            profile_path: None,
        }
    }
}

/// Load from `OCLIVE_DISTRO_PROFILE` file and `OCLIVE_DISTRO_ID`, or defaults (full capability).
#[must_use]
pub fn load_host_profile_from_env() -> HostProfile {
    let distro_id = std::env::var(ENV_DISTRO_ID)
        .ok()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| "default".into());
    let path = std::env::var(ENV_DISTRO_PROFILE)
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .map(PathBuf::from);
    if let Some(ref p) = path {
        if let Ok(mut profile) = load_host_profile_file(p) {
            profile.profile_path = Some(p.clone());
            tracing::info!(
                target: "oclive_desktop",
                distro_id = %profile.distro_id,
                path = %p.display(),
                skip_agent = profile.skip_agent,
                skip_complex_emotion = profile.skip_complex_emotion,
                prompt = ?profile.prompt_profile,
                "host profile loaded"
            );
            return profile;
        }
        tracing::warn!(
            target: "oclive_desktop",
            path = %p.display(),
            "failed to load OCLIVE_DISTRO_PROFILE; using defaults"
        );
    }
    HostProfile {
        distro_id,
        ..HostProfile::default()
    }
}

/// Parse a distro profile TOML file.
///
/// # Errors
///
/// Returns I/O or TOML parse errors, or unknown `plugin_backends` enum values.
pub fn load_host_profile_file(path: &Path) -> Result<HostProfile, String> {
    let raw = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    let file: DistroProfileFile = toml::from_str(&raw).map_err(|e| e.to_string())?;
    let mut profile = HostProfile::default();
    if let Some(id) = file.distro_id {
        profile.distro_id = id;
    }
    if let Some(ref hf) = file.host_flags {
        profile.skip_agent = hf.skip_agent.unwrap_or(false);
        profile.skip_complex_emotion = hf.skip_complex_emotion.unwrap_or(false);
    }
    if let Some(ref slots) = file.slots {
        if slots
            .complex_emotion
            .as_deref()
            .is_some_and(|s| s.eq_ignore_ascii_case("off"))
        {
            profile.skip_complex_emotion = true;
        }
    }
    if let Some(ref p) = file.prompt {
        if let Some(ref prof) = p.profile {
            profile.prompt_profile = PromptProfile::parse(prof);
        }
    }
    if let Some(ref pb) = file.plugin_backends {
        profile.backends_ceiling = Some(parse_plugin_backends_toml(pb)?);
    }
    if let Some(ref ui) = file.user_identity {
        profile.user_identity.default_id = ui
            .default_id
            .as_ref()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
        profile.user_identity.allowed_ids = ui.allowed_ids.as_ref().map(|ids| {
            ids.iter()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        });
    }
    if let Some(ref pp) = file.post_process {
        if let Some(ref chain) = pp.chain {
            profile.post_process.chain = PostProcessChain::parse(chain);
        }
    }
    Ok(profile)
}

fn parse_plugin_backends_toml(pb: &PluginBackendsToml) -> Result<PluginBackends, String> {
    Ok(PluginBackends {
        memory: parse_memory(pb.memory.as_deref())?,
        emotion: parse_emotion(pb.emotion.as_deref())?,
        event: parse_event(pb.event.as_deref())?,
        prompt: parse_prompt(pb.prompt.as_deref())?,
        llm: parse_llm(pb.llm.as_deref())?,
        agent: parse_agent(pb.agent.as_deref())?,
        ..Default::default()
    })
}

fn parse_memory(s: Option<&str>) -> Result<MemoryBackend, String> {
    match s {
        None => Ok(MemoryBackend::Builtin),
        Some(v) => match v {
            "builtin" => Ok(MemoryBackend::Builtin),
            "builtin_v2" => Ok(MemoryBackend::BuiltinV2),
            "remote" => Ok(MemoryBackend::Remote),
            "local" => Ok(MemoryBackend::Local),
            "directory" => Ok(MemoryBackend::Directory),
            other => Err(format!("unknown memory backend: {other}")),
        },
    }
}

fn parse_emotion(s: Option<&str>) -> Result<EmotionBackend, String> {
    match s {
        None => Ok(EmotionBackend::Builtin),
        Some(v) => match v {
            "builtin" => Ok(EmotionBackend::Builtin),
            "builtin_v2" => Ok(EmotionBackend::BuiltinV2),
            "remote" => Ok(EmotionBackend::Remote),
            "directory" => Ok(EmotionBackend::Directory),
            other => Err(format!("unknown emotion backend: {other}")),
        },
    }
}

fn parse_event(s: Option<&str>) -> Result<EventBackend, String> {
    match s {
        None => Ok(EventBackend::Builtin),
        Some(v) => match v {
            "builtin" => Ok(EventBackend::Builtin),
            "builtin_v2" => Ok(EventBackend::BuiltinV2),
            "remote" => Ok(EventBackend::Remote),
            "directory" => Ok(EventBackend::Directory),
            other => Err(format!("unknown event backend: {other}")),
        },
    }
}

fn parse_prompt(s: Option<&str>) -> Result<PromptBackend, String> {
    match s {
        None => Ok(PromptBackend::Builtin),
        Some(v) => match v {
            "builtin" => Ok(PromptBackend::Builtin),
            "builtin_v2" => Ok(PromptBackend::BuiltinV2),
            "remote" => Ok(PromptBackend::Remote),
            "directory" => Ok(PromptBackend::Directory),
            other => Err(format!("unknown prompt backend: {other}")),
        },
    }
}

fn parse_llm(s: Option<&str>) -> Result<LlmBackend, String> {
    match s {
        None => Ok(LlmBackend::Ollama),
        Some(v) => match v {
            "ollama" => Ok(LlmBackend::Ollama),
            "remote" => Ok(LlmBackend::Remote),
            "directory" => Ok(LlmBackend::Directory),
            other => Err(format!("unknown llm backend: {other}")),
        },
    }
}

fn parse_agent(s: Option<&str>) -> Result<AgentBackend, String> {
    match s {
        None => Ok(AgentBackend::Builtin),
        Some(v) => match v {
            "builtin" => Ok(AgentBackend::Builtin),
            "remote" => Ok(AgentBackend::Remote),
            "directory" => Ok(AgentBackend::Directory),
            other => Err(format!("unknown agent backend: {other}")),
        },
    }
}

/// Concise prompt overlay for VS Code–style distros (P4d minimal).
pub const DISTRO_CONCISE_PROMPT_OVERLAY: &str = "【发行版简洁模式】回复宜短、信息密度高；避免长段寒暄与重复上一轮内容；仍遵守人设与质量锚点。\n\n";

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::plugin_backends::{AgentBackend, LlmBackend, MemoryBackend};
    use std::io::Write;

    #[test]
    fn parse_vscode_profile_skips_agent_and_ce() {
        let dir = std::env::temp_dir().join(format!("oclive_host_profile_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("distro.oclive.toml");
        let raw = r#"
schema_version = 1
distro_id = "vscode"
[host_flags]
skip_agent = true
skip_complex_emotion = true
[prompt]
profile = "concise"
[plugin_backends]
memory = "builtin"
llm = "ollama"
"#;
        let mut f = std::fs::File::create(&file).unwrap();
        f.write_all(raw.as_bytes()).unwrap();
        let p = load_host_profile_file(&file).unwrap();
        assert_eq!(p.distro_id, "vscode");
        assert!(p.skip_agent);
        assert!(p.skip_complex_emotion);
        assert_eq!(p.prompt_profile, PromptProfile::Concise);
        let ceiling = p.backends_ceiling.as_ref().unwrap();
        assert_eq!(ceiling.memory, MemoryBackend::Builtin);
        assert_eq!(ceiling.llm, LlmBackend::Ollama);
        assert_eq!(ceiling.agent, AgentBackend::Builtin);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn apply_ceiling_replaces_role_backends() {
        use crate::models::plugin_backends::{
            EmotionBackend, EventBackend, PluginBackends, PromptBackend,
        };
        use crate::state::host_backends::apply_host_ceiling;
        let role = PluginBackends {
            memory: MemoryBackend::Remote,
            emotion: EmotionBackend::Remote,
            event: EventBackend::Remote,
            prompt: PromptBackend::Remote,
            llm: LlmBackend::Remote,
            agent: AgentBackend::Remote,
            ..Default::default()
        };
        let host = HostProfile {
            backends_ceiling: Some(PluginBackends {
                memory: MemoryBackend::Builtin,
                emotion: EmotionBackend::Builtin,
                event: EventBackend::Builtin,
                prompt: PromptBackend::Builtin,
                llm: LlmBackend::Ollama,
                agent: AgentBackend::Builtin,
                ..Default::default()
            }),
            ..HostProfile::default()
        };
        let eff = apply_host_ceiling(&role, &host);
        assert_eq!(eff.memory, MemoryBackend::Builtin);
        assert_eq!(eff.llm, LlmBackend::Ollama);
    }

    #[test]
    fn parse_user_identity_and_post_process_sections() {
        let dir = std::env::temp_dir().join(format!("oclive_host_profile2_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("distro.oclive.toml");
        let raw = r#"
distro_id = "vscode"
[user_identity]
default_id = "classmate"
allowed_ids = ["classmate", "friend"]
[post_process]
chain = "minimal"
"#;
        let mut f = std::fs::File::create(&file).unwrap();
        f.write_all(raw.as_bytes()).unwrap();
        let p = load_host_profile_file(&file).unwrap();
        assert_eq!(p.distro_id, "vscode");
        assert_eq!(p.user_identity.default_id.as_deref(), Some("classmate"));
        assert_eq!(
            p.user_identity.allowed_ids.as_ref().map(|v| v.len()),
            Some(2)
        );
        assert_eq!(p.post_process.chain, PostProcessChain::Minimal);
        let _ = std::fs::remove_dir_all(dir);
    }
}
