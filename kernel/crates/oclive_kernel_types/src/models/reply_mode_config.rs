//! Role pack `config.json` → `reply_mode` (independent side-channel `reply_mode`).

use serde::{Deserialize, Serialize};

pub const DEFAULT_REPLY_SEPARATOR: &str = "+++";
pub const DEFAULT_REPLY_SEGMENTS: usize = 2;
pub const MAX_REPLY_SEGMENTS: usize = 8;
pub const MAX_REPLY_SEPARATOR_CHARS: usize = 16;

fn default_reply_mode_segments() -> usize {
    DEFAULT_REPLY_SEGMENTS
}

fn default_reply_mode_separator() -> String {
    DEFAULT_REPLY_SEPARATOR.to_string()
}

fn default_reply_mode_delays() -> Vec<u32> {
    vec![0, 0]
}

/// Reply presentation strategy. v1 ships `single` and `burst`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ReplyModeKind {
    #[default]
    Single,
    Burst,
}

/// How the frontend reveals multiple segments.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ReplyModeStreaming {
    #[default]
    Live,
    Batch,
}

/// `config.json` → `reply_mode`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RolePackReplyModeConfig {
    #[serde(default)]
    pub mode: ReplyModeKind,
    #[serde(default = "default_reply_mode_segments")]
    pub segments: usize,
    #[serde(default = "default_reply_mode_separator")]
    pub separator: String,
    #[serde(default = "default_reply_mode_delays")]
    pub delays_ms: Vec<u32>,
    #[serde(default)]
    pub streaming: ReplyModeStreaming,
}

/// Read-only role info snapshot for frontend presentation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReplyModeInfoDto {
    pub mode: ReplyModeKind,
    pub segments: usize,
    pub separator: String,
    pub delays_ms: Vec<u32>,
    pub streaming: ReplyModeStreaming,
}

impl From<&RolePackReplyModeConfig> for ReplyModeInfoDto {
    fn from(cfg: &RolePackReplyModeConfig) -> Self {
        Self {
            mode: cfg.mode,
            segments: cfg.effective_segments(),
            separator: cfg.separator.clone(),
            delays_ms: cfg.delays_ms.clone(),
            streaming: cfg.streaming,
        }
    }
}

impl Default for RolePackReplyModeConfig {
    fn default() -> Self {
        Self {
            mode: ReplyModeKind::Single,
            segments: DEFAULT_REPLY_SEGMENTS,
            separator: DEFAULT_REPLY_SEPARATOR.to_string(),
            delays_ms: default_reply_mode_delays(),
            streaming: ReplyModeStreaming::Live,
        }
    }
}

impl RolePackReplyModeConfig {
    /// Whether multiple presentation segments are requested. Separator validity
    /// is checked by the host at effective-resolution time.
    #[must_use]
    pub fn enabled(&self) -> bool {
        self.mode == ReplyModeKind::Burst && self.segments > 1
    }

    /// Segment count clamped to the public protocol ceiling.
    #[must_use]
    pub fn effective_segments(&self) -> usize {
        self.segments.clamp(2, MAX_REPLY_SEGMENTS)
    }

    /// Display delay for segment `index` (zero-based); missing entries read 0.
    #[must_use]
    pub fn delay_for(&self, index: usize) -> u32 {
        self.delays_ms.get(index).copied().unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_single_and_passthrough() {
        let cfg = RolePackReplyModeConfig::default();
        assert_eq!(cfg.mode, ReplyModeKind::Single);
        assert!(!cfg.enabled());
        assert_eq!(cfg.separator, "+++");
    }

    #[test]
    fn burst_clamps_segments_and_delays() {
        let cfg = RolePackReplyModeConfig {
            mode: ReplyModeKind::Burst,
            segments: 99,
            separator: "【二发】".to_string(),
            delays_ms: vec![0, 300, 900],
            streaming: ReplyModeStreaming::Batch,
        };
        assert!(cfg.enabled());
        assert_eq!(cfg.effective_segments(), MAX_REPLY_SEGMENTS);
        assert_eq!(cfg.delay_for(0), 0);
        assert_eq!(cfg.delay_for(1), 300);
        assert_eq!(cfg.delay_for(2), 900);
        assert_eq!(cfg.delay_for(9), 0);
    }
}
