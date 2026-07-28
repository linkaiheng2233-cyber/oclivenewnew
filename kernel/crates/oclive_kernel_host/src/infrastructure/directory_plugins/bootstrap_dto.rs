//! Directory plugin bootstrap DTO assembly (shared by Tauri + bridge).

use crate::infrastructure::directory_plugins::{
    normalize_ui_slot_appearance_id, HostPluginsFile, OclivePluginManifest, UiSlotDecl,
};
use crate::infrastructure::plugin_protocol::plugin_asset_url;
use crate::infrastructure::plugin_state::{PluginStateFile, RolePluginState};
use crate::state::AppState;
use serde::Serialize;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginUiSlotDto {
    pub plugin_id: String,
    pub slot: String,
    pub appearance_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    pub entry: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vue_component: Option<String>,
    pub bridge_events: Vec<String>,
    pub url: String,
}

/// Embedded UI slot names accepted in non-shell plugin manifests.
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

fn pick_ui_slot_decl<'a>(
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

fn plugin_ui_slot_dto_from_decl(pid: &str, decl: &UiSlotDecl) -> Option<PluginUiSlotDto> {
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
    let url = plugin_asset_url(pid, &entry_norm);
    let bridge_events = decl
        .bridge
        .as_ref()
        .map(|bridge| {
            bridge
                .events
                .iter()
                .map(|event| event.trim())
                .filter(|event| !event.is_empty())
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default();
    Some(PluginUiSlotDto {
        plugin_id: pid.to_string(),
        slot: decl.slot.clone(),
        appearance_id: normalize_ui_slot_appearance_id(&decl.appearance_id),
        label: decl.label.clone(),
        entry: entry_norm,
        vue_component,
        bridge_events,
        url,
    })
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DirectoryPluginBootstrapDto {
    pub shell_url: Option<String>,
    pub shell_plugin_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shell_vue_entry: Option<String>,
    pub force_iframe_mode: bool,
    pub plugin_ids: Vec<String>,
    pub developer_mode: bool,
    pub subscribed_host_events: Vec<String>,
    pub ui_slots: Vec<PluginUiSlotDto>,
}

fn merge_manifest_bridge_events(manifest: &OclivePluginManifest, set: &mut HashSet<String>) {
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

/// Collect subscribed host events for enabled plugins (role plugin state).
#[must_use]
pub fn collect_subscribed_host_events(state: &AppState, pst: &PluginStateFile) -> Vec<String> {
    let mut set = HashSet::new();
    let roots = state.directory_plugins.plugin_roots.read();
    for (pid, entry) in roots.iter() {
        if pst.is_plugin_disabled(pid) {
            continue;
        }
        let Ok(manifest) = state
            .directory_plugins
            .load_manifest_cached(pid, &entry.root)
        else {
            continue;
        };
        merge_manifest_bridge_events(&manifest, &mut set);
    }
    subscribed_events_sorted_vec(set)
}

fn order_plugin_slots(mut slots: Vec<PluginUiSlotDto>, order: &[String]) -> Vec<PluginUiSlotDto> {
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

fn shell_plugin_id_resolved(
    host: &HostPluginsFile,
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

/// Shared by `get_directory_plugin_bootstrap` and `plugin_bridge_invoke`.
#[must_use]
pub fn directory_plugin_bootstrap_dto(
    state: &AppState,
    role_id: Option<String>,
) -> DirectoryPluginBootstrapDto {
    let rt = &state.directory_plugins;
    rt.ensure_plugin_roots_scanned();
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
    let shell_plugin_id_raw = shell_plugin_id_resolved(host, Some(&role_state));
    let shell_plugin_id = shell_plugin_id_raw.filter(|id| !pst.is_plugin_disabled(id));
    let shell_manifest = shell_plugin_id.as_ref().and_then(|pid| {
        let roots = rt.plugin_roots.read();
        let entry = roots.get(pid)?;
        rt.load_manifest_cached(pid, &entry.root).ok()
    });
    let shell_url = shell_manifest.as_ref().and_then(|manifest| {
        let pid = shell_plugin_id.as_ref()?;
        let sh = manifest.shell.as_ref()?;
        rt.shell_url_for(pid, sh.entry.as_str())
    });
    let shell_vue_entry = shell_manifest.as_ref().and_then(|manifest| {
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
    let roots = rt.plugin_roots.read();
    for (pid, entry) in roots.iter() {
        if pst.is_plugin_disabled(pid) {
            continue;
        }
        let Ok(manifest) = rt.load_manifest_cached(pid, &entry.root) else {
            continue;
        };
        merge_manifest_bridge_events(&manifest, &mut subscribed_set);
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
            let Some(picked) = pick_ui_slot_decl(&decls, sel) else {
                continue;
            };
            let Some(dto) = plugin_ui_slot_dto_from_decl(pid, picked) else {
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
        ui_slots,
    }
}

#[cfg(test)]
mod tests {
    use super::plugin_ui_slot_dto_from_decl;
    use crate::infrastructure::directory_plugins::UiSlotDecl;
    use crate::infrastructure::plugin_protocol::plugin_asset_url;

    #[test]
    fn slot_dto_carries_its_bridge_event_allowlist() {
        let decl: UiSlotDecl = serde_json::from_value(serde_json::json!({
            "slot": "sidebar",
            "entry": "slots/sidebar.html",
            "bridge": { "events": [" role:switched ", "message:sent", ""] }
        }))
        .unwrap();

        let dto = plugin_ui_slot_dto_from_decl("plugin.a", &decl).unwrap();
        assert_eq!(dto.url, plugin_asset_url("plugin.a", "slots/sidebar.html"));
        assert_eq!(dto.bridge_events, ["role:switched", "message:sent"]);
        let json = serde_json::to_value(dto).unwrap();
        assert_eq!(
            json["bridgeEvents"],
            serde_json::json!(["role:switched", "message:sent"])
        );
    }
}
