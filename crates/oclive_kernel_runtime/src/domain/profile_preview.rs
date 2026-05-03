//! 从磁盘路径预览 `profile` JSON（无 `KernelAppState` 依赖）。

use crate::api::BridgeApiError;
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfilePluginSpecDto {
    pub id: String,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub source: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfilePermissionsDto {
    #[serde(default)]
    pub predeclared: Vec<String>,
    #[serde(default)]
    pub require_confirm: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileBackendsDto {
    #[serde(default)]
    pub memory: Option<String>,
    #[serde(default)]
    pub emotion: Option<String>,
    #[serde(default)]
    pub event: Option<String>,
    #[serde(default)]
    pub prompt: Option<String>,
    #[serde(default)]
    pub llm: Option<String>,
    #[serde(default)]
    pub agent: Option<String>,
    #[serde(default)]
    pub complex_emotion: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfilePreviewDto {
    pub id: String,
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub developer_mode: bool,
    #[serde(default)]
    pub market_sources: Vec<String>,
    #[serde(default)]
    pub plugins: Vec<ProfilePluginSpecDto>,
    #[serde(default)]
    pub permissions: Option<ProfilePermissionsDto>,
    #[serde(default)]
    pub backends: Option<ProfileBackendsDto>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewProfileFromPathRequest {
    pub path: String,
}

#[derive(Debug, Deserialize)]
struct RawBackendCfg {
    #[serde(default)]
    backend: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RawBackends {
    #[serde(default)]
    memory: Option<RawBackendCfg>,
    #[serde(default)]
    emotion: Option<RawBackendCfg>,
    #[serde(default)]
    event: Option<RawBackendCfg>,
    #[serde(default)]
    prompt: Option<RawBackendCfg>,
    #[serde(default)]
    llm: Option<RawBackendCfg>,
    #[serde(default)]
    agent: Option<RawBackendCfg>,
    #[serde(default)]
    complex_emotion: Option<RawBackendCfg>,
}

#[derive(Debug, Deserialize)]
struct RawProfileFile {
    #[serde(default)]
    r#type: String,
    #[serde(default)]
    schema_version: String,
    id: String,
    name: String,
    version: String,
    #[serde(default)]
    developer_mode: bool,
    #[serde(default)]
    market_sources: Vec<String>,
    #[serde(default)]
    plugins: Vec<ProfilePluginSpecDto>,
    #[serde(default)]
    permissions: Option<ProfilePermissionsDto>,
    #[serde(default)]
    backends: Option<RawBackends>,
}

pub fn preview_profile_from_path(req: &PreviewProfileFromPathRequest) -> Result<ProfilePreviewDto, String> {
    let path = req.path.trim();
    if path.is_empty() {
        return Err(
            BridgeApiError::InvalidParameter {
                message: "path required".into(),
            }
            .to_string(),
        );
    }
    let text = fs::read_to_string(path).map_err(|e| {
        BridgeApiError::Io {
            message: format!("read profile failed: {}", e),
        }
        .to_string()
    })?;
    let raw: RawProfileFile = serde_json::from_str(&text).map_err(|e| {
        BridgeApiError::InvalidParameter {
            message: format!("invalid profile json: {}", e),
        }
        .to_string()
    })?;
    if raw.r#type.trim() != "profile" {
        return Err(
            BridgeApiError::InvalidParameter {
                message: "profile.type must be \"profile\"".into(),
            }
            .to_string(),
        );
    }
    if raw.schema_version.trim() != "1.0" {
        return Err(
            BridgeApiError::InvalidParameter {
                message: "profile.schema_version must be \"1.0\"".into(),
            }
            .to_string(),
        );
    }

    let backends = raw.backends.map(|b| ProfileBackendsDto {
        memory: b
            .memory
            .and_then(|x| x.backend)
            .map(|s| s.trim().to_string()),
        emotion: b
            .emotion
            .and_then(|x| x.backend)
            .map(|s| s.trim().to_string()),
        event: b
            .event
            .and_then(|x| x.backend)
            .map(|s| s.trim().to_string()),
        prompt: b
            .prompt
            .and_then(|x| x.backend)
            .map(|s| s.trim().to_string()),
        llm: b.llm.and_then(|x| x.backend).map(|s| s.trim().to_string()),
        agent: b
            .agent
            .and_then(|x| x.backend)
            .map(|s| s.trim().to_string()),
        complex_emotion: b
            .complex_emotion
            .and_then(|x| x.backend)
            .map(|s| s.trim().to_string()),
    });

    Ok(ProfilePreviewDto {
        id: raw.id,
        name: raw.name,
        version: raw.version,
        developer_mode: raw.developer_mode,
        market_sources: raw.market_sources,
        plugins: raw.plugins,
        permissions: raw.permissions,
        backends,
    })
}
