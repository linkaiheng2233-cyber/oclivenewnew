//! `pipeline.ocblueprint` schema_version 3 (dual-core P1 contract validation).
//!
//! **FROZEN (2026-06)**: v3 schema keys (`pipeline.stable` / `pipeline.experimental`, step DAG shape) are
//! contract-frozen for validation only. **Production Stable scheduling** does not execute blueprint `steps[]`
//! as a DSL — runtime order is **`process_message` → `turn_pipeline` → `co_present`** (see
//! [`BUS_FACTOR_NOTES.md`](../../../handoff/BUS_FACTOR_NOTES.md) §1).
//!
//! **Feature gate**: Experimental pipeline steps are consumed only when the host is built with the `dual_core`
//! Cargo feature **and** `runtime_config.dual_core.enabled` is true. Default desktop / `oclive-kernel-server`
//! builds keep experimental scheduling off (aligned with [`dual_pipeline.rs`](../oclive_kernel_host/src/domain/dual_pipeline.rs)
//! module freeze notes).
//!
//! **Disambiguation**: This file validates **disk JSON** for `schema_version: 3`. It is not the Rust `dual_pipeline`
//! orchestrator module and not a rename target for `pipeline.ocblueprint`.

use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use crate::blueprint_v2::{
    meta_to_disk_manifest, validate_slot_registry_contract, BlueprintMeta, SlotGroupEntry,
    SlotRegistryEntry, PIPELINE_BLUEPRINT_FILENAME,
};
use crate::pipeline_action::{parse_pipeline_action_kind, PipelineActionKind};
use crate::role_pack::merge_role_pack_scene_ids;
use crate::runtime_config::{validate_runtime_config, RuntimeConfig};
use crate::validate::{validate_disk_manifest, validate_min_runtime_version};

pub const BLUEPRINT_V3_SCHEMA_VERSION: u32 = 3;

/// Instances referenced by the stable core `pipeline.stable`; their `type` must be one of the six slots.
pub const STABLE_PIPELINE_SLOT_TYPES: &[&str] =
    &["memory", "emotion", "event", "prompt", "llm", "agent"];

/// P4 runtime: the seven `type` values the `PluginHost` facade supports (including the `complex_emotion` facility).
pub const PLUGIN_HOST_SLOT_TYPES: &[&str] = &[
    "memory",
    "emotion",
    "event",
    "prompt",
    "llm",
    "agent",
    "complex_emotion",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PipelineStep {
    pub action: String,
    #[serde(default)]
    pub depends_on: Vec<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DualPipelineDef {
    #[serde(default)]
    pub stable: Vec<PipelineStep>,
    #[serde(default)]
    pub experimental: Vec<PipelineStep>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BlueprintV3File {
    pub schema_version: u32,
    pub meta: BlueprintMeta,
    pub slot_registry: HashMap<String, SlotRegistryEntryV3>,
    #[serde(default)]
    pub groups: HashMap<String, SlotGroupEntry>,
    #[serde(default)]
    pub runtime_config: Option<RuntimeConfig>,
    #[serde(default)]
    pub pipeline: Option<DualPipelineDef>,
    #[serde(default)]
    #[allow(dead_code)]
    includes: Vec<crate::blueprint_includes::BlueprintIncludeEntry>,
    #[serde(default)]
    #[allow(dead_code)]
    expert_overlay: Option<BTreeMap<String, serde_json::Value>>,
}

/// v3 `slot_registry` instance (extends the v2 fields with `zone`).
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SlotRegistryEntryV3 {
    #[serde(rename = "type")]
    pub slot_type: String,
    pub label: String,
    pub backend: String,
    pub position: i64,
    #[serde(default)]
    pub plugin: Option<String>,
    #[serde(default)]
    pub plugins: Option<Vec<String>>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub local_memory_provider_id: Option<String>,
    #[serde(default)]
    pub zone: Option<Value>,
}

/// Validate v3 blueprint JSON.
///
/// # Errors
///
/// Returns `Err(Vec<String>)` when the contract is not satisfied.
pub fn validate_blueprint_v3_json(raw: &str, folder_name: Option<&str>) -> Result<(), Vec<String>> {
    let root: Value = serde_json::from_str(raw)
        .map_err(|e| vec![format!("pipeline.ocblueprint v3 JSON 语法错误: {}", e)])?;
    let object = root
        .as_object()
        .ok_or_else(|| vec!["pipeline.ocblueprint v3 根节点须为 JSON 对象".into()])?;
    let mut shape_errors = Vec::new();
    for field in ["expert_overlay", "runtime_config", "pipeline"] {
        if object.get(field).is_some_and(|value| !value.is_object()) {
            shape_errors.push(format!(
                "pipeline.ocblueprint v3.{field} 若存在则须为 JSON 对象"
            ));
        }
    }
    if !shape_errors.is_empty() {
        return Err(shape_errors);
    }
    let bp: BlueprintV3File = serde_json::from_value(root)
        .map_err(|e| vec![format!("pipeline.ocblueprint v3 结构不符合契约: {}", e)])?;
    validate_blueprint_v3_parsed(&bp, folder_name)
}

fn validate_blueprint_v3_parsed(
    bp: &BlueprintV3File,
    folder_name: Option<&str>,
) -> Result<(), Vec<String>> {
    let mut errs = Vec::new();

    if bp.schema_version != BLUEPRINT_V3_SCHEMA_VERSION {
        errs.push(format!(
            "schema_version 须为 {}（当前 {}）",
            BLUEPRINT_V3_SCHEMA_VERSION, bp.schema_version
        ));
    }

    crate::blueprint_v2::validate_blueprint_meta_core(&bp.meta, folder_name, &mut errs);

    if let Some(ref rc) = bp.runtime_config {
        if let Err(e) = validate_runtime_config(rc) {
            errs.extend(e);
        }
        if rc.inference_profile.is_some() {
            errs.push(
                "runtime_config.inference_profile 仅属于 Stable v4；冻结 v3 不接受该字段".into(),
            );
        }
        if rc.dual_core.as_ref().is_some_and(|d| d.enabled) && bp.pipeline.is_none() {
            errs.push(
                "runtime_config.dual_core.enabled 为 true 时须提供 pipeline.stable 和/或 pipeline.experimental"
                    .into(),
            );
        }
    }

    let registry = v3_registry_to_btree(&bp.slot_registry);
    let groups: BTreeMap<String, SlotGroupEntry> = bp
        .groups
        .iter()
        .map(|(key, group)| (key.clone(), group.clone()))
        .collect();
    errs.extend(validate_slot_registry_contract(&registry, &groups, true));

    let zone_map: HashMap<String, HashSet<String>> = bp
        .slot_registry
        .iter()
        .map(|(k, e)| (k.clone(), zones_of_entry(e.zone.as_ref())))
        .collect();

    for (key, slot) in &bp.slot_registry {
        if let Err(e) = validate_zone_value(slot.zone.as_ref()) {
            errs.push(format!("slot_registry[{key}].zone {e}"));
        }
    }

    if let Some(ref pipe) = bp.pipeline {
        if let Err(mut e) = validate_pipeline_steps(
            "pipeline.stable",
            &pipe.stable,
            &bp.slot_registry,
            &zone_map,
            true,
        ) {
            errs.append(&mut e);
        }
        if let Err(mut e) = validate_pipeline_steps(
            "pipeline.experimental",
            &pipe.experimental,
            &bp.slot_registry,
            &zone_map,
            false,
        ) {
            errs.append(&mut e);
        }
    }

    if errs.is_empty() {
        Ok(())
    } else {
        Err(errs)
    }
}

fn zones_of_entry(zone: Option<&Value>) -> HashSet<String> {
    let mut set = HashSet::new();
    match zone {
        None => {
            set.insert("stable".into());
        }
        Some(Value::String(s)) => {
            set.insert(normalize_zone(s));
        }
        Some(Value::Array(arr)) => {
            for v in arr {
                if let Some(s) = v.as_str() {
                    set.insert(normalize_zone(s));
                }
            }
            if set.is_empty() {
                set.insert("stable".into());
            }
        }
        _ => {}
    }
    if set.is_empty() {
        set.insert("stable".into());
    }
    set
}

fn validate_zone_value(zone: Option<&Value>) -> Result<(), String> {
    let is_valid_name = |value: &str| matches!(value, "stable" | "experimental");
    match zone {
        None => Ok(()),
        Some(Value::String(value)) if is_valid_name(value) => Ok(()),
        Some(Value::Array(values)) if !values.is_empty() => {
            if values
                .iter()
                .all(|value| value.as_str().is_some_and(is_valid_name))
            {
                Ok(())
            } else {
                Err("数组元素只能是 stable / experimental".into())
            }
        }
        Some(Value::Array(_)) => Err("数组不能为空".into()),
        Some(_) => Err("须为 stable / experimental 字符串或非空数组".into()),
    }
}

/// Whether a v3-compatible slot entry belongs to `stable` or `experimental`.
///
/// Missing or empty `zone` follows the v3 contract and belongs to `stable`.
#[must_use]
pub fn slot_registry_entry_in_zone(entry: &SlotRegistryEntry, zone: &str) -> bool {
    zones_of_entry(entry.zone.as_ref()).contains(&normalize_zone(zone))
}

fn normalize_zone(s: &str) -> String {
    match s.trim().to_ascii_lowercase().as_str() {
        "stable" => "stable".into(),
        "experimental" => "experimental".into(),
        other => other.into(),
    }
}

fn is_experimental_only(zones: &HashSet<String>) -> bool {
    zones.len() == 1 && zones.contains("experimental")
}

pub use crate::pipeline_action::parse_pipeline_action;

fn validate_pipeline_steps(
    label: &str,
    steps: &[PipelineStep],
    registry: &HashMap<String, SlotRegistryEntryV3>,
    zone_map: &HashMap<String, HashSet<String>>,
    stable_pipeline: bool,
) -> Result<(), Vec<String>> {
    let mut errs = Vec::new();
    if steps.is_empty() {
        return Ok(());
    }

    let mut actions: HashSet<String> = HashSet::new();
    for step in steps {
        if step.action.trim().is_empty() {
            errs.push(format!("{label}：action 不能为空"));
            continue;
        }
        if !actions.insert(step.action.clone()) {
            errs.push(format!("{label}：重复的 action「{}」", step.action));
        }
        let kind = match parse_pipeline_action_kind(&step.action) {
            Ok(k) => k,
            Err(e) => {
                errs.push(format!("{label}：{e}"));
                continue;
            }
        };
        if matches!(kind, PipelineActionKind::ExpertInvoke) {
            continue;
        }
        let PipelineActionKind::Slot {
            registry_key: key,
            method: _,
        } = kind
        else {
            continue;
        };
        let Some(entry) = registry.get(&key) else {
            errs.push(format!(
                "{label}：action「{}」引用的 registry 键「{key}」不存在于 slot_registry",
                step.action
            ));
            continue;
        };
        if stable_pipeline {
            if is_experimental_only(zone_map.get(&key).unwrap_or(&HashSet::new())) {
                errs.push(format!(
                    "{label}：不得引用仅 experimental zone 的实例「{key}」"
                ));
            }
            let t = entry.slot_type.trim();
            if !STABLE_PIPELINE_SLOT_TYPES.contains(&t) {
                errs.push(format!(
                    "{label}：实例「{key}」的 type「{t}」不在 Stable 六槽集合"
                ));
            }
        }
        for dep in &step.depends_on {
            if !actions.contains(dep) && !steps.iter().any(|s| s.action == *dep) {
                // dep must exist in same pipeline
            }
        }
    }

    let action_list: Vec<String> = steps.iter().map(|s| s.action.clone()).collect();
    for step in steps {
        for dep in &step.depends_on {
            if !action_list.contains(dep) {
                errs.push(format!(
                    "{label}：depends_on「{dep}」未在同一 pipeline 中声明为 action"
                ));
            }
        }
    }

    if let Some(cycle) = find_cycle(steps) {
        errs.push(format!("{label}：depends_on 存在环（涉及「{cycle}」）"));
    }

    if errs.is_empty() {
        Ok(())
    } else {
        Err(errs)
    }
}

fn find_cycle(steps: &[PipelineStep]) -> Option<String> {
    let mut graph: HashMap<String, Vec<String>> = HashMap::new();
    for step in steps {
        for dep in &step.depends_on {
            graph
                .entry(dep.clone())
                .or_default()
                .push(step.action.clone());
        }
    }
    let mut state: HashMap<String, u8> = HashMap::new();
    for step in steps {
        if dfs_cycle(&step.action, &graph, &mut state) {
            return Some(step.action.clone());
        }
        for dep in &step.depends_on {
            if dfs_cycle(dep, &graph, &mut state) {
                return Some(dep.clone());
            }
        }
    }
    None
}

/// Structured result after the host loads a v3 blueprint.
#[derive(Debug, Clone)]
pub struct BlueprintV3LoadResult {
    pub disk: crate::manifest::DiskRoleManifest,
    pub slot_registry: BTreeMap<String, SlotRegistryEntry>,
    pub groups: BTreeMap<String, SlotGroupEntry>,
    pub runtime_config: Option<RuntimeConfig>,
    pub pipeline_experimental: Vec<PipelineStep>,
    pub interaction_mode: Option<String>,
    pub featured: bool,
    pub deep_capsule_enabled: bool,
    pub preset_order: u32,
    pub remote_presence: Option<crate::disk_role_settings::RemotePresenceConfig>,
    pub autonomous_scene: Option<crate::disk_role_settings::AutonomousSceneConfig>,
    pub reply_quality_anchor: Option<String>,
}

fn v3_entry_to_slot_registry_entry(e: &SlotRegistryEntryV3) -> SlotRegistryEntry {
    SlotRegistryEntry {
        slot_type: e.slot_type.clone(),
        label: e.label.clone(),
        backend: e.backend.clone(),
        position: e.position,
        plugin: e.plugin.clone(),
        plugins: e.plugins.clone(),
        model: e.model.clone(),
        url: e.url.clone(),
        local_memory_provider_id: e.local_memory_provider_id.clone(),
        zone: e.zone.clone(),
        policy: None,
    }
}

fn v3_registry_to_btree(
    registry: &HashMap<String, SlotRegistryEntryV3>,
) -> BTreeMap<String, SlotRegistryEntry> {
    registry
        .iter()
        .map(|(k, e)| (k.clone(), v3_entry_to_slot_registry_entry(e)))
        .collect()
}

pub(crate) fn apply_runtime_config_to_disk(
    disk: &mut crate::manifest::DiskRoleManifest,
    rc: &RuntimeConfig,
) {
    if let Some(ref m) = rc.memory_config {
        disk.memory_config = m.clone();
    }
    if let Some(ref e) = rc.evolution {
        disk.evolution = e.clone();
    }
    if let Some(ref id) = rc.identity_binding {
        disk.identity_binding = *id;
    }
    if rc.ollama_model.is_some() {
        disk.ollama_model = rc.ollama_model.clone();
    }
}

fn blueprint_v3_file_to_load_result(bp: &BlueprintV3File) -> BlueprintV3LoadResult {
    let mut disk = meta_to_disk_manifest(&bp.meta);
    let interaction_mode = bp
        .runtime_config
        .as_ref()
        .and_then(|r| r.interaction_mode.clone())
        .or_else(|| bp.meta.interaction_mode.clone());
    let remote_presence = bp
        .runtime_config
        .as_ref()
        .and_then(|r| r.remote_presence.clone())
        .or_else(|| bp.meta.remote_presence.clone());
    let autonomous_scene = bp
        .runtime_config
        .as_ref()
        .and_then(|r| r.autonomous_scene.clone())
        .or_else(|| bp.meta.autonomous_scene.clone());
    let reply_quality_anchor = bp
        .runtime_config
        .as_ref()
        .and_then(|r| r.reply_quality_anchor.clone())
        .or_else(|| bp.meta.reply_quality_anchor.clone());

    if let Some(ref rc) = bp.runtime_config {
        apply_runtime_config_to_disk(&mut disk, rc);
    }

    let pipeline_experimental = bp
        .pipeline
        .as_ref()
        .map(|p| p.experimental.clone())
        .unwrap_or_default();

    BlueprintV3LoadResult {
        disk,
        slot_registry: v3_registry_to_btree(&bp.slot_registry),
        groups: bp
            .groups
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect(),
        runtime_config: bp.runtime_config.clone(),
        pipeline_experimental,
        interaction_mode,
        featured: bp.meta.featured,
        deep_capsule_enabled: bp.meta.deep_capsule_enabled,
        preset_order: bp.meta.preset_order,
        remote_presence,
        autonomous_scene,
        reply_quality_anchor,
    }
}

/// Validate and load `pipeline.ocblueprint` under a v3 role directory.
///
/// # Errors
///
/// Returns `Err(Vec<String>)` when contract or on-disk validation fails.
pub fn validate_role_pack_blueprint_v3_directory(
    role_dir: &Path,
    host_version: &str,
) -> Result<(), Vec<String>> {
    let mut legacy_errors = Vec::new();
    for legacy_name in ["manifest.json", "settings.json"] {
        let legacy_path = role_dir.join(legacy_name);
        if legacy_path.is_file() {
            legacy_errors.push(format!(
                "v3 角色包不得包含 {legacy_name}（已废弃）：{}",
                legacy_path.display()
            ));
        }
    }
    if !legacy_errors.is_empty() {
        return Err(legacy_errors);
    }

    let blueprint_path = role_dir.join(PIPELINE_BLUEPRINT_FILENAME);
    if !blueprint_path.is_file() {
        return Err(vec![format!(
            "缺少 {}：{}",
            PIPELINE_BLUEPRINT_FILENAME,
            blueprint_path.display()
        )]);
    }
    let raw = fs::read_to_string(&blueprint_path)
        .map_err(|e| vec![format!("读取 {} 失败: {}", blueprint_path.display(), e)])?;
    let folder_name = role_dir.file_name().and_then(|s| s.to_str());
    validate_blueprint_v3_json(&raw, folder_name)?;
    let resolved = crate::blueprint_includes::merge_blueprint_includes_strict(role_dir, &raw)?;
    validate_blueprint_v3_json(&resolved, folder_name)?;

    let bp: BlueprintV3File = serde_json::from_str(&resolved)
        .map_err(|e| vec![format!("pipeline.ocblueprint v3 结构不符合契约: {}", e)])?;
    let mut disk = meta_to_disk_manifest(&bp.meta);
    if let Some(ref rc) = bp.runtime_config {
        apply_runtime_config_to_disk(&mut disk, rc);
    }
    let merged_scenes = merge_role_pack_scene_ids(role_dir, &disk.scenes).map_err(|e| vec![e])?;
    validate_disk_manifest(&disk, &merged_scenes).map_err(|e| vec![e])?;
    validate_min_runtime_version(disk.min_runtime_version.as_deref(), host_version)
        .map_err(|e| vec![e])?;
    Ok(())
}

/// Load v3 blueprint from a role directory (includes `runtime_config` and `pipeline.experimental`).
///
/// # Errors
///
/// Returns `Err(Vec<String>)` when contract or on-disk validation fails.
pub fn load_blueprint_v3_for_role_dir(
    role_dir: &Path,
    host_version: &str,
) -> Result<BlueprintV3LoadResult, Vec<String>> {
    validate_role_pack_blueprint_v3_directory(role_dir, host_version)?;
    let raw = fs::read_to_string(role_dir.join(PIPELINE_BLUEPRINT_FILENAME))
        .map_err(|e| vec![format!("读取 {} 失败: {}", PIPELINE_BLUEPRINT_FILENAME, e)])?;
    let resolved = crate::blueprint_includes::merge_blueprint_includes_strict(role_dir, &raw)?;
    let folder_name = role_dir.file_name().and_then(|s| s.to_str()).unwrap_or("");
    validate_blueprint_v3_json(&resolved, Some(folder_name))?;
    let bp: BlueprintV3File = serde_json::from_str(&resolved)
        .map_err(|e| vec![format!("pipeline.ocblueprint v3 结构不符合契约: {}", e)])?;
    Ok(blueprint_v3_file_to_load_result(&bp))
}

fn dfs_cycle(
    node: &str,
    graph: &HashMap<String, Vec<String>>,
    state: &mut HashMap<String, u8>,
) -> bool {
    match state.get(node).copied().unwrap_or(0) {
        1 => return true,
        2 => return false,
        _ => {}
    }
    state.insert(node.to_string(), 1);
    if let Some(nexts) = graph.get(node) {
        for n in nexts {
            if dfs_cycle(n, graph, state) {
                return true;
            }
        }
    }
    state.insert(node.to_string(), 2);
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blueprint_dispatch::validate_blueprint_json_by_schema_version;

    fn minimal_v3_json() -> String {
        r#"{
          "schema_version": 3,
          "meta": {
            "id": "test",
            "name": "T",
            "version": "1",
            "author": "a",
            "description": "d",
            "relations": { "f": { "initial_favorability": 50, "favor_multiplier": 1.0 } },
            "default_relation": "f"
          },
          "slot_registry": {
            "emotion": { "type": "emotion", "label": "E", "backend": "builtin", "position": 1 },
            "llm": { "type": "llm", "label": "L", "backend": "ollama", "position": 2 }
          },
          "runtime_config": { "dual_core": { "enabled": false } }
        }"#
        .to_string()
    }

    #[test]
    fn accepts_valid_v3() {
        validate_blueprint_v3_json(&minimal_v3_json(), Some("test")).unwrap();
    }

    #[test]
    fn rejects_unknown_fields_invalid_zone_and_invalid_backend() {
        let mut unknown: Value = serde_json::from_str(&minimal_v3_json()).unwrap();
        unknown["vendor_blob"] = serde_json::json!({});
        let unknown_errors =
            validate_blueprint_v3_json(&unknown.to_string(), Some("test")).unwrap_err();
        assert!(unknown_errors
            .iter()
            .any(|error| error.contains("unknown field") || error.contains("未知")));

        let mut legacy_runtime_alias: Value = serde_json::from_str(&minimal_v3_json()).unwrap();
        legacy_runtime_alias["runtime_config"]["model"] = serde_json::json!("qwen");
        let alias_errors =
            validate_blueprint_v3_json(&legacy_runtime_alias.to_string(), Some("test"))
                .unwrap_err();
        assert!(alias_errors.iter().any(|error| error.contains("model")));

        let mut stable_v4_only: Value = serde_json::from_str(&minimal_v3_json()).unwrap();
        stable_v4_only["runtime_config"]["inference_profile"] = serde_json::json!({
            "generation": { "temperature": 0.8 }
        });
        let stable_v4_only_errors =
            validate_blueprint_v3_json(&stable_v4_only.to_string(), Some("test")).unwrap_err();
        assert!(stable_v4_only_errors
            .iter()
            .any(|error| error.contains("Stable v4")));

        let mut invalid_overlay: Value = serde_json::from_str(&minimal_v3_json()).unwrap();
        invalid_overlay["expert_overlay"] = Value::Null;
        let overlay_errors =
            validate_blueprint_v3_json(&invalid_overlay.to_string(), Some("test")).unwrap_err();
        assert!(overlay_errors
            .iter()
            .any(|error| error.contains("expert_overlay")));

        let mut bad_zone: Value = serde_json::from_str(&minimal_v3_json()).unwrap();
        bad_zone["slot_registry"]["llm"]["zone"] = serde_json::json!("background");
        let zone_errors =
            validate_blueprint_v3_json(&bad_zone.to_string(), Some("test")).unwrap_err();
        assert!(zone_errors.iter().any(|error| error.contains("zone")));

        let mut bad_backend: Value = serde_json::from_str(&minimal_v3_json()).unwrap();
        bad_backend["slot_registry"]["emotion"]["backend"] = serde_json::json!("ollama");
        let backend_errors =
            validate_blueprint_v3_json(&bad_backend.to_string(), Some("test")).unwrap_err();
        assert!(backend_errors.iter().any(|error| error.contains("backend")));
    }

    #[test]
    fn rejects_duplicate_positions_and_group_type_mismatch() {
        let mut value: Value = serde_json::from_str(&minimal_v3_json()).unwrap();
        value["slot_registry"]["emotion_second"] = serde_json::json!({
            "type": "emotion",
            "label": "E2",
            "backend": "builtin",
            "position": 1
        });
        value["groups"] = serde_json::json!({
            "bad": {
                "label": "Bad",
                "type": "memory",
                "members": ["llm"]
            }
        });
        let errors = validate_blueprint_v3_json(&value.to_string(), Some("test")).unwrap_err();
        assert!(errors.iter().any(|error| error.contains("position")));
        assert!(errors
            .iter()
            .any(|error| error.contains("不一致") || error.contains("groups")));
    }

    #[test]
    fn rejects_bad_action_format() {
        let mut j = minimal_v3_json();
        j = j.replace(
            "\"runtime_config\"",
            "\"pipeline\": { \"stable\": [{ \"action\": \"bad\", \"depends_on\": [] }] }, \"runtime_config\"",
        );
        let errs = validate_blueprint_v3_json(&j, Some("test")).unwrap_err();
        assert!(errs.iter().any(|e| e.contains("slot.<registry_key>")));
    }

    #[test]
    fn rejects_stable_refs_experimental_only_zone() {
        let raw = r#"{
          "schema_version": 3,
          "meta": {
            "id": "test", "name": "T", "version": "1", "author": "a", "description": "d",
            "relations": { "f": { "initial_favorability": 50, "favor_multiplier": 1.0 } },
            "default_relation": "f"
          },
          "slot_registry": {
            "exp_mem": { "type": "memory", "label": "X", "backend": "builtin", "position": 1, "zone": "experimental" },
            "llm": { "type": "llm", "label": "L", "backend": "ollama", "position": 2 }
          },
          "pipeline": {
            "stable": [{ "action": "slot.exp_mem.retrieve", "depends_on": [] }]
          }
        }"#;
        let errs = validate_blueprint_v3_json(raw, Some("test")).unwrap_err();
        assert!(errs.iter().any(|e| e.contains("experimental")));
    }

    #[test]
    fn load_v3_for_role_dir_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let role = dir.path().join("dual");
        fs::create_dir_all(role.join("scenes").join("default")).unwrap();
        let raw = minimal_v3_json().replace("\"test\"", "\"dual\"");
        fs::write(role.join(PIPELINE_BLUEPRINT_FILENAME), raw).unwrap();
        let loaded = load_blueprint_v3_for_role_dir(&role, "0.2.0").unwrap();
        assert_eq!(loaded.disk.id, "dual");
        assert!(loaded.runtime_config.is_some());
    }

    #[test]
    fn v2_warns_on_runtime_config() {
        let raw = r#"{
          "schema_version": 2,
          "meta": {
            "id": "x", "name": "n", "version": "1", "author": "a", "description": "d",
            "relations": { "f": { "initial_favorability": 50, "favor_multiplier": 1.0 } },
            "default_relation": "f"
          },
          "slot_registry": {
            "llm": { "type": "llm", "label": "L", "backend": "ollama", "position": 1 }
          },
          "runtime_config": { "interaction_mode": "immersive" }
        }"#;
        let warnings = validate_blueprint_json_by_schema_version(raw, None).unwrap();
        assert!(!warnings.is_empty());
    }

    #[test]
    fn slot_zone_defaults_to_stable_and_supports_dual_membership() {
        let mut entry = SlotRegistryEntry {
            slot_type: "llm".into(),
            label: "LLM".into(),
            backend: "directory".into(),
            position: 0,
            plugin: Some("com.example.lora".into()),
            plugins: None,
            model: None,
            url: None,
            local_memory_provider_id: None,
            zone: None,
            policy: None,
        };
        assert!(slot_registry_entry_in_zone(&entry, "stable"));
        assert!(!slot_registry_entry_in_zone(&entry, "experimental"));

        entry.zone = Some(serde_json::json!(["stable", "experimental"]));
        assert!(slot_registry_entry_in_zone(&entry, "stable"));
        assert!(slot_registry_entry_in_zone(&entry, "experimental"));
    }
}
