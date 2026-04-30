//! Module 9: Expert Models API (Tauri commands).

use crate::api::error::ApiError;
use crate::domain::chat_engine::conversation_state_role_id;
use crate::domain::expert_models::{compile_graph_to_llama_local_config, LLAMA_LOCAL_PLUGIN_ID};
use crate::infrastructure::plugin_data::write_config_json;
use crate::infrastructure::remote_plugin::{invoke_directory_plugin_rpc_blocking, RemoteRpcChannel};
use crate::models::dto::{
    ExpertModelsApplyResult, ExpertModelsApplyToSessionRequest, ExpertModelsClearRoleDefaultRequest,
    ExpertModelsClearSessionOverrideRequest, ExpertModelsEffectiveResponse,
    ExpertModelsGetEffectiveRequest, ExpertModelsSetRoleDefaultRequest,
    ExpertModelsSetSessionOverrideRequest,
};
use crate::models::{ExpertConfigSource, ExpertGraph, LlamaLocalPluginConfig, PromptStyleOverride};
use crate::state::AppState;
use serde_json::json;
use serde::Serialize;
use tauri::State;

fn parse_graph_json(raw: Option<String>) -> Result<Option<ExpertGraph>, String> {
    let Some(s) = raw else {
        return Ok(None);
    };
    let t = s.trim();
    if t.is_empty() {
        return Ok(None);
    }
    serde_json::from_str::<ExpertGraph>(t)
        .map(Some)
        .map_err(|e| ApiError::InvalidParameter {
            message: format!("invalid expert graph json: {}", e),
        }
        .to_string())
}

fn parse_prompt_style_json(raw: Option<String>) -> Result<Option<PromptStyleOverride>, String> {
    let Some(s) = raw else {
        return Ok(None);
    };
    let t = s.trim();
    if t.is_empty() {
        return Ok(None);
    }
    serde_json::from_str::<PromptStyleOverride>(t)
        .map(Some)
        .map_err(|e| ApiError::InvalidParameter {
            message: format!("invalid prompt style json: {}", e),
        }
        .to_string())
}

fn to_json_string<T: serde::Serialize>(v: &T) -> Result<String, String> {
    serde_json::to_string(v).map_err(|e| {
        ApiError::InvalidParameter {
            message: format!("serialize failed: {}", e),
        }
        .to_string()
    })
}

async fn effective_for_session(
    state: &AppState,
    role_id: &str,
    session_ns: &str,
) -> Result<(ExpertGraph, ExpertConfigSource, Option<PromptStyleOverride>, ExpertConfigSource), String>
{
    let role_default_graph_raw = state
        .expert_models_repo
        .get_expert_models_role_default_json(role_id)
        .await
        .map_err(|e| e.to_frontend_error())?;
    let sess_graph_raw = state
        .expert_models_repo
        .get_expert_models_session_override_json(session_ns)
        .await
        .map_err(|e| e.to_frontend_error())?;

    let role_default_style_raw = state
        .expert_models_repo
        .get_expert_prompt_style_role_default_json(role_id)
        .await
        .map_err(|e| e.to_frontend_error())?;
    let sess_style_raw = state
        .expert_models_repo
        .get_expert_prompt_style_session_override_json(session_ns)
        .await
        .map_err(|e| e.to_frontend_error())?;

    let sess_graph = parse_graph_json(sess_graph_raw)?;
    let role_graph = parse_graph_json(role_default_graph_raw)?;
    let (graph, graph_source) = if let Some(g) = sess_graph {
        (g, ExpertConfigSource::SessionOverride)
    } else if let Some(g) = role_graph {
        (g, ExpertConfigSource::RoleDefault)
    } else {
        (ExpertGraph::default(), ExpertConfigSource::PackDefault)
    };

    let sess_style = parse_prompt_style_json(sess_style_raw)?;
    let role_style = parse_prompt_style_json(role_default_style_raw)?;
    let (style, style_source) = if sess_style.is_some() {
        (sess_style, ExpertConfigSource::SessionOverride)
    } else if role_style.is_some() {
        (role_style, ExpertConfigSource::RoleDefault)
    } else {
        (None, ExpertConfigSource::PackDefault)
    };

    Ok((graph, graph_source, style, style_source))
}

#[tauri::command]
pub async fn expert_models_get_effective(
    req: ExpertModelsGetEffectiveRequest,
    state: State<'_, AppState>,
) -> Result<ExpertModelsEffectiveResponse, String> {
    let role_id = req.role_id.trim();
    if role_id.is_empty() {
        return Err(ApiError::InvalidParameter {
            message: "role_id required".into(),
        }
        .to_string());
    }
    let session_ns = conversation_state_role_id(role_id, req.session_id.as_deref());
    let (graph, graph_source, prompt_style, prompt_style_source) =
        effective_for_session(&state, role_id, session_ns.as_str()).await?;
    Ok(ExpertModelsEffectiveResponse {
        graph,
        prompt_style,
        graph_source,
        prompt_style_source,
    })
}

#[tauri::command]
pub async fn expert_models_set_session_override(
    req: ExpertModelsSetSessionOverrideRequest,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let role_id = req.role_id.trim();
    if role_id.is_empty() {
        return Err(ApiError::InvalidParameter {
            message: "role_id required".into(),
        }
        .to_string());
    }
    let session_ns = conversation_state_role_id(role_id, req.session_id.as_deref());
    state
        .db_manager
        .ensure_role_runtime(session_ns.as_str())
        .await
        .map_err(|e| e.to_frontend_error())?;

    let graph_json = to_json_string(&req.graph)?;
    state
        .expert_models_repo
        .set_expert_models_session_override_json(session_ns.as_str(), Some(graph_json.as_str()))
        .await
        .map_err(|e| e.to_frontend_error())?;

    let style_json = req
        .prompt_style
        .as_ref()
        .map(to_json_string)
        .transpose()?;
    state
        .expert_models_repo
        .set_expert_prompt_style_session_override_json(
            session_ns.as_str(),
            style_json.as_deref(),
        )
        .await
        .map_err(|e| e.to_frontend_error())?;
    Ok(())
}

#[tauri::command]
pub async fn expert_models_clear_session_override(
    req: ExpertModelsClearSessionOverrideRequest,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let role_id = req.role_id.trim();
    if role_id.is_empty() {
        return Err(ApiError::InvalidParameter {
            message: "role_id required".into(),
        }
        .to_string());
    }
    let session_ns = conversation_state_role_id(role_id, req.session_id.as_deref());
    state
        .db_manager
        .ensure_role_runtime(session_ns.as_str())
        .await
        .map_err(|e| e.to_frontend_error())?;
    state
        .expert_models_repo
        .set_expert_models_session_override_json(session_ns.as_str(), None)
        .await
        .map_err(|e| e.to_frontend_error())?;
    state
        .expert_models_repo
        .set_expert_prompt_style_session_override_json(session_ns.as_str(), None)
        .await
        .map_err(|e| e.to_frontend_error())?;
    Ok(())
}

#[tauri::command]
pub async fn expert_models_set_role_default(
    req: ExpertModelsSetRoleDefaultRequest,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let role_id = req.role_id.trim();
    if role_id.is_empty() {
        return Err(ApiError::InvalidParameter {
            message: "role_id required".into(),
        }
        .to_string());
    }
    state
        .db_manager
        .ensure_role_runtime(role_id)
        .await
        .map_err(|e| e.to_frontend_error())?;

    let graph_json = to_json_string(&req.graph)?;
    state
        .expert_models_repo
        .set_expert_models_role_default_json(role_id, Some(graph_json.as_str()))
        .await
        .map_err(|e| e.to_frontend_error())?;

    let style_json = req
        .prompt_style
        .as_ref()
        .map(to_json_string)
        .transpose()?;
    state
        .expert_models_repo
        .set_expert_prompt_style_role_default_json(role_id, style_json.as_deref())
        .await
        .map_err(|e| e.to_frontend_error())?;
    Ok(())
}

#[tauri::command]
pub async fn expert_models_clear_role_default(
    req: ExpertModelsClearRoleDefaultRequest,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let role_id = req.role_id.trim();
    if role_id.is_empty() {
        return Err(ApiError::InvalidParameter {
            message: "role_id required".into(),
        }
        .to_string());
    }
    state
        .db_manager
        .ensure_role_runtime(role_id)
        .await
        .map_err(|e| e.to_frontend_error())?;
    state
        .expert_models_repo
        .set_expert_models_role_default_json(role_id, None)
        .await
        .map_err(|e| e.to_frontend_error())?;
    state
        .expert_models_repo
        .set_expert_prompt_style_role_default_json(role_id, None)
        .await
        .map_err(|e| e.to_frontend_error())?;
    Ok(())
}

fn llama_models_gguf_dir(state: &AppState) -> std::path::PathBuf {
    state.directory_plugins.app_data_dir().join("models").join("gguf")
}

fn llama_loras_dir(state: &AppState) -> std::path::PathBuf {
    state
        .directory_plugins
        .app_data_dir()
        .join("models")
        .join("loras")
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalModelFileDto {
    pub name: String,
    pub path: String,
}

fn list_gguf_files(dir: &std::path::Path) -> Vec<LocalModelFileDto> {
    let mut out: Vec<LocalModelFileDto> = vec![];
    let Ok(rd) = std::fs::read_dir(dir) else {
        return out;
    };
    for ent in rd.flatten() {
        let p = ent.path();
        if !p.is_file() {
            continue;
        }
        let is_gguf = p
            .extension()
            .and_then(|s| s.to_str())
            .map(|s| s.eq_ignore_ascii_case("gguf"))
            .unwrap_or(false);
        if !is_gguf {
            continue;
        }
        let name = p
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();
        out.push(LocalModelFileDto {
            name,
            path: p.to_string_lossy().to_string(),
        });
    }
    out.sort_by(|a, b| a.name.to_ascii_lowercase().cmp(&b.name.to_ascii_lowercase()));
    out
}

#[tauri::command]
pub fn expert_models_list_local_base_models(
    state: State<'_, AppState>,
) -> Result<Vec<LocalModelFileDto>, String> {
    Ok(list_gguf_files(llama_models_gguf_dir(&state).as_path()))
}

#[tauri::command]
pub fn expert_models_list_local_loras(
    state: State<'_, AppState>,
) -> Result<Vec<LocalModelFileDto>, String> {
    let loras = llama_loras_dir(&state);
    let mut out = list_gguf_files(loras.as_path());
    // For M1 compatibility, allow placing LoRAs under models/gguf as well.
    out.extend(list_gguf_files(llama_models_gguf_dir(&state).as_path()));
    out.sort_by(|a, b| a.name.to_ascii_lowercase().cmp(&b.name.to_ascii_lowercase()));
    out.dedup_by(|a, b| a.path == b.path);
    Ok(out)
}

#[tauri::command]
pub async fn expert_models_apply_to_session(
    req: ExpertModelsApplyToSessionRequest,
    state: State<'_, AppState>,
) -> Result<ExpertModelsApplyResult, String> {
    let role_id = req.role_id.trim();
    if role_id.is_empty() {
        return Err(ApiError::InvalidParameter {
            message: "role_id required".into(),
        }
        .to_string());
    }
    let session_ns = conversation_state_role_id(role_id, req.session_id.as_deref());

    // Compute current effective graph (session override > role default > pack default(empty)).
    let (graph, _graph_src, _style, _style_src) =
        effective_for_session(&state, role_id, session_ns.as_str()).await?;

    let gguf_dir = llama_models_gguf_dir(&state);
    let loras_dir = llama_loras_dir(&state);
    let compiled =
        compile_graph_to_llama_local_config(&graph, gguf_dir.as_path(), loras_dir.as_path())
            .map_err(|e| e.to_frontend_error())?;

    let cfg_val = serde_json::to_value(&compiled).map_err(|e| e.to_string())?;
    write_config_json(&state, LLAMA_LOCAL_PLUGIN_ID, &cfg_val)?;

    // Restart trigger: dedicated, not generic rpc:invoke.
    let url = state
        .directory_plugins
        .ensure_rpc_url(LLAMA_LOCAL_PLUGIN_ID)
        .map_err(|e| e.to_string())?;
    let _ = invoke_directory_plugin_rpc_blocking(
        &url,
        "config_updated",
        json!({ "config": cfg_val }),
        RemoteRpcChannel::Plugin,
    );

    Ok(ExpertModelsApplyResult {
        ok: true,
        llama_plugin_id: LLAMA_LOCAL_PLUGIN_ID.to_string(),
        model_path: compiled.model_path.clone(),
        llama_args: compiled.llama_args.clone(),
    })
}

// Re-export config type to avoid unused warnings in some builds.
#[allow(dead_code)]
fn _ensure_types(cfg: LlamaLocalPluginConfig) -> LlamaLocalPluginConfig {
    cfg
}

