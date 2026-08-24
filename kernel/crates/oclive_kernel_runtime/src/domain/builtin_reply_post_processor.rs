//! Builtin Reply Post-Processor rules (`standard` / `minimal` profiles).

use oclive_kernel_contracts::reply_post_processor::{
    PostProcessInput, PostProcessOutput, ReplyPostProcessor,
};
use oclive_kernel_types::models::RolePackBuiltinReplyPostProcessorConfig;
use oclive_kernel_types::Result;

pub struct BuiltinReplyPostProcessor {
    config: RolePackBuiltinReplyPostProcessorConfig,
}

impl BuiltinReplyPostProcessor {
    #[must_use]
    pub fn new(config: RolePackBuiltinReplyPostProcessorConfig) -> Self {
        Self { config }
    }

    fn normalize_whitespace(text: &str, minimal: bool) -> String {
        let collapsed = text
            .lines()
            .map(str::trim_end)
            .collect::<Vec<_>>()
            .join("\n");
        if minimal {
            collapsed.trim().to_string()
        } else {
            let mut out = String::new();
            let mut blank_run = 0usize;
            for line in collapsed.lines() {
                if line.trim().is_empty() {
                    blank_run += 1;
                    if blank_run <= 1 {
                        out.push('\n');
                    }
                } else {
                    blank_run = 0;
                    if !out.is_empty() && !out.ends_with('\n') {
                        out.push('\n');
                    }
                    out.push_str(line);
                }
            }
            out.trim().to_string()
        }
    }

    fn strip_leading_quote(text: &str) -> String {
        let trimmed = text.trim_start();
        if (trimmed.starts_with('「') && trimmed.contains('」'))
            || (trimmed.starts_with('『') && trimmed.contains('』'))
        {
            if let Some(end) = trimmed.find(['」', '』']) {
                let after = trimmed[end + 1..].trim_start();
                if !after.is_empty() {
                    return after.to_string();
                }
            }
        }
        text.to_string()
    }

    fn strip_echo_remainder(remainder: &str) -> Option<String> {
        let first = remainder.chars().next()?;
        if !first.is_whitespace()
            && !matches!(
                first,
                '，' | ','
                    | '。'
                    | '.'
                    | '！'
                    | '!'
                    | '？'
                    | '?'
                    | '：'
                    | ':'
                    | '；'
                    | ';'
                    | '—'
                    | '-'
                    | '」'
                    | '』'
                    | '”'
                    | '"'
                    | '\''
            )
        {
            return None;
        }
        let cleaned = remainder
            .trim_start_matches(|c: char| {
                c.is_whitespace()
                    || matches!(
                        c,
                        '，' | ','
                            | '。'
                            | '.'
                            | '！'
                            | '!'
                            | '？'
                            | '?'
                            | '：'
                            | ':'
                            | '；'
                            | ';'
                            | '—'
                            | '-'
                            | '」'
                            | '』'
                            | '”'
                            | '"'
                            | '\''
                    )
            })
            .trim();
        (!cleaned.is_empty()).then(|| cleaned.to_string())
    }

    /// Remove only a leading, exact copy of this turn's user message. The
    /// boundary requirement preserves natural replies such as user `你好` →
    /// assistant `你好呀`, while filtering `用户说：“你好。” ……` and plain
    /// `你好。……` echo openings that slipped past prompt guardrails.
    fn strip_leading_user_echo(text: &str, user_message: &str) -> String {
        let user = user_message.trim();
        if user.is_empty() {
            return text.to_string();
        }

        let mut candidate = text.trim_start();
        for label in ["用户说：", "用户说:", "用户：", "用户:", "User:", "user:"] {
            if let Some(rest) = candidate.strip_prefix(label) {
                candidate = rest.trim_start();
                break;
            }
        }
        for open in ['「', '『', '“', '"', '\''] {
            if let Some(rest) = candidate.strip_prefix(open) {
                candidate = rest.trim_start();
                break;
            }
        }
        if let Some(remainder) = candidate.strip_prefix(user) {
            if let Some(cleaned) = Self::strip_echo_remainder(remainder) {
                return cleaned;
            }
        }
        text.to_string()
    }

    fn strip_duplicate_quoted_repetitions(text: &str) -> String {
        let mut out = text.to_string();
        for (open, close) in [('「', '」'), ('『', '』'), ('【', '】')] {
            let mut scan = 0usize;
            loop {
                let Some(rel_start) = out[scan..].find(open) else {
                    break;
                };
                let start = scan + rel_start;
                let body_start = start + open.len_utf8();
                let Some(rel_end) = out[body_start..].find(close) else {
                    break;
                };
                let body_end = body_start + rel_end;
                let close_end = body_end + close.len_utf8();
                let needle = out[body_start..body_end].trim().trim_matches(|c: char| {
                    matches!(
                        c,
                        '。' | '.' | '！' | '!' | '？' | '?' | '，' | ',' | '；' | ';'
                    )
                });
                let already_seen = needle.chars().count() >= 2 && out[..start].contains(needle);
                if already_seen {
                    out.replace_range(start..close_end, "");
                    scan = start;
                } else {
                    scan = close_end;
                }
            }
        }
        Self::normalize_whitespace(&out, false)
    }
}

impl ReplyPostProcessor for BuiltinReplyPostProcessor {
    fn process_reply(&self, input: PostProcessInput<'_>) -> Result<PostProcessOutput> {
        let minimal = self.config.profile.eq_ignore_ascii_case("minimal");
        let mut out = Self::normalize_whitespace(input.raw_reply, minimal);
        if self.config.strip_leading_quote.unwrap_or(!minimal) {
            out = Self::strip_leading_quote(&out);
        }
        if !minimal {
            out = Self::strip_leading_user_echo(&out, input.user_message);
        }
        if self.config.dedupe_quoted_repetitions.unwrap_or(false) {
            out = Self::strip_duplicate_quoted_repetitions(&out);
        }
        if let Some(max) = self.config.max_chars {
            if out.chars().count() > max as usize {
                out = out.chars().take(max as usize).collect();
            }
        }
        Ok(PostProcessOutput {
            display_reply: out,
            diagnostic: None,
        })
    }
}

/// Pass-through when `reply_post_processor.enabled = false`.
pub struct PassthroughReplyPostProcessor;

impl ReplyPostProcessor for PassthroughReplyPostProcessor {
    fn process_reply(&self, input: PostProcessInput<'_>) -> Result<PostProcessOutput> {
        Ok(PostProcessOutput {
            display_reply: input.raw_reply.to_string(),
            diagnostic: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn minimal_profile_trims_outer_whitespace() {
        let p = BuiltinReplyPostProcessor::new(RolePackBuiltinReplyPostProcessorConfig {
            profile: "minimal".to_string(),
            max_chars: None,
            strip_leading_quote: None,
            dedupe_quoted_repetitions: None,
        });
        let out = p
            .process_reply(PostProcessInput {
                raw_reply: "  hello \n\n world  ",
                user_message: "",
                role_id: "r",
                scene_id: "s",
                srid: "r",
                locale: "zh",
            })
            .expect("ok");
        assert_eq!(out.display_reply, "hello\n\n world");
    }

    #[test]
    fn max_chars_truncates() {
        let p = BuiltinReplyPostProcessor::new(RolePackBuiltinReplyPostProcessorConfig {
            profile: "standard".to_string(),
            max_chars: Some(3),
            strip_leading_quote: Some(false),
            dedupe_quoted_repetitions: None,
        });
        let out = p
            .process_reply(PostProcessInput {
                raw_reply: "abcdef",
                user_message: "",
                role_id: "r",
                scene_id: "s",
                srid: "r",
                locale: "zh",
            })
            .expect("ok");
        assert_eq!(out.display_reply, "abc");
    }

    #[test]
    fn standard_profile_strips_exact_leading_user_echo() {
        let p = BuiltinReplyPostProcessor::new(RolePackBuiltinReplyPostProcessorConfig {
            profile: "standard".to_string(),
            max_chars: None,
            strip_leading_quote: Some(false),
            dedupe_quoted_repetitions: None,
        });
        for raw in [
            "晚上好哦沐沐。今天怎么突然这么乖？",
            "用户说：\"晚上好哦沐沐\"。今天怎么突然这么乖？",
        ] {
            let out = p
                .process_reply(PostProcessInput {
                    raw_reply: raw,
                    user_message: "晚上好哦沐沐",
                    role_id: "mumu",
                    scene_id: "home",
                    srid: "mumu",
                    locale: "zh",
                })
                .expect("ok");
            assert_eq!(out.display_reply, "今天怎么突然这么乖？");
        }
    }

    #[test]
    fn standard_profile_preserves_non_echo_prefix_word() {
        let p = BuiltinReplyPostProcessor::new(RolePackBuiltinReplyPostProcessorConfig {
            profile: "standard".to_string(),
            max_chars: None,
            strip_leading_quote: Some(false),
            dedupe_quoted_repetitions: None,
        });
        let out = p
            .process_reply(PostProcessInput {
                raw_reply: "你好呀，今天回来得挺早。",
                user_message: "你好",
                role_id: "mumu",
                scene_id: "home",
                srid: "mumu",
                locale: "zh",
            })
            .expect("ok");
        assert_eq!(out.display_reply, "你好呀，今天回来得挺早。");
    }

    #[test]
    fn configured_dedupe_removes_later_quoted_repetition() {
        let p = BuiltinReplyPostProcessor::new(RolePackBuiltinReplyPostProcessorConfig {
            profile: "standard".to_string(),
            max_chars: None,
            strip_leading_quote: Some(false),
            dedupe_quoted_repetitions: Some(true),
        });
        for raw in [
            "菲比啾比！你好呀。「菲比啾比！」",
            "菲比啾比。明白，只保留自然语言。【菲比啾比】",
        ] {
            let out = p
                .process_reply(PostProcessInput {
                    raw_reply: raw,
                    user_message: "",
                    role_id: "phoebe-chubi",
                    scene_id: "default",
                    srid: "phoebe-chubi",
                    locale: "zh",
                })
                .expect("ok");
            assert!(!out.display_reply.contains('「'));
            assert!(!out.display_reply.contains('【'));
            assert_eq!(out.display_reply.matches("菲比啾比").count(), 1);
        }
    }
}
