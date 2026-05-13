//! Stable permission token registry (v1).
//!
//! This module is the single source of truth for:
//! - permission token strings shown in UI / index / profile
//! - bridge command -> permission token mapping (compat layer)
//!
//! Naming: keep tokens stable once shipped.

use crate::infrastructure::directory_plugins::OclivePluginManifest;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PermissionRisk {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionTokenInfo {
    pub token: &'static str,
    pub title: &'static str,
    pub description: &'static str,
    pub risk: PermissionRisk,
}

// NOTE: Keep the list short and human-friendly. This is v1; future fine-grained tokens can be added.
pub const PERMISSION_TOKENS_V1: &[PermissionTokenInfo] = &[
    PermissionTokenInfo {
        token: "read:conversation",
        title: "读取当前对话",
        description: "允许读取当前对话内容与摘要。",
        risk: PermissionRisk::Low,
    },
    PermissionTokenInfo {
        token: "read:conversations",
        title: "读取对话列表",
        description: "允许读取对话列表/历史索引。",
        risk: PermissionRisk::Low,
    },
    PermissionTokenInfo {
        token: "read:roles",
        title: "读取角色列表",
        description: "允许读取本机已安装角色的清单。",
        risk: PermissionRisk::Low,
    },
    PermissionTokenInfo {
        token: "read:current_role",
        title: "读取当前角色",
        description: "允许读取当前正在使用的角色信息。",
        risk: PermissionRisk::Low,
    },
    PermissionTokenInfo {
        token: "read:role_info",
        title: "读取角色运行时详情",
        description: "允许读取指定角色的运行时信息（桥接 get_role_info）。",
        risk: PermissionRisk::Low,
    },
    PermissionTokenInfo {
        token: "write:memory",
        title: "写入记忆",
        description: "允许写入/修改记忆数据。",
        risk: PermissionRisk::Medium,
    },
    PermissionTokenInfo {
        token: "write:emotion",
        title: "写入情绪",
        description: "允许写入/修改情绪数据。",
        risk: PermissionRisk::Medium,
    },
    PermissionTokenInfo {
        token: "write:event",
        title: "写入事件",
        description: "允许创建/修改事件记录。",
        risk: PermissionRisk::Medium,
    },
    PermissionTokenInfo {
        token: "write:prompt",
        title: "写入提示词配置",
        description: "允许写入与 Prompt 相关的配置/片段。",
        risk: PermissionRisk::Medium,
    },
    PermissionTokenInfo {
        token: "write:settings",
        title: "修改设置",
        description: "允许修改应用设置。",
        risk: PermissionRisk::Medium,
    },
    PermissionTokenInfo {
        token: "export:conversation",
        title: "导出对话",
        description: "允许导出对话记录到本地文件。",
        risk: PermissionRisk::Medium,
    },
    PermissionTokenInfo {
        token: "import:role",
        title: "导入角色包",
        description: "允许导入新的角色包到本机。",
        risk: PermissionRisk::High,
    },
    PermissionTokenInfo {
        token: "delete:role",
        title: "删除角色",
        description: "允许删除本机角色数据。",
        risk: PermissionRisk::High,
    },
    PermissionTokenInfo {
        token: "rpc:invoke",
        title: "目录插件 RPC 透传",
        description: "允许通过 directory_plugin_invoke 透传 JSON-RPC 调用（高风险）。",
        risk: PermissionRisk::High,
    },
    PermissionTokenInfo {
        token: "process:spawn",
        title: "启动子进程（目录插件）",
        description: "允许启动目录插件的 RPC 子进程（高风险）。",
        risk: PermissionRisk::High,
    },
    PermissionTokenInfo {
        token: "network:*",
        title: "网络访问（全部）",
        description: "允许访问网络（Remote HTTP 侧车 / 任意域名）。",
        risk: PermissionRisk::High,
    },
];

/// Bridge command name -> permission token mapping (compat layer).
///
/// If a command is not listed, we keep the historical behavior: treat the command as a token.
pub fn permission_token_for_bridge_command(cmd: &str) -> String {
    match cmd {
        "get_conversation" => "read:conversation".to_string(),
        "get_roles" | "list_roles" => "read:roles".to_string(),
        "get_current_role" => "read:current_role".to_string(),
        "get_role_info" => "read:role_info".to_string(),
        "update_memory" | "delete_memory" => "write:memory".to_string(),
        "update_emotion" => "write:emotion".to_string(),
        "update_event" => "write:event".to_string(),
        "update_prompt" => "write:prompt".to_string(),
        "export_conversation" => "export:conversation".to_string(),
        "import_role" => "import:role".to_string(),
        "delete_role" => "delete:role".to_string(),
        "update_settings" => "write:settings".to_string(),
        "get_conversation_list" => "read:conversations".to_string(),
        _ => cmd.to_string(),
    }
}

pub fn is_known_permission_token(token: &str) -> bool {
    let t = token.trim();
    if t.is_empty() {
        return false;
    }
    PERMISSION_TOKENS_V1.iter().any(|x| x.token == t)
}

/// 从目录插件 manifest 收集桥接 `invoke` 项对应的权限令牌（与 `permission_token_for_bridge_command` 一致）。
#[must_use]
pub fn bridge_permission_tokens_from_manifest(manifest: &OclivePluginManifest) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    if let Some(sh) = &manifest.shell {
        if let Some(b) = &sh.bridge {
            for x in &b.invoke {
                let t = x.trim();
                if t.is_empty() {
                    continue;
                }
                let perm = if t.contains(':') {
                    t.to_string()
                } else {
                    permission_token_for_bridge_command(t)
                };
                out.push(perm);
            }
        }
    }
    for us in &manifest.ui_slots {
        if let Some(b) = &us.bridge {
            for x in &b.invoke {
                let t = x.trim();
                if t.is_empty() {
                    continue;
                }
                let perm = if t.contains(':') {
                    t.to_string()
                } else {
                    permission_token_for_bridge_command(t)
                };
                out.push(perm);
            }
        }
    }
    out.sort();
    out.dedup();
    out
}
