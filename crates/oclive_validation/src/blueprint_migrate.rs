//! manifest.json + settings.json → `pipeline.ocblueprint` v2 (P6 migration).

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use serde_json::{json, Value};

use crate::blueprint_v2::{
    SlotRegistryEntry, BLUEPRINT_V2_SCHEMA_VERSION, PIPELINE_BLUEPRINT_FILENAME,
};
use crate::disk_role_settings::DiskRoleSettings;
use crate::manifest::DiskRoleManifest;
use crate::plugin_backends::{DirectoryPluginSlots, PluginBackends};

/// Converts a legacy role pack on disk to a v2 blueprint JSON value (does not write to disk).
///
/// # Errors
///
/// Returns `Err(Vec<String>)` when `manifest.json` is missing or parsing fails.
pub fn build_blueprint_v2_from_legacy_dir(role_dir: &Path) -> Result<Value, Vec<String>> {
    let manifest_path = role_dir.join("manifest.json");
    if !manifest_path.is_file() {
        return Err(vec![format!(
            "缺少 manifest.json：{}",
            manifest_path.display()
        )]);
    }
    let manifest_raw = fs::read_to_string(&manifest_path)
        .map_err(|e| vec![format!("读取 manifest.json 失败: {}", e)])?;
    let mut disk: DiskRoleManifest = serde_json::from_str(&manifest_raw)
        .map_err(|e| vec![format!("manifest.json 结构错误: {}", e)])?;

    let settings_path = role_dir.join("settings.json");
    let settings: Option<DiskRoleSettings> = if settings_path.is_file() {
        let raw = fs::read_to_string(&settings_path)
            .map_err(|e| vec![format!("读取 settings.json 失败: {}", e)])?;
        let s: DiskRoleSettings = serde_json::from_str(&raw)
            .map_err(|e| vec![format!("settings.json 结构错误: {}", e)])?;
        s.apply_to_manifest(&mut disk);
        Some(s)
    } else {
        None
    };

    let pb = settings
        .as_ref()
        .and_then(|s| s.plugin_backends.clone())
        .unwrap_or_default();
    let slot_registry = plugin_backends_to_slot_registry(&pb);

    let personality = if disk.default_personality.len() == 7 {
        Value::Array(disk.default_personality.iter().map(|x| json!(x)).collect())
    } else {
        json!({
            "stubbornness": 0.5,
            "clinginess": 0.5,
            "sensitivity": 0.5,
            "assertiveness": 0.5,
            "forgiveness": 0.5,
            "talkativeness": 0.5,
            "warmth": 0.5
        })
    };

    let mut meta = json!({
        "id": disk.id,
        "name": disk.name,
        "version": disk.version,
        "author": disk.author,
        "description": disk.description,
        "personality": personality,
        "relations": disk.user_relations,
        "default_relation": disk.default_relation,
        "scenes": disk.scenes,
        "evolution": disk.evolution,
        "memory_config": disk.memory_config,
        "identity_binding": disk.identity_binding,
        "dev_only": disk.dev_only,
    });
    if let Some(m) = disk.ollama_model.clone() {
        meta["ollama_model"] = json!(m);
    }
    if let Some(m) = disk.min_runtime_version.clone() {
        meta["min_runtime_version"] = json!(m);
    }
    if let Some(lt) = disk.life_trajectory.clone() {
        meta["life_trajectory"] = serde_json::to_value(lt).unwrap_or(Value::Null);
    }
    if let Some(ls) = disk.life_schedule.clone() {
        meta["life_schedule"] = serde_json::to_value(ls).unwrap_or(Value::Null);
    }
    if let Some(k) = disk.knowledge.clone() {
        meta["knowledge"] = serde_json::to_value(k).unwrap_or(Value::Null);
    }
    if let Some(ref s) = settings {
        if let Some(ref m) = s.interaction_mode {
            meta["interaction_mode"] = json!(m);
        }
        if let Some(ref rp) = s.remote_presence {
            meta["remote_presence"] = serde_json::to_value(rp).unwrap_or(Value::Null);
        }
        if let Some(ref asc) = s.autonomous_scene {
            meta["autonomous_scene"] = serde_json::to_value(asc).unwrap_or(Value::Null);
        }
        if let Some(ref a) = s.reply_quality_anchor {
            meta["reply_quality_anchor"] = json!(a);
        }
    }

    let registry_value = serde_json::to_value(&slot_registry)
        .map_err(|e| vec![format!("slot_registry 序列化失败: {}", e)])?;

    Ok(json!({
        "schema_version": BLUEPRINT_V2_SCHEMA_VERSION,
        "meta": meta,
        "slot_registry": registry_value,
    }))
}

/// Writes `pipeline.ocblueprint`; when `remove_legacy` is true, deletes manifest/settings.
///
/// # Errors
///
/// Returns `Err(Vec<String>)` on build or write failure.
pub fn migrate_role_pack_dir_to_blueprint_v2(
    role_dir: &Path,
    remove_legacy: bool,
) -> Result<(), Vec<String>> {
    let bp = build_blueprint_v2_from_legacy_dir(role_dir)?;
    let out = role_dir.join(PIPELINE_BLUEPRINT_FILENAME);
    let text = serde_json::to_string_pretty(&bp)
        .map_err(|e| vec![format!("序列化 blueprint 失败: {}", e)])?;
    fs::write(&out, text).map_err(|e| vec![format!("写入 {} 失败: {}", out.display(), e)])?;
    if remove_legacy {
        for name in ["manifest.json", "settings.json"] {
            let p = role_dir.join(name);
            if p.is_file() {
                fs::remove_file(&p)
                    .map_err(|e| vec![format!("删除 {} 失败: {}", p.display(), e)])?;
            }
        }
    }
    Ok(())
}

fn plugin_backends_to_slot_registry(pb: &PluginBackends) -> BTreeMap<String, SlotRegistryEntry> {
    let mut reg = BTreeMap::new();
    reg.insert(
        "memory".into(),
        entry_for(
            "memory",
            backend_snake(pb.memory),
            0,
            &pb.directory_plugins,
            pb,
        ),
    );
    reg.insert(
        "emotion".into(),
        entry_for(
            "emotion",
            backend_snake(pb.emotion),
            0,
            &pb.directory_plugins,
            pb,
        ),
    );
    reg.insert(
        "complex_emotion".into(),
        SlotRegistryEntry {
            slot_type: "complex_emotion".into(),
            label: "Complex emotion".into(),
            backend: "builtin".into(),
            position: 1,
            plugin: None,
            plugins: None,
            model: None,
            url: None,
            local_memory_provider_id: None,
            zone: None,
            policy: None,
        },
    );
    reg.insert(
        "event".into(),
        entry_for(
            "event",
            backend_snake(pb.event),
            0,
            &pb.directory_plugins,
            pb,
        ),
    );
    reg.insert(
        "prompt".into(),
        entry_for(
            "prompt",
            backend_snake(pb.prompt),
            0,
            &pb.directory_plugins,
            pb,
        ),
    );
    reg.insert(
        "llm".into(),
        entry_for("llm", backend_snake(pb.llm), 0, &pb.directory_plugins, pb),
    );
    reg.insert(
        "agent".into(),
        entry_for(
            "agent",
            backend_snake(pb.agent),
            0,
            &pb.directory_plugins,
            pb,
        ),
    );
    reg
}

fn entry_for(
    slot_type: &str,
    backend: String,
    position: i64,
    dir: &DirectoryPluginSlots,
    pb: &PluginBackends,
) -> SlotRegistryEntry {
    let plugin = match slot_type {
        "memory" => dir.memory.clone(),
        "emotion" => dir.emotion.clone(),
        "event" => dir.event.clone(),
        "prompt" => dir.prompt.clone(),
        "llm" => dir.llm.clone(),
        "agent" => dir.agent.clone(),
        _ => None,
    };
    SlotRegistryEntry {
        slot_type: slot_type.into(),
        label: slot_type.to_string(),
        backend,
        position,
        plugin,
        plugins: None,
        model: None,
        url: None,
        local_memory_provider_id: if slot_type == "memory" {
            pb.local_memory_provider_id.clone()
        } else {
            None
        },
        zone: None,
        policy: None,
    }
}

fn backend_snake<T: serde::Serialize>(v: T) -> String {
    serde_json::to_value(v)
        .ok()
        .and_then(|j| j.as_str().map(|s| s.to_string()))
        .unwrap_or_else(|| "builtin".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn migrate_minimal_legacy_pack() {
        let dir = tempfile::tempdir().unwrap();
        let role = dir.path().join("demo");
        fs::create_dir_all(&role).unwrap();
        let manifest = serde_json::json!({
            "id": "demo",
            "name": "D",
            "version": "0.1.0",
            "author": "a",
            "description": "d",
            "default_personality": [0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5],
            "relations": { "f": { "initial_favorability": 50.0, "favor_multiplier": 1.0 } },
            "default_relation": "f"
        });
        let mut f = fs::File::create(role.join("manifest.json")).unwrap();
        f.write_all(manifest.to_string().as_bytes()).unwrap();
        migrate_role_pack_dir_to_blueprint_v2(&role, true).unwrap();
        assert!(role.join(PIPELINE_BLUEPRINT_FILENAME).is_file());
        assert!(!role.join("manifest.json").exists());
    }
}
