//! `plugins/<id>/manifest.json`

use super::version::parse_manifest_version;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

/// 整壳 / UI 插槽页可调用的宿主能力白名单（`plugin_bridge_invoke`）。
#[derive(Debug, Clone, Deserialize, Default)]
pub struct BridgeConfig {
    /// 允许的 Tauri command 名（与 `invoke_handler` 注册名一致，如 `get_role_info`）。
    #[serde(default)]
    pub invoke: Vec<String>,
    /// 允许的 `event.listen` 事件名（可选；未实现时列表可为空）。
    #[serde(default)]
    pub events: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ShellSection {
    /// 相对插件根，如 `ui/index.html`
    pub entry: String,
    /// 可选：整壳主界面用原生 Vue 渲染（相对插件根的 `.vue`）；与 `entry` 二选一优先时见宿主引导逻辑。
    #[serde(default, rename = "vueEntry")]
    pub vue_entry: Option<String>,
    /// 非空时由宿主向该 HTML 注入 `window.OclivePluginBridge`。
    #[serde(default)]
    pub bridge: Option<BridgeConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ProcessSection {
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    /// 相对插件根的工作目录；缺省为插件根
    #[serde(default)]
    pub cwd: Option<String>,
}

/// 非整壳模式下在主界面挂载的 UI（官方语义插槽名见宿主 `EMBEDDED_UI_SLOT_NAMES`）。
///
/// **同一 `slot` 多条声明**：每条须有唯一 `appearance_id`（空字符串表示「默认」变体，同一 `slot` 至多一条）。
/// `label` 为管理界面展示用，可选。
#[derive(Debug, Clone, Deserialize)]
pub struct UiSlotDecl {
    pub slot: String,
    /// 同一 `slot` 多外观时用于区分；缺省或空字符串表示默认单外观。
    #[serde(default)]
    pub appearance_id: String,
    /// 管理界面、目录等展示用名称。
    #[serde(default)]
    pub label: Option<String>,
    pub entry: String,
    /// 可选：相对插件根的 `.vue` 路径，由主界面原生渲染（失败则回退 `entry` iframe）。
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
    /// 宿主扩展用：整壳深度集成插件建议设为 **`"ocliveplugin"`**（见 `plugin_bridge` 敏感命令门禁）。
    #[serde(default, rename = "type")]
    pub plugin_type: Option<String>,
    #[serde(default)]
    pub shell: Option<ShellSection>,
    /// 若存在 `shell`，按约定不参与插槽，避免与整壳重复。
    #[serde(default)]
    pub ui_slots: Vec<UiSlotDecl>,
    /// 可选：声明本插件提供的目录后端能力（如 `memory` / `emotion` / `event` / `prompt` / `llm`）。未声明时编写器视为全部可用。
    #[serde(default)]
    pub provides: Vec<String>,
    #[serde(default)]
    pub process: Option<ProcessSection>,
    /// stdout 就绪行前缀，默认 `OCLIVE_READY`
    #[serde(default = "default_ready_prefix")]
    pub ready_prefix: String,
    /// 可选：依赖的其他目录插件 id → semver 范围（如 `^2.0.0`、`>=1.0.0`）。
    #[serde(default)]
    pub dependencies: Option<HashMap<String, String>>,
}

/// 规范化 manifest 内相对路径，与请求 URI 中 `rel` 比较。
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
    /// 当前资源相对路径（插件根下）是否配置了 bridge，返回 `BridgeConfig`。
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

    /// 是否应注入桥接脚本：有 bridge 且 invoke 或 events 非空。
    pub fn should_inject_bridge(&self, rel: &str) -> bool {
        let Some(b) = self.bridge_for_asset_rel(rel) else {
            return false;
        };
        !b.invoke.is_empty() || !b.events.is_empty()
    }

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

/// 与持久化、`plugin_state` 比较时使用的 `appearance_id` 规范化（trim）。
#[must_use]
pub fn normalize_ui_slot_appearance_id(s: &str) -> String {
    s.trim().to_string()
}

fn normalize_appearance_id(s: &str) -> String {
    normalize_ui_slot_appearance_id(s)
}

/// 同一 manifest 内：每个 `(slot, appearance_id)` 至多出现一次（`appearance_id` 按 trim 后比较；空为默认键）。
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
