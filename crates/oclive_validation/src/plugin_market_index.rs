//! Plugin market index (`plugins.json`) validation (v1).
//!
//! This is intentionally **no-code**: it validates the JSON contract and safety constraints.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashSet;

fn default_entry_type() -> String {
    "plugin".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginMarketIndexFileDisk {
    #[serde(default, alias = "generated_at")]
    pub generated_at: Option<String>,
    #[serde(default)]
    pub plugins: Vec<PluginMarketIndexEntryDisk>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginMarketIndexEntryDisk {
    #[serde(rename = "type", default = "default_entry_type")]
    pub entry_type: String,
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub author: String,
    pub version: String,
    #[serde(default)]
    pub git: String,
    #[serde(default)]
    pub permissions: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub changelog: Option<String>,
    #[serde(default)]
    pub dependencies: std::collections::HashMap<String, String>,
    #[serde(default)]
    pub publisher: Option<String>,
    #[serde(default)]
    pub public_keys: Vec<Value>,
    #[serde(default)]
    pub versions: Vec<Value>,

    /// `type=module` only: no-code module spec.
    #[serde(default)]
    pub module: Option<PluginIndexModuleSpecDisk>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginIndexModuleSpecDisk {
    #[serde(default)]
    pub plugins: Vec<PluginIndexModulePluginSpecDisk>,
    /// Mirrors runtime `PluginBackendsOverride` shape; we keep it as JSON to decouple from runtime types.
    #[serde(default)]
    pub backends: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginIndexModulePluginSpecDisk {
    pub id: String,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub source: Option<String>,
}

pub fn validate_plugin_market_index_v1(text: &str) -> Result<(), String> {
    let file: PluginMarketIndexFileDisk =
        serde_json::from_str(text).map_err(|e| format!("plugins.json：解析失败：{}", e))?;
    for (i, e) in file.plugins.iter().enumerate() {
        validate_entry(i, e)?;
    }
    Ok(())
}

fn validate_entry(i: usize, e: &PluginMarketIndexEntryDisk) -> Result<(), String> {
    let ty = e.entry_type.trim();
    if ty.is_empty() {
        return Err(format!("plugins.json：plugins[{}].type 不能为空", i));
    }
    if e.id.trim().is_empty() {
        return Err(format!("plugins.json：plugins[{}].id 不能为空", i));
    }
    if e.name.trim().is_empty() {
        return Err(format!("plugins.json：plugins[{}].name 不能为空", i));
    }
    if e.version.trim().is_empty() {
        return Err(format!("plugins.json：plugins[{}].version 不能为空", i));
    }

    match ty {
        "plugin" => {
            if e.git.trim().is_empty() {
                return Err(format!(
                    "plugins.json：plugins[{}] type=plugin 时 git 必填",
                    i
                ));
            }
            if e.module.is_some() {
                return Err(format!(
                    "plugins.json：plugins[{}] type=plugin 不允许出现 module 字段",
                    i
                ));
            }
        }
        "module" => {
            if !e.git.trim().is_empty() {
                return Err(format!(
                    "plugins.json：plugins[{}] type=module 时禁止提供 git（模块本身无代码）",
                    i
                ));
            }
            if !e.versions.is_empty() {
                return Err(format!(
                    "plugins.json：plugins[{}] type=module 时禁止提供 versions（模块本身无代码）",
                    i
                ));
            }
            if !e.public_keys.is_empty() {
                return Err(format!(
                    "plugins.json：plugins[{}] type=module 时禁止提供 publicKeys（模块本身无代码）",
                    i
                ));
            }
            if e.module.is_none() {
                return Err(format!(
                    "plugins.json：plugins[{}] type=module 时必须提供 module 字段",
                    i
                ));
            }
            let m = e.module.as_ref().unwrap();
            let mut ids = HashSet::<String>::new();
            for (j, p) in m.plugins.iter().enumerate() {
                let pid = p.id.trim();
                if pid.is_empty() {
                    return Err(format!(
                        "plugins.json：plugins[{}].module.plugins[{}].id 不能为空",
                        i, j
                    ));
                }
                if !ids.insert(pid.to_string()) {
                    return Err(format!(
                        "plugins.json：plugins[{}].module.plugins 出现重复 id：{}",
                        i, pid
                    ));
                }
            }
        }
        "profile" => {
            // Reserved: profile entries are accepted for forward-compat, but must remain no-code.
            if !e.git.trim().is_empty() {
                return Err(format!(
                    "plugins.json：plugins[{}] type=profile 时禁止提供 git（profile 本身无代码）",
                    i
                ));
            }
        }
        other => {
            return Err(format!(
                "plugins.json：plugins[{}].type={} 不受支持（允许：plugin|module|profile）",
                i, other
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn module_rejects_git() {
        let text = r#"{
          "plugins":[
            {"type":"module","id":"m","name":"M","version":"1.0.0","git":"https://x","module":{"plugins":[]}}
          ]
        }"#;
        assert!(validate_plugin_market_index_v1(text).is_err());
    }

    #[test]
    fn module_accepts_no_code() {
        let text = r#"{
          "plugins":[
            {"type":"module","id":"m","name":"M","version":"1.0.0","git":"","module":{"plugins":[{"id":"p1"}]}}
          ]
        }"#;
        assert!(validate_plugin_market_index_v1(text).is_ok());
    }
}

