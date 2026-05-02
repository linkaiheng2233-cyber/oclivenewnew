//! 从目录插件 manifest 收集桥接 `invoke` 项对应的 **权限令牌**（与内核
//! `permission_token_for_bridge_command` 一致），供安装预览与 `plugin_bridge` 种子共用。

use crate::domain::permission_tokens::permission_token_for_bridge_command;
use crate::infrastructure::directory_plugins::OclivePluginManifest;

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
