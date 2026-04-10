use crate::error::AppError;
use crate::models::dto::TimeStateResponse;
use crate::state::AppState;

/// 时间跳转后批量生成独白（使用已算好的虚拟时间，避免循环依赖 `time` ↔ `monologue`）
pub async fn generate_monologue_lines(
    state: &AppState,
    role_id: &str,
    ts: &TimeStateResponse,
    count: usize,
) -> Result<Vec<String>, String> {
    if count == 0 {
        return Ok(vec![]);
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

    let templates = state.storage.scene_monologue_templates(role_id, &scene);
    let hint = if templates.is_empty() {
        String::new()
    } else {
        format!(
            "\n可参考场景独白模板（可化用语气，不必照抄）：\n{}\n",
            templates.join("\n")
        )
    };

    let prompts: Vec<String> = (0..count.min(3))
        .map(|i| {
            if i == 0 {
                format!(
                    "你是「{}」。当前虚拟时间：{}。当前场景 id：{}{}\
                    \n请用第一人称写一句简短的内心独白（中文，35 字以内），不要加引号或前缀。",
                    role.name, ts.iso_datetime, scene, hint
                )
            } else if i == 1 {
                format!(
                    "你是「{}」。情绪延续上一刻，用另一种口吻再写一句内心独白（中文，35 字以内），不要加引号。",
                    role.name
                )
            } else {
                format!(
                    "你是「{}」。从更细微的感受再写一句独白（中文，30 字以内），不要加引号。",
                    role.name
                )
            }
        })
        .collect();

    let pl = state.resolved_plugins_for(role.as_ref());
    let ollama_model = role.resolve_ollama_model(state.ollama_model.as_str());
    let mut out = Vec::new();
    for (i, p) in prompts.into_iter().enumerate() {
        let text = match pl.llm.generate(ollama_model.as_str(), &p).await {
            Ok(s) => s,
            Err(e) => {
                if !templates.is_empty() {
                    let idx = (ts.virtual_time_ms as usize)
                        .wrapping_add(i)
                        .wrapping_add(templates.len())
                        % templates.len();
                    log::warn!("jump monologue LLM failed, scene template [{}]: {}", idx, e);
                    templates[idx].clone()
                } else {
                    return Err(AppError::OllamaError(e.to_string()).to_frontend_error());
                }
            }
        };
        let t = text.trim().to_string();
        if !t.is_empty() {
            out.push(t);
        }
    }
    Ok(out)
}
