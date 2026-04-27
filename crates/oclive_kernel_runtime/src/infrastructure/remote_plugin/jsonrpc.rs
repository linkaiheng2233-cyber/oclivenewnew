//! 最小 JSON-RPC 2.0 over HTTP POST（与 `creator-docs/plugin-and-architecture/REMOTE_PLUGIN_PROTOCOL.md` 一致）。

use crate::error::{AppError, Result};
use serde_json::{json, Value};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

static NEXT_ID: AtomicU64 = AtomicU64::new(1);
const PROTOCOL_HEADER_NAME: &str = "x-oclive-remote-protocol";
const PROTOCOL_HEADER_VALUE: &str = "oclive-remote-jsonrpc-v1";
const CLIENT_VERSION_HEADER_NAME: &str = "x-oclive-client-version";
/// 错误信息里原始响应体的最大长度（避免反向代理 HTML 把日志撑爆）。
const BODY_PREVIEW_MAX: usize = 512;

/// 日志与 `AppError` 文案中的通道标签（`call_async` 同时服务于 plugin 端点与 LLM 端点）。
#[derive(Clone, Copy, Debug)]
pub enum RemoteRpcChannel {
    Plugin,
    Llm,
}

impl RemoteRpcChannel {
    fn label(self) -> &'static str {
        match self {
            Self::Plugin => "remote_plugin",
            Self::Llm => "remote_llm",
        }
    }
}

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

fn classify_reqwest_error(e: &reqwest::Error) -> &'static str {
    if e.is_timeout() {
        "timeout"
    } else if e.is_connect() {
        "connect"
    } else if e.is_status() {
        "status"
    } else if e.is_request() {
        "request"
    } else if e.is_decode() {
        "decode"
    } else {
        "transport"
    }
}

fn body_preview(text: &str) -> String {
    let t = text.trim();
    if t.is_empty() {
        return "(empty)".to_string();
    }
    if t.len() <= BODY_PREVIEW_MAX {
        return t.to_string();
    }
    let mut end = BODY_PREVIEW_MAX.min(t.len());
    while end > 0 && !t.is_char_boundary(end) {
        end -= 1;
    }
    format!("{}… (truncated bytes={})", &t[..end], t.len())
}

pub fn call_blocking(
    channel: RemoteRpcChannel,
    client: &reqwest::blocking::Client,
    url: &str,
    method: &str,
    params: Value,
    bearer_token: Option<&str>,
) -> Result<Value> {
    let id = next_id();
    let t0 = Instant::now();
    let ch = channel.label();
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
    let resp = req.send().map_err(|e| {
        let kind = classify_reqwest_error(&e);
        let ms = t0.elapsed().as_millis();
        log::warn!(
            target: "oclive_plugin",
            "{} rpc_fail kind={} phase=send method={} url={} duration_ms={} err={}",
            ch,
            kind,
            method,
            url,
            ms,
            e
        );
        AppError::OllamaError(format!(
            "{} transport kind={} method={} url={} err={}",
            ch, kind, method, url, e
        ))
    })?;
    let status = resp.status();
    let text = resp.text().map_err(|e| {
        let ms = t0.elapsed().as_millis();
        log::warn!(
            target: "oclive_plugin",
            "{} rpc_fail kind=read_body method={} url={} duration_ms={} err={}",
            ch,
            method,
            url,
            ms,
            e
        );
        AppError::OllamaError(format!("{} body read: {}", ch, e))
    })?;
    if !status.is_success() {
        let ms = t0.elapsed().as_millis();
        log::warn!(
            target: "oclive_plugin",
            "{} rpc_fail kind=http_status method={} url={} status={} duration_ms={} body={}",
            ch,
            method,
            url,
            status,
            ms,
            body_preview(&text)
        );
        return Err(AppError::OllamaError(format!(
            "{} http_status method={} url={} status={} body={}",
            ch,
            method,
            url,
            status,
            body_preview(&text)
        )));
    }
    log::debug!(
        target: "oclive_plugin",
        "{} rpc_ok method={} url={} duration_ms={}",
        ch,
        method,
        url,
        t0.elapsed().as_millis()
    );
    parse_jsonrpc_result(&text, method, id)
}

pub async fn call_async(
    channel: RemoteRpcChannel,
    client: &reqwest::Client,
    url: &str,
    method: &str,
    params: Value,
    bearer_token: Option<&str>,
) -> Result<Value> {
    let id = next_id();
    let t0 = Instant::now();
    let ch = channel.label();
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
    let resp = req.send().await.map_err(|e| {
        let kind = classify_reqwest_error(&e);
        let ms = t0.elapsed().as_millis();
        log::warn!(
            target: "oclive_plugin",
            "{} rpc_fail kind={} phase=send method={} url={} duration_ms={} err={}",
            ch,
            kind,
            method,
            url,
            ms,
            e
        );
        AppError::OllamaError(format!(
            "{} transport kind={} method={} url={} err={}",
            ch, kind, method, url, e
        ))
    })?;
    let status = resp.status();
    let text = resp.text().await.map_err(|e| {
        let ms = t0.elapsed().as_millis();
        log::warn!(
            target: "oclive_plugin",
            "{} rpc_fail kind=read_body method={} url={} duration_ms={} err={}",
            ch,
            method,
            url,
            ms,
            e
        );
        AppError::OllamaError(format!("{} body read: {}", ch, e))
    })?;
    if !status.is_success() {
        let ms = t0.elapsed().as_millis();
        log::warn!(
            target: "oclive_plugin",
            "{} rpc_fail kind=http_status method={} url={} status={} duration_ms={} body={}",
            ch,
            method,
            url,
            status,
            ms,
            body_preview(&text)
        );
        return Err(AppError::OllamaError(format!(
            "{} http_status method={} url={} status={} body={}",
            ch,
            method,
            url,
            status,
            body_preview(&text)
        )));
    }
    log::debug!(
        target: "oclive_plugin",
        "{} rpc_ok method={} url={} duration_ms={}",
        ch,
        method,
        url,
        t0.elapsed().as_millis()
    );
    parse_jsonrpc_result(&text, method, id)
}

fn json_request_id_matches(id: &Value, expected: u64) -> bool {
    match id {
        Value::Number(n) => n.as_u64() == Some(expected),
        Value::String(s) => s.trim().parse::<u64>().ok() == Some(expected),
        _ => false,
    }
}

fn parse_jsonrpc_result(text: &str, method: &str, expected_id: u64) -> Result<Value> {
    let trim = text.trim();
    if trim.is_empty() {
        return Err(AppError::OllamaError(format!(
            "jsonrpc empty_body method={}",
            method
        )));
    }
    let v: Value = serde_json::from_str(trim).map_err(|e| {
        AppError::OllamaError(format!(
            "jsonrpc parse method={} err={} raw={}",
            method,
            e,
            body_preview(trim)
        ))
    })?;
    let jsonrpc_ok = v
        .get("jsonrpc")
        .and_then(|x| x.as_str())
        .map(|s| s == "2.0")
        .unwrap_or(false);
    if !jsonrpc_ok {
        return Err(AppError::OllamaError(format!(
            "jsonrpc invalid version method={} raw={}",
            method,
            body_preview(trim)
        )));
    }
    let Some(idv) = v.get("id") else {
        return Err(AppError::OllamaError(format!(
            "jsonrpc missing id method={} raw={}",
            method,
            body_preview(trim)
        )));
    };
    if !json_request_id_matches(idv, expected_id) {
        let actual = match idv {
            Value::Number(n) => n.to_string(),
            Value::String(s) => s.clone(),
            _ => idv.to_string(),
        };
        return Err(AppError::OllamaError(format!(
            "jsonrpc id mismatch method={} expected={} actual={} raw={}",
            method,
            expected_id,
            actual,
            body_preview(trim)
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
    fn parse_jsonrpc_result_ok_string_id() {
        let text = r#"{"jsonrpc":"2.0","id":"42","result":{"ordered_ids":["a"]}}"#;
        let v = parse_jsonrpc_result(text, "memory.rank", 42).unwrap();
        assert_eq!(v["ordered_ids"][0], "a");
    }

    #[test]
    fn parse_jsonrpc_result_empty_body() {
        assert!(parse_jsonrpc_result("", "m", 1).is_err());
        assert!(parse_jsonrpc_result("   \n", "m", 1).is_err());
    }

    #[test]
    fn body_preview_truncates_long_text() {
        let long = "x".repeat(BODY_PREVIEW_MAX + 50);
        let p = body_preview(&long);
        assert!(p.contains("truncated"));
        assert!(p.len() < long.len());
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

    #[test]
    fn parse_jsonrpc_result_rejects_id_mismatch() {
        let text = r#"{"jsonrpc":"2.0","id":3,"result":{"ordered_ids":["a"]}}"#;
        assert!(parse_jsonrpc_result(text, "memory.rank", 1).is_err());
    }

    #[test]
    fn code_name_maps_protocol_codes() {
        assert_eq!(code_name(-32601), "method_not_found");
        assert_eq!(code_name(-32010), "plugin_timeout");
    }
}
