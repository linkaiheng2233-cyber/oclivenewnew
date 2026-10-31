//! Display-only stream hygiene. The unchanged raw outcome is parsed by the parent.

use super::{
    EMO_MARKER_CLOSE, EMO_MARKER_OPEN, INTERNAL_STATUS_MARKERS, LABEL_PREFIXES, MALFORMED_OPENERS,
};
use oclive_kernel_contracts::LlmTokenSink;
use std::sync::{Arc, Mutex};

#[derive(Default)]
enum State {
    #[default]
    Text,
    EmotionBlock,
    MalformedTail,
}

#[derive(Default)]
struct DisplayStream {
    state: State,
    pending: String,
}

fn openers() -> impl Iterator<Item = &'static str> {
    std::iter::once(EMO_MARKER_OPEN)
        .chain(MALFORMED_OPENERS)
        .chain(LABEL_PREFIXES)
        .chain(INTERNAL_STATUS_MARKERS)
}

/// Number of trailing UTF-8 bytes that could still become a reserved delimiter.
fn ambiguous_suffix(text: &str, markers: impl Iterator<Item = &'static str>) -> usize {
    markers
        .flat_map(|marker| {
            marker
                .char_indices()
                .skip(1)
                .map(move |(end, _)| &marker[..end])
        })
        .filter(|prefix| text.ends_with(prefix))
        .map(str::len)
        .max()
        .unwrap_or(0)
}

impl DisplayStream {
    fn push(&mut self, token: &str) -> String {
        if matches!(self.state, State::MalformedTail) {
            return String::new();
        }
        self.pending.push_str(token);
        let mut visible = String::new();
        loop {
            match self.state {
                State::Text => {
                    let next = openers()
                        .filter_map(|marker| self.pending.find(marker).map(|at| (at, marker)))
                        .min_by_key(|(at, _)| *at);
                    if let Some((at, marker)) = next {
                        visible.push_str(&self.pending[..at]);
                        self.pending.drain(..at + marker.len());
                        if marker == EMO_MARKER_OPEN {
                            self.state = State::EmotionBlock;
                        } else if !INTERNAL_STATUS_MARKERS.contains(&marker) {
                            // Malformed marker/JSON is withheld; authoritative final parsing
                            // restores legitimate quoted text that only looked like metadata.
                            self.pending.clear();
                            self.state = State::MalformedTail;
                            break;
                        }
                    } else {
                        let held = ambiguous_suffix(&self.pending, openers());
                        let end = self.pending.len() - held;
                        visible.push_str(&self.pending[..end]);
                        self.pending.drain(..end);
                        break;
                    }
                }
                State::EmotionBlock => {
                    if let Some(at) = self.pending.find(EMO_MARKER_CLOSE) {
                        self.pending.drain(..at + EMO_MARKER_CLOSE.len());
                        self.state = State::Text;
                    } else {
                        let held =
                            ambiguous_suffix(&self.pending, std::iter::once(EMO_MARKER_CLOSE));
                        self.pending.drain(..self.pending.len() - held);
                        break;
                    }
                }
                State::MalformedTail => break,
            }
        }
        visible
    }
}

pub(crate) fn display_token_sink(downstream: LlmTokenSink) -> LlmTokenSink {
    let filter = Mutex::new(DisplayStream::default());
    Arc::new(move |token| {
        // On poison fail closed: raw internal metadata must not become visible.
        let visible = match filter.lock() {
            Ok(mut filter) => filter.push(token),
            Err(_) => return,
        };
        if !visible.is_empty() {
            downstream(&visible);
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check_all_splits(raw: &str, expected: &str) {
        for split in raw
            .char_indices()
            .map(|(i, _)| i)
            .chain(std::iter::once(raw.len()))
        {
            let mut filter = DisplayStream::default();
            let visible = filter.push(&raw[..split]) + &filter.push(&raw[split..]);
            assert_eq!(visible, expected, "split {split} of {raw}");
            assert!(filter.pending.len() < 20);
        }
        let mut filter = DisplayStream::default();
        let visible: String = raw.chars().map(|c| filter.push(&c.to_string())).collect();
        assert_eq!(visible, expected);
    }

    #[test]
    fn hides_markers_before_any_prefix_can_flash_and_keeps_later_dialogue() {
        check_all_splits(
            "你好！[EMO]{\"labels\":[\"joy\"],\"intensity\":0.8}[/EMO]继续。",
            "你好！继续。",
        );
        check_all_splits("a[EMO]invalid[/EMO]b[EMO]{}[/EMO]c", "abc");
        check_all_splits("可见[EMO]{\"labels\":[\"joy\"]", "可见");
    }

    #[test]
    fn hides_all_supported_malformed_markers_and_status_strings() {
        for opener in MALFORMED_OPENERS.into_iter().chain(LABEL_PREFIXES) {
            check_all_splits(&format!("正文{opener}internal payload"), "正文");
        }
        for status in INTERNAL_STATUS_MARKERS {
            check_all_splits(&format!("前{status}后"), "前后");
        }
    }

    #[test]
    fn preserves_ordinary_unicode_markdown_and_json() {
        check_all_splits(
            "你好🙂 [链接](https://example.test) `x[0]` （普通说明） {\"foo\":123}",
            "你好🙂 [链接](https://example.test) `x[0]` （普通说明） {\"foo\":123}",
        );
    }

    #[test]
    fn memory_is_bounded_for_large_hidden_payloads() {
        let mut filter = DisplayStream::default();
        assert_eq!(filter.push("正文[EMO]"), "正文");
        for _ in 0..10_000 {
            assert!(filter.push("隐藏数据\"labels\": [joy]").is_empty());
            assert!(filter.pending.len() < EMO_MARKER_CLOSE.len());
        }
        assert_eq!(filter.push("[/EMO]结束"), "结束");
    }
}
