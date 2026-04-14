//! `plugins/<id>/manifest.json`

use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Clone, Deserialize)]
pub struct ShellSection {
    /// 相对插件根，如 `ui/index.html`
    pub entry: String,
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

#[derive(Debug, Clone, Deserialize)]
pub struct OclivePluginManifest {
    pub schema_version: u32,
    pub id: String,
    pub version: String,
    #[serde(default)]
    pub shell: Option<ShellSection>,
    #[serde(default)]
    pub process: Option<ProcessSection>,
    /// stdout 就绪行前缀，默认 `OCLIVE_READY`
    #[serde(default = "default_ready_prefix")]
    pub ready_prefix: String,
}

fn default_ready_prefix() -> String {
    "OCLIVE_READY".to_string()
}

impl OclivePluginManifest {
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
        Ok(m)
    }
}
