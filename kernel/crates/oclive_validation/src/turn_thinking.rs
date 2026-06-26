//! `config.json` → `turn_thinking` validation.

use std::path::Path;

const VALID_SIGNALS: &[&str] = &[
    "long_message",
    "high_arousal",
    "high_sadness",
    "high_anger",
    "high_fear",
    "this_turn_event",
    "recent_event",
    "keyword",
    "deep_latch_active",
];

const VALID_EVENT_TYPES: &[&str] = &[
    "Quarrel",
    "Apology",
    "Praise",
    "Complaint",
    "Confession",
    "Joke",
    "Ignore",
];

fn validate_signal_rule(rule: &serde_json::Value, path: &str) -> Vec<String> {
    let mut errs = Vec::new();
    let Some(signal) = rule.get("signal").and_then(|v| v.as_str()) else {
        errs.push(format!("{path} 须含 signal 字段"));
        return errs;
    };
    if !VALID_SIGNALS.contains(&signal) {
        errs.push(format!(
            "{path}.signal 须为 {} 之一（当前「{signal}」）",
            VALID_SIGNALS.join(" | ")
        ));
        return errs;
    }
    match signal {
        "long_message" => {
            if let Some(v) = rule.get("min_chars") {
                match v.as_u64() {
                    Some(n) if n >= 1 => {}
                    _ => errs.push(format!("{path}.min_chars 须为正整数")),
                }
            }
        }
        "this_turn_event" | "recent_event" => {
            let Some(events) = rule.get("events").and_then(|v| v.as_array()) else {
                errs.push(format!("{path}.events 须为非空数组"));
                return errs;
            };
            if events.is_empty() {
                errs.push(format!("{path}.events 须至少含一项"));
            }
            for (i, ev) in events.iter().enumerate() {
                let Some(name) = ev.as_str() else {
                    errs.push(format!("{path}.events[{i}] 须为字符串"));
                    continue;
                };
                if !VALID_EVENT_TYPES.contains(&name) {
                    errs.push(format!(
                        "{path}.events[{i}] 须为 {} 之一（当前「{name}」）",
                        VALID_EVENT_TYPES.join(" | ")
                    ));
                }
            }
        }
        "keyword" => {
            let Some(keywords) = rule.get("keywords").and_then(|v| v.as_array()) else {
                errs.push(format!("{path}.keywords 须为非空数组"));
                return errs;
            };
            if keywords.is_empty() {
                errs.push(format!("{path}.keywords 须至少含一项"));
            }
        }
        _ => {}
    }
    errs
}

fn validate_event_name_list(values: &[serde_json::Value], path: &str) -> Vec<String> {
    let mut errs = Vec::new();
    for (i, v) in values.iter().enumerate() {
        let Some(name) = v.as_str() else {
            errs.push(format!("{path}[{i}] 须为字符串"));
            continue;
        };
        if !VALID_EVENT_TYPES.contains(&name) {
            errs.push(format!(
                "{path}[{i}] 须为 {} 之一（当前「{name}」）",
                VALID_EVENT_TYPES.join(" | ")
            ));
        }
    }
    errs
}

/// Validate `turn_thinking` JSON object (from parsed `config.json`).
#[must_use]
pub fn validate_turn_thinking_config(value: &serde_json::Value) -> Vec<String> {
    let mut errs = Vec::new();

    if let Some(deep_when) = value.get("deep_when") {
        if let Some(or) = deep_when.get("or").and_then(|v| v.as_array()) {
            for (i, rule) in or.iter().enumerate() {
                errs.extend(validate_signal_rule(
                    rule,
                    &format!("config.json turn_thinking.deep_when.or[{i}]"),
                ));
            }
        } else if deep_when.get("or").is_some() {
            errs.push("config.json turn_thinking.deep_when.or 须为数组".into());
        }
        if let Some(and) = deep_when.get("and").and_then(|v| v.as_array()) {
            for (gi, group) in and.iter().enumerate() {
                let path = format!("config.json turn_thinking.deep_when.and[{gi}]");
                let Some(all) = group.get("all").and_then(|v| v.as_array()) else {
                    errs.push(format!("{path}.all 须为非空数组"));
                    continue;
                };
                if all.is_empty() {
                    errs.push(format!("{path}.all 须至少含一项"));
                }
                for (i, rule) in all.iter().enumerate() {
                    errs.extend(validate_signal_rule(rule, &format!("{path}.all[{i}]")));
                }
            }
        } else if deep_when.get("and").is_some() {
            errs.push("config.json turn_thinking.deep_when.and 须为数组".into());
        }
    }

    if let Some(latch) = value.get("latch") {
        if let Some(enter) = latch.get("enter_on").and_then(|v| v.as_array()) {
            errs.extend(validate_event_name_list(
                enter,
                "config.json turn_thinking.latch.enter_on",
            ));
        }
        if let Some(exit) = latch.get("exit_on").and_then(|v| v.as_array()) {
            errs.extend(validate_event_name_list(
                exit,
                "config.json turn_thinking.latch.exit_on",
            ));
        }
    }

    if let Some(ephemeral) = value.get("ephemeral_archive") {
        if let Some(ttl) = ephemeral.get("ttl_turns") {
            match ttl.as_u64() {
                Some(n) if (1..=8).contains(&n) => {}
                _ => errs
                    .push("config.json turn_thinking.ephemeral_archive.ttl_turns 须在 1–8".into()),
            }
        }
        if let Some(max) = ephemeral.get("max_chars") {
            match max.as_u64() {
                Some(n) if (1..=500).contains(&n) => {}
                _ => {
                    errs.push(
                        "config.json turn_thinking.ephemeral_archive.max_chars 须在 1–500".into(),
                    );
                }
            }
        }
        if let Some(events) = ephemeral.get("update_on_events").and_then(|v| v.as_array()) {
            errs.extend(validate_event_name_list(
                events,
                "config.json turn_thinking.ephemeral_archive.update_on_events",
            ));
        }
    }

    errs
}

/// Validate `turn_thinking` section in `config.json` when present.
///
/// # Errors
///
/// Returns validation messages when `turn_thinking` is present and invalid.
pub fn validate_turn_thinking_config_file(config_path: &Path) -> Result<(), Vec<String>> {
    let Ok(raw) = std::fs::read_to_string(config_path) else {
        return Ok(());
    };
    let Ok(root) = serde_json::from_str::<serde_json::Value>(&raw) else {
        return Ok(());
    };
    let Some(section) = root.get("turn_thinking") else {
        return Ok(());
    };
    let errs = validate_turn_thinking_config(section);
    if errs.is_empty() {
        Ok(())
    } else {
        Err(errs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_turn_thinking_config_passes() {
        let v = serde_json::json!({
            "deep_when": {
                "or": [{ "signal": "this_turn_event", "events": ["Quarrel"] }],
                "and": [{
                    "all": [
                        { "signal": "long_message", "min_chars": 40 },
                        { "signal": "high_sadness" }
                    ]
                }]
            },
            "latch": { "enter_on": ["Quarrel"], "exit_on": ["Apology"] },
            "ephemeral_archive": {
                "enabled": true,
                "ttl_turns": 3,
                "max_chars": 200,
                "update_on_events": ["Quarrel", "Apology"]
            }
        });
        assert!(validate_turn_thinking_config(&v).is_empty());
    }

    #[test]
    fn invalid_signal_rejected() {
        let v = serde_json::json!({
            "deep_when": { "or": [{ "signal": "unknown_signal" }] }
        });
        assert!(!validate_turn_thinking_config(&v).is_empty());
    }

    #[test]
    fn ttl_out_of_range_rejected() {
        let v = serde_json::json!({
            "ephemeral_archive": { "ttl_turns": 12 }
        });
        assert!(!validate_turn_thinking_config(&v).is_empty());
    }
}
