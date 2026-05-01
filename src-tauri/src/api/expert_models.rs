//! Module 9: Expert Models API (Tauri commands).

use crate::api::error::ApiError;
use crate::api::role::set_session_plugin_backend_impl;
use crate::domain::chat_engine::conversation_state_role_id;
use crate::domain::expert_models::{compile_graph_to_llama_local_config, LLAMA_LOCAL_PLUGIN_ID};
use crate::infrastructure::plugin_data::write_config_json;
use crate::infrastructure::remote_plugin::{
    invoke_directory_plugin_rpc_blocking, RemoteRpcChannel,
};
use crate::models::dto::SetSessionPluginBackendRequest;
use crate::models::dto::{
    ExpertModelsApplyResult, ExpertModelsApplyToSessionRequest,
    ExpertModelsClearRoleDefaultRequest, ExpertModelsClearRunsRequest,
    ExpertModelsClearSessionOverrideRequest, ExpertModelsEffectiveResponse,
    ExpertModelsGetEffectiveRequest, ExpertModelsGetRunDetailRequest,
    ExpertModelsGetRunDetailResponse, ExpertModelsListRunsResponse,
    ExpertModelsRollbackToRunRequest, ExpertModelsRunDetailDto, ExpertModelsRunSummaryDto,
    ExpertModelsSetRoleDefaultRequest, ExpertModelsSetRunPinnedRequest,
    ExpertModelsSetSessionOverrideRequest, ExpertWorkflowDto, ExpertWorkflowSummaryDto,
    ExpertWorkflowsDeleteRequest, ExpertWorkflowsGetRequest, ExpertWorkflowsListResponse,
    ExpertWorkflowsSaveRequest,
};
use crate::models::{ExpertConfigSource, ExpertGraph, LlamaLocalPluginConfig, PromptStyleOverride};
use crate::state::AppState;
use chrono::Utc;
use serde::Serialize;
use serde_json::json;
use tauri::State;
use uuid::Uuid;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct RunTargetSummary {
    #[serde(default)]
    base_name: String,
    #[serde(default)]
    lora_count: u32,
    #[serde(default)]
    has_prompt_style: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct RunApplyOutcome {
    #[serde(default)]
    ok: bool,
    #[serde(default)]
    error: Option<String>,
    #[serde(default)]
    model_path: Option<String>,
    #[serde(default)]
    llama_args: Option<String>,
    #[serde(default)]
    duration_ms: Option<i64>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct ExpertModelsRunEntry {
    #[serde(default)]
    at_ms: i64,
    #[serde(default)]
    pinned: bool,
    /// Rollback snapshot: previous effective before apply.
    graph: ExpertGraph,
    #[serde(default)]
    prompt_style: Option<PromptStyleOverride>,
    /// Target summary (what we tried to apply).
    #[serde(default)]
    target: Option<RunTargetSummary>,
    /// Target config for retry/detail (optional for backward compatibility).
    #[serde(default)]
    target_graph: Option<ExpertGraph>,
    #[serde(default)]
    target_prompt_style: Option<PromptStyleOverride>,
    /// Apply outcome (optional for backward compatibility).
    #[serde(default)]
    apply: Option<RunApplyOutcome>,
}

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
        .map_err(|e| {
            ApiError::InvalidParameter {
                message: format!("invalid expert graph json: {}", e),
            }
            .to_string()
        })
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
        .map_err(|e| {
            ApiError::InvalidParameter {
                message: format!("invalid prompt style json: {}", e),
            }
            .to_string()
        })
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
) -> Result<
    (
        ExpertGraph,
        ExpertConfigSource,
        Option<PromptStyleOverride>,
        ExpertConfigSource,
    ),
    String,
> {
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

fn parse_run_entries(raw: Option<String>) -> Vec<ExpertModelsRunEntry> {
    let Some(s) = raw else { return vec![] };
    let t = s.trim();
    if t.is_empty() {
        return vec![];
    }
    // Prefer typed parse.
    if let Ok(list) = serde_json::from_str::<Vec<ExpertModelsRunEntry>>(t) {
        return list;
    }
    // Backwards compatibility: old history stored as Vec<Value>.
    let vals = serde_json::from_str::<Vec<serde_json::Value>>(t).unwrap_or_default();
    vals.into_iter()
        .filter_map(|v| serde_json::from_value::<ExpertModelsRunEntry>(v).ok())
        .collect()
}

fn to_run_entries_json(items: &[ExpertModelsRunEntry]) -> Option<String> {
    if items.is_empty() {
        return None;
    }
    serde_json::to_string(items).ok()
}

fn graph_summary(graph: &ExpertGraph, style: &Option<PromptStyleOverride>) -> (String, u32, bool) {
    (
        pick_base_name(graph),
        count_enabled_loras(graph),
        style.is_some(),
    )
}

fn idx_from_latest_to_pos(len: usize, idx_from_latest: u32) -> Option<usize> {
    let idx = idx_from_latest as usize;
    len.checked_sub(1)?.checked_sub(idx)
}

fn trim_runs_keep_pinned(
    mut runs: Vec<ExpertModelsRunEntry>,
    max_len: usize,
) -> Vec<ExpertModelsRunEntry> {
    if runs.len() <= max_len {
        return runs;
    }
    let mut over = runs.len().saturating_sub(max_len);
    // Drop oldest non-pinned first.
    while over > 0 {
        if let Some(idx) = runs.iter().position(|r| !r.pinned) {
            runs.remove(idx);
            over -= 1;
            continue;
        }
        break;
    }
    // If still over, drop oldest regardless.
    if runs.len() > max_len {
        let extra = runs.len() - max_len;
        runs.drain(0..extra);
    }
    runs
}

fn base_file_name(p: &str) -> String {
    let t = p.trim();
    if t.is_empty() {
        return "".to_string();
    }
    t.rsplit(['\\', '/']).next().unwrap_or(t).to_string()
}

fn count_enabled_loras(graph: &ExpertGraph) -> u32 {
    graph
        .nodes
        .iter()
        .filter_map(|n| match n {
            crate::models::expert_models::ExpertNode::LoraAdapter { enabled, .. } => Some(*enabled),
            _ => None,
        })
        .filter(|x| *x)
        .count() as u32
}

fn pick_base_name(graph: &ExpertGraph) -> String {
    for n in &graph.nodes {
        if let crate::models::expert_models::ExpertNode::BaseModel { gguf_path, .. } = n {
            return base_file_name(gguf_path);
        }
    }
    "".to_string()
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
    let run_raw = state
        .expert_models_repo
        .get_expert_models_run_history_json(session_ns.as_str())
        .await
        .map_err(|e| e.to_frontend_error())?;
    let can_rollback_last_run = !parse_run_entries(run_raw).is_empty();
    Ok(ExpertModelsEffectiveResponse {
        graph,
        prompt_style,
        graph_source,
        prompt_style_source,
        can_rollback_last_run,
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

    let style_json = req.prompt_style.as_ref().map(to_json_string).transpose()?;
    state
        .expert_models_repo
        .set_expert_prompt_style_session_override_json(session_ns.as_str(), style_json.as_deref())
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

    let style_json = req.prompt_style.as_ref().map(to_json_string).transpose()?;
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
    state
        .directory_plugins
        .app_data_dir()
        .join("models")
        .join("gguf")
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
    out.sort_by(|a, b| {
        a.name
            .to_ascii_lowercase()
            .cmp(&b.name.to_ascii_lowercase())
    });
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
    out.sort_by(|a, b| {
        a.name
            .to_ascii_lowercase()
            .cmp(&b.name.to_ascii_lowercase())
    });
    out.dedup_by(|a, b| a.path == b.path);
    Ok(out)
}

fn ensure_dir(p: &std::path::Path) -> Result<(), String> {
    std::fs::create_dir_all(p).map_err(|e| e.to_string())
}

fn sanitize_file_name(name: &str) -> String {
    // Keep it simple and safe: no path separators.
    name.replace(['\\', '/', ':'], "_")
}

fn unique_dest_path(dir: &std::path::Path, file_name: &str) -> std::path::PathBuf {
    let base = sanitize_file_name(file_name);
    let stem = std::path::Path::new(&base)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("model")
        .to_string();
    let ext = std::path::Path::new(&base)
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("gguf")
        .to_string();
    let mut cand = dir.join(format!("{}.{}", stem, ext));
    if !cand.exists() {
        return cand;
    }
    for i in 2..=999 {
        cand = dir.join(format!("{}_{}.{}", stem, i, ext));
        if !cand.exists() {
            return cand;
        }
    }
    dir.join(format!("{}_{}_copy.{}", stem, Uuid::new_v4(), ext))
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExpertModelsImportGgufRequest {
    /// Absolute path selected via dialog on the user machine.
    pub source_path: String,
}

fn import_gguf_into_dir(
    dir: &std::path::Path,
    source_path: &str,
) -> Result<LocalModelFileDto, String> {
    let src = std::path::PathBuf::from(source_path.trim());
    if !src.is_file() {
        return Err(ApiError::InvalidParameter {
            message: "source_path must be a file".into(),
        }
        .to_string());
    }
    let is_gguf = src
        .extension()
        .and_then(|s| s.to_str())
        .map(|s| s.eq_ignore_ascii_case("gguf"))
        .unwrap_or(false);
    if !is_gguf {
        return Err(ApiError::InvalidParameter {
            message: "only .gguf files are supported".into(),
        }
        .to_string());
    }
    ensure_dir(dir)?;
    let file_name = src
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("model.gguf");
    let dest = unique_dest_path(dir, file_name);
    std::fs::copy(&src, &dest).map_err(|e| e.to_string())?;
    Ok(LocalModelFileDto {
        name: dest
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string(),
        path: dest.to_string_lossy().to_string(),
    })
}

#[tauri::command]
pub fn expert_models_import_base_gguf(
    req: ExpertModelsImportGgufRequest,
    state: State<'_, AppState>,
) -> Result<LocalModelFileDto, String> {
    let dir = llama_models_gguf_dir(&state);
    import_gguf_into_dir(dir.as_path(), req.source_path.as_str())
}

#[tauri::command]
pub fn expert_models_import_lora_gguf(
    req: ExpertModelsImportGgufRequest,
    state: State<'_, AppState>,
) -> Result<LocalModelFileDto, String> {
    let dir = llama_loras_dir(&state);
    import_gguf_into_dir(dir.as_path(), req.source_path.as_str())
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

    // Push a rollback snapshot (previous effective) before applying, and record the apply outcome.
    // This provides a "Module 9 Ctrl+Z" at the session scope, and keeps a lightweight "queue-like" log.
    let (prev_graph, _pgs, prev_style, _pss) =
        effective_for_session(&state, role_id, session_ns.as_str()).await?;
    // Current effective graph (session override > role default > pack default(empty)) is the target we are applying.
    let (graph, _graph_src, style, _style_src) =
        effective_for_session(&state, role_id, session_ns.as_str()).await?;

    let run_raw_prev = state
        .expert_models_repo
        .get_expert_models_run_history_json(session_ns.as_str())
        .await
        .map_err(|e| e.to_frontend_error())?;
    let mut runs = parse_run_entries(run_raw_prev);
    let snapshot_idx = runs.len();
    runs.push(ExpertModelsRunEntry {
        at_ms: Utc::now().timestamp_millis(),
        pinned: false,
        graph: prev_graph,
        prompt_style: prev_style,
        target: Some(RunTargetSummary {
            base_name: pick_base_name(&graph),
            lora_count: count_enabled_loras(&graph),
            has_prompt_style: style.is_some(),
        }),
        target_graph: Some(graph.clone()),
        target_prompt_style: style.clone(),
        apply: None,
    });
    // keep last 30 (prefer keeping pinned)
    runs = trim_runs_keep_pinned(runs, 30);

    let gguf_dir = llama_models_gguf_dir(&state);
    let loras_dir = llama_loras_dir(&state);
    let started = std::time::Instant::now();
    let apply_result: Result<ExpertModelsApplyResult, String> = (async {
        let compiled =
            compile_graph_to_llama_local_config(&graph, gguf_dir.as_path(), loras_dir.as_path())
                .map_err(|e| e.to_frontend_error())?;

        let cfg_val = serde_json::to_value(&compiled).map_err(|e| e.to_string())?;
        write_config_json(&state, LLAMA_LOCAL_PLUGIN_ID, &cfg_val)?;

        // Ensure the current session uses the directory LLM backend (mechanism), pointing to llama local plugin.
        let _ = set_session_plugin_backend_impl(
            &state,
            &SetSessionPluginBackendRequest {
                role_id: role_id.to_string(),
                session_id: req.session_id.clone(),
                module: "llm".to_string(),
                backend: Some(Some("directory".to_string())),
                local_memory_provider_id: None,
                directory_plugin_id: Some(LLAMA_LOCAL_PLUGIN_ID.to_string()),
            },
        )
        .await;

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
    })
    .await;
    let duration_ms: i64 = started.elapsed().as_millis() as i64;

    // Persist apply outcome back onto the entry (best-effort).
    if let Some(v) = runs.get_mut(snapshot_idx) {
        v.apply = Some(match &apply_result {
            Ok(r) => RunApplyOutcome {
                ok: true,
                error: None,
                model_path: r.model_path.clone(),
                llama_args: r.llama_args.clone(),
                duration_ms: Some(duration_ms),
            },
            Err(e) => RunApplyOutcome {
                ok: false,
                error: Some(e.clone()),
                model_path: None,
                llama_args: None,
                duration_ms: Some(duration_ms),
            },
        });
    }
    let run_json = to_run_entries_json(runs.as_slice());
    let _ = state
        .expert_models_repo
        .set_expert_models_run_history_json(session_ns.as_str(), run_json.as_deref())
        .await;

    apply_result
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExpertModelsRollbackLastRunRequest {
    pub role_id: String,
    #[serde(default)]
    pub session_id: Option<String>,
}

#[tauri::command]
pub async fn expert_models_rollback_last_run(
    req: ExpertModelsRollbackLastRunRequest,
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
    state
        .db_manager
        .ensure_role_runtime(session_ns.as_str())
        .await
        .map_err(|e| e.to_frontend_error())?;
    let raw = state
        .expert_models_repo
        .get_expert_models_run_history_json(session_ns.as_str())
        .await
        .map_err(|e| e.to_frontend_error())?;
    let mut runs = parse_run_entries(raw);
    let last = runs.pop().ok_or_else(|| {
        ApiError::InvalidParameter {
            message: "no previous run to rollback".into(),
        }
        .to_string()
    })?;
    let graph = last.graph;
    let style = last.prompt_style;

    // Set session override to the snapshot.
    let graph_json = to_json_string(&graph)?;
    state
        .expert_models_repo
        .set_expert_models_session_override_json(session_ns.as_str(), Some(graph_json.as_str()))
        .await
        .map_err(|e| e.to_frontend_error())?;
    let style_json = style.as_ref().map(to_json_string).transpose()?;
    state
        .expert_models_repo
        .set_expert_prompt_style_session_override_json(session_ns.as_str(), style_json.as_deref())
        .await
        .map_err(|e| e.to_frontend_error())?;

    // Persist trimmed history.
    let run_json = to_run_entries_json(runs.as_slice());
    state
        .expert_models_repo
        .set_expert_models_run_history_json(session_ns.as_str(), run_json.as_deref())
        .await
        .map_err(|e| e.to_frontend_error())?;

    // Apply (compile → config → restart) using existing path.
    expert_models_apply_to_session(
        ExpertModelsApplyToSessionRequest {
            role_id: role_id.to_string(),
            session_id: req.session_id.clone(),
        },
        state,
    )
    .await
}

#[tauri::command]
pub async fn expert_models_list_runs(
    req: ExpertModelsGetEffectiveRequest,
    state: State<'_, AppState>,
) -> Result<ExpertModelsListRunsResponse, String> {
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
    let raw = state
        .expert_models_repo
        .get_expert_models_run_history_json(session_ns.as_str())
        .await
        .map_err(|e| e.to_frontend_error())?;
    let runs = parse_run_entries(raw);
    // Stored in chronological order; present latest first.
    let mut items: Vec<ExpertModelsRunSummaryDto> = vec![];
    for (i, e) in runs.iter().rev().enumerate() {
        let (t_base, t_loras, t_ps) = e
            .target
            .as_ref()
            .map(|t| (t.base_name.clone(), t.lora_count, t.has_prompt_style))
            .or_else(|| {
                e.target_graph.as_ref().map(|g| {
                    let s = &e.target_prompt_style;
                    (pick_base_name(g), count_enabled_loras(g), s.is_some())
                })
            })
            .unwrap_or_else(|| {
                let s = &e.prompt_style;
                (
                    pick_base_name(&e.graph),
                    count_enabled_loras(&e.graph),
                    s.is_some(),
                )
            });
        let (apply_ok, apply_error, apply_duration_ms) = e
            .apply
            .as_ref()
            .map(|a| (Some(a.ok), a.error.clone(), a.duration_ms))
            .unwrap_or((None, None, None));
        items.push(ExpertModelsRunSummaryDto {
            index_from_latest: i as u32,
            at_ms: e.at_ms,
            pinned: e.pinned,
            target_base_name: t_base,
            target_lora_count: t_loras,
            target_has_prompt_style: t_ps,
            apply_ok,
            apply_error,
            apply_duration_ms,
        });
        if items.len() >= 30 {
            break;
        }
    }
    Ok(ExpertModelsListRunsResponse { items })
}

#[tauri::command]
pub async fn expert_models_get_run_detail(
    req: ExpertModelsGetRunDetailRequest,
    state: State<'_, AppState>,
) -> Result<ExpertModelsGetRunDetailResponse, String> {
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
    let raw = state
        .expert_models_repo
        .get_expert_models_run_history_json(session_ns.as_str())
        .await
        .map_err(|e| e.to_frontend_error())?;
    let runs = parse_run_entries(raw);
    let pos = idx_from_latest_to_pos(runs.len(), req.index_from_latest).ok_or_else(|| {
        ApiError::InvalidParameter {
            message: "index out of range".into(),
        }
        .to_string()
    })?;
    let e = runs
        .get(pos)
        .cloned()
        .ok_or_else(|| "run missing".to_string())?;
    let snapshot_graph = e.graph.clone();
    let snapshot_style = e.prompt_style.clone();
    let (snapshot_base, snapshot_loras, snapshot_ps) =
        graph_summary(&snapshot_graph, &snapshot_style);

    let target_graph = e.target_graph.clone();
    let target_style = e.target_prompt_style.clone();
    let (t_base, t_loras, t_ps) = e
        .target
        .as_ref()
        .map(|t| (t.base_name.clone(), t.lora_count, t.has_prompt_style))
        .or_else(|| {
            target_graph
                .as_ref()
                .map(|g| graph_summary(g, &target_style))
        })
        .unwrap_or_else(|| graph_summary(&snapshot_graph, &snapshot_style));
    let (apply_ok, apply_error, apply_model_path, apply_llama_args, apply_duration_ms) = e
        .apply
        .as_ref()
        .map(|a| {
            (
                Some(a.ok),
                a.error.clone(),
                a.model_path.clone(),
                a.llama_args.clone(),
                a.duration_ms,
            )
        })
        .unwrap_or((None, None, None, None, None));

    Ok(ExpertModelsGetRunDetailResponse {
        item: ExpertModelsRunDetailDto {
            index_from_latest: req.index_from_latest,
            at_ms: e.at_ms,
            pinned: e.pinned,
            snapshot_graph,
            snapshot_prompt_style: snapshot_style,
            snapshot_base_name: snapshot_base,
            snapshot_lora_count: snapshot_loras,
            snapshot_has_prompt_style: snapshot_ps,
            target_graph,
            target_prompt_style: target_style,
            target_base_name: t_base,
            target_lora_count: t_loras,
            target_has_prompt_style: t_ps,
            apply_ok,
            apply_error,
            apply_model_path,
            apply_llama_args,
            apply_duration_ms,
        },
    })
}

#[tauri::command]
pub async fn expert_models_clear_runs(
    req: ExpertModelsClearRunsRequest,
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
    let raw = state
        .expert_models_repo
        .get_expert_models_run_history_json(session_ns.as_str())
        .await
        .map_err(|e| e.to_frontend_error())?;
    let mut runs = parse_run_entries(raw);
    let mode = req.mode.as_deref().unwrap_or("all");
    let keep_pinned = req.keep_pinned.unwrap_or(mode != "all");
    runs = runs
        .into_iter()
        .filter(|r| {
            if keep_pinned && r.pinned {
                return true;
            }
            let ok = r.apply.as_ref().map(|a| a.ok);
            let should_remove = match mode {
                "ok" => ok == Some(true),
                "failed" => ok == Some(false),
                "unpinned" => !r.pinned,
                "all" | _ => true,
            };
            !should_remove
        })
        .collect();
    let out = to_run_entries_json(runs.as_slice());
    state
        .expert_models_repo
        .set_expert_models_run_history_json(session_ns.as_str(), out.as_deref())
        .await
        .map_err(|e| e.to_frontend_error())?;
    Ok(())
}

#[tauri::command]
pub async fn expert_models_set_run_pinned(
    req: ExpertModelsSetRunPinnedRequest,
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
    let raw = state
        .expert_models_repo
        .get_expert_models_run_history_json(session_ns.as_str())
        .await
        .map_err(|e| e.to_frontend_error())?;
    let mut runs = parse_run_entries(raw);
    let pos = idx_from_latest_to_pos(runs.len(), req.index_from_latest).ok_or_else(|| {
        ApiError::InvalidParameter {
            message: "index out of range".into(),
        }
        .to_string()
    })?;
    if let Some(e) = runs.get_mut(pos) {
        e.pinned = req.pinned;
    }
    let out = to_run_entries_json(runs.as_slice());
    state
        .expert_models_repo
        .set_expert_models_run_history_json(session_ns.as_str(), out.as_deref())
        .await
        .map_err(|e| e.to_frontend_error())?;
    Ok(())
}

#[tauri::command]
pub async fn expert_models_rollback_to_run(
    req: ExpertModelsRollbackToRunRequest,
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
    state
        .db_manager
        .ensure_role_runtime(session_ns.as_str())
        .await
        .map_err(|e| e.to_frontend_error())?;
    let raw = state
        .expert_models_repo
        .get_expert_models_run_history_json(session_ns.as_str())
        .await
        .map_err(|e| e.to_frontend_error())?;
    let mut runs = parse_run_entries(raw);
    if runs.is_empty() {
        return Err(ApiError::InvalidParameter {
            message: "no runs".into(),
        }
        .to_string());
    }
    let target_pos_from_start = idx_from_latest_to_pos(runs.len(), req.index_from_latest)
        .ok_or_else(|| {
            ApiError::InvalidParameter {
                message: "index out of range".into(),
            }
            .to_string()
        })?;
    let target = runs
        .get(target_pos_from_start)
        .cloned()
        .ok_or_else(|| "target run missing".to_string())?;
    // Trim newer runs (keep up to the ones older than target).
    runs.truncate(target_pos_from_start);

    let graph = target.graph;
    let style = target.prompt_style;

    let graph_json = to_json_string(&graph)?;
    state
        .expert_models_repo
        .set_expert_models_session_override_json(session_ns.as_str(), Some(graph_json.as_str()))
        .await
        .map_err(|e| e.to_frontend_error())?;
    let style_json = style.as_ref().map(to_json_string).transpose()?;
    state
        .expert_models_repo
        .set_expert_prompt_style_session_override_json(session_ns.as_str(), style_json.as_deref())
        .await
        .map_err(|e| e.to_frontend_error())?;

    let run_json = to_run_entries_json(runs.as_slice());
    state
        .expert_models_repo
        .set_expert_models_run_history_json(session_ns.as_str(), run_json.as_deref())
        .await
        .map_err(|e| e.to_frontend_error())?;

    expert_models_apply_to_session(
        ExpertModelsApplyToSessionRequest {
            role_id: role_id.to_string(),
            session_id: req.session_id.clone(),
        },
        state,
    )
    .await
}

// ===== Module 9: Expert Workflows (global preset library in app_settings) =====

const EXPERT_WORKFLOWS_APP_SETTING_KEY: &str = "expert_models_workflows_v1";

fn parse_workflow_list(raw: Option<String>) -> Result<Vec<ExpertWorkflowDto>, String> {
    let Some(s) = raw else {
        return Ok(vec![]);
    };
    let t = s.trim();
    if t.is_empty() {
        return Ok(vec![]);
    }
    serde_json::from_str::<Vec<ExpertWorkflowDto>>(t).map_err(|e| {
        ApiError::InvalidParameter {
            message: format!("invalid workflows json: {}", e),
        }
        .to_string()
    })
}

async fn load_workflows(state: &AppState) -> Result<Vec<ExpertWorkflowDto>, String> {
    let raw = state
        .db_manager
        .get_app_setting(EXPERT_WORKFLOWS_APP_SETTING_KEY)
        .await
        .map_err(|e| e.to_frontend_error())?;
    parse_workflow_list(raw)
}

async fn store_workflows(state: &AppState, list: &[ExpertWorkflowDto]) -> Result<(), String> {
    let raw = serde_json::to_string(list).map_err(|e| e.to_string())?;
    state
        .db_manager
        .upsert_app_setting(EXPERT_WORKFLOWS_APP_SETTING_KEY, raw.as_str())
        .await
        .map_err(|e| e.to_frontend_error())?;
    Ok(())
}

#[tauri::command]
pub async fn expert_workflows_list(
    state: State<'_, AppState>,
) -> Result<ExpertWorkflowsListResponse, String> {
    let list = load_workflows(&state).await?;
    let mut items: Vec<ExpertWorkflowSummaryDto> = list
        .into_iter()
        .map(|w| ExpertWorkflowSummaryDto {
            id: w.id,
            name: w.name,
            updated_at_ms: w.updated_at_ms,
        })
        .collect();
    items.sort_by(|a, b| {
        b.updated_at_ms
            .cmp(&a.updated_at_ms)
            .then(a.name.cmp(&b.name))
    });
    Ok(ExpertWorkflowsListResponse { items })
}

#[tauri::command]
pub async fn expert_workflows_get(
    req: ExpertWorkflowsGetRequest,
    state: State<'_, AppState>,
) -> Result<ExpertWorkflowDto, String> {
    let id = req.id.trim();
    if id.is_empty() {
        return Err(ApiError::InvalidParameter {
            message: "id required".into(),
        }
        .to_string());
    }
    let list = load_workflows(&state).await?;
    let found = list.into_iter().find(|w| w.id == id);
    found.ok_or_else(|| {
        ApiError::InvalidParameter {
            message: format!("workflow not found: {}", id),
        }
        .to_string()
    })
}

#[tauri::command]
pub async fn expert_workflows_save(
    req: ExpertWorkflowsSaveRequest,
    state: State<'_, AppState>,
) -> Result<ExpertWorkflowDto, String> {
    let name = req.name.trim();
    if name.is_empty() {
        return Err(ApiError::InvalidParameter {
            message: "name required".into(),
        }
        .to_string());
    }
    let mut list = load_workflows(&state).await?;
    let now_ms = Utc::now().timestamp_millis();
    let id = req
        .id
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .unwrap_or_else(|| Uuid::new_v4().to_string());
    let dto = ExpertWorkflowDto {
        id: id.clone(),
        name: name.to_string(),
        updated_at_ms: now_ms,
        graph: req.graph,
        prompt_style: req.prompt_style,
    };
    // upsert by id
    if let Some(pos) = list.iter().position(|w| w.id == id) {
        list[pos] = dto.clone();
    } else {
        list.push(dto.clone());
    }
    store_workflows(&state, list.as_slice()).await?;
    Ok(dto)
}

#[tauri::command]
pub async fn expert_workflows_delete(
    req: ExpertWorkflowsDeleteRequest,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let id = req.id.trim();
    if id.is_empty() {
        return Err(ApiError::InvalidParameter {
            message: "id required".into(),
        }
        .to_string());
    }
    let mut list = load_workflows(&state).await?;
    let before = list.len();
    list.retain(|w| w.id != id);
    if list.len() == before {
        return Ok(());
    }
    store_workflows(&state, list.as_slice()).await?;
    Ok(())
}

// Re-export config type to avoid unused warnings in some builds.
#[allow(dead_code)]
fn _ensure_types(cfg: LlamaLocalPluginConfig) -> LlamaLocalPluginConfig {
    cfg
}
