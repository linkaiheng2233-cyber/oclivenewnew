use crate::error::{AppError, Result};
use crate::infrastructure::ollama_timeouts;
use reqwest::Client;
use std::sync::Arc;
use std::sync::LazyLock;

static OLLAMA_HTTP_CLIENT: LazyLock<Client> = LazyLock::new(|| {
    Client::builder()
        .pool_max_idle_per_host(4)
        .build()
        .expect("ollama reqwest client")
});
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Ollama request body.
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

/// Ollama response body.
#[derive(Debug, Deserialize)]
pub struct OllamaResponse {
    pub response: String,
    pub model: String,
    pub created_at: String,
    pub done: bool,
}

/// Ollama HTTP client.
pub struct OllamaClient {
    base_url: String,
    client: Client,
    timeout: Duration,
}

fn normalize_base_url(url: String) -> String {
    url.trim_end_matches('/').to_string()
}

impl OllamaClient {
    /// Creates a new Ollama client.
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: normalize_base_url(base_url.into()),
            client: OLLAMA_HTTP_CLIENT.clone(),
            timeout: ollama_timeouts::http_client_timeout(),
        }
    }

    /// Sets the request timeout.
    #[must_use]
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
    /// # Errors
    ///
    /// Returns [`Err`] with a human-readable message when the operation fails.
    /// Checks whether the Ollama service is reachable.
    pub async fn health_check(&self) -> Result<bool> {
        let url = format!("{}/api/tags", self.base_url);

        match self.client.get(&url).timeout(self.timeout).send().await {
            Ok(response) => Ok(response.status().is_success()),
            Err(_) => Ok(false),
        }
    }
    /// # Errors
    ///
    /// Returns [`Err`] with a human-readable message when the operation fails.
    /// Lists available models.
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
    /// # Errors
    ///
    /// Returns [`Err`] with a human-readable message when the operation fails.
    /// Calls Ollama to generate a reply.
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
            // 404 often means missing model or wrong URL; body often contains {"error":"..."}
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
    /// # Errors
    ///
    /// Returns [`Err`] with a human-readable message when the operation fails.
    /// Calls Ollama to generate a reply with streaming, invoking `on_token` per chunk.
    pub async fn generate_stream_with_callback(
        &self,
        model: &str,
        prompt: &str,
        temperature: Option<f32>,
        top_p: Option<f32>,
        on_token: Arc<dyn Fn(&str) + Send + Sync>,
    ) -> Result<String> {
        use futures_util::StreamExt;

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
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(AppError::OllamaError(format!(
                "Ollama returned status: {} — {}",
                status,
                body.chars().take(400).collect::<String>()
            )));
        }

        let mut stream = response.bytes_stream();
        let mut line_buf = String::new();
        let mut full_response = String::new();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk
                .map_err(|e| AppError::OllamaError(format!("Stream read failed: {}", e)))?;
            line_buf.push_str(&String::from_utf8_lossy(&chunk));
            while let Some(pos) = line_buf.find('\n') {
                let line = line_buf.drain(..=pos).collect::<String>();
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                if let Ok(json) = serde_json::from_str::<OllamaResponse>(line) {
                    if !json.response.is_empty() {
                        full_response.push_str(&json.response);
                        on_token(json.response.as_str());
                    }
                }
            }
        }
        if !line_buf.trim().is_empty() {
            if let Ok(json) = serde_json::from_str::<OllamaResponse>(line_buf.trim()) {
                if !json.response.is_empty() {
                    full_response.push_str(&json.response);
                    on_token(json.response.as_str());
                }
            }
        }

        Ok(full_response)
    }

    /// Buffered streaming (legacy): reads full body then merges lines.
    pub async fn generate_stream(
        &self,
        model: &str,
        prompt: &str,
        temperature: Option<f32>,
        top_p: Option<f32>,
    ) -> Result<String> {
        self.generate_stream_with_callback(
            model,
            prompt,
            temperature,
            top_p,
            Arc::new(|_| {}),
        )
        .await
    }

    /// Register a local GGUF (or bin) as an Ollama model via `POST /api/create`.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::AppError`] when the HTTP request fails or Ollama rejects the create payload.
    pub async fn create_model_from_path(&self, name: &str, model_path: &str) -> Result<()> {
        let url = format!("{}/api/create", self.base_url);
        let path_escaped = model_path.replace('\\', "/");
        let modelfile = format!("FROM \"{path_escaped}\"\n");
        let body = serde_json::json!({
            "name": name.trim(),
            "modelfile": modelfile,
        });
        let response = self
            .client
            .post(&url)
            .json(&body)
            .timeout(self.timeout)
            .send()
            .await
            .map_err(|e| AppError::OllamaError(format!("create model request: {e}")))?;
        let status = response.status();
        let text = response
            .text()
            .await
            .map_err(|e| AppError::OllamaError(format!("create model body: {e}")))?;
        if !status.is_success() {
            return Err(AppError::OllamaError(format!(
                "create model HTTP {status}: {}",
                text.chars().take(500).collect::<String>()
            )));
        }
        Ok(())
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
        assert!(!json.contains("\"top_p\"")); // should be omitted
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
        let client = OllamaClient::new("http://localhost:9999"); // unused port
        let result = client.health_check().await;
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }
}
