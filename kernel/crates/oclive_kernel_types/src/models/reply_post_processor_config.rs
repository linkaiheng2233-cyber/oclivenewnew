//! Role pack `config.json` → `reply_post_processor` (parallel to `memory`; not a six-slot).

use serde::{Deserialize, Serialize};

fn default_reply_post_processor_backend() -> ReplyPostProcessorBackendKind {
    ReplyPostProcessorBackendKind::Builtin
}

fn default_builtin_profile() -> String {
    "standard".to_string()
}

/// `config.json` → `reply_post_processor`
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RolePackReplyPostProcessorConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_reply_post_processor_backend")]
    pub backend: ReplyPostProcessorBackendKind,
    #[serde(default)]
    pub builtin: RolePackBuiltinReplyPostProcessorConfig,
    #[serde(default)]
    pub remote: RolePackRemoteReplyPostProcessorConfig,
    #[serde(default)]
    pub directory: RolePackDirectoryReplyPostProcessorConfig,
}

impl Default for RolePackReplyPostProcessorConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            backend: ReplyPostProcessorBackendKind::Builtin,
            builtin: RolePackBuiltinReplyPostProcessorConfig::default(),
            remote: RolePackRemoteReplyPostProcessorConfig::default(),
            directory: RolePackDirectoryReplyPostProcessorConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ReplyPostProcessorBackendKind {
    #[default]
    Builtin,
    Remote,
    Directory,
}

/// `reply_post_processor.builtin`
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RolePackBuiltinReplyPostProcessorConfig {
    #[serde(default = "default_builtin_profile")]
    pub profile: String,
    #[serde(default)]
    pub max_chars: Option<u32>,
    #[serde(default)]
    pub strip_leading_quote: Option<bool>,
}

impl Default for RolePackBuiltinReplyPostProcessorConfig {
    fn default() -> Self {
        Self {
            profile: default_builtin_profile(),
            max_chars: None,
            strip_leading_quote: None,
        }
    }
}

/// `reply_post_processor.remote` (Phase 2 wiring; validated at pack time).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct RolePackRemoteReplyPostProcessorConfig {
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub timeout_ms: Option<u32>,
}

/// `reply_post_processor.directory` (Phase 2 wiring; validated at pack time).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct RolePackDirectoryReplyPostProcessorConfig {
    #[serde(default)]
    pub plugin_id: String,
}
