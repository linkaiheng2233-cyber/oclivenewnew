//! `pipeline.ocblueprint` schema_version 3 (dual-core P1 contract validation).

use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use crate::blueprint_includes::validate_includes;
use crate::blueprint_v2::{
    meta_to_disk_manifest, BlueprintMeta, SlotGroupEntry, SlotRegistryEntry,
    BLUEPRINT_V2_SCHEMA_VERSION, PIPELINE_BLUEPRINT_FILENAME,
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
pub struct PipelineStep {
    pub action: String,
    #[serde(default)]
    pub depends_on: Vec<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct DualPipelineDef {
    #[serde(default)]
    pub stable: Vec<PipelineStep>,
    #[serde(default)]
    pub experimental: Vec<PipelineStep>,
}

#[derive(Debug, Clone, Deserialize)]
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
    includes: Vec<crate::blueprint_includes::BlueprintIncludeEntry>,
    #[serde(default)]
    #[allow(dead_code)]
    expert_overlay: Option<serde_json::Value>,
}

/// v3 `slot_registry` instance (extends the v2 fields with `zone`).
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
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

/// Dispatch by `schema_version`: 2 → v2 validation; 3 → v3 validation.
///
/// # Errors
///
/// Returns `Err(Vec<String>)` on an unknown version or contract failure.
pub fn validate_blueprint_json_by_schema_version(
    raw: &str,
    folder_name: Option<&str>,
) -> Result<Vec<String>, Vec<String>> {
    let root: Value = serde_json::from_str(raw)
        .map_err(|e| vec![format!("pipeline.ocblueprint JSON 语法错误: {}", e)])?;
    let version = root
        .get("schema_version")
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as u32;
    match version {
        BLUEPRINT_V2_SCHEMA_VERSION => {
            crate::blueprint_v2::validate_blueprint_v2_json(raw)?;
            Ok(warnings_for_v2_runtime_config(&root))
        }
        BLUEPRINT_V3_SCHEMA_VERSION => {
            validate_blueprint_v3_json(raw, folder_name)?;
            Ok(Vec::new())
        }
        other => Err(vec![format!(
            "pipeline.ocblueprint：不支持的 schema_version {other}（支持 {} 或 {}）",
            BLUEPRINT_V2_SCHEMA_VERSION, BLUEPRINT_V3_SCHEMA_VERSION
        )]),
    }
}

fn warnings_for_v2_runtime_config(root: &Value) -> Vec<String> {
    if root.get("runtime_config").is_some() {
        vec![
            "注意：schema_version 2 下顶层 runtime_config 不参与宿主加载（将被忽略）；请升级到 schema_version 3 或把字段写在 meta（过渡期）"
                .into(),
        ]
    } else {
        Vec::new()
    }
}

/// Validate v3 blueprint JSON.
///
/// # Errors
///
/// Returns `Err(Vec<String>)` when the contract is not satisfied.
pub fn validate_blueprint_v3_json(raw: &str, folder_name: Option<&str>) -> Result<(), Vec<String>> {
    let bp: BlueprintV3File = serde_json::from_str(raw)
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
        if rc.dual_core.as_ref().is_some_and(|d| d.enabled) && bp.pipeline.is_none() {
            errs.push(
                "runtime_config.dual_core.enabled 为 true 时须提供 pipeline.stable 和/或 pipeline.experimental"
                    .into(),
            );
        }
    }

    if bp.slot_registry.is_empty() {
        errs.push("slot_registry 不能为空".into());
    }

    let mut llm_count = 0usize;
    let zone_map: HashMap<String, HashSet<String>> = bp
        .slot_registry
        .iter()
        .map(|(k, e)| (k.clone(), zones_of_entry(e.zone.as_ref())))
        .collect();

    for (key, slot) in &bp.slot_registry {
        if slot.label.trim().is_empty() {
            errs.push(format!("slot_registry[{key}].label 不能为空"));
        }
        let t = slot.slot_type.trim();
        if !PLUGIN_HOST_SLOT_TYPES.contains(&t) {
            errs.push(format!(
                "slot_registry[{key}].type「{t}」不在 PluginHost 门面集合（允许: {}）",
                PLUGIN_HOST_SLOT_TYPES.join(", ")
            ));
        }
        if t == "llm" {
            llm_count += 1;
        }
    }
    if llm_count == 0 {
        errs.push("slot_registry 须至少包含一个 type: llm 实例".into());
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
    pub preset_order: u32,
    pub remote_presence: Option<crate::disk_role_settings::RemotePresenceConfig>,
    pub autonomous_scene: Option<crate::disk_role_settings::AutonomousSceneConfig>,
    pub reply_quality_anchor: Option<String>,
}

/// Read `schema_version` from blueprint JSON (`None` when parsing fails).
#[must_use]
pub fn blueprint_schema_version_from_raw(raw: &str) -> Option<u32> {
    serde_json::from_str::<Value>(raw)
        .ok()
        .and_then(|v| v.get("schema_version").and_then(|n| n.as_u64()))
        .map(|n| n as u32)
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

fn apply_runtime_config_to_disk(disk: &mut crate::manifest::DiskRoleManifest, rc: &RuntimeConfig) {
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
    if let Ok(bp) = serde_json::from_str::<BlueprintV3File>(&raw) {
        if !bp.includes.is_empty() {
            validate_includes(role_dir, &bp.includes)?;
        }
    }
    validate_blueprint_v3_json(&raw, folder_name)?;

    let bp: BlueprintV3File = serde_json::from_str(&raw)
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
    let resolved = crate::blueprint_includes::merge_blueprint_includes_lenient(role_dir, &raw);
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
}
