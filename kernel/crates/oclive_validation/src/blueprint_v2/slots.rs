//! Slot registry validation and slot registry file generation.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::path::Path;

use serde_json::Value;

use super::{
    validate_blueprint_v2_json_with_context, BlueprintV2ValidateContext, SlotGroupEntry,
    SlotRegistryEntry, BLUEPRINT_V2_SCHEMA_VERSION, GROUP_SLOT_TYPES, PIPELINE_BLUEPRINT_FILENAME,
    SLOT_TYPES,
};

/// Writes `slot_registry` back to a supported `pipeline.ocblueprint` version
/// while preserving all other fields; validates the matching v2/v3/v4
/// contract before write.
///
/// # Errors
///
/// Returns `Err(Vec<String>)` when blueprint is missing, JSON is invalid, or validation fails.
pub fn write_role_pack_blueprint_slot_registry(
    role_dir: &Path,
    slot_registry: &BTreeMap<String, SlotRegistryEntry>,
    host_version: &str,
) -> Result<(), Vec<String>> {
    let blueprint_path = role_dir.join(PIPELINE_BLUEPRINT_FILENAME);
    if !blueprint_path.is_file() {
        return Err(vec![format!(
            "缺少 {}：{}",
            PIPELINE_BLUEPRINT_FILENAME,
            blueprint_path.display()
        )]);
    }
    let raw = fs::read_to_string(&blueprint_path)
        .map_err(|e| vec![format!("读取 {} 失败: {}", PIPELINE_BLUEPRINT_FILENAME, e)])?;
    let version = crate::blueprint_dispatch::blueprint_schema_version_from_raw(&raw).unwrap_or(0);
    let mut root: Value = serde_json::from_str(&raw).map_err(|e| {
        vec![format!(
            "{} JSON 解析失败: {}",
            PIPELINE_BLUEPRINT_FILENAME, e
        )]
    })?;
    let obj = root
        .as_object_mut()
        .ok_or_else(|| vec![format!("{} 根节点须为对象", PIPELINE_BLUEPRINT_FILENAME)])?;
    let reg_val = serde_json::to_value(slot_registry)
        .map_err(|e| vec![format!("slot_registry 序列化失败: {e}")])?;
    obj.insert("slot_registry".to_string(), reg_val);
    let out = serde_json::to_string_pretty(&root)
        .map_err(|e| vec![format!("{} 序列化失败: {e}", PIPELINE_BLUEPRINT_FILENAME)])?;
    let folder_name = role_dir.file_name().and_then(|s| s.to_str()).unwrap_or("");
    validate_blueprint_for_slot_registry_write(&out, version, folder_name, role_dir, host_version)?;
    let resolved = crate::blueprint_includes::merge_blueprint_includes_strict(role_dir, &out)?;
    validate_blueprint_for_slot_registry_write(
        &resolved,
        version,
        folder_name,
        role_dir,
        host_version,
    )?;
    fs::write(&blueprint_path, format!("{out}\n"))
        .map_err(|e| vec![format!("写入 {} 失败: {e}", blueprint_path.display())])?;
    Ok(())
}

fn validate_blueprint_for_slot_registry_write(
    raw: &str,
    version: u32,
    folder_name: &str,
    role_dir: &Path,
    host_version: &str,
) -> Result<(), Vec<String>> {
    match version {
        BLUEPRINT_V2_SCHEMA_VERSION => validate_blueprint_v2_json_with_context(
            raw,
            BlueprintV2ValidateContext {
                folder_name: Some(folder_name),
                role_dir: Some(role_dir),
                host_version: Some(host_version),
            },
        ),
        crate::blueprint_v3::BLUEPRINT_V3_SCHEMA_VERSION => {
            crate::blueprint_v3::validate_blueprint_v3_json(raw, Some(folder_name))
        }
        crate::blueprint_v4::BLUEPRINT_V4_SCHEMA_VERSION => {
            crate::blueprint_v4::validate_blueprint_v4_json(raw, Some(folder_name))?;
            crate::blueprint_v4::validate_blueprint_v4_extension_payloads_for_raw(role_dir, raw)
        }
        unsupported => Err(vec![format!(
            "pipeline.ocblueprint：不支持的 schema_version {unsupported}（支持 {BLUEPRINT_V2_SCHEMA_VERSION}、{} 或 {}）",
            crate::blueprint_v3::BLUEPRINT_V3_SCHEMA_VERSION,
            crate::blueprint_v4::BLUEPRINT_V4_SCHEMA_VERSION
        )]),
    }
}

fn validate_blueprint_groups(
    groups: &BTreeMap<String, SlotGroupEntry>,
    registry: &BTreeMap<String, SlotRegistryEntry>,
) -> Vec<String> {
    let mut errs = Vec::new();
    let mut member_owner: HashMap<&str, &str> = HashMap::new();

    for (group_id, group) in groups {
        if group_id.trim().is_empty() {
            errs.push("groups 键名不能为空".into());
            continue;
        }
        if group.label.trim().is_empty() {
            errs.push(format!("groups[{group_id}].label 不能为空"));
        }
        let gt = group.group_type.trim();
        if !GROUP_SLOT_TYPES.contains(&gt) {
            errs.push(format!(
                "groups[{group_id}].type「{gt}」非法（允许: {}）",
                GROUP_SLOT_TYPES.join(", ")
            ));
            continue;
        }
        if group.members.is_empty() {
            errs.push(format!("groups[{group_id}].members 不能为空"));
            continue;
        }
        for member in &group.members {
            let m = member.trim();
            if m.is_empty() {
                errs.push(format!("groups[{group_id}].members 含空键名"));
                continue;
            }
            let Some(slot) = registry.get(m) else {
                errs.push(format!(
                    "groups[{group_id}].members 引用未知 slot_registry 键「{m}」"
                ));
                continue;
            };
            if slot.slot_type.trim() != gt {
                errs.push(format!(
                    "groups[{group_id}].members「{m}」的 type 为「{}」，与 groups.type「{gt}」不一致",
                    slot.slot_type.trim()
                ));
            }
            if let Some(prev) = member_owner.insert(m, group_id.as_str()) {
                errs.push(format!(
                    "slot_registry 键「{m}」同时属于 groups「{prev}」与「{group_id}」"
                ));
            }
        }
    }
    errs
}

pub(crate) fn validate_slot_registry_contract(
    registry: &BTreeMap<String, SlotRegistryEntry>,
    groups: &BTreeMap<String, SlotGroupEntry>,
    allow_zone: bool,
) -> Vec<String> {
    let mut errs = Vec::new();
    if registry.is_empty() {
        errs.push("slot_registry 不能为空".into());
    }

    let mut llm_count = 0usize;
    let mut positions_by_type: HashMap<&str, HashSet<i64>> = HashMap::new();

    for (key, slot) in registry {
        if key.trim().is_empty() {
            errs.push("slot_registry 键名不能为空".into());
            continue;
        }
        if slot.label.trim().is_empty() {
            errs.push(format!("slot_registry[{key}].label 不能为空"));
        }

        let t = slot.slot_type.trim();
        if !SLOT_TYPES.contains(&t) {
            errs.push(format!(
                "slot_registry[{key}].type「{t}」非法（允许: {}）",
                SLOT_TYPES.join(", ")
            ));
            continue;
        }

        if slot.position < 0 {
            errs.push(format!("slot_registry[{key}].position 须为非负整数"));
        }

        if !positions_by_type
            .entry(t)
            .or_default()
            .insert(slot.position)
        {
            errs.push(format!(
                "slot_registry：type「{t}」下 position {} 重复（B5）",
                slot.position
            ));
        }

        if t == "llm" {
            llm_count += 1;
        }

        if !allow_zone && slot.zone.is_some() {
            errs.push(format!(
                "slot_registry[{key}].zone 仅属于冻结 v3，v2 不接受该字段"
            ));
        }
        if slot.policy.is_some() {
            errs.push(format!(
                "slot_registry[{key}].policy 未进入 v2/v3/v4 磁盘契约；当前公开语义保持按 position 的既定合并策略"
            ));
        }

        if let Err(e) = validate_slot_backend_and_fields(key, slot) {
            errs.push(e);
        }
    }

    if llm_count == 0 {
        errs.push("slot_registry 须至少包含一个 type: llm 的实例".into());
    }

    errs.extend(validate_blueprint_groups(groups, registry));
    errs
}

fn validate_slot_backend_and_fields(key: &str, slot: &SlotRegistryEntry) -> Result<(), String> {
    let t = slot.slot_type.trim();
    let b = slot.backend.trim();

    let allowed = allowed_backends_for_type(t);
    if !allowed.contains(&b) {
        return Err(format!(
            "slot_registry[{key}]：type「{t}」的 backend「{b}」非法（允许: {}）",
            allowed.join(", ")
        ));
    }

    let has_plugin = slot
        .plugin
        .as_ref()
        .map(|s| !s.trim().is_empty())
        .unwrap_or(false);
    let plugins: Vec<&str> = slot
        .plugins
        .as_ref()
        .map(|ps| {
            ps.iter()
                .map(|s| s.as_str())
                .filter(|s| !s.trim().is_empty())
                .collect()
        })
        .unwrap_or_default();
    let has_plugins = !plugins.is_empty();

    if t != "agent" && has_plugin && has_plugins {
        return Err(format!(
            "slot_registry[{key}]：非 agent 槽位不得同时包含 plugin 与 plugins（S4）"
        ));
    }

    if t == "agent" && b != "directory" && has_plugins {
        return Err(format!(
            "slot_registry[{key}]：agent backend 为「{b}」时不得包含 plugins（S3）"
        ));
    }

    if b == "directory" && !has_plugin && !has_plugins {
        return Err(format!(
            "slot_registry[{key}]：backend 为 directory 时须指定 plugin 或 plugins"
        ));
    }

    if t == "llm" && b == "ollama" {
        if let Some(ref m) = slot.model {
            if m.trim().is_empty() {
                return Err(format!(
                    "slot_registry[{key}]：ollama 槽位的 model 若存在则不得为空"
                ));
            }
        }
    }

    Ok(())
}

pub(crate) fn allowed_backends_for_type(slot_type: &str) -> &'static [&'static str] {
    match slot_type {
        "memory" => &["builtin", "remote", "directory", "local", "none"],
        "emotion" | "event" | "prompt" => &["builtin", "remote", "directory", "none"],
        "llm" => &["ollama", "remote", "directory", "none"],
        "agent" => &["builtin", "remote", "directory", "none"],
        "complex_emotion" => &["builtin", "remote", "directory", "none"],
        _ => &[],
    }
}
