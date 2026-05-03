//! 目录插件私有配置：`{app_data}/plugin-data/{plugin_id}/config.json`（与桌面 `plugin_data` 路径一致）。

use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

fn config_path(app_data_dir: &Path, plugin_id: &str) -> PathBuf {
    app_data_dir
        .join("plugin-data")
        .join(plugin_id.trim())
        .join("config.json")
}

pub fn write_plugin_config_json(
    app_data_dir: &Path,
    plugin_id: &str,
    value: &Value,
) -> Result<(), String> {
    let p = config_path(app_data_dir, plugin_id);
    if let Some(parent) = p.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let raw = serde_json::to_string_pretty(value).map_err(|e| e.to_string())?;
    fs::write(&p, raw).map_err(|e| e.to_string())?;
    Ok(())
}
