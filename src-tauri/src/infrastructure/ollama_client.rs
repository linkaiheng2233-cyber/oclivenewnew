use crate::error::{AppError, Result};
use crate::infrastructure::ollama_timeouts;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Ollama 请求体
#[derive(Debug, Serialize)]
pub struct OllamaRequest {
    pub model: String,
    pub prompt: String,
    pub stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
}

/// Ollama 响应体
#[derive(Debug, Deserialize)]
pub struct OllamaResponse {
    pub response: String,
    pub model: String,
    pub created_at: String,
    pub done: bool,
}

/// Ollama 客户端
pub struct OllamaClient {
    base_url: String,
    client: Client,
    timeout: Duration,
}

fn normalize_base_url(url: String) -> String {
    url.trim_end_matches('/').to_string()
}

impl OllamaClient {
    /// 创建新的 Ollama 客户端
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: normalize_base_url(base_url.into()),
            client: Client::new(),
            timeout: ollama_timeouts::http_client_timeout(),
        }
    }

    /// 设置超时时间
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// 检查 Ollama 服务是否可用
    pub async fn health_check(&self) -> Result<bool> {
        let url = format!("{}/api/tags", self.base_url);

        match self.client.get(&url).timeout(self.timeout).send().await {
            Ok(response) => Ok(response.status().is_success()),
            Err(_) => Ok(false),
        }
    }

    /// 获取可用的模型列表
    pub async fn list_models(&self) -> Result<Vec<String>> {
        let url = format!("{}/api/tags", self.base_url);

        #[derive(Deserialize)]
        struct TagsResponse {
            models: Option<Vec<ModelInfo>>,
        }

        #[derive(Deserialize)]
        struct ModelInfo {
            name: String,
        }

        let response = self
            .client
            .get(&url)
            .timeout(self.timeout)
            .send()
            .await
            .map_err(|e| AppError::OllamaError(format!("Failed to list models: {}", e)))?;

        let tags: TagsResponse = response
            .json()
            .await
            .map_err(|e| AppError::OllamaError(format!("Failed to parse models: {}", e)))?;

        let models = tags
            .models
            .unwrap_or_default()
            .into_iter()
            .map(|m| m.name)
            .collect();

        Ok(models)
    }

    /// 调用 Ollama 生成回复
    pub async fn generate(
        &self,
        model: &str,
        prompt: &str,
        temperature: Option<f32>,
        top_p: Option<f32>,
    ) -> Result<String> {
        let url = format!("{}/api/generate", self.base_url);

        let request = OllamaRequest {
            model: model.to_string(),
            prompt: prompt.to_string(),
            stream: false,
            temperature,
            top_p,
        };

        let response = self
            .client
            .post(&url)
            .json(&request)
            .timeout(self.timeout)
            .send()
            .await
            .map_err(|e| AppError::OllamaError(format!("Request failed: {}", e)))?;

        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|e| AppError::OllamaError(format!("Failed to read response body: {}", e)))?;

        if !status.is_success() {
            // 404 多为模型不存在或 URL 错误；body 里常有 {"error":"..."}
            return Err(AppError::OllamaError(format!(
                "HTTP {} — {} (请求: POST {}/api/generate, model={})",
                status,
                body.chars().take(800).collect::<String>(),
                self.base_url,
                model
            )));
        }

        let ollama_response: OllamaResponse = serde_json::from_str(&body).map_err(|e| {
            AppError::OllamaError(format!(
                "Failed to parse response: {} — body: {}",
                e,
                body.chars().take(400).collect::<String>()
            ))
        })?;

        Ok(ollama_response.response)
    }

    /// 调用 Ollama 生成回复（带流式处理）
    pub async fn generate_stream(
        &self,
        model: &str,
        prompt: &str,
        temperature: Option<f32>,
        top_p: Option<f32>,
    ) -> Result<String> {
        let url = format!("{}/api/generate", self.base_url);

        let request = OllamaRequest {
            model: model.to_string(),
            prompt: prompt.to_string(),
            stream: true,
            temperature,
            top_p,
        };

        let response = self
            .client
            .post(&url)
            .json(&request)
            .timeout(self.timeout)
            .send()
            .await
            .map_err(|e| AppError::OllamaError(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(AppError::OllamaError(format!(
                "Ollama returned status: {}",
                response.status()
            )));
        }

        let text = response
            .text()
            .await
            .map_err(|e| AppError::OllamaError(format!("Failed to read response: {}", e)))?;

        // 解析流式响应，合并所有 response 字段
        let mut full_response = String::new();
        for line in text.lines() {
            if let Ok(json) = serde_json::from_str::<OllamaResponse>(line) {
                full_response.push_str(&json.response);
            }
        }

        Ok(full_response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ollama_client_new() {
        let client = OllamaClient::new("http://localhost:11434");
        assert_eq!(client.base_url, "http://localhost:11434");
    }

    #[test]
    fn test_ollama_client_with_timeout() {
        let client =
            OllamaClient::new("http://localhost:11434").with_timeout(Duration::from_secs(60));
        assert_eq!(client.timeout, Duration::from_secs(60));
    }

    #[test]
    fn test_ollama_request_serialization() {
        let request = OllamaRequest {
            model: "llama2".to_string(),
            prompt: "Hello".to_string(),
            stream: false,
            temperature: Some(0.7),
            top_p: None,
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"model\":\"llama2\""));
        assert!(json.contains("\"prompt\":\"Hello\""));
        assert!(json.contains("\"temperature\":0.7"));
        assert!(!json.contains("\"top_p\"")); // 应该被跳过
    }

    #[test]
    fn test_ollama_response_deserialization() {
        let json = r#"{
            "response": "Hello there!",
            "model": "llama2",
            "created_at": "2024-01-01T00:00:00Z",
            "done": true
        }"#;

        let response: OllamaResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.response, "Hello there!");
        assert_eq!(response.model, "llama2");
        assert!(response.done);
    }

    #[tokio::test]
    async fn test_health_check_offline() {
        let client = OllamaClient::new("http://localhost:9999"); // 不存在的端口
        let result = client.health_check().await;
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }
}
