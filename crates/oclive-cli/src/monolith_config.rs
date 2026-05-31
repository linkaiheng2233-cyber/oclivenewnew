//! `monolith.toml` parsing, validation, and weld-set resolution.

use anyhow::{bail, Context, Result};
use serde::Deserialize;
use std::collections::HashSet;

/// Seven weld keys in the order aligned with `plugin_backends` / RFC (modules 1–6 + `complex_emotion`, the orchestration demo order).
pub const SLOT_IDS: [&str; 7] = [
    "memory",
    "emotion",
    "event",
    "prompt",
    "llm",
    "agent",
    "complex_emotion",
];

#[derive(Debug, Clone, Default, Deserialize)]
pub struct MonolithDualCoreSection {
    #[serde(default)]
    pub enabled: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MonolithFile {
    pub monolith: MonolithSection,
    #[serde(default)]
    pub dual_core: MonolithDualCoreSection,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MonolithSection {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub weld_modules: Vec<String>,
    #[serde(default)]
    pub exclude: Vec<String>,
}

/// Whether each slot uses static welding (`true`) or keeps a trait/PluginHost-style placeholder (`false`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WeldPlan {
    pub welded: [bool; 7],
}

impl WeldPlan {
    pub fn all_welded() -> Self {
        Self { welded: [true; 7] }
    }

    /// Whether any slot still uses a PluginHost / trait placeholder.
    pub fn any_dynamic_slot(&self) -> bool {
        self.welded.iter().any(|&w| !w)
    }
}

pub fn parse_monolith_toml(text: &str) -> Result<MonolithFile> {
    toml::from_str(text).context("parse monolith.toml as TOML")
}

/// `weld_modules` and `exclude` must not both be non-empty; keys must be within the seven weld keys.
pub fn validate_monolith_section(m: &MonolithSection) -> Result<()> {
    let known: HashSet<&str> = SLOT_IDS.iter().copied().collect();
    if !m.weld_modules.is_empty() && !m.exclude.is_empty() {
        bail!(
            "monolith.toml: `weld_modules` and `exclude` cannot both be non-empty; pick one: \
             explicit weld list, or `weld_modules = []` with `exclude` for weld-all-then-exclude."
        );
    }
    for w in &m.weld_modules {
        if !known.contains(w.as_str()) {
            bail!("monolith.toml: unknown slot `{w}` (valid: memory, emotion, event, prompt, llm, agent, complex_emotion)");
        }
    }
    for e in &m.exclude {
        if !known.contains(e.as_str()) {
            bail!("monolith.toml: exclude contains unknown slot `{e}`");
        }
    }
    Ok(())
}

/// Called after [`validate_monolith_section`] has already passed.
pub fn resolve_weld_plan(m: &MonolithSection) -> WeldPlan {
    if !m.weld_modules.is_empty() {
        let mut welded = [false; 7];
        for w in &m.weld_modules {
            if let Some(i) = SLOT_IDS.iter().position(|&s| s == w.as_str()) {
                welded[i] = true;
            }
        }
        return WeldPlan { welded };
    }
    let excluded: HashSet<&str> = m.exclude.iter().map(|s| s.as_str()).collect();
    let mut welded = [true; 7];
    for (i, name) in SLOT_IDS.iter().enumerate() {
        if excluded.contains(name) {
            welded[i] = false;
        }
    }
    WeldPlan { welded }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sec(w: Vec<&str>, e: Vec<&str>) -> MonolithSection {
        MonolithSection {
            enabled: true,
            weld_modules: w.into_iter().map(String::from).collect(),
            exclude: e.into_iter().map(String::from).collect(),
        }
    }

    #[test]
    fn rejects_both_lists_non_empty() {
        let m = sec(vec!["memory"], vec!["agent"]);
        assert!(validate_monolith_section(&m).is_err());
    }

    #[test]
    fn empty_lists_full_weld() {
        let m = sec(vec![], vec![]);
        validate_monolith_section(&m).unwrap();
        let p = resolve_weld_plan(&m);
        assert_eq!(p, WeldPlan { welded: [true; 7] });
    }

    #[test]
    fn exclude_only() {
        let m = sec(vec![], vec!["agent", "llm"]);
        validate_monolith_section(&m).unwrap();
        let p = resolve_weld_plan(&m);
        assert!(p.welded[0]); // memory
        assert!(!p.welded[5]); // agent
        assert!(!p.welded[4]); // llm
    }

    #[test]
    fn explicit_weld_only() {
        let m = sec(vec!["memory", "emotion"], vec![]);
        validate_monolith_section(&m).unwrap();
        let p = resolve_weld_plan(&m);
        assert!(p.welded[0] && p.welded[1]);
        assert!(!p.welded[2]);
    }
}
