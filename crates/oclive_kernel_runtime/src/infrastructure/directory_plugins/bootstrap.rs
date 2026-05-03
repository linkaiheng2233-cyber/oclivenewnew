//! Bootstrap JSON、`subscribed_host_events` 与插槽排序纯逻辑。

use super::manifest::{normalize_ui_slot_appearance_id, OclivePluginManifest, UiSlotDecl};
use super::DirectoryPluginRuntime;
use crate::infrastructure::plugin_state::{PluginStateFile, RolePluginState};
use crate::models::dto::{DirectoryPluginBootstrapDto, PluginUiSlotDto};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

/// 桌面发行版默认插件静态资源 URL 前缀（不含末尾 `/`）；调用方可传入同等语义字符串。
pub const DEFAULT_DIRECTORY_PLUGIN_ASSET_BASE_URL: &str = "https://ocliveplugin.localhost";

/// 非整壳插件可声明的嵌入 UI 插槽（与前端消费一致）。
pub const EMBEDDED_UI_SLOT_NAMES: &[&str] = &[
    "chat_toolbar",
    "settings.panel",
    "role.detail",
    "sidebar",
    "chat.header",
    "settings.plugins",
    "settings.advanced",
    "overlay.floating",
    "launcher.palette",
    "debug.dock",
];

pub fn pick_ui_slot_decl<'a>(
    decls: &[&'a UiSlotDecl],
    selected: Option<&str>,
) -> Option<&'a UiSlotDecl> {
    if decls.is_empty() {
        return None;
    }
    let want = selected
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(normalize_ui_slot_appearance_id);
    if let Some(ref w) = want {
        for d in decls {
            if normalize_ui_slot_appearance_id(&d.appearance_id) == *w {
                return Some(*d);
            }
        }
    }
    for d in decls {
        if normalize_ui_slot_appearance_id(&d.appearance_id).is_empty() {
            return Some(*d);
        }
    }
    Some(decls[0])
}

fn plugin_ui_slot_dto_from_decl(
    pid: &str,
    decl: &UiSlotDecl,
    plugin_asset_base: &str,
) -> Option<PluginUiSlotDto> {
    let entry = decl.entry.trim().trim_start_matches(['/', '\\']);
    if entry.is_empty() {
        return None;
    }
    let entry_norm = entry.replace('\\', "/");
    let vue_component = decl
        .vue_component
        .as_ref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.replace('\\', "/"));
    let base = plugin_asset_base.trim_end_matches('/');
    let url = format!("{}/{}/{}", base, pid, entry_norm);
    Some(PluginUiSlotDto {
        plugin_id: pid.to_string(),
        slot: decl.slot.clone(),
        appearance_id: normalize_ui_slot_appearance_id(&decl.appearance_id),
        label: decl.label.clone(),
        entry: entry_norm,
        vue_component,
        url,
    })
}

/// 将 manifest 内 `shell.bridge` / `ui_slots[].bridge` 的 `events` 并入集合（与 `is_host_event_subscribed` 语义一致）。
pub fn merge_manifest_bridge_events(manifest: &OclivePluginManifest, set: &mut HashSet<String>) {
    if let Some(sh) = &manifest.shell {
        if let Some(b) = &sh.bridge {
            for e in &b.events {
                let t = e.trim();
                if !t.is_empty() {
                    set.insert(t.to_string());
                }
            }
        }
    }
    for us in &manifest.ui_slots {
        if let Some(b) = &us.bridge {
            for e in &b.events {
                let t = e.trim();
                if !t.is_empty() {
                    set.insert(t.to_string());
                }
            }
        }
    }
}

fn subscribed_events_sorted_vec(set: HashSet<String>) -> Vec<String> {
    let mut v: Vec<String> = set.into_iter().collect();
    v.sort_unstable();
    v
}

/// 收集「未全局禁用」的插件在 `shell.bridge` / `ui_slots[].bridge` 中声明的 `events`。
pub fn collect_subscribed_host_events(
    plugin_roots: &HashMap<String, PathBuf>,
    pst: &PluginStateFile,
) -> Vec<String> {
    let mut set = HashSet::new();
    for (pid, root) in plugin_roots.iter() {
        if pst.is_plugin_disabled(pid) {
            continue;
        }
        let Ok(manifest) = OclivePluginManifest::load_from_dir(root) else {
            continue;
        };
        merge_manifest_bridge_events(&manifest, &mut set);
    }
    subscribed_events_sorted_vec(set)
}

pub fn is_host_event_subscribed(
    plugin_roots: &HashMap<String, PathBuf>,
    pst: &PluginStateFile,
    event: &str,
) -> bool {
    let ev = event.trim();
    if ev.is_empty() {
        return false;
    }
    collect_subscribed_host_events(plugin_roots, pst)
        .iter()
        .any(|s| s == ev)
}

/// 对**同一插槽**的条目按 `plugin_state.slot_order[slot]` 排序。
pub fn order_plugin_slots(mut slots: Vec<PluginUiSlotDto>, order: &[String]) -> Vec<PluginUiSlotDto> {
    let mut by_id: HashMap<String, PluginUiSlotDto> =
        slots.drain(..).map(|s| (s.plugin_id.clone(), s)).collect();
    let mut out = Vec::new();
    for id in order {
        if let Some(s) = by_id.remove(id.as_str()) {
            out.push(s);
        }
    }
    let mut rest: Vec<_> = by_id.into_values().collect();
    rest.sort_by(|a, b| a.plugin_id.cmp(&b.plugin_id));
    out.extend(rest);
    out
}

pub fn shell_plugin_id_resolved(
    host: &super::HostPluginsFile,
    role: Option<&RolePluginState>,
) -> Option<String> {
    if let Ok(v) = std::env::var("OCLIVE_SHELL_PLUGIN_ID") {
        let t = v.trim().to_string();
        if !t.is_empty() {
            return Some(t);
        }
    }
    if let Some(rs) = role {
        let t = rs.shell_plugin_id.trim();
        if !t.is_empty() {
            return Some(t.to_string());
        }
    }
    host.shell_plugin_id
        .as_ref()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

/// `role_id`：当前角色；省略时尝试 `oclive_last_role_id.txt`，再回退旧版全局插件状态。
pub fn directory_plugin_bootstrap_dto(
    rt: &DirectoryPluginRuntime,
    role_id: Option<String>,
    plugin_asset_base: &str,
) -> DirectoryPluginBootstrapDto {
    let host = rt.host();
    let rid = role_id
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .or_else(|| rt.read_last_role_id_from_disk());
    let role_state = if let Some(ref id) = rid {
        let mut s = rt.role_plugin_state_for(id);
        rt.sanitize_role_shell(&mut s);
        s
    } else {
        let mut s = RolePluginState {
            shell_plugin_id: String::new(),
            slots: rt.effective_slots(),
        };
        rt.sanitize_role_shell(&mut s);
        s
    };
    let pst = &role_state.slots;
    let mut plugin_ids_sorted: Vec<String> = rt.plugin_roots.read().keys().cloned().collect();
    plugin_ids_sorted.retain(|id| !pst.is_plugin_disabled(id));
    plugin_ids_sorted.sort_unstable();
    let roots = rt.plugin_roots.read();
    let manifests: HashMap<String, OclivePluginManifest> = roots
        .iter()
        .filter_map(|(pid, root)| {
            OclivePluginManifest::load_from_dir(root)
                .ok()
                .map(|m| (pid.clone(), m))
        })
        .collect();
    let shell_plugin_id_raw = shell_plugin_id_resolved(&host, Some(&role_state));
    let shell_plugin_id = shell_plugin_id_raw.filter(|id| !pst.is_plugin_disabled(id));
    let shell_url = shell_plugin_id.as_ref().and_then(|pid| {
        let manifest = manifests.get(pid)?;
        let entry = manifest.shell.as_ref()?.entry.as_str();
        rt.shell_url_for(pid, entry)
    });
    let shell_vue_entry = shell_plugin_id.as_ref().and_then(|pid| {
        let manifest = manifests.get(pid)?;
        let sh = manifest.shell.as_ref()?;
        let ve = sh.vue_entry.as_ref()?.trim();
        if ve.is_empty() {
            None
        } else {
            Some(ve.replace('\\', "/"))
        }
    });

    let mut ui_slots = Vec::new();
    let mut subscribed_set = HashSet::new();
    for (pid, manifest) in manifests.iter() {
        if pst.is_plugin_disabled(pid) {
            continue;
        }
        merge_manifest_bridge_events(manifest, &mut subscribed_set);
        if manifest.shell.is_some() {
            continue;
        }
        let appearance_for = pst.slot_appearance.get(pid);
        let mut by_slot: HashMap<String, Vec<&UiSlotDecl>> = HashMap::new();
        for decl in &manifest.ui_slots {
            if !EMBEDDED_UI_SLOT_NAMES.contains(&decl.slot.as_str()) {
                continue;
            }
            by_slot.entry(decl.slot.clone()).or_default().push(decl);
        }
        for (slot_name, decls) in by_slot {
            if pst.is_slot_contribution_disabled(&slot_name, pid) {
                continue;
            }
            let sel = appearance_for
                .and_then(|m| m.get(&slot_name))
                .map(|s| s.as_str());
            let decl_refs: Vec<&UiSlotDecl> = decls.to_vec();
            let Some(picked) = pick_ui_slot_decl(&decl_refs, sel) else {
                continue;
            };
            let Some(dto) = plugin_ui_slot_dto_from_decl(pid, picked, plugin_asset_base) else {
                continue;
            };
            ui_slots.push(dto);
        }
    }
    let mut ui_slots_ordered = Vec::new();
    for slot_name in EMBEDDED_UI_SLOT_NAMES {
        let mut bucket: Vec<_> = ui_slots
            .iter()
            .filter(|s| s.slot == *slot_name)
            .cloned()
            .collect();
        let order = pst
            .slot_order
            .get(*slot_name)
            .map(|v| v.as_slice())
            .unwrap_or(&[]);
        bucket = order_plugin_slots(bucket, order);
        ui_slots_ordered.extend(bucket);
    }
    let ui_slots = ui_slots_ordered;

    let subscribed_host_events = subscribed_events_sorted_vec(subscribed_set);

    DirectoryPluginBootstrapDto {
        shell_url,
        shell_plugin_id,
        shell_vue_entry,
        force_iframe_mode: pst.force_iframe_mode,
        plugin_ids: plugin_ids_sorted,
        developer_mode: host.developer_effective(),
        subscribed_host_events,
        supported_ui_slots: EMBEDDED_UI_SLOT_NAMES
            .iter()
            .map(|s| (*s).to_string())
            .collect(),
        ui_slots,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::directory_plugins::HostPluginsFile;

    #[test]
    fn pick_ui_slot_prefers_matching_appearance() {
        let d1 = UiSlotDecl {
            slot: "chat_toolbar".into(),
            appearance_id: "".into(),
            label: None,
            entry: "a.html".into(),
            vue_component: None,
            bridge: None,
        };
        let d2 = UiSlotDecl {
            slot: "chat_toolbar".into(),
            appearance_id: "dark".into(),
            label: None,
            entry: "b.html".into(),
            vue_component: None,
            bridge: None,
        };
        let refs: Vec<&UiSlotDecl> = vec![&d1, &d2];
        assert_eq!(
            pick_ui_slot_decl(&refs, Some("dark"))
                .map(|d| d.entry.as_str())
                .unwrap(),
            "b.html"
        );
        assert_eq!(
            pick_ui_slot_decl(&refs, None)
                .map(|d| d.entry.as_str())
                .unwrap(),
            "a.html"
        );
    }

    #[test]
    fn shell_plugin_id_prefers_role_over_host_file() {
        let host = HostPluginsFile {
            shell_plugin_id: Some("from_file".into()),
            ..Default::default()
        };
        let role = RolePluginState {
            shell_plugin_id: "from_role".into(),
            ..Default::default()
        };
        assert_eq!(
            shell_plugin_id_resolved(&host, Some(&role)).as_deref(),
            Some("from_role")
        );
        assert_eq!(
            shell_plugin_id_resolved(&host, None).as_deref(),
            Some("from_file")
        );
    }
}
