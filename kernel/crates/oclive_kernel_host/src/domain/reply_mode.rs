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
    cfg.fallback_leads = cfg.sanitized_leads();
    Some(cfg)
}

fn push_segment(segments: &mut Vec<String>, current: &mut String) {
    let segment = current.trim().to_string();
    if !segment.is_empty() {
        segments.push(segment);
    }
    current.clear();
}

/// True when a line is a segment boundary: its trimmed content either equals the
/// separator exactly, or is the separator followed only by trailing punctuation
/// (small models often append 。 / . / ! to a standalone marker line).
fn is_separator_boundary(line: &str, separator: &str) -> bool {
    let t = line.trim();
    if t == separator {
        return true;
    }
    if let Some(rest) = t.strip_prefix(separator) {
        if !rest.is_empty()
            && rest.chars().all(|c| {
                matches!(
                    c,
                    '。' | '，' | '！' | '？' | '…' | '、' | '.' | ',' | '!' | '?' | ';' | '~'
                )
            })
        {
            return true;
        }
    }
    false
}

/// Sentence-terminal characters that may directly precede a burst lead-in when
/// a weak local model merges two bursts into one paragraph.
const BURST_LEAD_PRECEDERS: &[char] = &['。', '！', '？', '!', '?', '…', '；', ';'];

/// Byte offset of the first pack-declared burst lead that sits at a sentence or
/// line boundary, or `None` when no such lead is present.
fn burst_lead_boundary(text: &str, leads: &[String]) -> Option<usize> {
    if leads.is_empty() {
        return None;
    }
    let chars: Vec<char> = text.chars().collect();
    let lead_chars: Vec<Vec<char>> = leads.iter().map(|l| l.chars().collect()).collect();
    let mut byte = 0usize;
    for (i, ch) in chars.iter().enumerate() {
        if i >= 1 {
            let prev = chars[i - 1];
            if prev == '\n' || BURST_LEAD_PRECEDERS.contains(&prev) {
                for lc in &lead_chars {
                    if chars[i..].starts_with(lc) {
                        return Some(byte);
                    }
                }
            }
        }
        byte += ch.len_utf8();
    }
    None
}

/// When a line ends with the separator right after a sentence-terminal
/// character (weak models often append the marker to the end of the last burst
/// line), return the prefix to keep so the marker can be stripped. C++-style
/// text is untouched because its suffix is not preceded by a terminal.
fn trailing_marker_prefix<'a>(line: &'a str, separator: &str) -> Option<&'a str> {
    let t = line.trim_end();
    if !t.ends_with(separator) {
        return None;
    }
    let pre = t[..t.len() - separator.len()].trim_end();
    if pre.is_empty() {
        return None;
    }
    let last = pre.chars().last()?;
    if BURST_LEAD_PRECEDERS.contains(&last) {
        Some(pre)
    } else {
        None
    }
}

/// Cap segment count by merging overflow into the last segment.
fn cap_and_merge(mut segments: Vec<String>, max_segments: usize) -> Vec<String> {
    if segments.len() > max_segments {
        let tail = segments[max_segments - 1..].join("\n\n");
        segments.truncate(max_segments - 1);
        segments.push(tail);
    }
    segments
}

/// Split one model reply into presentation segments on standalone separator lines.
///
/// A line whose trimmed content exactly equals `separator` (or the separator
/// followed only by trailing punctuation) is a boundary. When the model never
/// emits the separator protocol, two degradations apply in order: blank-line
/// paragraphs, then pack-declared burst lead-ins at sentence boundaries.
/// Extra segments beyond `max_segments` are merged into the last segment, and
/// empty segments are dropped. An invalid separator or `max_segments <= 1`
/// returns the whole trimmed reply as one segment.
#[must_use]
pub fn split_reply_segments(
    raw: &str,
    separator: &str,
    max_segments: usize,
    leads: &[String],
) -> Vec<String> {
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

    // 1) Standalone separator lines are the primary protocol.
    let mut segments: Vec<String> = Vec::with_capacity(max_segments.min(MAX_REPLY_SEGMENTS));
    let mut current = String::new();
    let mut saw_boundary = false;
    for line in normalized.split('\n') {
        if let Some(prefix) = trailing_marker_prefix(line, separator) {
            if !current.is_empty() {
                current.push('\n');
            }
            current.push_str(prefix);
            saw_boundary = true;
            push_segment(&mut segments, &mut current);
        } else if is_separator_boundary(line, separator) {
            saw_boundary = true;
            push_segment(&mut segments, &mut current);
        } else {
            if !current.is_empty() {
                current.push('\n');
            }
            current.push_str(line);
        }
    }
    push_segment(&mut segments, &mut current);
    if saw_boundary {
        return cap_and_merge(segments, max_segments);
    }

    // 2) Degradation: blank-line paragraphs. Weak local models often separate
    //    their bursts with an empty line instead of the separator protocol.
    let paragraphs: Vec<String> = normalized
        .split("\n\n")
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect();
    if paragraphs.len() > 1 {
        return cap_and_merge(paragraphs, max_segments);
    }

    // 3) Degradation: pack-declared burst lead-ins at sentence boundaries
    //    (the model wrote the second burst inline, e.g. ……。——而且，……).
    if let Some(i) = burst_lead_boundary(&normalized, leads) {
        let (head, tail) = normalized.split_at(i);
        let head = head.trim().to_string();
        let tail = tail.trim().to_string();
        let mut segs = Vec::with_capacity(2);
        if !head.is_empty() {
            segs.push(head);
        }
        if !tail.is_empty() {
            segs.push(tail);
        }
        return cap_and_merge(segs, max_segments);
    }

    let whole = normalized.trim().to_string();
    if whole.is_empty() {
        Vec::new()
    } else {
        vec![whole]
    }
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
    let segments = split_reply_segments(raw, &cfg.separator, cfg.segments, &cfg.sanitized_leads());
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
///
/// Small local models follow "absolute prohibition" phrasing more reliably than
/// gentle phrasing or worked examples, so the instruction states the rule twice
/// and never invites the model to reason about the protocol.
#[must_use]
pub fn reply_output_format_instruction(segments: usize, separator: &str) -> String {
    format!(
        "你的回复必须分成 {segments} 段。每一段写完后必须换行，单独输出一行分隔符（这一行只有分隔符本身，不允许添加任何文字或标点）：\n{separator}\n然后再换行写下一段。绝对不允许把各段连成一段，绝对不允许省略分隔符这一行。"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn no_leads() -> Vec<String> {
        Vec::new()
    }

    #[test]
    fn splits_on_standalone_separator() {
        let raw = "第一发\n\n+++\n\n第二发";
        assert_eq!(
            split_reply_segments(raw, "+++", 2, &no_leads()),
            vec!["第一发", "第二发"]
        );
    }

    #[test]
    fn ignores_inline_separator_occurrences() {
        let raw = "C+++ 代码\n\na +++ b\n\n+++\n\n第二发";
        assert_eq!(
            split_reply_segments(raw, "+++", 2, &no_leads()),
            vec!["C+++ 代码\n\na +++ b", "第二发"]
        );
    }

    #[test]
    fn normalizes_crlf() {
        let raw = "第一发\r\n\r\n+++\r\n\r\n第二发";
        assert_eq!(
            split_reply_segments(raw, "+++", 2, &no_leads()),
            vec!["第一发", "第二发"]
        );
    }

    #[test]
    fn missing_separator_returns_single_segment() {
        let raw = "只有一段，第二发没有来。";
        assert_eq!(split_reply_segments(raw, "+++", 2, &no_leads()), vec![raw]);
    }

    #[test]
    fn caps_and_merges_overflow() {
        let raw = "一\n+++\n二\n+++\n三\n+++\n四";
        assert_eq!(
            split_reply_segments(raw, "+++", 2, &no_leads()),
            vec!["一", "二\n\n三\n\n四"]
        );
    }

    #[test]
    fn drops_empty_segments() {
        let raw = "第一发\n\n+++\n\n+++\n\n第二发";
        assert_eq!(
            split_reply_segments(raw, "+++", 3, &no_leads()),
            vec!["第一发", "第二发"]
        );
    }

    #[test]
    fn empty_raw_returns_empty_segments() {
        assert_eq!(
            split_reply_segments("  \n \n", "+++", 2, &no_leads()),
            Vec::<String>::new()
        );
    }

    #[test]
    fn supports_custom_unicode_separator() {
        let raw = "第一发\n【二发】\n第二发";
        assert_eq!(
            split_reply_segments(raw, "【二发】", 2, &no_leads()),
            vec!["第一发", "第二发"]
        );
    }

    #[test]
    fn splits_on_separator_with_trailing_punctuation() {
        let raw = "第一发\n\n+++。\n\n第二发";
        assert_eq!(
            split_reply_segments(raw, "+++", 2, &no_leads()),
            vec!["第一发", "第二发"]
        );
        let raw2 = "第一发\n+++!\n第二发";
        assert_eq!(
            split_reply_segments(raw2, "+++", 2, &no_leads()),
            vec!["第一发", "第二发"]
        );
    }

    #[test]
    fn does_not_split_on_separator_with_non_punctuation_suffix() {
        // Single line: the +++abc suffix is not a separator boundary and there
        // are no blank-line paragraphs, so the reply stays one segment.
        let raw = "C+++代码 +++abc 正文";
        assert_eq!(split_reply_segments(raw, "+++", 2, &no_leads()), vec![raw]);
    }

    #[test]
    fn blank_line_fallback_splits_when_separator_absent() {
        let raw = "——晚上好，孩子。射击场的氛围让人振奋。\n\n而且，今天打靶表现如何？";
        assert_eq!(
            split_reply_segments(raw, "+++", 2, &no_leads()),
            vec![
                "——晚上好，孩子。射击场的氛围让人振奋。",
                "而且，今天打靶表现如何？"
            ]
        );
    }

    #[test]
    fn lead_phrase_fallback_splits_inline_second_burst() {
        let raw = "——听到这话，我感到欣慰。——而且，射击需要默契。";
        let leads = vec!["——".to_string(), "而且".to_string()];
        assert_eq!(
            split_reply_segments(raw, "+++", 2, &leads),
            vec!["——听到这话，我感到欣慰。", "——而且，射击需要默契。"]
        );
    }

    #[test]
    fn lead_phrase_requires_sentence_boundary() {
        let raw = "射击而且配合很重要。";
        let leads = vec!["而且".to_string()];
        assert_eq!(split_reply_segments(raw, "+++", 2, &leads), vec![raw]);
    }

    #[test]
    fn strips_trailing_marker_at_end_of_last_burst_line() {
        let raw = "第一发\n——而且，第二发。+++\n——不过，第三发。";
        let leads = vec!["——".to_string(), "而且".to_string()];
        assert_eq!(
            split_reply_segments(raw, "+++", 2, &leads),
            vec!["第一发\n——而且，第二发。", "——不过，第三发。"]
        );
    }

    #[test]
    fn does_not_strip_cpp_style_suffix_as_marker() {
        let raw = "C+++\n正文";
        assert_eq!(
            split_reply_segments(raw, "+++", 2, &no_leads()),
            vec!["C+++\n正文"]
        );
    }

    #[test]
    fn separator_protocol_wins_over_fallbacks() {
        let raw = "第一发\n+++\n第二发";
        let leads = vec!["而且".to_string()];
        assert_eq!(
            split_reply_segments(raw, "+++", 2, &leads),
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
        assert!(instruction.contains("绝对不允许省略分隔符"));
    }
}
