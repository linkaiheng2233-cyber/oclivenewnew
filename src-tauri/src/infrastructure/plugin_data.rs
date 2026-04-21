//! 插件私有配置：`{app_data}/plugin-data/{plugin_id}/config.json`。

use crate::infrastructure::directory_plugins::{OclivePluginManifest, UiSchemaField};
use crate::state::AppState;
use serde_json::{json, Map, Value};
use std::fs;
use std::path::PathBuf;

fn plugin_data_dir(state: &AppState, plugin_id: &str) -> PathBuf {
    state
        .directory_plugins
        .app_data_dir()
        .join("plugin-data")
        .join(plugin_id.trim())
}

fn config_path(state: &AppState, plugin_id: &str) -> PathBuf {
    plugin_data_dir(state, plugin_id).join("config.json")
}

fn default_object_from_schema(fields: &[UiSchemaField]) -> Value {
    let mut m = Map::new();
    for f in fields {
        let k = f.key.trim();
        if k.is_empty() {
            continue;
        }
        if let Some(ref d) = f.default {
            m.insert(k.to_string(), d.clone());
        } else {
            let v = match f.field_type.as_str() {
                "number" => json!(0),
                "bool" | "boolean" => json!(false),
                _ => json!(""),
            };
            m.insert(k.to_string(), v);
        }
    }
    Value::Object(m)
}

/// 安装后若尚无配置文件，则按 `ui_schema.fields[].default` 写入默认 JSON。
pub fn ensure_default_config_for_manifest(state: &AppState, manifest: &OclivePluginManifest) {
    let pid = manifest.id.trim();
    if pid.is_empty() {
        return;
    }
    let Some(ref schema) = manifest.ui_schema else {
        return;
    };
    if schema.fields.is_empty() {
        return;
    }
    let p = config_path(state, pid);
    if p.exists() {
        return;
    }
    if let Some(parent) = p.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let body = default_object_from_schema(&schema.fields);
    if let Ok(raw) = serde_json::to_string_pretty(&body) {
        let _ = fs::write(&p, raw);
    }
}

pub fn read_config_json(state: &AppState, plugin_id: &str) -> Result<Value, String> {
    let p = config_path(state, plugin_id);
    if !p.is_file() {
        return Ok(json!({}));
    }
    let raw = fs::read_to_string(&p).map_err(|e| e.to_string())?;
    serde_json::from_str(&raw).map_err(|e| e.to_string())
}

pub fn write_config_json(state: &AppState, plugin_id: &str, value: &Value) -> Result<(), String> {
    let p = config_path(state, plugin_id);
    if let Some(parent) = p.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let raw = serde_json::to_string_pretty(value).map_err(|e| e.to_string())?;
    fs::write(&p, raw).map_err(|e| e.to_string())
}
