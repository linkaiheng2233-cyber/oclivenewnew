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
}

impl ReplyPostProcessor for BuiltinReplyPostProcessor {
    fn process_reply(&self, input: PostProcessInput<'_>) -> Result<PostProcessOutput> {
        let minimal = self.config.profile.eq_ignore_ascii_case("minimal");
        let mut out = Self::normalize_whitespace(input.raw_reply, minimal);
        if self.config.strip_leading_quote.unwrap_or(!minimal) {
            out = Self::strip_leading_quote(&out);
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
}
