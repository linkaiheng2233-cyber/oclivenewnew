//! `pipeline.ocblueprint` schema_version 2 校验（角色包 SSOT）。

use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::disk_role_settings::{AutonomousSceneConfig, RemotePresenceConfig};
use crate::manifest::{
    DiskRoleManifest, EvolutionConfigDisk, IdentityBinding, KnowledgePackConfigDisk,
    MemoryConfigDisk, UserRelationDisk,
};
use crate::plugin_backends::{
    AgentBackend, DirectoryPluginSlots, EmotionBackend, EventBackend, LlmBackend, MemoryBackend,
    PluginBackends, PromptBackend,
};
use crate::role_pack::{merge_role_pack_scene_ids, validate_default_personality_vector};
use crate::validate::{
    validate_disk_manifest, validate_interaction_mode_pack_setting,
    validate_knowledge_manifest_disk, validate_min_runtime_version,
};

pub const BLUEPRINT_V2_SCHEMA_VERSION: u32 = 2;
pub const PIPELINE_BLUEPRINT_FILENAME: &str = "pipeline.ocblueprint";

const FORBIDDEN_ROOT_KEYS: &[&str] = &["module_relations", "steps", "entry"];

const SLOT_TYPES: &[&str] = &[
    "memory",
    "emotion",
    "event",
    "prompt",
    "llm",
    "agent",
    "complex_emotion",
];

const PERSONALITY_OBJECT_KEYS: &[&str] = &[
    "stubbornness",
    "clinginess",
    "sensitivity",
    "assertiveness",
    "forgiveness",
    "talkativeness",
    "warmth",
];

/// 蓝图 `slot_registry` 单实例（与 `pipeline.ocblueprint` v2 文件一致）。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SlotRegistryEntry {
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
}

/// 校验通过后的 v2 蓝图加载结果（供宿主 `RoleStorage` 映射为 `Role`）。
#[derive(Debug, Clone)]
pub struct BlueprintV2LoadResult {
    pub disk: DiskRoleManifest,
    pub slot_registry: BTreeMap<String, SlotRegistryEntry>,
    pub interaction_mode: Option<String>,
    pub remote_presence: Option<RemotePresenceConfig>,
    pub autonomous_scene: Option<AutonomousSceneConfig>,
    pub reply_quality_anchor: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct BlueprintV2File {
    schema_version: u32,
    meta: BlueprintMeta,
    slot_registry: BTreeMap<String, SlotRegistryEntry>,
}

#[derive(Debug, Clone, Deserialize)]
struct BlueprintMeta {
    pub id: String,
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    #[serde(default)]
    pub personality: Option<serde_json::Value>,
    #[serde(default)]
    pub relations: HashMap<String, UserRelationDisk>,
    #[serde(default)]
    pub default_relation: String,
    #[serde(default)]
    pub scenes: Vec<String>,
    #[serde(default)]
    pub evolution: EvolutionConfigDisk,
    #[serde(default)]
    pub memory_config: MemoryConfigDisk,
    #[serde(default)]
    pub identity_binding: IdentityBinding,
    #[serde(default, alias = "model")]
    pub ollama_model: Option<String>,
    #[serde(default)]
    pub interaction_mode: Option<String>,
    #[serde(default)]
    pub knowledge: Option<KnowledgePackConfigDisk>,
    #[serde(default)]
    pub min_runtime_version: Option<String>,
    #[serde(default)]
    pub dev_only: bool,
    #[serde(default)]
    pub remote_presence: Option<RemotePresenceConfig>,
    #[serde(default)]
    pub autonomous_scene: Option<AutonomousSceneConfig>,
    #[serde(default)]
    pub reply_quality_anchor: Option<String>,
}

/// 可选上下文：目录名校验、`scenes/` 合并、`min_runtime_version` 与宿主比较。
#[derive(Debug, Clone, Copy, Default)]
pub struct BlueprintV2ValidateContext<'a> {
    /// 角色包目录名，与 `meta.id` 比对（R4）。
    pub folder_name: Option<&'a str>,
    /// 提供时合并 `scenes/` 子目录并跑完整 `validate_disk_manifest`。
    pub role_dir: Option<&'a Path>,
    /// 提供时校验 `meta.min_runtime_version`。
    pub host_version: Option<&'a str>,
}

/// 校验 v2 蓝图 JSON 文本（槽位 + meta 结构；不含宿主版本时可省略 `min_runtime`）。
///
/// # Errors
///
/// 契约不符时返回 `Err(Vec<String>)`。
pub fn validate_blueprint_v2_json(raw: &str) -> Result<(), Vec<String>> {
    validate_blueprint_v2_json_with_context(raw, BlueprintV2ValidateContext::default())
}

/// 带上下文的 v2 蓝图 JSON 校验（CLI / 目录校验共用）。
///
/// # Errors
///
/// 契约不符时返回 `Err(Vec<String>)`。
pub fn validate_blueprint_v2_json_with_context(
    raw: &str,
    ctx: BlueprintV2ValidateContext<'_>,
) -> Result<(), Vec<String>> {
    let bp = parse_blueprint_v2_root(raw)?;
    validate_blueprint_v2_file(&bp, ctx)
}

fn parse_blueprint_v2_root(raw: &str) -> Result<BlueprintV2File, Vec<String>> {
    let root: Value = match serde_json::from_str(raw) {
        Ok(v) => v,
        Err(e) => return Err(vec![format!("pipeline.ocblueprint JSON 语法错误: {}", e)]),
    };

    let mut errs = Vec::new();
    if let Value::Object(map) = &root {
        for key in FORBIDDEN_ROOT_KEYS {
            if map.contains_key(*key) {
                errs.push(format!(
                    "pipeline.ocblueprint：禁止顶层字段「{key}」（B3：module_relations 仅运行时派生；steps/entry 已废弃）"
                ));
            }
        }
    } else {
        return Err(vec!["pipeline.ocblueprint 根节点须为 JSON 对象".into()]);
    }

    if !errs.is_empty() {
        return Err(errs);
    }

    serde_json::from_value(root)
        .map_err(|e| vec![format!("pipeline.ocblueprint 结构不符合 v2 契约: {}", e)])
}

fn validate_blueprint_v2_file(
    bp: &BlueprintV2File,
    ctx: BlueprintV2ValidateContext<'_>,
) -> Result<(), Vec<String>> {
    validate_blueprint_v2_parsed(bp, ctx.folder_name)?;

    let disk = meta_to_disk_manifest(&bp.meta);
    let merged_scenes = merged_scenes_for_validate(ctx.role_dir, &disk.scenes)?;
    if let Err(e) = validate_disk_manifest(&disk, &merged_scenes) {
        return Err(vec![e]);
    }

    if let Some(ref m) = bp.meta.interaction_mode {
        if let Err(e) = validate_interaction_mode_pack_setting(Some(m.as_str())) {
            return Err(vec![e]);
        }
    }

    if let Some(host) = ctx.host_version {
        if let Err(e) = validate_min_runtime_version(bp.meta.min_runtime_version.as_deref(), host) {
            return Err(vec![e]);
        }
    }

    Ok(())
}

fn merged_scenes_for_validate(
    role_dir: Option<&Path>,
    manifest_scenes: &[String],
) -> Result<Vec<String>, Vec<String>> {
    match role_dir {
        Some(dir) => merge_role_pack_scene_ids(dir, manifest_scenes).map_err(|e| vec![e]),
        None => {
            let mut scenes: Vec<String> = manifest_scenes
                .iter()
                .filter(|s| !s.trim().is_empty())
                .cloned()
                .collect();
            if scenes.is_empty() {
                scenes.push("default".into());
            }
            Ok(scenes)
        }
    }
}

/// 校验角色包目录（v2 SSOT：`pipeline.ocblueprint`；不得含 manifest/settings）。
///
/// # Errors
///
/// 缺少文件、读盘失败或校验未通过时返回 `Err(Vec<String>)`。
pub fn validate_role_pack_blueprint_v2_directory(
    role_dir: &Path,
    host_version: &str,
) -> Result<(), Vec<String>> {
    let mut errs = Vec::new();

    let manifest_path = role_dir.join("manifest.json");
    if manifest_path.is_file() {
        errs.push(format!(
            "v2 角色包不得包含 manifest.json（已废弃）：{}",
            manifest_path.display()
        ));
    }
    let settings_path = role_dir.join("settings.json");
    if settings_path.is_file() {
        errs.push(format!(
            "v2 角色包不得包含 settings.json（已废弃）：{}",
            settings_path.display()
        ));
    }

    let blueprint_path = role_dir.join(PIPELINE_BLUEPRINT_FILENAME);
    if !blueprint_path.is_file() {
        errs.push(format!(
            "缺少 {}：{}",
            PIPELINE_BLUEPRINT_FILENAME,
            blueprint_path.display()
        ));
        return Err(errs);
    }

    if !errs.is_empty() {
        return Err(errs);
    }

    let raw = match fs::read_to_string(&blueprint_path) {
        Ok(s) => s,
        Err(e) => {
            return Err(vec![format!(
                "读取 {} 失败: {}",
                PIPELINE_BLUEPRINT_FILENAME, e
            )]);
        }
    };

    let folder_name = role_dir.file_name().and_then(|s| s.to_str()).unwrap_or("");
    validate_blueprint_v2_json_with_context(
        &raw,
        BlueprintV2ValidateContext {
            folder_name: Some(folder_name),
            role_dir: Some(role_dir),
            host_version: Some(host_version),
        },
    )
}

/// 将 `slot_registry` 写回 `pipeline.ocblueprint`（保留 `meta` 等其余字段），写前全量校验。
///
/// # Errors
///
/// 缺少蓝图、JSON 非法或校验未通过时返回 `Err(Vec<String>)`。
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
    validate_blueprint_v2_json_with_context(
        &out,
        BlueprintV2ValidateContext {
            folder_name: Some(folder_name),
            role_dir: Some(role_dir),
            host_version: Some(host_version),
        },
    )?;
    fs::write(&blueprint_path, format!("{out}\n"))
        .map_err(|e| vec![format!("写入 {} 失败: {e}", blueprint_path.display())])?;
    Ok(())
}

/// 读取并校验角色包目录中的 v2 蓝图，返回宿主加载用结构。
///
/// # Errors
///
/// 与 [`validate_role_pack_blueprint_v2_directory`] 相同。
pub fn load_blueprint_v2_for_role_dir(
    role_dir: &Path,
    host_version: &str,
) -> Result<BlueprintV2LoadResult, Vec<String>> {
    validate_role_pack_blueprint_v2_directory(role_dir, host_version)?;
    let raw = fs::read_to_string(role_dir.join(PIPELINE_BLUEPRINT_FILENAME))
        .map_err(|e| vec![format!("读取 {} 失败: {}", PIPELINE_BLUEPRINT_FILENAME, e)])?;
    let bp = parse_blueprint_v2_root(&raw)?;
    Ok(blueprint_v2_file_to_load_result(&bp))
}

/// 将 `slot_registry` 折叠为现网六槽 `PluginBackends`（同 type **last-wins**，按 `position`）。
#[must_use]
pub fn slot_registry_to_plugin_backends(
    registry: &BTreeMap<String, SlotRegistryEntry>,
) -> PluginBackends {
    let mut winners: HashMap<&str, (&str, &SlotRegistryEntry)> = HashMap::new();
    for (key, entry) in registry {
        let t = entry.slot_type.trim();
        let keep = winners
            .get(t)
            .map(|(_, e)| entry.position >= e.position)
            .unwrap_or(true);
        if keep {
            winners.insert(t, (key.as_str(), entry));
        }
    }

    let mut pb = PluginBackends::default();
    let mut dir = DirectoryPluginSlots::default();

    if let Some((_, e)) = winners.get("memory") {
        if let Ok(b) = parse_backend_wire::<MemoryBackend>(&e.backend) {
            pb.memory = b;
        }
        if b_is_local(&e.backend) {
            pb.local_memory_provider_id = e.local_memory_provider_id.clone();
        }
        if b_is_directory(&e.backend) {
            dir.memory = single_plugin_id(e);
        }
    }
    if let Some((_, e)) = winners.get("emotion") {
        if let Ok(b) = parse_backend_wire::<EmotionBackend>(&e.backend) {
            pb.emotion = b;
        }
        if b_is_directory(&e.backend) {
            dir.emotion = single_plugin_id(e);
        }
    }
    if let Some((_, e)) = winners.get("event") {
        if let Ok(b) = parse_backend_wire::<EventBackend>(&e.backend) {
            pb.event = b;
        }
        if b_is_directory(&e.backend) {
            dir.event = single_plugin_id(e);
        }
    }
    if let Some((_, e)) = winners.get("prompt") {
        if let Ok(b) = parse_backend_wire::<PromptBackend>(&e.backend) {
            pb.prompt = b;
        }
        if b_is_directory(&e.backend) {
            dir.prompt = single_plugin_id(e);
        }
    }
    if let Some((_, e)) = winners.get("llm") {
        if let Ok(b) = parse_backend_wire::<LlmBackend>(&e.backend) {
            pb.llm = b;
        }
        if b_is_directory(&e.backend) {
            dir.llm = single_plugin_id(e);
        }
    }
    if let Some((_, e)) = winners.get("agent") {
        if let Ok(b) = parse_backend_wire::<AgentBackend>(&e.backend) {
            pb.agent = b;
        }
        if b_is_directory(&e.backend) {
            dir.agent = single_plugin_id(e).or_else(|| {
                e.plugins
                    .as_ref()
                    .and_then(|ps| ps.first())
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
            });
        }
    }

    pb.directory_plugins = dir;
    pb
}

fn blueprint_v2_file_to_load_result(bp: &BlueprintV2File) -> BlueprintV2LoadResult {
    BlueprintV2LoadResult {
        disk: meta_to_disk_manifest(&bp.meta),
        slot_registry: bp.slot_registry.clone(),
        interaction_mode: bp.meta.interaction_mode.clone(),
        remote_presence: bp.meta.remote_presence.clone(),
        autonomous_scene: bp.meta.autonomous_scene.clone(),
        reply_quality_anchor: bp.meta.reply_quality_anchor.clone(),
    }
}

fn parse_backend_wire<T: serde::de::DeserializeOwned>(backend: &str) -> Result<T, ()> {
    serde_json::from_value(Value::String(backend.trim().to_string())).map_err(|_| ())
}

fn b_is_directory(backend: &str) -> bool {
    backend.trim() == "directory"
}

fn b_is_local(backend: &str) -> bool {
    backend.trim() == "local"
}

/// 会话级对单实例的覆盖（不落盘）。
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct SlotOverridePatch {
    #[serde(default)]
    pub backend: Option<String>,
    #[serde(default)]
    pub plugin: Option<String>,
    #[serde(default)]
    pub plugins: Option<Vec<String>>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub local_memory_provider_id: Option<String>,
}

impl SlotOverridePatch {
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.backend.as_ref().is_none_or(|s| s.trim().is_empty())
            && self.plugin.as_ref().is_none_or(|s| s.trim().is_empty())
            && self.plugins.as_ref().is_none_or(|v| v.is_empty())
            && self.model.as_ref().is_none_or(|s| s.trim().is_empty())
            && self
                .local_memory_provider_id
                .as_ref()
                .is_none_or(|s| s.trim().is_empty())
    }
}

/// 将包默认 `slot_registry` 与命名空间覆盖合并为 effective 视图。
#[must_use]
pub fn effective_slot_registry(
    pack: &BTreeMap<String, SlotRegistryEntry>,
    overrides: &BTreeMap<String, SlotOverridePatch>,
) -> BTreeMap<String, SlotRegistryEntry> {
    let mut out = pack.clone();
    for (key, patch) in overrides {
        if patch.is_empty() {
            continue;
        }
        if let Some(entry) = out.get_mut(key) {
            apply_slot_override(entry, patch);
        }
    }
    out
}

/// 默认六槽模块名 → `slot_registry` 键（C1 薄包装）。
#[must_use]
pub fn default_slot_key_for_module(module: &str) -> Option<&'static str> {
    match module.trim().to_ascii_lowercase().as_str() {
        "memory" => Some("memory"),
        "emotion" => Some("emotion"),
        "event" => Some("event"),
        "prompt" => Some("prompt"),
        "llm" => Some("llm"),
        "agent" => Some("agent"),
        "complex_emotion" => Some("complex_emotion"),
        _ => None,
    }
}

pub fn apply_slot_override(entry: &mut SlotRegistryEntry, patch: &SlotOverridePatch) {
    if let Some(ref b) = patch.backend {
        let t = b.trim();
        if !t.is_empty() {
            entry.backend = t.to_string();
        }
    }
    if patch.plugin.is_some() {
        entry.plugin = patch.plugin.clone();
    }
    if patch.plugins.is_some() {
        entry.plugins = patch.plugins.clone();
    }
    if patch.model.is_some() {
        entry.model = patch.model.clone();
    }
    if patch.local_memory_provider_id.is_some() {
        entry.local_memory_provider_id = patch.local_memory_provider_id.clone();
    }
}

fn single_plugin_id(entry: &SlotRegistryEntry) -> Option<String> {
    entry
        .plugin
        .as_ref()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

/// 同 `type` 内按 `position` 升序排列的实例列表（P3 多实例解析）。
#[must_use]
pub fn slot_registry_instances_sorted(
    registry: &BTreeMap<String, SlotRegistryEntry>,
    slot_type: &str,
) -> Vec<(String, SlotRegistryEntry)> {
    let want = slot_type.trim();
    let mut v: Vec<_> = registry
        .iter()
        .filter(|(_, e)| e.slot_type.trim() == want)
        .map(|(k, e)| (k.clone(), e.clone()))
        .collect();
    v.sort_by_key(|(_, e)| e.position);
    v
}

/// 单实例 → 折叠后的六槽 `PluginBackends`（仅该实例 `type` 对应槽非默认）。
#[must_use]
pub fn plugin_backends_for_slot_entry(entry: &SlotRegistryEntry) -> PluginBackends {
    let mut one = BTreeMap::new();
    one.insert("_".to_string(), entry.clone());
    slot_registry_to_plugin_backends(&one)
}

/// 所有 `type: agent` 且 `backend: directory` 的 `plugin` / `plugins[]` 合并（去重、字典序）。
#[must_use]
pub fn merged_agent_directory_plugin_ids(
    registry: &BTreeMap<String, SlotRegistryEntry>,
) -> Vec<String> {
    let mut ids = Vec::new();
    for (_, entry) in slot_registry_instances_sorted(registry, "agent") {
        if entry.backend.trim() != "directory" {
            continue;
        }
        if let Some(p) = single_plugin_id(&entry) {
            ids.push(p);
        }
        if let Some(ps) = &entry.plugins {
            for p in ps {
                let t = p.trim();
                if !t.is_empty() {
                    ids.push(t.to_string());
                }
            }
        }
    }
    ids.sort();
    ids.dedup();
    ids
}

fn validate_blueprint_v2_parsed(
    bp: &BlueprintV2File,
    folder_name: Option<&str>,
) -> Result<(), Vec<String>> {
    let mut errs = Vec::new();

    if bp.schema_version != BLUEPRINT_V2_SCHEMA_VERSION {
        errs.push(format!(
            "pipeline.ocblueprint：schema_version 须为 {}（当前 {}）",
            BLUEPRINT_V2_SCHEMA_VERSION, bp.schema_version
        ));
    }

    if bp.meta.id.trim().is_empty() {
        errs.push("meta.id 不能为空".into());
    }
    if let Some(dir) = folder_name {
        if bp.meta.id.trim() != dir {
            errs.push(format!(
                "meta.id「{}」与角色包目录名「{}」不一致（R4：ERROR）",
                bp.meta.id.trim(),
                dir
            ));
        }
    }

    if let Some(ref p) = bp.meta.personality {
        if let Err(e) = validate_meta_personality(p) {
            errs.push(e);
        }
    }

    if bp.meta.relations.is_empty() {
        errs.push("meta.relations 至少需要配置一种用户身份".into());
    }

    if bp.meta.name.trim().is_empty() {
        errs.push("meta.name 不能为空".into());
    }

    if bp.slot_registry.is_empty() {
        errs.push("slot_registry 不能为空".into());
    }

    let mut llm_count = 0usize;
    let mut positions_by_type: HashMap<&str, HashSet<i64>> = HashMap::new();

    for (key, slot) in &bp.slot_registry {
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

        if let Err(e) = validate_slot_backend_and_fields(key, slot) {
            errs.push(e);
        }
    }

    if llm_count == 0 {
        errs.push("slot_registry 须至少包含一个 type: llm 的实例".into());
    }

    if let Some(ref k) = bp.meta.knowledge {
        if let Err(e) = validate_knowledge_manifest_disk(k) {
            errs.push(e);
        }
    }

    if errs.is_empty() {
        Ok(())
    } else {
        Err(errs)
    }
}

fn validate_meta_personality(value: &Value) -> Result<(), String> {
    match value {
        Value::Array(arr) => {
            let floats: Result<Vec<f32>, _> = arr
                .iter()
                .map(|v| {
                    v.as_f64()
                        .ok_or_else(|| "personality 数组元素须为数字".to_string())
                        .map(|x| x as f32)
                })
                .collect();
            let floats = floats?;
            validate_default_personality_vector(&floats)
                .map_err(|e| e.replace("manifest：", "meta.personality："))
        }
        Value::Object(map) => {
            let mut vec = Vec::with_capacity(7);
            for key in PERSONALITY_OBJECT_KEYS {
                let Some(v) = map.get(*key) else {
                    return Err(format!(
                        "meta.personality 对象缺少键「{key}」（须含七维: {}）",
                        PERSONALITY_OBJECT_KEYS.join(", ")
                    ));
                };
                let Some(n) = v.as_f64() else {
                    return Err(format!("meta.personality.{key} 须为数字"));
                };
                vec.push(n as f32);
            }
            validate_default_personality_vector(&vec)
                .map_err(|e| e.replace("manifest：", "meta.personality："))
        }
        _ => Err("meta.personality 须为七键对象或长度 7 的数组".into()),
    }
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

fn allowed_backends_for_type(slot_type: &str) -> &'static [&'static str] {
    match slot_type {
        "memory" => &["builtin", "builtin_v2", "remote", "directory", "local"],
        "emotion" | "event" | "prompt" => &["builtin", "builtin_v2", "remote", "directory"],
        "llm" => &["ollama", "remote", "directory"],
        "agent" => &["builtin", "remote", "directory"],
        "complex_emotion" => &["builtin", "remote", "directory"],
        _ => &[],
    }
}

fn meta_to_disk_manifest(meta: &BlueprintMeta) -> DiskRoleManifest {
    let default_personality = meta
        .personality
        .as_ref()
        .and_then(personality_to_vector)
        .unwrap_or_default();

    DiskRoleManifest {
        id: meta.id.clone(),
        name: meta.name.clone(),
        version: meta.version.clone(),
        author: meta.author.clone(),
        description: meta.description.clone(),
        ollama_model: meta.ollama_model.clone(),
        default_personality,
        evolution: meta.evolution.clone(),
        scenes: meta.scenes.clone(),
        user_relations: meta.relations.clone(),
        default_relation: meta.default_relation.clone(),
        memory_config: meta.memory_config.clone(),
        identity_binding: meta.identity_binding,
        life_trajectory: None,
        life_schedule: None,
        dev_only: meta.dev_only,
        knowledge: meta.knowledge.clone(),
        min_runtime_version: meta.min_runtime_version.clone(),
    }
}

fn personality_to_vector(value: &Value) -> Option<Vec<f32>> {
    match value {
        Value::Array(arr) => {
            let mut out = Vec::new();
            for v in arr {
                out.push(v.as_f64()? as f32);
            }
            Some(out)
        }
        Value::Object(map) => {
            let mut out = Vec::with_capacity(7);
            for key in PERSONALITY_OBJECT_KEYS {
                out.push(map.get(*key)?.as_f64()? as f32);
            }
            Some(out)
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn minimal_v2_json() -> String {
        r#"{
          "schema_version": 2,
          "meta": {
            "id": "demo.pack",
            "name": "Demo",
            "version": "0.1.0",
            "author": "t",
            "description": "d",
            "personality": [0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5],
            "relations": {
              "friend": { "initial_favorability": 50.0, "favor_multiplier": 1.0 }
            },
            "default_relation": "friend",
            "scenes": ["default"]
          },
          "slot_registry": {
            "memory": { "type": "memory", "label": "Memory", "backend": "builtin", "position": 0 },
            "emotion": { "type": "emotion", "label": "Emotion", "backend": "builtin", "position": 0 },
            "event": { "type": "event", "label": "Event", "backend": "builtin", "position": 0 },
            "prompt": { "type": "prompt", "label": "Prompt", "backend": "builtin", "position": 0 },
            "llm": { "type": "llm", "label": "LLM", "backend": "ollama", "position": 0 },
            "agent": { "type": "agent", "label": "Agent", "backend": "builtin", "position": 0 },
            "complex_emotion": { "type": "complex_emotion", "label": "Complex", "backend": "builtin", "position": 1 }
          }
        }"#
        .to_string()
    }

    #[test]
    fn valid_minimal_v2_passes() {
        validate_blueprint_v2_json(&minimal_v2_json()).unwrap();
    }

    #[test]
    fn rejects_module_relations() {
        let mut v: Value = serde_json::from_str(&minimal_v2_json()).unwrap();
        v.as_object_mut()
            .unwrap()
            .insert("module_relations".into(), serde_json::json!({}));
        let errs = validate_blueprint_v2_json(&v.to_string()).unwrap_err();
        assert!(errs.iter().any(|e| e.contains("module_relations")));
    }

    #[test]
    fn rejects_schema_version_not_2() {
        let raw = minimal_v2_json().replace("\"schema_version\": 2", "\"schema_version\": 1");
        let errs = validate_blueprint_v2_json(&raw).unwrap_err();
        assert!(errs.iter().any(|e| e.contains("schema_version")));
    }

    #[test]
    fn rejects_no_llm() {
        let raw = r#"{
          "schema_version": 2,
          "meta": {
            "id": "x", "name": "X", "version": "0.1.0", "author": "a", "description": "d",
            "relations": { "f": { "initial_favorability": 50.0, "favor_multiplier": 1.0 } },
            "default_relation": "f"
          },
          "slot_registry": {
            "memory": { "type": "memory", "label": "M", "backend": "builtin", "position": 0 }
          }
        }"#;
        let errs = validate_blueprint_v2_json(raw).unwrap_err();
        assert!(errs.iter().any(|e| e.contains("llm")));
    }

    #[test]
    fn rejects_duplicate_position_same_type() {
        let raw = r#"{
          "schema_version": 2,
          "meta": {
            "id": "x", "name": "X", "version": "0.1.0", "author": "a", "description": "d",
            "relations": { "f": { "initial_favorability": 50.0, "favor_multiplier": 1.0 } },
            "default_relation": "f"
          },
          "slot_registry": {
            "llm_a": { "type": "llm", "label": "A", "backend": "ollama", "position": 0 },
            "llm_b": { "type": "llm", "label": "B", "backend": "ollama", "position": 0 }
          }
        }"#;
        let errs = validate_blueprint_v2_json(raw).unwrap_err();
        assert!(errs.iter().any(|e| e.contains("position")));
    }

    #[test]
    fn rejects_invalid_relation_favorability_via_disk_manifest() {
        let raw = r#"{
          "schema_version": 2,
          "meta": {
            "id": "x", "name": "X", "version": "0.1.0", "author": "a", "description": "d",
            "relations": { "f": { "initial_favorability": 200.0, "favor_multiplier": 1.0 } },
            "default_relation": "f"
          },
          "slot_registry": {
            "llm": { "type": "llm", "label": "L", "backend": "ollama", "position": 0 }
          }
        }"#;
        let errs = validate_blueprint_v2_json(raw).unwrap_err();
        assert!(errs.iter().any(|e| e.contains("favorability")));
    }

    #[test]
    fn slot_registry_last_wins_maps_to_plugin_backends() {
        let mut reg = BTreeMap::new();
        reg.insert(
            "llm_a".into(),
            SlotRegistryEntry {
                slot_type: "llm".into(),
                label: "A".into(),
                backend: "ollama".into(),
                position: 0,
                plugin: None,
                plugins: None,
                model: Some("model-a".into()),
                url: None,
                local_memory_provider_id: None,
            },
        );
        reg.insert(
            "llm_b".into(),
            SlotRegistryEntry {
                slot_type: "llm".into(),
                label: "B".into(),
                backend: "remote".into(),
                position: 1,
                plugin: None,
                plugins: None,
                model: None,
                url: None,
                local_memory_provider_id: None,
            },
        );
        let pb = slot_registry_to_plugin_backends(&reg);
        assert_eq!(pb.llm, LlmBackend::Remote);
    }

    #[test]
    fn blueprint_v2_directory_minimal_pack() {
        let dir = tempfile::tempdir().unwrap();
        let role = dir.path().join("demo.pack");
        fs::create_dir_all(role.join("scenes").join("default")).unwrap();
        let bp = minimal_v2_json().replace("demo.pack", "demo.pack");
        fs::write(role.join(PIPELINE_BLUEPRINT_FILENAME), bp).unwrap();
        validate_role_pack_blueprint_v2_directory(&role, "999.0.0").unwrap();
    }

    #[test]
    fn blueprint_v2_directory_rejects_legacy_manifest() {
        let dir = tempfile::tempdir().unwrap();
        let role = dir.path().join("demo.pack");
        fs::create_dir_all(&role).unwrap();
        fs::write(role.join(PIPELINE_BLUEPRINT_FILENAME), minimal_v2_json()).unwrap();
        fs::write(role.join("manifest.json"), "{}").unwrap();
        let errs = validate_role_pack_blueprint_v2_directory(&role, "999.0.0").unwrap_err();
        assert!(errs.iter().any(|e| e.contains("manifest.json")));
    }

    #[test]
    fn rejects_directory_without_plugin() {
        let raw = r#"{
          "schema_version": 2,
          "meta": {
            "id": "x", "name": "X", "version": "0.1.0", "author": "a", "description": "d",
            "relations": { "f": { "initial_favorability": 50.0, "favor_multiplier": 1.0 } },
            "default_relation": "f"
          },
          "slot_registry": {
            "llm": { "type": "llm", "label": "L", "backend": "directory", "position": 0 }
          }
        }"#;
        let errs = validate_blueprint_v2_json(raw).unwrap_err();
        assert!(errs.iter().any(|e| e.contains("directory")));
    }

    #[test]
    fn write_role_pack_blueprint_slot_registry_persists_and_validates() {
        let dir = tempfile::tempdir().unwrap();
        let role = dir.path().join("demo.pack");
        fs::create_dir_all(role.join("scenes").join("default")).unwrap();
        fs::write(role.join(PIPELINE_BLUEPRINT_FILENAME), minimal_v2_json()).unwrap();
        let mut reg = BTreeMap::new();
        reg.insert(
            "llm".into(),
            SlotRegistryEntry {
                slot_type: "llm".into(),
                label: "L".into(),
                backend: "remote".into(),
                position: 0,
                plugin: None,
                plugins: None,
                model: None,
                url: None,
                local_memory_provider_id: None,
            },
        );
        write_role_pack_blueprint_slot_registry(&role, &reg, "999.0.0").unwrap();
        let loaded = load_blueprint_v2_for_role_dir(&role, "999.0.0").unwrap();
        assert_eq!(loaded.slot_registry.get("llm").unwrap().backend, "remote");
    }
}
