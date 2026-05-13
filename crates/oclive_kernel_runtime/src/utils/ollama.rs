use crate::infrastructure::ollama_timeouts;
use once_cell::sync::Lazy;
use reqwest::Client;
use serde_json::{json, Value};
use std::error::Error;
use tokio::time::timeout;

static OLLAMA_CLIENT: Lazy<Client> = Lazy::new(|| {
    Client::builder()
        .build()
        .expect("Failed to build HTTP client")
});

// 调用 Ollama API 生成文本
pub async fn ollama_generate(
    model: &str,
    prompt: &str,
    max_tokens: u32,
    temperature: f64,
) -> Result<String, Box<dyn Error>> {
    let url = "http://localhost:11434/api/generate";

    let body = json!({
        "model": model,
        "prompt": prompt,
        "stream": false,
        "options": {
            "num_predict": max_tokens,
            "temperature": temperature
        }
    });

    let call_future = async {
        let resp = OLLAMA_CLIENT.post(url).json(&body).send().await?;
        let json_resp: Value = resp.json().await?;

        if let Some(result_str) = json_resp.get("response").and_then(|v| v.as_str()) {
            Ok(result_str.to_string())
        } else {
            Err("Ollama API response missing `response` field".into())
        }
    };

    // 带超时保护调用（与主路径 `OllamaClient` 分离，默认更短；见 `ollama_timeouts`）
    let result = timeout(ollama_timeouts::legacy_utils_call_timeout(), call_future).await;

    match result {
        Ok(Ok(text)) => Ok(text),
        Ok(Err(e)) => Err(e),
        Err(_) => Err("Ollama generate call timed out".into()),
    }
}

// 可以扩展支持云端OpenAI兼容API，待后续设计
