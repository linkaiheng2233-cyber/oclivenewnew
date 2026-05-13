//! 插件管理面板用的 catalog 条目构建（无全局缓存；短时 TTL 由宿主维护）。

use super::bootstrap::EMBEDDED_UI_SLOT_NAMES;
use super::dependency::dependency_report;
use super::install_meta::read_plugin_install_meta;
use super::manifest::{normalize_ui_slot_appearance_id, OclivePluginManifest};
use super::parse_manifest_version;
use crate::models::dto::{DirectoryPluginCatalogEntry, UiSlotVariantDto};
use semver::Version;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

pub fn build_directory_plugin_catalog(
    plugin_roots: &HashMap<String, PathBuf>,
) -> Vec<DirectoryPluginCatalogEntry> {
    let mut version_by_id: HashMap<String, Version> = HashMap::new();
    let mut manifests: Vec<(String, PathBuf, OclivePluginManifest)> = Vec::new();
    for (pid, root) in plugin_roots.iter() {
        if let Ok(m) = OclivePluginManifest::load_from_dir(root) {
            if let Some(v) = parse_manifest_version(&m.version) {
                version_by_id.insert(pid.clone(), v);
            }
            manifests.push((pid.clone(), root.clone(), m));
        }
    }
    let mut out: Vec<DirectoryPluginCatalogEntry> = manifests
        .into_iter()
        .map(|(pid, root, manifest)| {
            let install_meta = read_plugin_install_meta(&root);
            let is_shell = manifest.shell.is_some();
            let has_ui_settings = manifest.ui_template.is_some()
                || manifest
                    .ui_schema
                    .as_ref()
                    .map(|s| !s.fields.is_empty())
                    .unwrap_or(false);
            let has_rpc_process = manifest.process.is_some();
            let declares_rpc_methods = !manifest.rpc_methods.is_empty();
            let mut ui_slot_names: Vec<String> = Vec::new();
            let mut seen_slot: HashSet<String> = HashSet::new();
            let mut ui_slot_variants: Vec<UiSlotVariantDto> = Vec::new();
            for u in &manifest.ui_slots {
                if !EMBEDDED_UI_SLOT_NAMES.contains(&u.slot.as_str()) {
                    continue;
                }
                ui_slot_variants.push(UiSlotVariantDto {
                    slot: u.slot.clone(),
                    appearance_id: normalize_ui_slot_appearance_id(&u.appearance_id),
                    label: u.label.clone(),
                });
                if seen_slot.insert(u.slot.clone()) {
                    ui_slot_names.push(u.slot.clone());
                }
            }
            let (dependency_status, dependency_issues) =
                dependency_report(&manifest, &version_by_id);
            DirectoryPluginCatalogEntry {
                id: pid.clone(),
                version: manifest.version.clone(),
                plugin_type: manifest.plugin_type.clone(),
                has_ui_settings,
                has_rpc_process,
                declares_rpc_methods,
                is_shell,
                ui_slot_names,
                ui_slot_variants,
                provides: manifest.provides.clone(),
                dependency_status,
                dependency_issues,
                install_meta,
            }
        })
        .collect();
    out.sort_by(|a, b| a.id.cmp(&b.id));
    out
}
