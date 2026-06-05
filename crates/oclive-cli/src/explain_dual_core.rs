//! `oclive explain slot.emotion.analyze` / `DUAL_CORE` — experimental-core method documentation.

use anyhow::{bail, Result};
use serde::Serialize;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize)]
pub struct DualCoreMethodExplain {
    pub query: String,
    pub slot_type: String,
    pub method: String,
    pub co_present_stage: String,
    pub action_format: String,
    pub example: String,
    pub doc: String,
}

const SPECS: &[(&str, &str, &str, &str)] = &[
    ("memory", "retrieve", "memory_rank", "slot.memory.retrieve"),
    (
        "emotion",
        "analyze",
        "user_emotion_analyze",
        "slot.emotion.analyze",
    ),
    ("event", "detect", "event_estimate", "slot.event.detect"),
    ("prompt", "assemble", "build_prompt", "slot.prompt.assemble"),
    ("llm", "generate", "llm_generate", "slot.llm.generate"),
    ("agent", "process", "agent_process", "slot.agent.process"),
    (
        "complex_emotion",
        "resolve_turn",
        "complex_emotion_resolve_turn",
        "slot.complex_emotion.resolve_turn",
    ),
];

pub fn find_method_registry_md() -> Option<PathBuf> {
    if let Ok(root) = std::env::var("OCLIVE_ROOT") {
        let p = PathBuf::from(root).join("creator-docs/dual-core/METHOD_REGISTRY.md");
        if p.is_file() {
            return Some(p);
        }
    }
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let p = manifest.join("../../creator-docs/dual-core/METHOD_REGISTRY.md");
    if p.is_file() {
        return Some(p.canonicalize().unwrap_or(p));
    }
    None
}

pub fn explain_dual_core_query(code: &str, json: bool) -> Result<()> {
    let raw = code.trim();
    let upper = raw.to_ascii_uppercase();

    if upper == "DUAL_CORE" || upper == "DUAL-CORE" || upper == "DUAL_CORE_METHODS" {
        if json {
            let list: Vec<DualCoreMethodExplain> = SPECS
                .iter()
                .map(|(t, m, stage, ex)| DualCoreMethodExplain {
                    query: format!("{t}.{m}"),
                    slot_type: (*t).into(),
                    method: (*m).into(),
                    co_present_stage: (*stage).into(),
                    action_format: "slot.<registry_key>.<method>".into(),
                    example: (*ex).into(),
                    doc: find_method_registry_md()
                        .map(|p| p.display().to_string())
                        .unwrap_or_default(),
                })
                .collect();
            println!("{}", serde_json::to_string_pretty(&list)?);
        } else {
            println!("Dual-core experimental methods (pipeline.experimental):\n");
            for (t, m, stage, ex) in SPECS {
                println!("  {ex}");
                println!("    type: {t}  method: {m}  co_present: {stage}\n");
            }
            if let Some(p) = find_method_registry_md() {
                println!("Full registry: {}", p.display());
            }
        }
        return Ok(());
    }

    let (slot_type, method) = parse_action_query(raw)?;
    let Some((t, m, stage, ex)) = SPECS
        .iter()
        .find(|(st, sm, _, _)| *st == slot_type && *sm == method)
    else {
        bail!("unknown experimental method: {slot_type}.{method} (try `oclive explain DUAL_CORE`)");
    };

    let entry = DualCoreMethodExplain {
        query: raw.to_string(),
        slot_type: (*t).into(),
        method: (*m).into(),
        co_present_stage: (*stage).into(),
        action_format: "slot.<registry_key>.<method>".into(),
        example: (*ex).into(),
        doc: find_method_registry_md()
            .map(|p| p.display().to_string())
            .unwrap_or_default(),
    };

    if json {
        println!("{}", serde_json::to_string_pretty(&entry)?);
        return Ok(());
    }

    println!("oclive explain {raw}\n");
    println!("Slot type:     {}", entry.slot_type);
    println!("Method:        {}", entry.method);
    println!("Co-present:    {}", entry.co_present_stage);
    println!("Action format: {}", entry.action_format);
    println!("Example:       {}", entry.example);
    if !entry.doc.is_empty() {
        println!("\nDoc: {}", entry.doc);
        if let Ok(body) = fs::read_to_string(&entry.doc) {
            if let Some(section) = extract_md_section(&body, &entry.method) {
                println!("\n---\n{section}");
            }
        }
    }
    Ok(())
}

/// Parse `slot.<key>.<method>` or `<type>.<method>`.
pub fn parse_action_query(raw: &str) -> Result<(String, String)> {
    if let Some(rest) = raw.strip_prefix("slot.") {
        let parts: Vec<&str> = rest.split('.').collect();
        if parts.len() >= 2 {
            let key = parts[0].trim();
            let method = parts[1..].join(".");
            if !key.is_empty() && !method.is_empty() {
                if let Some((t, m, _, _)) = SPECS.iter().find(|(_, sm, _, _)| *sm == method) {
                    return Ok(((*t).to_string(), (*m).to_string()));
                }
                bail!("unknown method suffix in action (registry key '{key}' ignored for lookup)");
            }
        }
    }
    if let Some((t, m, _, _)) = SPECS
        .iter()
        .find(|(st, sm, _, _)| format!("{st}.{sm}") == raw)
    {
        return Ok(((*t).to_string(), (*m).to_string()));
    }
    bail!("expected slot.<registry_key>.<method> or <type>.<method> (e.g. slot.emotion.analyze)");
}

fn extract_md_section(body: &str, method: &str) -> Option<String> {
    let anchor = format!("### `{method}`");
    let start = body.find(&anchor)?;
    let rest = &body[start..];
    let end = rest[anchor.len()..]
        .find("\n### `")
        .map(|i| anchor.len() + i)
        .unwrap_or(rest.len());
    Some(rest[..end].trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_slot_action() {
        let (t, m) = parse_action_query("slot.my_emotion.analyze").unwrap();
        assert_eq!(t, "emotion");
        assert_eq!(m, "analyze");
    }
}
