//! `plugins/<id>/manifest.json`

use super::version::parse_manifest_version;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

/// Host capability whitelist for shell / UI slot pages (`plugin_bridge_invoke`).
#[derive(Debug, Clone, Deserialize, Default)]
pub struct BridgeConfig {
    /// Allowed Tauri command names (match `invoke_handler` registration, e.g. `get_role_info`).
    #[serde(default)]
    pub invoke: Vec<String>,
    /// Allowed `event.listen` event names (optional; may be empty if unimplemented).
    #[serde(default)]
    pub events: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ShellSection {
    /// Relative to plugin root, e.g. `ui/index.html`
    pub entry: String,
    /// Optional: native Vue shell entry (`.vue` relative to plugin root); host picks vs `entry` per bootstrap rules.
    #[serde(default, rename = "vueEntry")]
    pub vue_entry: Option<String>,
    /// When non-empty, host injects `window.OclivePluginBridge` into that HTML.
    #[serde(default)]
    pub bridge: Option<BridgeConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ProcessSection {
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    /// Working directory relative to plugin root; defaults to plugin root
    #[serde(default)]
    pub cwd: Option<String>,
}

/// UI mounted in the main window when not in shell mode (official slot names: host `EMBEDDED_UI_SLOT_NAMES`).
///
/// **Multiple declarations per `slot`**: each must have a unique `appearance_id` (empty string = default variant; at most one per `slot`).
/// `label` is optional display text for the manager UI.
/// Dynamic form fields for plugin settings (contract with `PLUGIN_INDEX.md` / frontend `PluginSettings.vue`).
#[derive(Debug, Clone, Deserialize)]
pub struct UiSchemaField {
    pub key: String,
    pub label: String,
    #[serde(rename = "type")]
    pub field_type: String,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub default: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct UiSchemaSection {
    #[serde(default)]
    pub fields: Vec<UiSchemaField>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UiSlotDecl {
    pub slot: String,
    /// Distinguishes multiple appearances for the same `slot`; empty = default single appearance.
    #[serde(default)]
    pub appearance_id: String,
    /// Display name for manager UI, catalog, etc.
    #[serde(default)]
    pub label: Option<String>,
    pub entry: String,
    /// Optional: `.vue` path relative to plugin root; host renders natively (falls back to `entry` iframe on failure).
    #[serde(default, rename = "vueComponent")]
    pub vue_component: Option<String>,
    #[serde(default)]
    pub bridge: Option<BridgeConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OclivePluginManifest {
    pub schema_version: u32,
    pub id: String,
    pub version: String,
    /// Host extension: deep-integration shell plugins should use **`"ocliveplugin"`** (see `plugin_bridge` sensitive-command gate).
    #[serde(default, rename = "type")]
    pub plugin_type: Option<String>,
    #[serde(default)]
    pub shell: Option<ShellSection>,
    /// When `shell` is set, excluded from slots by convention to avoid duplication.
    #[serde(default)]
    pub ui_slots: Vec<UiSlotDecl>,
    /// Optional: directory backend capabilities this plugin provides (e.g. `memory` / `emotion` / `event` / `prompt` / `llm`). Unset = all available in editor.
    #[serde(default)]
    pub provides: Vec<String>,
    #[serde(default)]
    pub process: Option<ProcessSection>,
    /// stdout ready-line prefix, default `OCLIVE_READY`
    #[serde(default = "default_ready_prefix")]
    pub ready_prefix: String,
    /// Optional: declared JSON-RPC method names (developer debug panel fallback; may merge with runtime `rpc.discover`).
    #[serde(default, rename = "rpcMethods")]
    pub rpc_methods: Vec<String>,
    /// Optional: other directory plugin ids → semver range (e.g. `^2.0.0`, `>=1.0.0`).
    #[serde(default)]
    pub dependencies: Option<HashMap<String, String>>,
    /// Optional: `endpoint-config` / `provider-selector` / `slot-selector` / `switch-toggle`.
    #[serde(default, rename = "uiTemplate")]
    pub ui_template: Option<String>,
    /// Optional: dynamic form schema (`fields` array).
    #[serde(default, rename = "uiSchema")]
    pub ui_schema: Option<UiSchemaSection>,
    /// Optional: high-risk capability declarations (PLUGIN_V1 permissions); omitted = `[]`.
    #[serde(default)]
    pub permissions: Vec<String>,
    /// Optional: plugin description (simple manager list detail).
    #[serde(default)]
    pub description: Option<String>,
    /// Optional: author or organization.
    #[serde(default)]
    pub author: Option<String>,
    /// Optional: auto-write role pack `slot_registry` on install (PLUGIN_V1 · `slot_attachment`).
    #[serde(default, rename = "slot_attachment")]
    pub slot_attachment: Option<serde_json::Value>,
}

/// Normalize relative paths in manifest for comparison with request URI `rel`.
#[must_use]
pub fn normalize_plugin_rel(s: &str) -> String {
    s.replace('\\', "/")
        .trim()
        .trim_start_matches("./")
        .to_string()
}

fn default_ready_prefix() -> String {
    "OCLIVE_READY".to_string()
}

impl OclivePluginManifest {
    /// Whether the given asset `rel` (under plugin root) has bridge config; returns `BridgeConfig` when set.
    #[must_use]
    pub fn bridge_for_asset_rel(&self, rel: &str) -> Option<&BridgeConfig> {
        let n = normalize_plugin_rel(rel);
        if let Some(sh) = &self.shell {
            if normalize_plugin_rel(&sh.entry) == n {
                return sh.bridge.as_ref();
            }
            if let Some(ref vc) = sh.vue_entry {
                let vc = vc.trim();
                if !vc.is_empty() && normalize_plugin_rel(vc) == n {
                    return sh.bridge.as_ref();
                }
            }
        }
        for us in &self.ui_slots {
            if normalize_plugin_rel(&us.entry) == n {
                return us.bridge.as_ref();
            }
            if let Some(ref vc) = us.vue_component {
                if !vc.trim().is_empty() && normalize_plugin_rel(vc) == n {
                    return us.bridge.as_ref();
                }
            }
        }
        None
    }

    /// Whether to inject bridge script: bridge present and invoke or events non-empty.
    #[must_use]
    pub fn should_inject_bridge(&self, rel: &str) -> bool {
        let Some(b) = self.bridge_for_asset_rel(rel) else {
            return false;
        };
        !b.invoke.is_empty() || !b.events.is_empty()
    }
    /// # Errors
    ///
    /// Returns [`Err`] with a human-readable message when the operation fails.
    pub fn load_from_dir(dir: &Path) -> Result<Self, String> {
        let p = dir.join("manifest.json");
        let raw = std::fs::read_to_string(&p).map_err(|e| format!("{}: {}", p.display(), e))?;
        let m: OclivePluginManifest =
            serde_json::from_str(&raw).map_err(|e| format!("{}: {}", p.display(), e))?;
        if m.schema_version != 1 {
            return Err(format!(
                "manifest {}: unsupported schema_version {}",
                p.display(),
                m.schema_version
            ));
        }
        if m.id.trim().is_empty() {
            return Err(format!("manifest {}: id empty", p.display()));
        }
        if m.version.trim().is_empty() {
            return Err(format!("manifest {}: version empty", p.display()));
        }
        if parse_manifest_version(&m.version).is_none() {
            return Err(format!(
                "manifest {}: version must be valid semver (e.g. 1.2.3), got {:?}",
                p.display(),
                m.version
            ));
        }
        validate_ui_slot_appearance_ids(&m)?;
        oclive_validation::validate_permissions_list(&m.permissions)
            .map_err(|e| format!("manifest {}: {}", p.display(), e))?;
        if m.slot_attachment.is_some() {
            oclive_validation::parse_slot_attachments_from_manifest_json(&raw)
                .map_err(|e| format!("manifest {}: {}", p.display(), e))?;
        }
        if let Some(ref sh) = m.shell {
            if sh.entry.trim().is_empty() {
                return Err(format!(
                    "manifest {}: shell.entry required when shell is set",
                    p.display()
                ));
            }
        }
        Ok(m)
    }
}

/// Normalized `appearance_id` for persistence and `plugin_state` comparison (trim).
#[must_use]
pub fn normalize_ui_slot_appearance_id(s: &str) -> String {
    s.trim().to_string()
}

fn normalize_appearance_id(s: &str) -> String {
    normalize_ui_slot_appearance_id(s)
}

/// Within one manifest: each `(slot, appearance_id)` at most once (`appearance_id` compared after trim; empty = default key).
fn validate_ui_slot_appearance_ids(m: &OclivePluginManifest) -> Result<(), String> {
    use std::collections::HashSet;
    let mut seen: HashSet<(String, String)> = HashSet::new();
    for us in &m.ui_slots {
        let slot = us.slot.trim().to_string();
        let aid = normalize_appearance_id(&us.appearance_id);
        let key = (slot, aid);
        if !seen.insert(key.clone()) {
            return Err(format!(
                "manifest: duplicate ui_slots for slot {:?} with appearance_id {:?}",
                key.0, key.1
            ));
        }
    }
    Ok(())
}
