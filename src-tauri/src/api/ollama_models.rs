//! Ollama 轻量降级：健康检查、列出模型名、删除。实现于 `oclive_kernel_runtime::domain::ollama_host_commands`。

pub use oclive_kernel_runtime::domain::ollama_host_commands::OllamaModelsDeleteRequest;

#[tauri::command]
pub async fn ollama_models_health() -> Result<bool, String> {
    oclive_kernel_runtime::domain::ollama_host_commands::ollama_models_health()
        .await
        .map_err(|e| e.to_frontend_error())
}

#[tauri::command]
pub async fn ollama_models_list_names() -> Result<Vec<String>, String> {
    oclive_kernel_runtime::domain::ollama_host_commands::ollama_models_list_names()
        .await
        .map_err(|e| e.to_frontend_error())
}

#[tauri::command]
pub async fn ollama_models_delete(req: OllamaModelsDeleteRequest) -> Result<(), String> {
    oclive_kernel_runtime::domain::ollama_host_commands::ollama_models_delete(&req)
        .await
        .map_err(|e| e.to_frontend_error())
}
