//! Ollama 轻量操作：健康检查、列出模型名、删除（与桌面 `api/ollama_models` 行为一致）。

use crate::error::Result;
use crate::infrastructure::ollama_client::OllamaClient;
use serde::Deserialize;

fn client_from_env() -> OllamaClient {
    OllamaClient::new(
        std::env::var("OLLAMA_BASE_URL").unwrap_or_else(|_| "http://localhost:11434".to_string()),
    )
}

pub async fn ollama_models_health() -> Result<bool> {
    client_from_env().health_check().await
}

pub async fn ollama_models_list_names() -> Result<Vec<String>> {
    client_from_env().list_models().await
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OllamaModelsDeleteRequest {
    pub name: String,
}

pub async fn ollama_models_delete(req: &OllamaModelsDeleteRequest) -> Result<()> {
    client_from_env().delete_model(req.name.as_str()).await
}
