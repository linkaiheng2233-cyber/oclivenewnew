//! Ollama 轻量操作：健康检查、列出模型名、删除（与桌面 `api/ollama_models` 行为一致）。
//!
//! `default-llm-providers` 关闭时本模块 API 仍保留，但返回明确错误（未编译内置 Ollama 客户端）。

#[cfg(not(feature = "default-llm-providers"))]
use crate::error::AppError;
use crate::error::Result;
use serde::Deserialize;

#[cfg(feature = "default-llm-providers")]
use crate::infrastructure::ollama_client::OllamaClient;

#[cfg(feature = "default-llm-providers")]
fn client_from_env() -> OllamaClient {
    OllamaClient::new(
        std::env::var("OLLAMA_BASE_URL").unwrap_or_else(|_| "http://localhost:11434".to_string()),
    )
}

#[cfg(not(feature = "default-llm-providers"))]
fn disabled_err<T>() -> Result<T> {
    Err(AppError::InvalidParameter(
        "built-in Ollama client disabled (compile without default-llm-providers)".into(),
    ))
}

pub async fn ollama_models_health() -> Result<bool> {
    #[cfg(not(feature = "default-llm-providers"))]
    {
        return disabled_err();
    }
    #[cfg(feature = "default-llm-providers")]
    {
        client_from_env().health_check().await
    }
}

pub async fn ollama_models_list_names() -> Result<Vec<String>> {
    #[cfg(not(feature = "default-llm-providers"))]
    {
        return disabled_err();
    }
    #[cfg(feature = "default-llm-providers")]
    {
        client_from_env().list_models().await
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OllamaModelsDeleteRequest {
    pub name: String,
}

pub async fn ollama_models_delete(_req: &OllamaModelsDeleteRequest) -> Result<()> {
    #[cfg(not(feature = "default-llm-providers"))]
    {
        return disabled_err();
    }
    #[cfg(feature = "default-llm-providers")]
    {
        client_from_env().delete_model(_req.name.as_str()).await
    }
}
