//! `oclive explain <CODE>` — human-readable error code reference.

use anyhow::{bail, Result};
use clap::Parser;
use serde::Serialize;
use std::fs;
use std::path::PathBuf;

#[derive(Parser, Debug)]
pub struct ExplainArgs {
    /// Error code (SCREAMING_SNAKE_CASE)
    pub code: String,
    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExplainEntry {
    pub code: String,
    pub meaning: String,
    pub causes: String,
    pub suggestions: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
}

pub fn run(args: ExplainArgs) -> Result<()> {
    let code = args.code.trim().to_ascii_uppercase();
    let path = find_error_codes_md().ok_or_else(|| {
        anyhow::anyhow!("ERROR_CODES.md not found (set OCLIVE_ROOT or run from oclivenewnew)")
    })?;
    let raw = fs::read_to_string(&path)?;
    let entries = parse_error_codes(&raw);
    let Some(entry) = entries.into_iter().find(|e| e.code == code) else {
        if args.json {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "ok": false,
                    "code": code,
                    "message": "unknown error code"
                }))?
            );
        } else {
            println!("Unknown error code: {code}");
            println!("See {} for the full table.", path.display());
        }
        bail!("unknown error code: {code}");
    };

    if args.json {
        println!("{}", serde_json::to_string_pretty(&entry)?);
        return Ok(());
    }

    println!("oclive explain {code}\n");
    println!("Meaning:    {}", entry.meaning);
    println!("Causes:     {}", entry.causes);
    println!("Suggestions: {}", entry.suggestions);
    if let Some(h) = &entry.hint {
        println!("Hint:       {h}");
    }
    println!("\nSource: {}", path.display());
    Ok(())
}

pub fn find_error_codes_md() -> Option<PathBuf> {
    if let Ok(root) = std::env::var("OCLIVE_ROOT") {
        let p = PathBuf::from(root).join("creator-docs/getting-started/ERROR_CODES.md");
        if p.is_file() {
            return Some(p);
        }
    }
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let p = manifest.join("../../creator-docs/getting-started/ERROR_CODES.md");
    if p.is_file() {
        return Some(p.canonicalize().unwrap_or(p));
    }
    let mut dir = std::env::current_dir().ok()?;
    for _ in 0..8 {
        let cand = dir.join("creator-docs/getting-started/ERROR_CODES.md");
        if cand.is_file() {
            return Some(cand);
        }
        if !dir.pop() {
            break;
        }
    }
    None
}

fn parse_error_codes(raw: &str) -> Vec<ExplainEntry> {
    let mut entries = Vec::new();
    let mut current_code: Option<String> = None;
    let mut meaning = String::new();
    let mut causes = String::new();
    let mut suggestions = String::new();
    let mut hint = None;

    let flush = |entries: &mut Vec<ExplainEntry>,
                 code: &mut Option<String>,
                 meaning: &mut String,
                 causes: &mut String,
                 suggestions: &mut String,
                 hint: &mut Option<String>| {
        if let Some(c) = code.take() {
            entries.push(ExplainEntry {
                code: c,
                meaning: std::mem::take(meaning),
                causes: std::mem::take(causes),
                suggestions: std::mem::take(suggestions),
                hint: hint.take(),
            });
        }
    };

    for line in raw.lines() {
        if let Some(rest) = line.strip_prefix("<!-- code:") {
            if let Some(code_part) = rest.strip_suffix("-->") {
                flush(
                    &mut entries,
                    &mut current_code,
                    &mut meaning,
                    &mut causes,
                    &mut suggestions,
                    &mut hint,
                );
                current_code = Some(code_part.trim().to_string());
                continue;
            }
        }
        if let Some(c) = parse_table_row(line) {
            if current_code.as_ref().is_some_and(|cur| cur != &c.0) {
                flush(
                    &mut entries,
                    &mut current_code,
                    &mut meaning,
                    &mut causes,
                    &mut suggestions,
                    &mut hint,
                );
            }
            current_code = Some(c.0);
            meaning = c.1;
            causes = c.2;
            suggestions = c.3;
            hint = c.4;
        }
    }
    flush(
        &mut entries,
        &mut current_code,
        &mut meaning,
        &mut causes,
        &mut suggestions,
        &mut hint,
    );
    entries
}

fn parse_table_row(line: &str) -> Option<(String, String, String, String, Option<String>)> {
    let line = line.trim();
    if !line.starts_with('|') || line.contains("---") || line.contains("code |") {
        return None;
    }
    let parts: Vec<&str> = line
        .split('|')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();
    if parts.len() < 4 {
        return None;
    }
    let code = parts[0].trim_matches('`').to_string();
    if !code.chars().all(|c| c.is_ascii_uppercase() || c == '_') {
        return None;
    }
    let hint = parts.get(4).map(|s| s.to_string());
    Some((
        code,
        parts[1].to_string(),
        parts[2].to_string(),
        parts[3].to_string(),
        hint,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_anchor_and_table() {
        let md = r#"
<!-- code:EMPTY_MESSAGE -->
| `EMPTY_MESSAGE` | empty input | spaces only | type visible chars |
"#;
        let e = parse_error_codes(md);
        assert!(e.iter().any(|x| x.code == "EMPTY_MESSAGE"));
    }
}
