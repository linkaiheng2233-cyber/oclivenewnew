//! Distro capability profile — loaded from `distro.oclive.toml` (P1 contract, P4 runtime).

use crate::error::AppError;
use crate::models::plugin_backends::{
    AgentBackend, EmotionBackend, EventBackend, LlmBackend, MemoryBackend, PluginBackends,
    PromptBackend,
};
use oclive_kernel_runtime::distro_oclive_file::{
    parse_distro_oclive_file, DistroOcliveFile, PluginBackendsToml,
};
use std::path::{Path, PathBuf};

pub const ENV_DISTRO_ID: &str = "OCLIVE_DISTRO_ID";
pub const ENV_DISTRO_PROFILE: &str = "OCLIVE_DISTRO_PROFILE";
pub const ENV_THEATER_DIRECTOR_PLUGIN: &str = "OCLIVE_THEATER_DIRECTOR_PLUGIN";

/// Distro theater scene director directory plugin id.
#[derive(Debug, Clone, Default)]
pub struct TheaterProfile {
    pub director_plugin: Option<String>,
}

/// Distro default User Identity Prompt Template id (when session has no explicit choice).
#[derive(Debug, Clone, Default)]
pub struct UserIdentityProfile {
    pub default_id: Option<String>,
    pub allowed_ids: Option<Vec<String>>,
}

/// Distro interaction mode defaults and UX hints for official releases.
#[derive(Debug, Clone)]
pub struct InteractionProfile {
    pub default_mode: oclive_kernel_types::InteractionMode,
    pub allow_mode_switch: bool,
    pub immersive_unlock_hint_after_turns: u32,
}

impl Default for InteractionProfile {
    fn default() -> Self {
        Self {
            default_mode: oclive_kernel_types::InteractionMode::PureChat,
            allow_mode_switch: true,
            immersive_unlock_hint_after_turns: 10,
        }
    }
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

/// Favor-tier expression hints for Prompt status summary (optional distro overlay).
#[derive(Debug, Clone, Default)]
pub struct StateExpressionProfile {
    pub favor_high: Option<String>,
    pub favor_mid: Option<String>,
    pub favor_low: Option<String>,
}

impl StateExpressionProfile {
    #[must_use]
    pub fn hint_for_favor(&self, favorability: f64) -> &str {
        let score = favorability.clamp(0.0, 100.0);
        if score >= 65.0 {
            self.favor_high.as_deref().unwrap_or("")
        } else if score >= 40.0 {
            self.favor_mid.as_deref().unwrap_or("")
        } else {
            self.favor_low.as_deref().unwrap_or("")
        }
    }
}

/// Effective host policy for kernel runtime (process-level, v1).
#[derive(Debug, Clone)]
pub struct HostProfile {
    pub distro_id: String,
    pub skip_agent: bool,
    pub skip_complex_emotion: bool,
    /// When false, skip pre-LLM `generate_tag` for event impact (rules-only `EventDetector`).
    pub event_impact_llm: bool,
    pub prompt_profile: PromptProfile,
    pub backends_ceiling: Option<PluginBackends>,
    pub user_identity: UserIdentityProfile,
    pub post_process: PostProcessChainProfile,
    pub state_expression: Option<StateExpressionProfile>,
    /// Path passed to child kernel via `OCLIVE_DISTRO_PROFILE` when spawned.
    pub profile_path: Option<PathBuf>,
    /// Distro memory retrieval density (`default` = 8, `light` = 4 relevant memories).
    pub memory_retrieval: MemoryRetrievalMode,
    pub interaction: InteractionProfile,
    /// Distro visual gating: `off` | `image_only` | `stage_full` (None = no gating).
    pub visual_presentation_mode: Option<String>,
    pub theater: TheaterProfile,
    pub turn_thinking: TurnThinkingProfile,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MemoryRetrievalMode {
    #[default]
    Default,
    Light,
}

impl MemoryRetrievalMode {
    #[must_use]
    pub fn parse(s: &str) -> Self {
        if s.eq_ignore_ascii_case("light") {
            Self::Light
        } else {
            Self::Default
        }
    }

    #[must_use]
    pub fn retrieval_limit(&self) -> usize {
        match self {
            Self::Default => 8,
            Self::Light => 4,
        }
    }
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

    #[must_use]
    pub fn is_concise(self) -> bool {
        matches!(self, Self::Concise)
    }
}

/// Per-turn latency policy (`[turn_thinking]` in `distro.oclive.toml`).
#[derive(Debug, Clone)]
pub struct TurnThinkingProfile {
    pub default: TurnThinkingDefault,
    pub fast_skip_complex_emotion: bool,
    pub auto_deep_min_chars: usize,
    pub auto_deep_keywords: Vec<String>,
    pub fast_knowledge_limit: usize,
    pub fast_memory_cap: usize,
    /// When `Some(true)`, force Deep capsule on Small+Deep when file exists; `Some(false)` blocks.
    pub deep_capsule: Option<bool>,
    /// When `Some(true)`, Deep+Ollama uses `build_prompt_segments` for llama.cpp prefix reuse.
    pub prompt_prefix_cache: Option<bool>,
}

impl Default for TurnThinkingProfile {
    fn default() -> Self {
        Self {
            default: TurnThinkingDefault::Auto,
            fast_skip_complex_emotion: true,
            auto_deep_min_chars: 80,
            auto_deep_keywords: vec!["认真".into(), "很重要".into(), "别敷衍".into()],
            fast_knowledge_limit: 4,
            fast_memory_cap: 4,
            deep_capsule: None,
            prompt_prefix_cache: None,
        }
    }
}

#[must_use]
pub fn prompt_prefix_cache_effective(host: &HostProfile) -> bool {
    if std::env::var("OCLIVE_PROMPT_PREFIX_CACHE")
        .ok()
        .is_some_and(|v| v == "1" || v.eq_ignore_ascii_case("true"))
    {
        return true;
    }
    host.turn_thinking.prompt_prefix_cache == Some(true)
}

#[must_use]
pub fn bench_telemetry_enabled() -> bool {
    std::env::var("OCLIVE_BENCH_TELEMETRY")
        .ok()
        .is_some_and(|v| v == "1" || v.eq_ignore_ascii_case("true"))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TurnThinkingDefault {
    #[default]
    Auto,
    Fast,
    Deep,
}

impl TurnThinkingDefault {
    #[must_use]
    pub fn parse(s: &str) -> Self {
        match s.trim().to_ascii_lowercase().as_str() {
            "fast" => Self::Fast,
            "deep" => Self::Deep,
            _ => Self::Auto,
        }
    }
}

fn host_profile_from_distro_file(
    file: &DistroOcliveFile,
) -> std::result::Result<HostProfile, AppError> {
    let mut profile = HostProfile::default();
    if let Some(ref id) = file.distro_id {
        profile.distro_id = id.clone();
    }
    let (skip_agent, skip_complex_emotion) = file.parse_distro_skip_flags();
    profile.skip_agent = skip_agent;
    profile.skip_complex_emotion = skip_complex_emotion;
    if let Some(ref hf) = file.host_flags {
        if let Some(v) = hf.event_impact_llm {
            profile.event_impact_llm = v;
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
    if let Some(ref se) = file.state_expression {
        profile.state_expression = Some(StateExpressionProfile {
            favor_high: se
                .favor_high
                .as_ref()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty()),
            favor_mid: se
                .favor_mid
                .as_ref()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty()),
            favor_low: se
                .favor_low
                .as_ref()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty()),
        });
    }
    if let Some(ref mem) = file.memory {
        if let Some(ref mode) = mem.retrieval {
            profile.memory_retrieval = MemoryRetrievalMode::parse(mode);
        }
    }
    if let Some(ref ix) = file.interaction {
        if let Some(ref mode) = ix.default_mode {
            profile.interaction.default_mode =
                oclive_kernel_types::InteractionMode::seed_default(Some(mode.as_str()), None);
        }
        if let Some(allow) = ix.allow_mode_switch {
            profile.interaction.allow_mode_switch = allow;
        }
        if let Some(turns) = ix.immersive_unlock_hint_after_turns {
            profile.interaction.immersive_unlock_hint_after_turns = turns.max(1);
        }
    }
    if let Some(ref vp) = file.visual_presentation {
        profile.visual_presentation_mode = vp
            .mode
            .as_ref()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
    }
    if let Some(ref th) = file.theater {
        profile.theater.director_plugin = th
            .director_plugin
            .as_ref()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
    }
    if let Some(ref tt) = file.turn_thinking {
        if let Some(ref d) = tt.default {
            profile.turn_thinking.default = TurnThinkingDefault::parse(d);
        }
        if let Some(v) = tt.fast_skip_complex_emotion {
            profile.turn_thinking.fast_skip_complex_emotion = v;
        }
        if let Some(n) = tt.auto_deep_min_chars {
            profile.turn_thinking.auto_deep_min_chars = n.max(20);
        }
        if let Some(ref kws) = tt.auto_deep_keywords {
            profile.turn_thinking.auto_deep_keywords = kws
                .iter()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }
        if let Some(n) = tt.fast_knowledge_limit {
            profile.turn_thinking.fast_knowledge_limit = n.clamp(1, 8);
        }
        if let Some(n) = tt.fast_memory_cap {
            profile.turn_thinking.fast_memory_cap = n.clamp(1, 8);
        }
        if let Some(v) = tt.deep_capsule {
            profile.turn_thinking.deep_capsule = Some(v);
        }
        if let Some(v) = tt.prompt_prefix_cache {
            profile.turn_thinking.prompt_prefix_cache = Some(v);
        }
    }
    Ok(profile)
}

impl Default for HostProfile {
    fn default() -> Self {
        Self {
            distro_id: "default".into(),
            skip_agent: false,
            skip_complex_emotion: false,
            event_impact_llm: true,
            prompt_profile: PromptProfile::Full,
            backends_ceiling: None,
            user_identity: UserIdentityProfile::default(),
            post_process: PostProcessChainProfile::default(),
            state_expression: None,
            profile_path: None,
            memory_retrieval: MemoryRetrievalMode::Default,
            interaction: InteractionProfile::default(),
            visual_presentation_mode: None,
            theater: TheaterProfile::default(),
            turn_thinking: TurnThinkingProfile::default(),
        }
    }
}

/// Whether theater director plugins should be indexed for the active host profile.
#[must_use]
pub fn theater_director_enabled(profile: &HostProfile) -> bool {
    profile.theater_director_enabled()
}

impl HostProfile {
    /// Whether this distro profile (or env override) enables the theater director directory plugin.
    #[must_use]
    pub fn theater_director_enabled(&self) -> bool {
        if self
            .theater
            .director_plugin
            .as_ref()
            .is_some_and(|s| !s.trim().is_empty())
        {
            return true;
        }
        std::env::var(ENV_THEATER_DIRECTOR_PLUGIN)
            .ok()
            .is_some_and(|s| !s.trim().is_empty())
    }

    #[must_use]
    pub fn state_expression_hint(&self, favorability: f64) -> &str {
        self.state_expression
            .as_ref()
            .map(|s| s.hint_for_favor(favorability))
            .unwrap_or("")
    }

    /// Effective profile summary for `/health` and kernel scheduling (runtime truth).
    #[must_use]
    pub fn active_profile_summary(&self) -> Option<oclive_kernel_types::ActiveProfileSummary> {
        let loaded = self.profile_path.is_some()
            || (!self.distro_id.is_empty() && self.distro_id != "default");
        if !loaded {
            return None;
        }

        let mut enabled = vec![
            "memory".into(),
            "emotion".into(),
            "event".into(),
            "prompt".into(),
            "llm".into(),
        ];
        let mut disabled = Vec::new();
        if self.skip_agent {
            disabled.push("agent".into());
            enabled.retain(|m| m != "agent");
        } else {
            enabled.push("agent".into());
        }
        if self.skip_complex_emotion {
            disabled.push("complex_emotion".into());
            enabled.retain(|m| m != "complex_emotion");
        } else {
            enabled.push("complex_emotion".into());
        }

        let post_process_profile = match self.post_process.chain {
            PostProcessChain::Minimal => Some("minimal".into()),
            PostProcessChain::Standard => Some("standard".into()),
        };
        let prompt_profile = match self.prompt_profile {
            PromptProfile::Concise => Some("concise".into()),
            PromptProfile::Full => Some("full".into()),
        };

        Some(oclive_kernel_types::ActiveProfileSummary {
            distro_id: Some(self.distro_id.clone()),
            enabled_modules: enabled,
            disabled_modules: disabled,
            post_process_profile,
            prompt_profile,
            default_interaction_mode: Some(self.interaction.default_mode.as_str().to_string()),
            allow_mode_switch: self.interaction.allow_mode_switch,
            immersive_unlock_hint_after_turns: self.interaction.immersive_unlock_hint_after_turns,
        })
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
pub fn load_host_profile_file(path: &Path) -> std::result::Result<HostProfile, String> {
    let file = parse_distro_oclive_file(path)?;
    host_profile_from_distro_file(&file).map_err(|e| e.to_string())
}

fn parse_plugin_backends_toml(
    pb: &PluginBackendsToml,
) -> std::result::Result<PluginBackends, AppError> {
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

fn parse_memory(s: Option<&str>) -> std::result::Result<MemoryBackend, AppError> {
    match s {
        None => Ok(MemoryBackend::Builtin),
        Some(v) => match v {
            "builtin" => Ok(MemoryBackend::Builtin),
            "builtin_v2" => Ok(MemoryBackend::Builtin),
            "remote" => Ok(MemoryBackend::Remote),
            "local" => Ok(MemoryBackend::Local),
            "directory" => Ok(MemoryBackend::Directory),
            "none" => Ok(MemoryBackend::None),
            other => Err(AppError::InvalidParameter(format!(
                "unknown memory backend: {other}"
            ))),
        },
    }
}

fn parse_emotion(s: Option<&str>) -> std::result::Result<EmotionBackend, AppError> {
    match s {
        None => Ok(EmotionBackend::Builtin),
        Some(v) => match v {
            "builtin" => Ok(EmotionBackend::Builtin),
            "builtin_v2" => Ok(EmotionBackend::Builtin),
            "remote" => Ok(EmotionBackend::Remote),
            "directory" => Ok(EmotionBackend::Directory),
            "none" => Ok(EmotionBackend::None),
            other => Err(AppError::InvalidParameter(format!(
                "unknown emotion backend: {other}"
            ))),
        },
    }
}

fn parse_event(s: Option<&str>) -> std::result::Result<EventBackend, AppError> {
    match s {
        None => Ok(EventBackend::Builtin),
        Some(v) => match v {
            "builtin" => Ok(EventBackend::Builtin),
            "builtin_v2" => Ok(EventBackend::Builtin),
            "remote" => Ok(EventBackend::Remote),
            "directory" => Ok(EventBackend::Directory),
            "none" => Ok(EventBackend::None),
            other => Err(AppError::InvalidParameter(format!(
                "unknown event backend: {other}"
            ))),
        },
    }
}

fn parse_prompt(s: Option<&str>) -> std::result::Result<PromptBackend, AppError> {
    match s {
        None => Ok(PromptBackend::Builtin),
        Some(v) => match v {
            "builtin" => Ok(PromptBackend::Builtin),
            "builtin_v2" => Ok(PromptBackend::Builtin),
            "remote" => Ok(PromptBackend::Remote),
            "directory" => Ok(PromptBackend::Directory),
            "none" => Ok(PromptBackend::None),
            other => Err(AppError::InvalidParameter(format!(
                "unknown prompt backend: {other}"
            ))),
        },
    }
}

fn parse_llm(s: Option<&str>) -> std::result::Result<LlmBackend, AppError> {
    match s {
        None => Ok(LlmBackend::Ollama),
        Some(v) => match v {
            "ollama" => Ok(LlmBackend::Ollama),
            "remote" => Ok(LlmBackend::Remote),
            "directory" => Ok(LlmBackend::Directory),
            "none" => Ok(LlmBackend::None),
            other => Err(AppError::InvalidParameter(format!(
                "unknown llm backend: {other}"
            ))),
        },
    }
}

fn parse_agent(s: Option<&str>) -> std::result::Result<AgentBackend, AppError> {
    match s {
        None => Ok(AgentBackend::Builtin),
        Some(v) => match v {
            "builtin" => Ok(AgentBackend::Builtin),
            "remote" => Ok(AgentBackend::Remote),
            "directory" => Ok(AgentBackend::Directory),
            "none" => Ok(AgentBackend::None),
            other => Err(AppError::InvalidParameter(format!(
                "unknown agent backend: {other}"
            ))),
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
    fn active_profile_summary_from_loaded_vscode_profile() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../examples/distro-profiles");
        let mut p = load_host_profile_file(&root.join("vscode.oclive.toml")).unwrap();
        p.profile_path = Some(root.join("vscode.oclive.toml"));
        let summary = p.active_profile_summary().expect("summary");
        assert_eq!(summary.distro_id.as_deref(), Some("vscode"));
        assert!(summary.disabled_modules.contains(&"agent".to_string()));
        assert_eq!(summary.prompt_profile.as_deref(), Some("concise"));
        assert_eq!(
            summary.default_interaction_mode.as_deref(),
            Some("pure_chat")
        );
        assert_eq!(summary.immersive_unlock_hint_after_turns, 10);
    }

    #[test]
    fn desktop_chat_profile_pure_chat_default() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../examples/distro-profiles");
        let p = load_host_profile_file(&root.join("desktop-chat.oclive.toml")).unwrap();
        assert_eq!(p.distro_id, "desktop-chat");
        assert!(!p.interaction.default_mode.is_immersive());
        assert_eq!(p.interaction.immersive_unlock_hint_after_turns, 10);
    }

    #[test]
    fn active_profile_summary_none_for_default_host() {
        let p = HostProfile::default();
        assert!(p.active_profile_summary().is_none());
    }

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

    #[test]
    fn parse_state_expression_section() {
        let dir = std::env::temp_dir().join(format!("oclive_host_profile3_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("distro.oclive.toml");
        let raw = r#"
distro_id = "vscode"
[state_expression]
favor_high = "更信任用户的技术判断，少寒暄"
favor_mid = "保持友好但不越界"
favor_low = "保持距离与礼貌"
"#;
        let mut f = std::fs::File::create(&file).unwrap();
        f.write_all(raw.as_bytes()).unwrap();
        let p = load_host_profile_file(&file).unwrap();
        let se = p.state_expression.as_ref().unwrap();
        assert_eq!(se.hint_for_favor(70.0), "更信任用户的技术判断，少寒暄");
        assert_eq!(se.hint_for_favor(50.0), "保持友好但不越界");
        assert_eq!(se.hint_for_favor(20.0), "保持距离与礼貌");
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn desktop_and_vscode_state_expression_differ_at_same_favor() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../examples/distro-profiles");
        let vscode = load_host_profile_file(&root.join("vscode.oclive.toml")).unwrap();
        let desktop = load_host_profile_file(&root.join("desktop.oclive.toml")).unwrap();
        let favor = 70.0;
        assert_ne!(
            vscode.state_expression_hint(favor),
            desktop.state_expression_hint(favor)
        );
    }
}
