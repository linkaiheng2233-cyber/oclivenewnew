//! 最小 JSON-RPC 2.0 over HTTP POST（与 `creator-docs/plugin-and-architecture/REMOTE_PLUGIN_PROTOCOL.md` 一致）。

use crate::error::{AppError, Result};
use serde_json::{json, Value};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

fn next_id() -> u64 {
    NEXT_ID.fetch_add(1, Ordering::SeqCst)
}

pub fn call_blocking(
    client: &reqwest::blocking::Client,
    url: &str,
    method: &str,
    params: Value,
    bearer_token: Option<&str>,
) -> Result<Value> {
    let body = json!({
        "jsonrpc": "2.0",
        "id": next_id(),
        "method": method,
        "params": params,
    });
    let mut req = client.post(url).json(&body);
    if let Some(t) = bearer_token {
        req = req.bearer_auth(t);
    }
    let resp = req
        .send()
        .map_err(|e| AppError::OllamaError(format!("remote_plugin http: {}", e)))?;
    let status = resp.status();
    let text = resp
        .text()
        .map_err(|e| AppError::OllamaError(format!("remote_plugin body: {}", e)))?;
    if !status.is_success() {
        return Err(AppError::OllamaError(format!(
            "remote_plugin status={} body={}",
            status, text
        )));
    }
    parse_jsonrpc_result(&text)
}

pub async fn call_async(
    client: &reqwest::Client,
    url: &str,
    method: &str,
    params: Value,
    bearer_token: Option<&str>,
) -> Result<Value> {
    let body = json!({
        "jsonrpc": "2.0",
        "id": next_id(),
        "method": method,
        "params": params,
    });
    let mut req = client.post(url).json(&body);
    if let Some(t) = bearer_token {
        req = req.bearer_auth(t);
    }
    let resp = req
        .send()
        .await
        .map_err(|e| AppError::OllamaError(format!("remote_llm http: {}", e)))?;
    let status = resp.status();
    let text = resp
        .text()
        .await
        .map_err(|e| AppError::OllamaError(format!("remote_llm body: {}", e)))?;
    if !status.is_success() {
        return Err(AppError::OllamaError(format!(
            "remote_llm status={} body={}",
            status, text
        )));
    }
    parse_jsonrpc_result(&text)
}

fn parse_jsonrpc_result(text: &str) -> Result<Value> {
    let v: Value = serde_json::from_str(text)
        .map_err(|e| AppError::OllamaError(format!("jsonrpc parse: {}", e)))?;
    if let Some(err) = v.get("error") {
        return Err(AppError::OllamaError(format!("jsonrpc error: {}", err)));
    }
    v.get("result")
        .cloned()
        .ok_or_else(|| AppError::OllamaError("jsonrpc missing result".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_jsonrpc_result_ok() {
        let text = r#"{"jsonrpc":"2.0","id":1,"result":{"ordered_ids":["a"]}}"#;
        let v = parse_jsonrpc_result(text).unwrap();
        assert_eq!(v["ordered_ids"][0], "a");
    }

    #[test]
    fn parse_jsonrpc_result_err_field() {
        let text = r#"{"jsonrpc":"2.0","id":1,"error":{"code":-1,"message":"x"}}"#;
        assert!(parse_jsonrpc_result(text).is_err());
    }
}
