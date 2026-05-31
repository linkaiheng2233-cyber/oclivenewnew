//! Directory plugin `manifest.json` → `permissions` field: aligned with host runtime gates and PLUGIN_V1 permission spec.

/// Allows plugin to spawn child processes (requires user grant `process:spawn`).
pub const PROCESS_SPAWN: &str = "process:spawn";
/// Allows plugin or Remote backend outbound HTTP (requires user grant `network:*`).
pub const NETWORK_WILDCARD: &str = "network:*";
/// MCP over HTTP transport (requires user grant `mcp:http`, per server `id`).
pub const MCP_HTTP: &str = "mcp:http";
/// MCP over stdio transport (requires user grant `mcp:stdio`, per server `id`).
pub const MCP_STDIO: &str = "mcp:stdio";

/// All permission identifiers allowed by spec (fixed order, for validation and doc cross-reference).
pub const ALLOWED: &[&str] = &[PROCESS_SPAWN, NETWORK_WILDCARD, MCP_HTTP, MCP_STDIO];

/// Remote sidecar (`OCLIVE_REMOTE_PLUGIN_URL`) grant `id` under `network:*`.
pub const NETWORK_GRANT_REMOTE_PLUGIN: &str = "remote:plugin";
/// Remote LLM (`OCLIVE_REMOTE_LLM_URL`) grant `id` under `network:*`.
pub const NETWORK_GRANT_REMOTE_LLM: &str = "remote:llm";

/// Validate `permissions` array: unknown values error; empty array is valid.
///
/// # Errors
///
/// Returns `Err` when any entry is not in [`ALLOWED`].
pub fn validate_permissions_list(permissions: &[String]) -> Result<(), String> {
    for p in permissions {
        let t = p.trim();
        if t.is_empty() {
            return Err("目录插件 manifest：permissions 含空字符串".to_string());
        }
        if !ALLOWED.contains(&t) {
            return Err(format!(
                "目录插件 manifest：permissions 含未知权限「{}」；允许值为 {}",
                t,
                ALLOWED.join("、")
            ));
        }
    }
    Ok(())
}

/// Parse and validate `permissions` from manifest JSON root object (defaults to `[]`).
///
/// # Errors
///
/// Invalid JSON or `permissions` validation failure.
pub fn validate_directory_plugin_manifest_permissions(manifest_json: &str) -> Result<(), String> {
    let v: serde_json::Value = serde_json::from_str(manifest_json)
        .map_err(|e| format!("目录插件 manifest.json JSON 语法错误: {}", e))?;
    let Some(obj) = v.as_object() else {
        return Err("目录插件 manifest.json 根须为对象".to_string());
    };
    let permissions = parse_permissions_from_map(obj)?;
    validate_permissions_list(&permissions)
}

fn parse_permissions_from_map(
    obj: &serde_json::Map<String, serde_json::Value>,
) -> Result<Vec<String>, String> {
    match obj.get("permissions") {
        None => Ok(Vec::new()),
        Some(serde_json::Value::Array(arr)) => {
            let mut out = Vec::with_capacity(arr.len());
            for (i, v) in arr.iter().enumerate() {
                let Some(s) = v.as_str() else {
                    return Err(format!("目录插件 manifest：permissions[{}] 须为字符串", i));
                };
                out.push(s.to_string());
            }
            Ok(out)
        }
        Some(_) => Err("目录插件 manifest：permissions 须为字符串数组".to_string()),
    }
}

/// Whether `process:spawn` is declared (legacy compat: omitted `permissions` with a `process` block).
#[must_use]
pub fn manifest_declares_process_spawn(permissions: &[String], has_process_section: bool) -> bool {
    if permissions.iter().any(|p| p.trim() == PROCESS_SPAWN) {
        return has_process_section;
    }
    permissions.is_empty() && has_process_section
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allowed_permissions_pass() {
        validate_permissions_list(&[PROCESS_SPAWN.into(), NETWORK_WILDCARD.into()]).unwrap();
    }

    #[test]
    fn empty_permissions_ok() {
        validate_permissions_list(&[]).unwrap();
    }

    #[test]
    fn unknown_permission_rejected() {
        let err = validate_permissions_list(&["process".into()]).unwrap_err();
        assert!(err.contains("process:spawn"));
    }

    #[test]
    fn missing_permissions_field_ok() {
        let json = r#"{"schema_version":1,"id":"x","version":"1.0.0"}"#;
        validate_directory_plugin_manifest_permissions(json).unwrap();
    }

    #[test]
    fn illegal_permission_in_manifest_json() {
        let json = r#"{"permissions":["fs:read"]}"#;
        assert!(validate_directory_plugin_manifest_permissions(json).is_err());
    }

    #[test]
    fn legacy_process_without_permissions_field() {
        assert!(manifest_declares_process_spawn(&[], true));
        assert!(!manifest_declares_process_spawn(&[], false));
    }

    #[test]
    fn explicit_network_only_no_spawn() {
        assert!(!manifest_declares_process_spawn(
            &[NETWORK_WILDCARD.into()],
            true
        ));
    }
}
