//! Ollama 轻量降级：健康检查、列出模型名、删除。主路径为内置 GGUF + `com.oclive.llama.local`。

use crate::infrastructure::ollama_client::OllamaClient;
use serde::Deserialize;

fn client_from_env() -> OllamaClient {
    OllamaClient::new(
        std::env::var("OLLAMA_BASE_URL")
            .unwrap_or_else(|_| "http://localhost:11434".to_string()),
    )
}

#[tauri::command]
pub async fn ollama_models_health() -> Result<bool, String> {
    let c = client_from_env();
    c.health_check()
        .await
        .map_err(|e| e.to_frontend_error())
}

#[tauri::command]
pub async fn ollama_models_list_names() -> Result<Vec<String>, String> {
    let c = client_from_env();
    c.list_models()
        .await
        .map_err(|e| e.to_frontend_error())
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OllamaModelsDeleteRequest {
    pub name: String,
}

#[tauri::command]
pub async fn ollama_models_delete(req: OllamaModelsDeleteRequest) -> Result<(), String> {
    let c = client_from_env();
    c.delete_model(req.name.as_str())
        .await
        .map_err(|e| e.to_frontend_error())
}
