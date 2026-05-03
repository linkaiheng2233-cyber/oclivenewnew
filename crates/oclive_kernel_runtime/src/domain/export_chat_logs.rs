//! 聊天记录导出：桌面 `export_chat_logs` 与 OOCP `session.export_chat_logs` 共用实现。

use crate::domain::plugin_resolution_debug::build_plugin_resolution_debug_info;
use crate::error::{AppError, Result};
use crate::models::dto::{
    ExportChatLogsRequest, ExportChatLogsResponse, PluginResolutionDebugInfo,
};
use crate::state::KernelAppState;
use chrono::Local;
use serde::Serialize;
use serde_json::{json, Value};

/// JSON 根字段 `app`，与历史桌面导出保持一致。
pub const EXPORT_CHAT_LOG_JSON_APP: &str = "oclivenewnew";

#[derive(Debug, Clone, Serialize)]
struct ExportTurn {
    at: String,
    scene: Option<String>,
    user: String,
    bot: String,
}

#[derive(Debug, Serialize)]
struct ExportRoleBlock {
    role_id: String,
    role_name: String,
    turns: Vec<ExportTurn>,
}

#[derive(Debug, Serialize)]
struct ExportJsonRoot {
    exported_at: String,
    app: &'static str,
    roles: Vec<ExportRoleBlock>,
    #[serde(skip_serializing_if = "Option::is_none")]
    plugin_resolution_debug: Option<PluginResolutionDebugInfo>,
}

pub fn sanitize_filename(s: &str) -> String {
    s.chars()
        .map(|c| {
            if matches!(c, '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|') {
                '_'
            } else {
                c
            }
        })
        .collect()
}

async fn load_export_turns(state: &KernelAppState, role_id: &str) -> Result<Vec<ExportTurn>> {
    let rows = state.db_manager.list_short_term_turns(role_id).await?;
    Ok(rows
        .into_iter()
        .map(|(user, bot, _emotion, scene, at)| ExportTurn {
            user,
            bot,
            scene,
            at,
        })
        .collect())
}

fn build_txt_mumu(
    roles: &[(String, String, Vec<ExportTurn>)],
    plugin_debug: Option<&PluginResolutionDebugInfo>,
) -> String {
    let mut s = String::new();
    s.push_str("# 沐沐 聊天记录\n");
    s.push_str(&format!("导出时间: {}\n\n", Local::now().to_rfc3339()));
    if let Some(d) = plugin_debug {
        s.push_str("## 插件解析诊断\n");
        s.push_str(&format!(
            "app_version: {} api_version: {} schema_version: {}\n",
            d.app_version, d.api_version, d.schema_version
        ));
        s.push_str(&format!("session_namespace: {}\n", d.session_namespace));
        s.push_str(&format!(
            "pack_default: mem={:?} emotion={:?} event={:?} prompt={:?} llm={:?} agent={:?}\n",
            d.plugin_backends_pack_default.memory,
            d.plugin_backends_pack_default.emotion,
            d.plugin_backends_pack_default.event,
            d.plugin_backends_pack_default.prompt,
            d.plugin_backends_pack_default.llm,
            d.plugin_backends_pack_default.agent
        ));
        s.push_str(&format!(
            "effective: mem={:?}({:?}) emotion={:?}({:?}) event={:?}({:?}) prompt={:?}({:?}) llm={:?}({:?}) agent={:?}({:?})\n",
            d.plugin_backends_effective.memory,
            d.plugin_backends_effective_sources.memory,
            d.plugin_backends_effective.emotion,
            d.plugin_backends_effective_sources.emotion,
            d.plugin_backends_effective.event,
            d.plugin_backends_effective_sources.event,
            d.plugin_backends_effective.prompt,
            d.plugin_backends_effective_sources.prompt,
            d.plugin_backends_effective.llm,
            d.plugin_backends_effective_sources.llm,
            d.plugin_backends_effective.agent,
            d.plugin_backends_effective_sources.agent
        ));
        s.push_str(&format!(
            "env: llm_override={} remote_plugin_url={} remote_llm_url={}\n\n",
            d.llm_env_override.as_deref().unwrap_or("none"),
            if d.remote_plugin_url_configured {
                "set"
            } else {
                "unset"
            },
            if d.remote_llm_url_configured {
                "set"
            } else {
                "unset"
            }
        ));
        s.push_str(&format!(
            "local_providers: count={} ids={}\n\n",
            d.local_provider_count,
            if d.local_provider_ids.is_empty() {
                "none".to_string()
            } else {
                d.local_provider_ids.join(",")
            }
        ));
    }
    for (id, name, turns) in roles {
        s.push_str(&format!("=== {} ({}) ===\n", name, id));
        for t in turns {
            let sc = t.scene.as_deref().unwrap_or("-");
            s.push_str(&format!(
                "[{}] 场景: {}\n用户: {}\n沐沐: {}\n\n",
                t.at, sc, t.user, t.bot
            ));
        }
        s.push('\n');
    }
    s
}

/// 桌面 / HTTP `export_chat_logs`；`app_version` 由嵌入方传入（如 `env!(\"CARGO_PKG_VERSION\")`）。
pub async fn export_chat_logs(
    state: &KernelAppState,
    req: &ExportChatLogsRequest,
    app_version: &str,
) -> Result<ExportChatLogsResponse> {
    let fmt = req.format.to_lowercase();
    if fmt != "json" && fmt != "txt" {
        return Err(AppError::InvalidParameter(
            "format must be json or txt".to_string(),
        ));
    }

    let date = Local::now().format("%Y-%m-%d").to_string();
    let mut blocks: Vec<(String, String, Vec<ExportTurn>)> = Vec::new();

    let include_plugin_debug = req.include_plugin_resolution_debug && !req.all_roles;

    if req.all_roles {
        let roles = state.storage.load_all_roles()?;
        for r in roles {
            let turns = load_export_turns(state, &r.id).await?;
            blocks.push((r.id.clone(), r.name.clone(), turns));
        }
        let filename = format!("沐沐_聊天记录_全部角色_{}.{}", date, fmt);
        let content = if fmt == "json" {
            let root = ExportJsonRoot {
                exported_at: Local::now().to_rfc3339(),
                app: EXPORT_CHAT_LOG_JSON_APP,
                roles: blocks
                    .iter()
                    .map(|(id, name, turns)| ExportRoleBlock {
                        role_id: id.clone(),
                        role_name: name.clone(),
                        turns: turns.clone(),
                    })
                    .collect(),
                plugin_resolution_debug: None,
            };
            serde_json::to_string_pretty(&root).map_err(AppError::SerializationError)?
        } else {
            build_txt_mumu(&blocks, None)
        };
        return Ok(ExportChatLogsResponse {
            content,
            suggested_filename: sanitize_filename(&filename),
        });
    }

    let rid = req.role_id.as_deref().ok_or_else(|| {
        AppError::InvalidParameter("role_id required when all_roles is false".to_string())
    })?;
    let role = state.load_role_cached(rid)?;
    let turns = load_export_turns(state, rid).await?;
    blocks.push((role.id.clone(), role.name.clone(), turns));
    let plugin_debug = if include_plugin_debug {
        Some(
            build_plugin_resolution_debug_info(state, rid, req.session_id.as_deref(), app_version)
                .await?,
        )
    } else {
        None
    };

    let filename = format!(
        "沐沐_聊天记录_{}_{}.{}",
        sanitize_filename(&role.name),
        date,
        fmt
    );
    let content = if fmt == "json" {
        let root = ExportJsonRoot {
            exported_at: Local::now().to_rfc3339(),
            app: EXPORT_CHAT_LOG_JSON_APP,
            roles: blocks
                .iter()
                .map(|(id, name, turns)| ExportRoleBlock {
                    role_id: id.clone(),
                    role_name: name.clone(),
                    turns: turns.clone(),
                })
                .collect(),
            plugin_resolution_debug: plugin_debug,
        };
        serde_json::to_string_pretty(&root).map_err(AppError::SerializationError)?
    } else {
        build_txt_mumu(&blocks, plugin_debug.as_ref())
    };

    Ok(ExportChatLogsResponse {
        content,
        suggested_filename: filename,
    })
}

/// OOCP `session.export_chat_logs`：单角色、与历史 OOCP 结果形态一致。
pub async fn export_session_chat_logs_oocp_value(
    state: &KernelAppState,
    role_id: &str,
    format: &str,
) -> Result<Value> {
    let fmt = format.trim().to_ascii_lowercase();
    if fmt != "json" && fmt != "txt" {
        return Err(AppError::InvalidParameter(
            "format must be json or txt".to_string(),
        ));
    }

    let turn_rows = state.db_manager.list_short_term_turns(role_id).await?;
    let date = Local::now().format("%Y-%m-%d").to_string();
    let role = state.load_role_cached(role_id)?;

    let suggested_filename = format!(
        "Oclive_chat_{}_{}.{}",
        sanitize_filename(&role.name),
        date,
        fmt
    );
    let content = if fmt == "json" {
        let items: Vec<Value> = turn_rows
            .iter()
            .map(|(user, bot, _emotion, scene, at)| {
                json!({
                    "at": at,
                    "scene": scene,
                    "user": user,
                    "bot": bot,
                })
            })
            .collect();
        serde_json::to_string_pretty(&json!({
            "exported_at": Local::now().to_rfc3339(),
            "app": "oclive",
            "role_id": role.id,
            "role_name": role.name,
            "turns": items,
        }))
        .map_err(AppError::SerializationError)?
    } else {
        let mut s = String::new();
        s.push_str(&format!(
            "# Oclive Chat Logs\nrole: {} ({})\nexported_at: {}\n\n",
            role.name,
            role.id,
            Local::now().to_rfc3339()
        ));
        for (user, bot, _emotion, scene, at) in &turn_rows {
            let sc = scene.as_deref().unwrap_or("-");
            s.push_str(&format!(
                "[{}] scene: {}\nuser: {}\nbot: {}\n\n",
                at, sc, user, bot
            ));
        }
        s
    };

    Ok(json!({
        "format": fmt,
        "suggested_filename": suggested_filename,
        "content": content,
    }))
}
