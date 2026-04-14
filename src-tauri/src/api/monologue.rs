use crate::api::time::get_time_state_impl;
use crate::error::AppError;
use crate::models::dto::{GenerateMonologueRequest, GenerateMonologueResponse};
use crate::state::AppState;
use tauri::State;

pub async fn generate_monologue_impl(
    state: &AppState,
    role_id: &str,
) -> Result<GenerateMonologueResponse, String> {
    if !state
        .db_manager
        .role_runtime_exists(role_id)
        .await
        .map_err(|e| e.to_frontend_error())?
    {
        return Err(AppError::InvalidParameter(
            "Role runtime not initialized; call load_role first".to_string(),
        )
        .to_frontend_error());
    }

    let role = state
        .load_role_cached(role_id)
        .map_err(|e| e.to_frontend_error())?;
    let scene = state
        .db_manager
        .get_current_scene(role_id)
        .await
        .map_err(|e| e.to_frontend_error())?
        .unwrap_or_else(|| "default".to_string());
    let ts = get_time_state_impl(state, role_id).await?;

    let templates = state.storage.scene_monologue_templates(role_id, &scene);
    let hint = if templates.is_empty() {
        String::new()
    } else {
        format!(
            "\n可参考场景独白模板（可化用语气，不必照抄）：\n{}\n",
            templates.join("\n")
        )
    };

    let prompt = format!(
        "你是「{}」。当前虚拟时间：{}。当前场景 id：{}{}\
        \n请用第一人称写一句简短的内心独白（中文，35 字以内），不要加引号或前缀。",
        role.name, ts.iso_datetime, scene, hint
    );

    let pl = state.resolved_plugins_for_session(&role, Some(role_id));
    let ollama_model = role.resolve_ollama_model(state.ollama_model.as_str());
    let text = match pl.llm.generate(ollama_model.as_str(), &prompt).await {
        Ok(s) => s,
        Err(e) => {
            if !templates.is_empty() {
                let idx =
                    (ts.virtual_time_ms as usize).wrapping_add(templates.len()) % templates.len();
                log::warn!(
                    "monologue LLM failed, using scene template [{}]: {}",
                    idx,
                    e
                );
                templates[idx].clone()
            } else {
                return Err(AppError::OllamaError(e.to_string()).to_frontend_error());
            }
        }
    };

    Ok(GenerateMonologueResponse {
        text: text.trim().to_string(),
    })
}

#[tauri::command]
pub async fn generate_monologue(
    req: GenerateMonologueRequest,
    state: State<'_, AppState>,
) -> Result<GenerateMonologueResponse, String> {
    generate_monologue_impl(&state, &req.role_id).await
}
