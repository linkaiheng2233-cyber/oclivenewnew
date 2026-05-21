//! 目录插件 `manifest.json` → `permissions` 字段：与宿主运行时门禁、PLUGIN_V1 权限规范一致。

/// 允许插件 spawn 子进程（须用户授权 `process:spawn`）。
pub const PROCESS_SPAWN: &str = "process:spawn";
/// 允许插件或 Remote 后端发起出站 HTTP（须用户授权 `network:*`）。
pub const NETWORK_WILDCARD: &str = "network:*";
/// MCP over HTTP 传输（须用户授权 `mcp:http`，按 server `id` 粒度）。
pub const MCP_HTTP: &str = "mcp:http";
/// MCP over stdio 传输（须用户授权 `mcp:stdio`，按 server `id` 粒度）。
pub const MCP_STDIO: &str = "mcp:stdio";

/// 规范允许的全部权限标识（顺序固定，供校验与文档对照）。
pub const ALLOWED: &[&str] = &[PROCESS_SPAWN, NETWORK_WILDCARD, MCP_HTTP, MCP_STDIO];

/// Remote 侧车（`OCLIVE_REMOTE_PLUGIN_URL`）在 `network:*` 下的 grant `id`。
pub const NETWORK_GRANT_REMOTE_PLUGIN: &str = "remote:plugin";
/// Remote LLM（`OCLIVE_REMOTE_LLM_URL`）在 `network:*` 下的 grant `id`。
pub const NETWORK_GRANT_REMOTE_LLM: &str = "remote:llm";

/// 校验 `permissions` 数组：未知值报错；空数组合法。
///
/// # Errors
///
/// 含不在 [`ALLOWED`] 内的条目时返回 `Err`。
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

/// 从 manifest JSON 根对象解析并校验 `permissions`（缺省视为 `[]`）。
///
/// # Errors
///
/// JSON 非法或 `permissions` 校验失败。
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

/// 是否声明需要 `process:spawn`（含旧版兼容：省略 `permissions` 且存在 `process` 块）。
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
