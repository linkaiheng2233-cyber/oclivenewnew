use crate::api::role::build_plugin_resolution_debug_info;
use crate::error::AppError;
use crate::models::dto::{
    ExportChatLogsRequest, ExportChatLogsResponse, PluginResolutionDebugInfo,
};
use crate::state::AppState;
use chrono::Local;
use serde::Serialize;
use tauri::State;

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

fn sanitize_filename(s: &str) -> String {
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

async fn load_turns(state: &AppState, role_id: &str) -> Result<Vec<ExportTurn>, String> {
    let rows = state
        .db_manager
        .list_short_term_turns(role_id)
        .await
        .map_err(|e| e.to_frontend_error())?;
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

fn build_txt(
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
            "pack_default: mem={:?} emotion={:?} event={:?} prompt={:?} llm={:?}\n",
            d.plugin_backends_pack_default.memory,
            d.plugin_backends_pack_default.emotion,
            d.plugin_backends_pack_default.event,
            d.plugin_backends_pack_default.prompt,
            d.plugin_backends_pack_default.llm
        ));
        s.push_str(&format!(
            "effective: mem={:?}({:?}) emotion={:?}({:?}) event={:?}({:?}) prompt={:?}({:?}) llm={:?}({:?})\n",
            d.plugin_backends_effective.memory,
            d.plugin_backends_effective_sources.memory,
            d.plugin_backends_effective.emotion,
            d.plugin_backends_effective_sources.emotion,
            d.plugin_backends_effective.event,
            d.plugin_backends_effective_sources.event,
            d.plugin_backends_effective.prompt,
            d.plugin_backends_effective_sources.prompt,
            d.plugin_backends_effective.llm,
            d.plugin_backends_effective_sources.llm
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

pub async fn export_chat_logs_impl(
    state: &AppState,
    req: &ExportChatLogsRequest,
) -> Result<ExportChatLogsResponse, String> {
    let fmt = req.format.to_lowercase();
    if fmt != "json" && fmt != "txt" {
        return Err(
            AppError::InvalidParameter("format must be json or txt".to_string())
                .to_frontend_error(),
        );
    }

    let date = Local::now().format("%Y-%m-%d").to_string();
    let mut blocks: Vec<(String, String, Vec<ExportTurn>)> = Vec::new();

    let include_plugin_debug = req.include_plugin_resolution_debug && !req.all_roles;

    if req.all_roles {
        let roles = state
            .storage
            .load_all_roles()
            .map_err(|e| e.to_frontend_error())?;
        for r in roles {
            let turns = load_turns(state, &r.id).await?;
            blocks.push((r.id.clone(), r.name.clone(), turns));
        }
        let filename = format!("沐沐_聊天记录_全部角色_{}.{}", date, fmt);
        let content = if fmt == "json" {
            let root = ExportJsonRoot {
                exported_at: Local::now().to_rfc3339(),
                app: "oclivenewnew",
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
            serde_json::to_string_pretty(&root)
                .map_err(|e| AppError::SerializationError(e).to_frontend_error())?
        } else {
            build_txt(&blocks, None)
        };
        return Ok(ExportChatLogsResponse {
            content,
            suggested_filename: sanitize_filename(&filename),
        });
    }

    let rid = req.role_id.as_deref().ok_or_else(|| {
        AppError::InvalidParameter("role_id required when all_roles is false".to_string())
            .to_frontend_error()
    })?;
    let role = state
        .load_role_cached(rid)
        .map_err(|e| e.to_frontend_error())?;
    let turns = load_turns(state, rid).await?;
    blocks.push((role.id.clone(), role.name.clone(), turns));
    let plugin_debug = if include_plugin_debug {
        Some(build_plugin_resolution_debug_info(state, rid, req.session_id.as_deref()).await?)
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
            app: "oclivenewnew",
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
        serde_json::to_string_pretty(&root)
            .map_err(|e| AppError::SerializationError(e).to_frontend_error())?
    } else {
        build_txt(&blocks, plugin_debug.as_ref())
    };

    Ok(ExportChatLogsResponse {
        content,
        suggested_filename: filename,
    })
}

#[tauri::command]
pub async fn export_chat_logs(
    req: ExportChatLogsRequest,
    state: State<'_, AppState>,
) -> Result<ExportChatLogsResponse, String> {
    export_chat_logs_impl(&state, &req).await
}
