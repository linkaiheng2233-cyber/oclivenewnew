//! 本地插件桥接抽象（Phase 2 骨架）：统一 provider 注册与版本门禁。

use oclive_validation::validate_min_runtime_version_for_local_plugin;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// 本地插件规范版本（`schema_version`）当前支持值。
pub const LOCAL_PLUGIN_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LocalPluginCapability {
    Memory,
    Emotion,
    Event,
    Prompt,
    Llm,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocalPluginProviderDescriptor {
    pub provider_id: String,
    /// `schema_version` / `min_runtime_version` 与文档规范保持一致，用于宿主门禁。
    pub schema_version: u32,
    #[serde(default)]
    pub min_runtime_version: Option<String>,
    #[serde(default)]
    pub capabilities: Vec<LocalPluginCapability>,
}

/// Provider 发现桥接接口（后续可由 WASM / Native Process 两种实现提供）。
pub trait LocalPluginBridge: Send + Sync {
    fn bridge_name(&self) -> &'static str;
    fn discover_providers(&self) -> Vec<LocalPluginProviderDescriptor>;
}

/// 文件清单桥接：从目录中的 JSON 描述文件发现本地 provider。
///
/// 约定：扫描目录下 `*.json`，每个文件应是 `LocalPluginProviderDescriptor`。
pub struct FileManifestLocalPluginBridge {
    manifest_dir: PathBuf,
}

impl FileManifestLocalPluginBridge {
    #[must_use]
    pub fn new(manifest_dir: impl AsRef<Path>) -> Self {
        Self {
            manifest_dir: manifest_dir.as_ref().to_path_buf(),
        }
    }

    #[must_use]
    pub fn manifest_dir(&self) -> &Path {
        &self.manifest_dir
    }
}

impl LocalPluginBridge for FileManifestLocalPluginBridge {
    fn bridge_name(&self) -> &'static str {
        "file_manifest"
    }

    fn discover_providers(&self) -> Vec<LocalPluginProviderDescriptor> {
        let mut out = Vec::new();
        let Ok(entries) = fs::read_dir(&self.manifest_dir) else {
            return out;
        };
        for entry in entries.flatten() {
            let p = entry.path();
            let is_json = p
                .extension()
                .and_then(|s| s.to_str())
                .map(|s| s.eq_ignore_ascii_case("json"))
                .unwrap_or(false);
            if !is_json {
                continue;
            }
            let text = match fs::read_to_string(&p) {
                Ok(v) => v,
                Err(e) => {
                    log::warn!(
                        target: "oclive_plugin",
                        "local plugin manifest read failed path={} err={}",
                        p.display(),
                        e
                    );
                    continue;
                }
            };
            match serde_json::from_str::<LocalPluginProviderDescriptor>(&text) {
                Ok(desc) => out.push(desc),
                Err(e) => log::warn!(
                    target: "oclive_plugin",
                    "local plugin manifest parse failed path={} err={}",
                    p.display(),
                    e
                ),
            }
        }
        out
    }
}

#[derive(Default)]
pub struct LocalPluginRegistry {
    providers: HashMap<String, Arc<LocalPluginProviderDescriptor>>,
}

impl LocalPluginRegistry {
    pub fn register_provider(
        &mut self,
        descriptor: LocalPluginProviderDescriptor,
    ) -> Result<(), String> {
        if descriptor.provider_id.trim().is_empty() {
            return Err("local plugin provider_id 不能为空".to_string());
        }
        if descriptor.schema_version == 0 {
            return Err(format!(
                "local plugin provider={} schema_version 不能为 0；请使用 schema_version={}",
                descriptor.provider_id, LOCAL_PLUGIN_SCHEMA_VERSION
            ));
        }
        if descriptor.schema_version != LOCAL_PLUGIN_SCHEMA_VERSION {
            return Err(format!(
                "local plugin provider={} schema_version={} 不受支持（当前仅支持 {}）",
                descriptor.provider_id, descriptor.schema_version, LOCAL_PLUGIN_SCHEMA_VERSION
            ));
        }
        validate_min_runtime_version_for_local_plugin(
            descriptor.min_runtime_version.as_deref(),
            env!("CARGO_PKG_VERSION"),
        )?;
        self.providers
            .insert(descriptor.provider_id.clone(), Arc::new(descriptor));
        Ok(())
    }

    #[must_use]
    pub fn providers_for_capability(
        &self,
        capability: LocalPluginCapability,
    ) -> Vec<Arc<LocalPluginProviderDescriptor>> {
        self.providers
            .values()
            .filter(|d| d.capabilities.contains(&capability))
            .cloned()
            .collect()
    }

    #[must_use]
    pub fn all_providers(&self) -> Vec<Arc<LocalPluginProviderDescriptor>> {
        self.providers.values().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn register_rejects_unsupported_schema() {
        let mut registry = LocalPluginRegistry::default();
        let err = registry
            .register_provider(LocalPluginProviderDescriptor {
                provider_id: "demo".to_string(),
                schema_version: 999,
                min_runtime_version: None,
                capabilities: vec![LocalPluginCapability::Memory],
            })
            .expect_err("unsupported schema must fail");
        assert!(err.contains("schema_version"));
    }

    #[test]
    fn register_rejects_future_runtime_requirement() {
        let mut registry = LocalPluginRegistry::default();
        let err = registry
            .register_provider(LocalPluginProviderDescriptor {
                provider_id: "demo".to_string(),
                schema_version: LOCAL_PLUGIN_SCHEMA_VERSION,
                min_runtime_version: Some("99.0.0".to_string()),
                capabilities: vec![LocalPluginCapability::Memory],
            })
            .expect_err("future runtime requirement must fail");
        assert!(err.contains("本地插件"));
        assert!(err.contains("最低"));
    }

    #[test]
    fn register_rejects_schema_version_zero() {
        let mut registry = LocalPluginRegistry::default();
        let err = registry
            .register_provider(LocalPluginProviderDescriptor {
                provider_id: "demo".to_string(),
                schema_version: 0,
                min_runtime_version: None,
                capabilities: vec![LocalPluginCapability::Memory],
            })
            .expect_err("schema 0 must fail");
        assert!(err.contains("不能为 0"));
    }

    #[test]
    fn register_rejects_invalid_min_runtime_semver() {
        let mut registry = LocalPluginRegistry::default();
        let err = registry
            .register_provider(LocalPluginProviderDescriptor {
                provider_id: "demo".to_string(),
                schema_version: LOCAL_PLUGIN_SCHEMA_VERSION,
                min_runtime_version: Some("v1.x".to_string()),
                capabilities: vec![LocalPluginCapability::Memory],
            })
            .expect_err("bad semver");
        assert!(err.contains("本地插件"));
    }

    #[test]
    fn register_and_query_capability_works() {
        let mut registry = LocalPluginRegistry::default();
        registry
            .register_provider(LocalPluginProviderDescriptor {
                provider_id: "demo".to_string(),
                schema_version: LOCAL_PLUGIN_SCHEMA_VERSION,
                min_runtime_version: None,
                capabilities: vec![LocalPluginCapability::Memory, LocalPluginCapability::Prompt],
            })
            .expect("register");
        assert_eq!(registry.providers_for_capability(LocalPluginCapability::Memory).len(), 1);
        assert_eq!(registry.providers_for_capability(LocalPluginCapability::Llm).len(), 0);
    }

    #[test]
    fn file_manifest_bridge_discovers_json_descriptors() {
        let dir = TempDir::new().expect("tempdir");
        let p = dir.path().join("demo.json");
        let mut f = std::fs::File::create(&p).expect("create");
        let body = r#"{
  "provider_id": "demo.local",
  "schema_version": 1,
  "capabilities": ["memory", "prompt"]
}"#;
        f.write_all(body.as_bytes()).expect("write");
        let bridge = FileManifestLocalPluginBridge::new(dir.path());
        let providers = bridge.discover_providers();
        assert_eq!(providers.len(), 1);
        assert_eq!(providers[0].provider_id, "demo.local");
    }
}
