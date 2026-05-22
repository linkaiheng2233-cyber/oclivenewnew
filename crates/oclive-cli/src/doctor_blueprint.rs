//! `oclive doctor` v2 蓝图专项检查（`roles/*/pipeline.ocblueprint`）。

use crate::doctor_cmd::DoctorCheck;
use oclive_validation::{
    validate_blueprint_v2_json, validate_role_pack_blueprint_v2_directory, PIPELINE_BLUEPRINT_FILENAME,
};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

pub fn blueprint_v2_checks(root: &Path) -> Vec<DoctorCheck> {
    let roles = root.join("roles");
    if !roles.is_dir() {
        return vec![DoctorCheck::ok(
            "blueprint_file_format",
            "no roles/ directory; blueprint checks skipped",
        )];
    }

    let mut dirs = Vec::new();
    if let Ok(rd) = fs::read_dir(&roles) {
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() && p.join(PIPELINE_BLUEPRINT_FILENAME).is_file() {
                dirs.push(p);
            }
        }
    }
    if dirs.is_empty() {
        return vec![
            DoctorCheck::ok("blueprint_file_format", "no pipeline.ocblueprint under roles/"),
            DoctorCheck::ok("slot_registry_llm", "skipped (no v2 blueprint packs)"),
            DoctorCheck::ok("slot_position_unique", "skipped (no v2 blueprint packs)"),
        ];
    }

    let mut format_errs = Vec::new();
    let mut llm_errs = Vec::new();
    let mut pos_errs = Vec::new();

    for dir in &dirs {
        let name = dir.file_name().and_then(|s| s.to_str()).unwrap_or("?");
        let path = dir.join(PIPELINE_BLUEPRINT_FILENAME);
        let raw = match fs::read_to_string(&path) {
            Ok(s) => s,
            Err(e) => {
                format_errs.push(format!("{name}: read failed: {e}"));
                continue;
            }
        };
        if let Err(errs) = validate_blueprint_v2_json(&raw) {
            format_errs.push(format!("{name}: {}", errs.join("; ")));
        }
        if let Err(errs) = validate_role_pack_blueprint_v2_directory(dir, env!("CARGO_PKG_VERSION")) {
            for e in errs {
                if e.contains("llm") || e.contains("LLM") {
                    llm_errs.push(format!("{name}: {e}"));
                } else if e.contains("position") {
                    pos_errs.push(format!("{name}: {e}"));
                } else if !format_errs.iter().any(|x| x.contains(name)) {
                    format_errs.push(format!("{name}: {e}"));
                }
            }
        } else {
            check_llm_and_position_local(&raw, name, &mut llm_errs, &mut pos_errs);
        }
    }

    vec![
        if format_errs.is_empty() {
            DoctorCheck::ok(
                "blueprint_file_format",
                format!("{} v2 blueprint pack(s) — JSON valid", dirs.len()),
            )
        } else {
            DoctorCheck::fail(
                "blueprint_file_format",
                format!("{} pack(s) with format errors", format_errs.len()),
                Some(format_errs.join("\n    ")),
            )
        },
        if llm_errs.is_empty() {
            DoctorCheck::ok(
                "slot_registry_llm",
                "each pack has at least one type: llm slot",
            )
        } else {
            DoctorCheck::fail(
                "slot_registry_llm",
                "missing or invalid llm slot",
                Some(llm_errs.join("\n    ")),
            )
        },
        if pos_errs.is_empty() {
            DoctorCheck::ok("slot_position_unique", "no duplicate position per type")
        } else {
            DoctorCheck::fail(
                "slot_position_unique",
                "duplicate position under same type",
                Some(pos_errs.join("\n    ")),
            )
        },
    ]
}

fn check_llm_and_position_local(
    raw: &str,
    name: &str,
    llm_errs: &mut Vec<String>,
    pos_errs: &mut Vec<String>,
) {
    let Ok(v) = serde_json::from_str::<serde_json::Value>(raw) else {
        return;
    };
    let Some(reg) = v.get("slot_registry").and_then(|r| r.as_object()) else {
        llm_errs.push(format!("{name}: slot_registry missing"));
        return;
    };
    let mut llm = 0usize;
    let mut by_type: HashMap<String, HashSet<i64>> = HashMap::new();
    for (key, slot) in reg {
        let t = slot
            .get("type")
            .and_then(|x| x.as_str())
            .unwrap_or("")
            .trim()
            .to_string();
        if t == "llm" {
            llm += 1;
        }
        let pos = slot.get("position").and_then(|x| x.as_i64()).unwrap_or(-1);
        if pos >= 0 && !by_type.entry(t.clone()).or_default().insert(pos) {
            pos_errs.push(format!("{name}: type「{t}」position {pos} duplicate"));
        }
        let _ = key;
    }
    if llm == 0 {
        llm_errs.push(format!("{name}: slot_registry needs at least one type: llm"));
    }
}
