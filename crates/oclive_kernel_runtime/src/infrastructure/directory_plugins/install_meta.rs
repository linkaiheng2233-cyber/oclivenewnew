//! `.oclive_install.json`（与桌面 `plugin_installer` 路径约定一致）。

use crate::error::{AppError, Result};
use crate::models::dto::PluginInstallMetaDto;
use std::path::Path;

pub fn read_plugin_install_meta(root: &Path) -> Option<PluginInstallMetaDto> {
    let p = root.join(".oclive_install.json");
    let raw = std::fs::read_to_string(p).ok()?;
    serde_json::from_str(&raw).ok()
}

pub fn write_plugin_install_meta(root: &Path, meta: &PluginInstallMetaDto) -> Result<()> {
    let p = root.join(".oclive_install.json");
    let raw = serde_json::to_string_pretty(meta).map_err(AppError::from)?;
    std::fs::write(p, raw).map_err(AppError::IoError)?;
    Ok(())
}
