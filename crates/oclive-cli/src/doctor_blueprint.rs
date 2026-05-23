//! `oclive doctor` 蓝图专项检查（`roles/*/pipeline.ocblueprint`，按 `schema_version` 分流 v2 / v3）。

use crate::doctor_cmd::DoctorCheck;
use oclive_validation::{
    validate_blueprint_v2_json, validate_blueprint_v3_json,
    validate_role_pack_blueprint_v2_directory, validate_role_pack_blueprint_v3_directory,
    BLUEPRINT_V3_SCHEMA_VERSION, PIPELINE_BLUEPRINT_FILENAME,
};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

struct PackDir {
    path: PathBuf,
    name: String,
    schema_version: u32,
}

pub fn blueprint_checks(root: &Path) -> Vec<DoctorCheck> {
    let roles = root.join("roles");
    if !roles.is_dir() {
        return vec![DoctorCheck::ok(
            "blueprint_file_format",
            "no roles/ directory; blueprint checks skipped",
        )];
    }

    let packs = collect_blueprint_packs(&roles);
    if packs.is_empty() {
        return vec![
            DoctorCheck::ok("blueprint_file_format", "no pipeline.ocblueprint under roles/"),
            DoctorCheck::ok("slot_registry_llm", "skipped (no blueprint packs)"),
            DoctorCheck::ok("slot_position_unique", "skipped (no blueprint packs)"),
        ];
    }

    let v2: Vec<_> = packs.iter().filter(|p| p.schema_version != BLUEPRINT_V3_SCHEMA_VERSION).collect();
    let v3: Vec<_> = packs
        .iter()
        .filter(|p| p.schema_version == BLUEPRINT_V3_SCHEMA_VERSION)
        .collect();

    let mut out = Vec::new();
    out.extend(run_v2_checks(&v2));
    out.extend(run_v3_checks(&v3));
    out
}

/// 兼容旧调用点。
pub fn blueprint_v2_checks(root: &Path) -> Vec<DoctorCheck> {
    blueprint_checks(root)
}

fn collect_blueprint_packs(roles: &Path) -> Vec<PackDir> {
    let mut out = Vec::new();
    let Ok(rd) = fs::read_dir(roles) else {
        return out;
    };
    for e in rd.flatten() {
        let p = e.path();
        if !p.is_dir() {
            continue;
        }
        let bp = p.join(PIPELINE_BLUEPRINT_FILENAME);
        if !bp.is_file() {
            continue;
        }
        let name = p.file_name().and_then(|s| s.to_str()).unwrap_or("?").to_string();
        let schema_version = fs::read_to_string(&bp)
            .ok()
            .and_then(|raw| serde_json::from_str::<serde_json::Value>(&raw).ok())
            .and_then(|v| v.get("schema_version").and_then(|x| x.as_u64()))
            .unwrap_or(2) as u32;
        out.push(PackDir {
            path: p,
            name,
            schema_version,
        });
    }
    out
}

fn run_v2_checks(packs: &[&PackDir]) -> Vec<DoctorCheck> {
    if packs.is_empty() {
        return vec![
            DoctorCheck::ok("blueprint_file_format", "no v2 blueprint packs under roles/"),
            DoctorCheck::ok("slot_registry_llm", "skipped (no v2 blueprint packs)"),
            DoctorCheck::ok("slot_position_unique", "skipped (no v2 blueprint packs)"),
        ];
    }

    let mut format_errs = Vec::new();
    let mut llm_errs = Vec::new();
    let mut pos_errs = Vec::new();

    for pack in packs {
        let path = pack.path.join(PIPELINE_BLUEPRINT_FILENAME);
        let raw = match fs::read_to_string(&path) {
            Ok(s) => s,
            Err(e) => {
                format_errs.push(format!("{}: read failed: {e}", pack.name));
                continue;
            }
        };
        if let Err(errs) = validate_blueprint_v2_json(&raw) {
            format_errs.push(format!("{}: {}", pack.name, errs.join("; ")));
        }
        if let Err(errs) =
            validate_role_pack_blueprint_v2_directory(&pack.path, env!("CARGO_PKG_VERSION"))
        {
            for e in errs {
                if e.contains("llm") || e.contains("LLM") {
                    llm_errs.push(format!("{}: {e}", pack.name));
                } else if e.contains("position") {
                    pos_errs.push(format!("{}: {e}", pack.name));
                } else if !format_errs.iter().any(|x| x.contains(&pack.name)) {
                    format_errs.push(format!("{}: {e}", pack.name));
                }
            }
        } else {
            check_llm_and_position_local(&raw, &pack.name, &mut llm_errs, &mut pos_errs);
        }
    }

    vec![
        format_check("blueprint_file_format", "v2", packs.len(), &format_errs),
        llm_check("slot_registry_llm", &llm_errs),
        position_check("slot_position_unique", &pos_errs),
    ]
}

fn run_v3_checks(packs: &[&PackDir]) -> Vec<DoctorCheck> {
    if packs.is_empty() {
        return vec![
            DoctorCheck::ok("blueprint_v3_file_format", "no v3 blueprint packs under roles/"),
            DoctorCheck::ok("slot_registry_v3_llm", "skipped (no v3 blueprint packs)"),
            DoctorCheck::ok("slot_position_v3_unique", "skipped (no v3 blueprint packs)"),
        ];
    }

    let mut format_errs = Vec::new();
    let mut llm_errs = Vec::new();
    let mut pos_errs = Vec::new();

    for pack in packs {
        let path = pack.path.join(PIPELINE_BLUEPRINT_FILENAME);
        let raw = match fs::read_to_string(&path) {
            Ok(s) => s,
            Err(e) => {
                format_errs.push(format!("{}: read failed: {e}", pack.name));
                continue;
            }
        };
        if let Err(errs) = validate_blueprint_v3_json(&raw, Some(&pack.name)) {
            format_errs.push(format!("{}: {}", pack.name, errs.join("; ")));
        }
        if let Err(errs) =
            validate_role_pack_blueprint_v3_directory(&pack.path, env!("CARGO_PKG_VERSION"))
        {
            for e in errs {
                if e.contains("llm") || e.contains("LLM") {
                    llm_errs.push(format!("{}: {e}", pack.name));
                } else if e.contains("position") {
                    pos_errs.push(format!("{}: {e}", pack.name));
                } else if !format_errs.iter().any(|x| x.contains(&pack.name)) {
                    format_errs.push(format!("{}: {e}", pack.name));
                }
            }
        } else {
            check_llm_and_position_local(&raw, &pack.name, &mut llm_errs, &mut pos_errs);
        }
    }

    vec![
        format_check("blueprint_v3_file_format", "v3", packs.len(), &format_errs),
        llm_check("slot_registry_v3_llm", &llm_errs),
        position_check("slot_position_v3_unique", &pos_errs),
    ]
}

fn format_check(id: &str, label: &str, count: usize, errs: &[String]) -> DoctorCheck {
    if errs.is_empty() {
        DoctorCheck::ok(id, format!("{count} {label} blueprint pack(s) — JSON valid"))
    } else {
        DoctorCheck::fail(
            id,
            format!("{} pack(s) with format errors", errs.len()),
            Some(errs.join("\n    ")),
        )
    }
}

fn llm_check(id: &str, errs: &[String]) -> DoctorCheck {
    if errs.is_empty() {
        DoctorCheck::ok(id, "each pack has at least one type: llm slot")
    } else {
        DoctorCheck::fail(
            id,
            "missing or invalid llm slot",
            Some(errs.join("\n    ")),
        )
    }
}

fn position_check(id: &str, errs: &[String]) -> DoctorCheck {
    if errs.is_empty() {
        DoctorCheck::ok(id, "no duplicate position per type")
    } else {
        DoctorCheck::fail(
            id,
            "duplicate position under same type",
            Some(errs.join("\n    ")),
        )
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    fn write_v3_pack(dir: &Path) {
        fs::create_dir_all(dir).unwrap();
        let bp = dir.join(PIPELINE_BLUEPRINT_FILENAME);
        let raw = include_str!("../tests/fixtures/valid_blueprint_v3.json");
        fs::write(bp, raw).unwrap();
        fs::create_dir_all(dir.join("scenes/default")).unwrap();
        fs::write(
            dir.join("scenes/default/scene.json"),
            r#"{"id":"default","name":"Default"}"#,
        )
        .unwrap();
        fs::write(dir.join("core_personality.txt"), "test").unwrap();
    }

    #[test]
    fn doctor_v3_checks_pass_for_valid_fixture() {
        let tmp = tempfile::tempdir().unwrap();
        let roles = tmp.path().join("roles/v3role");
        write_v3_pack(&roles);
        let checks = blueprint_checks(tmp.path());
        let ids: Vec<_> = checks.iter().map(|c| c.id.as_str()).collect();
        assert!(ids.contains(&"blueprint_v3_file_format"));
        assert!(ids.contains(&"slot_registry_v3_llm"));
        assert!(ids.contains(&"slot_position_v3_unique"));
        for c in &checks {
            if c.id.starts_with("blueprint_v3") || c.id.starts_with("slot_registry_v3") || c.id.starts_with("slot_position_v3") {
                assert!(c.status == "ok", "{}: {}", c.id, c.message);
            }
        }
    }

    #[test]
    fn doctor_v2_and_v3_split_by_schema_version() {
        let tmp = tempfile::tempdir().unwrap();
        let v2dir = tmp.path().join("roles/fixture.valid");
        fs::create_dir_all(&v2dir).unwrap();
        let v2raw = include_str!("../tests/fixtures/valid_blueprint.json");
        fs::write(v2dir.join(PIPELINE_BLUEPRINT_FILENAME), v2raw).unwrap();
        fs::create_dir_all(v2dir.join("scenes/default")).unwrap();
        fs::write(
            v2dir.join("scenes/default/scene.json"),
            r#"{"id":"default","name":"Default"}"#,
        )
        .unwrap();
        fs::write(v2dir.join("core_personality.txt"), "test").unwrap();
        write_v3_pack(&tmp.path().join("roles/v3role"));
        let checks = blueprint_checks(tmp.path());
        assert!(checks.iter().any(|c| c.id == "blueprint_file_format" && c.status == "ok"));
        assert!(checks.iter().any(|c| c.id == "blueprint_v3_file_format" && c.status == "ok"));
    }
}
