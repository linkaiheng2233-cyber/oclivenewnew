//! 最小 JSON-RPC 2.0 over HTTP POST（与 `creator-docs/plugin-and-architecture/REMOTE_PLUGIN_PROTOCOL.md` 一致）。

use crate::error::{AppError, Result};
use serde_json::{json, Value};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_ID: AtomicU64 = AtomicU64::new(1);
const PROTOCOL_HEADER_NAME: &str = "x-oclive-remote-protocol";
const PROTOCOL_HEADER_VALUE: &str = "oclive-remote-jsonrpc-v1";
const CLIENT_VERSION_HEADER_NAME: &str = "x-oclive-client-version";

fn next_id() -> u64 {
    NEXT_ID.fetch_add(1, Ordering::SeqCst)
}

fn code_name(code: i64) -> &'static str {
    match code {
        -32700 => "parse_error",
        -32600 => "invalid_request",
        -32601 => "method_not_found",
        -32602 => "invalid_params",
        -32603 => "internal_error",
        -32010 => "plugin_timeout",
        -32011 => "auth_failed",
        -32012 => "rate_limited",
        -32013 => "upstream_unavailable",
        _ => "application_error",
    }
}

pub fn call_blocking(
    client: &reqwest::blocking::Client,
    url: &str,
    method: &str,
    params: Value,
    bearer_token: Option<&str>,
) -> Result<Value> {
    let id = next_id();
    let body = json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": method,
        "params": params,
    });
    let mut req = client
        .post(url)
        .header(PROTOCOL_HEADER_NAME, PROTOCOL_HEADER_VALUE)
        .header(CLIENT_VERSION_HEADER_NAME, env!("CARGO_PKG_VERSION"))
        .json(&body);
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
    parse_jsonrpc_result(&text, method, id)
}

pub async fn call_async(
    client: &reqwest::Client,
    url: &str,
    method: &str,
    params: Value,
    bearer_token: Option<&str>,
) -> Result<Value> {
    let id = next_id();
    let body = json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": method,
        "params": params,
    });
    let mut req = client
        .post(url)
        .header(PROTOCOL_HEADER_NAME, PROTOCOL_HEADER_VALUE)
        .header(CLIENT_VERSION_HEADER_NAME, env!("CARGO_PKG_VERSION"))
        .json(&body);
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
    parse_jsonrpc_result(&text, method, id)
}

fn parse_jsonrpc_result(text: &str, method: &str, expected_id: u64) -> Result<Value> {
    let v: Value = serde_json::from_str(text)
        .map_err(|e| AppError::OllamaError(format!("jsonrpc parse: {}", e)))?;
    let jsonrpc_ok = v
        .get("jsonrpc")
        .and_then(|x| x.as_str())
        .map(|s| s == "2.0")
        .unwrap_or(false);
    if !jsonrpc_ok {
        return Err(AppError::OllamaError(format!(
            "jsonrpc invalid version method={} raw={}",
            method, text
        )));
    }
    let id = v.get("id").and_then(|x| x.as_u64()).ok_or_else(|| {
        AppError::OllamaError(format!("jsonrpc missing/invalid id method={}", method))
    })?;
    if id != expected_id {
        return Err(AppError::OllamaError(format!(
            "jsonrpc id mismatch method={} expected={} actual={}",
            method, expected_id, id
        )));
    }
    if let Some(err) = v.get("error") {
        let code = err.get("code").and_then(|x| x.as_i64()).unwrap_or(-32000);
        let msg = err
            .get("message")
            .and_then(|x| x.as_str())
            .unwrap_or("unknown");
        let data = err.get("data").cloned().unwrap_or(Value::Null);
        return Err(AppError::OllamaError(format!(
            "jsonrpc error method={} code={}({}) message={} data={}",
            method,
            code,
            code_name(code),
            msg,
            data
        )));
    }
    v.get("result")
        .cloned()
        .ok_or_else(|| AppError::OllamaError(format!("jsonrpc missing result method={}", method)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_jsonrpc_result_ok() {
        let text = r#"{"jsonrpc":"2.0","id":1,"result":{"ordered_ids":["a"]}}"#;
        let v = parse_jsonrpc_result(text, "memory.rank", 1).unwrap();
        assert_eq!(v["ordered_ids"][0], "a");
    }

    #[test]
    fn parse_jsonrpc_result_err_field() {
        let text = r#"{"jsonrpc":"2.0","id":1,"error":{"code":-1,"message":"x"}}"#;
        assert!(parse_jsonrpc_result(text, "memory.rank", 1).is_err());
    }

    #[test]
    fn parse_jsonrpc_result_rejects_invalid_version() {
        let text = r#"{"jsonrpc":"1.0","id":1,"result":{}}"#;
        assert!(parse_jsonrpc_result(text, "memory.rank", 1).is_err());
    }

    #[test]
    fn parse_jsonrpc_result_rejects_missing_id() {
        let text = r#"{"jsonrpc":"2.0","result":{"ordered_ids":["a"]}}"#;
        assert!(parse_jsonrpc_result(text, "memory.rank", 1).is_err());
    }
}
