//! Reply presentation segments for the `reply_mode` side channel.
//!
//! Pure helpers shared by post-LLM segmentation and prompt assembly. No I/O and
//! no role-specific logic live here; role packs only configure values.

use crate::models::{Role, RolePackReplyModeConfig};
pub use crate::models::{
    DEFAULT_REPLY_SEGMENTS, DEFAULT_REPLY_SEPARATOR, MAX_REPLY_SEGMENTS, MAX_REPLY_SEPARATOR_CHARS,
};

/// Whether a separator is safe to match as a full-line protocol marker.
///
/// The separator must be non-empty, contain no newlines, have no leading or
/// trailing whitespace, and stay within the public length cap.
#[must_use]
pub fn valid_reply_separator(separator: &str) -> bool {
    let trimmed = separator.trim();
    !trimmed.is_empty()
        && trimmed == separator
        && !separator.contains(['\r', '\n'])
        && separator.chars().count() <= MAX_REPLY_SEPARATOR_CHARS
        && !separator.chars().all(char::is_whitespace)
}

/// Effective role pack reply mode, or `None` when presentation stays single.
/// Invalid separators are rejected and logged, never guessed.
#[must_use]
pub fn effective_reply_mode(role: &Role) -> Option<RolePackReplyModeConfig> {
    let mut cfg = role.pack_reply_mode_config.clone();
    if !cfg.enabled() {
        return None;
    }
    let separator = cfg.separator.trim().to_string();
    if !valid_reply_separator(&separator) {
        tracing::warn!(
            target: "oclive_reply_mode",
            role_id = %role.id,
            separator = %cfg.separator,
            "invalid reply_mode separator; falling back to single"
        );
        return None;
    }
    cfg.separator = separator;
    cfg.segments = cfg.effective_segments();
    Some(cfg)
}

fn push_segment(segments: &mut Vec<String>, current: &mut String) {
    let segment = current.trim().to_string();
    if !segment.is_empty() {
        segments.push(segment);
    }
    current.clear();
}

/// Split one model reply into presentation segments on standalone separator lines.
///
/// Only a line whose trimmed content exactly equals `separator` is a boundary.
/// Extra segments beyond `max_segments` are merged into the last segment, and
/// empty segments are dropped. An invalid separator or `max_segments <= 1`
/// returns the whole trimmed reply as one segment.
#[must_use]
pub fn split_reply_segments(raw: &str, separator: &str, max_segments: usize) -> Vec<String> {
    if max_segments <= 1 || !valid_reply_separator(separator) {
        let whole = raw.replace("\r\n", "\n").replace('\r', "\n");
        let trimmed = whole.trim().to_string();
        return if trimmed.is_empty() {
            Vec::new()
        } else {
            vec![trimmed]
        };
    }

    let normalized = raw.replace("\r\n", "\n").replace('\r', "\n");
    let mut segments: Vec<String> = Vec::with_capacity(max_segments.min(MAX_REPLY_SEGMENTS));
    let mut current = String::new();

    for line in normalized.split('\n') {
        if line.trim() == separator {
            push_segment(&mut segments, &mut current);
        } else {
            if !current.is_empty() {
                current.push('\n');
            }
            current.push_str(line);
        }
    }
    push_segment(&mut segments, &mut current);

    if segments.len() > max_segments {
        let tail = segments[max_segments - 1..].join("\n\n");
        segments.truncate(max_segments - 1);
        segments.push(tail);
    }
    segments
}

/// Split result ready for DTO assembly and separator-free persistence.
#[derive(Debug, Clone, PartialEq)]
pub struct ReplyModePresentation {
    pub segments: Vec<String>,
    pub delays_ms: Vec<u32>,
    pub joined: String,
}

/// Split a final display reply according to the effective role pack config.
#[must_use]
pub fn present_reply(cfg: &RolePackReplyModeConfig, raw: &str) -> ReplyModePresentation {
    let segments = split_reply_segments(raw, &cfg.separator, cfg.segments);
    let delays_ms = segments
        .iter()
        .enumerate()
        .map(|(index, _)| cfg.delay_for(index))
        .collect::<Vec<_>>();
    let joined = segments.join("\n");
    ReplyModePresentation {
        segments,
        delays_ms,
        joined,
    }
}

/// Generic output-format instruction appended to the prompt when a reply mode
/// asks the model for multiple segments.
#[must_use]
pub fn reply_output_format_instruction(segments: usize, separator: &str) -> String {
    format!(
        "本次回复需要分成 {segments} 段。每段之间，单独输出一行分隔符：\n{separator}\n分隔符前后不要添加任何文字、标点或解释。"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_on_standalone_separator() {
        let raw = "第一发\n\n+++\n\n第二发";
        assert_eq!(
            split_reply_segments(raw, "+++", 2),
            vec!["第一发", "第二发"]
        );
    }

    #[test]
    fn ignores_inline_separator_occurrences() {
        let raw = "C+++ 代码\n\na +++ b\n\n+++\n\n第二发";
        assert_eq!(
            split_reply_segments(raw, "+++", 2),
            vec!["C+++ 代码\n\na +++ b", "第二发"]
        );
    }

    #[test]
    fn normalizes_crlf() {
        let raw = "第一发\r\n\r\n+++\r\n\r\n第二发";
        assert_eq!(
            split_reply_segments(raw, "+++", 2),
            vec!["第一发", "第二发"]
        );
    }

    #[test]
    fn missing_separator_returns_single_segment() {
        let raw = "只有一段，第二发没有来。";
        assert_eq!(split_reply_segments(raw, "+++", 2), vec![raw]);
    }

    #[test]
    fn caps_and_merges_overflow() {
        let raw = "一\n+++\n二\n+++\n三\n+++\n四";
        assert_eq!(
            split_reply_segments(raw, "+++", 2),
            vec!["一", "二\n\n三\n\n四"]
        );
    }

    #[test]
    fn drops_empty_segments() {
        let raw = "第一发\n\n+++\n\n+++\n\n第二发";
        assert_eq!(
            split_reply_segments(raw, "+++", 3),
            vec!["第一发", "第二发"]
        );
    }

    #[test]
    fn empty_raw_returns_empty_segments() {
        assert_eq!(
            split_reply_segments("  \n \n", "+++", 2),
            Vec::<String>::new()
        );
    }

    #[test]
    fn supports_custom_unicode_separator() {
        let raw = "第一发\n【二发】\n第二发";
        assert_eq!(
            split_reply_segments(raw, "【二发】", 2),
            vec!["第一发", "第二发"]
        );
    }

    #[test]
    fn rejects_invalid_separators() {
        assert!(!valid_reply_separator(""));
        assert!(!valid_reply_separator("   "));
        assert!(!valid_reply_separator(" +++ "));
        assert!(!valid_reply_separator("+ +\n"));
        assert!(!valid_reply_separator(
            &"+".repeat(MAX_REPLY_SEPARATOR_CHARS + 1)
        ));
        assert!(valid_reply_separator("+++"));
        assert!(valid_reply_separator("【二发】"));
    }

    #[test]
    fn instruction_contains_protocol_values() {
        let instruction = reply_output_format_instruction(2, "|||");
        assert!(instruction.contains("分成 2 段"));
        assert!(instruction.contains("|||"));
    }
}
