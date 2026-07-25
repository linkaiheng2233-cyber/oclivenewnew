//! Minimal JSON-RPC 2.0 over HTTP POST (consistent with `creator-docs/plugin-and-architecture/REMOTE_PLUGIN_PROTOCOL.md`).

use crate::error::{AppError, Result};
use crate::infrastructure::high_risk_grants::HighRiskGrantStore;
use futures_util::StreamExt;
use serde_json::{json, Value};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

static NEXT_ID: AtomicU64 = AtomicU64::new(1);
const PROTOCOL_HEADER_NAME: &str = "x-oclive-remote-protocol";
const PROTOCOL_HEADER_VALUE: &str = "oclive-remote-jsonrpc-v1";
const CLIENT_VERSION_HEADER_NAME: &str = "x-oclive-client-version";
/// Maximum length of the raw response body included in error messages (prevents reverse-proxy HTML from blowing up the logs).
const BODY_PREVIEW_MAX: usize = 512;
const STREAM_LINE_MAX_BYTES: usize = 1024 * 1024;
const STREAM_OUTPUT_MAX_BYTES: usize = 16 * 1024 * 1024;

/// Channel label used in logs and `AppError` messages (`call_async` serves both the plugin endpoint and the LLM endpoint).
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

#[allow(clippy::too_many_arguments)]
pub fn call_blocking(
    channel: RemoteRpcChannel,
    client: &reqwest::Client,
    url: &str,
    method: &str,
    params: Value,
    bearer_token: Option<&str>,
    network_grant: Option<(&HighRiskGrantStore, &str)>,
    request_timeout: Duration,
) -> Result<Value> {
    crate::utils::block_on::block_on(call_async(
        channel,
        client,
        url,
        method,
        params,
        bearer_token,
        network_grant,
        request_timeout,
    ))
}

#[allow(clippy::too_many_arguments)]
pub async fn call_async(
    channel: RemoteRpcChannel,
    client: &reqwest::Client,
    url: &str,
    method: &str,
    params: Value,
    bearer_token: Option<&str>,
    network_grant: Option<(&HighRiskGrantStore, &str)>,
    request_timeout: Duration,
) -> Result<Value> {
    let (id, t0, resp) = send_async(
        channel,
        client,
        url,
        method,
        params,
        bearer_token,
        network_grant,
        request_timeout,
    )
    .await?;
    let ch = channel.label();
    let status = resp.status();
    let text = resp.text().await.map_err(|e| {
        let ms = t0.elapsed().as_millis();
        tracing::warn!(
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
        tracing::warn!(
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
    tracing::debug!(
        target: "oclive_plugin",
        "{} rpc_ok method={} url={} duration_ms={}",
        ch,
        method,
        url,
        t0.elapsed().as_millis()
    );
    parse_jsonrpc_result(&text, method, id)
}

#[allow(clippy::too_many_arguments)]
async fn send_async(
    channel: RemoteRpcChannel,
    client: &reqwest::Client,
    url: &str,
    method: &str,
    params: Value,
    bearer_token: Option<&str>,
    network_grant: Option<(&HighRiskGrantStore, &str)>,
    request_timeout: Duration,
) -> Result<(u64, Instant, reqwest::Response)> {
    if let Some((grants, grant_id)) = network_grant {
        grants.require_network(grant_id)?;
    }
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
        .timeout(request_timeout)
        .header(PROTOCOL_HEADER_NAME, PROTOCOL_HEADER_VALUE)
        .header(CLIENT_VERSION_HEADER_NAME, env!("CARGO_PKG_VERSION"))
        .json(&body);
    if let Some(t) = bearer_token {
        req = req.bearer_auth(t);
    }
    let resp = req.send().await.map_err(|e| {
        let kind = classify_reqwest_error(&e);
        let ms = t0.elapsed().as_millis();
        tracing::warn!(
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
    Ok((id, t0, resp))
}

/// JSON-RPC-over-NDJSON extension for incremental LLM output.
///
/// Every line is a JSON-RPC envelope with the original request id. `token`
/// events are delivered immediately; exactly one `done` event must terminate
/// the stream.
#[allow(clippy::too_many_arguments)]
pub async fn call_async_stream(
    channel: RemoteRpcChannel,
    client: &reqwest::Client,
    url: &str,
    method: &str,
    params: Value,
    bearer_token: Option<&str>,
    network_grant: Option<(&HighRiskGrantStore, &str)>,
    request_timeout: Duration,
    on_token: &(dyn Fn(&str) + Send + Sync),
) -> Result<Value> {
    let (id, t0, resp) = send_async(
        channel,
        client,
        url,
        method,
        params,
        bearer_token,
        network_grant,
        request_timeout,
    )
    .await?;
    let ch = channel.label();
    let status = resp.status();
    if !status.is_success() {
        let text = resp.text().await.unwrap_or_default();
        let ms = t0.elapsed().as_millis();
        tracing::warn!(
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
    let content_type = resp
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_ascii_lowercase();
    if !content_type.starts_with("application/x-ndjson") {
        let text = resp.text().await.unwrap_or_default();
        return Err(AppError::OllamaError(format!(
            "jsonrpc stream invalid content_type method={} expected=application/x-ndjson actual={} body={}",
            method,
            content_type,
            body_preview(&text)
        )));
    }

    let mut pending = Vec::<u8>::new();
    let mut state = JsonRpcStreamState::default();
    let mut body = resp.bytes_stream();
    while let Some(next) = body.next().await {
        let chunk = next.map_err(|e| {
            AppError::OllamaError(format!(
                "{} stream body read method={} url={} err={}",
                ch, method, url, e
            ))
        })?;
        pending.extend_from_slice(&chunk);
        if pending.len() > STREAM_LINE_MAX_BYTES && !pending.contains(&b'\n') {
            return Err(AppError::OllamaError(format!(
                "jsonrpc stream line too large method={} max_bytes={}",
                method, STREAM_LINE_MAX_BYTES
            )));
        }
        while let Some(newline) = pending.iter().position(|byte| *byte == b'\n') {
            if newline > STREAM_LINE_MAX_BYTES {
                return Err(AppError::OllamaError(format!(
                    "jsonrpc stream line too large method={} max_bytes={}",
                    method, STREAM_LINE_MAX_BYTES
                )));
            }
            let line = pending.drain(..=newline).collect::<Vec<_>>();
            state.accept_line(&line[..line.len().saturating_sub(1)], method, id, on_token)?;
        }
    }
    if !pending.is_empty() {
        if pending.len() > STREAM_LINE_MAX_BYTES {
            return Err(AppError::OllamaError(format!(
                "jsonrpc stream line too large method={} max_bytes={}",
                method, STREAM_LINE_MAX_BYTES
            )));
        }
        state.accept_line(&pending, method, id, on_token)?;
    }
    let result = state.finish(method)?;
    tracing::debug!(
        target: "oclive_plugin",
        "{} rpc_stream_ok method={} url={} duration_ms={}",
        ch,
        method,
        url,
        t0.elapsed().as_millis()
    );
    Ok(result)
}

#[derive(Default)]
struct JsonRpcStreamState {
    output: String,
    done: bool,
    prompt_eval_ms: Option<u64>,
}

impl JsonRpcStreamState {
    fn accept_line(
        &mut self,
        line: &[u8],
        method: &str,
        expected_id: u64,
        on_token: &(dyn Fn(&str) + Send + Sync),
    ) -> Result<()> {
        let raw = std::str::from_utf8(line)
            .map_err(|e| {
                AppError::OllamaError(format!("jsonrpc stream utf8 method={} err={}", method, e))
            })?
            .trim();
        if raw.is_empty() {
            return Ok(());
        }
        let envelope: Value = serde_json::from_str(raw).map_err(|e| {
            AppError::OllamaError(format!(
                "jsonrpc stream parse method={} err={} raw={}",
                method,
                e,
                body_preview(raw)
            ))
        })?;
        let result = jsonrpc_envelope_result(&envelope, method, expected_id, raw)?;
        let event = result
            .get("event")
            .and_then(Value::as_str)
            .unwrap_or_default();
        match event {
            "token" => {
                if self.done {
                    return Err(AppError::OllamaError(format!(
                        "jsonrpc stream token after done method={}",
                        method
                    )));
                }
                let text = result.get("text").and_then(Value::as_str).ok_or_else(|| {
                    AppError::OllamaError(format!(
                        "jsonrpc stream token missing text method={}",
                        method
                    ))
                })?;
                if self.output.len().saturating_add(text.len()) > STREAM_OUTPUT_MAX_BYTES {
                    return Err(AppError::OllamaError(format!(
                        "jsonrpc stream output too large method={} max_bytes={}",
                        method, STREAM_OUTPUT_MAX_BYTES
                    )));
                }
                self.output.push_str(text);
                on_token(text);
            }
            "done" => {
                if self.done {
                    return Err(AppError::OllamaError(format!(
                        "jsonrpc stream duplicate done method={}",
                        method
                    )));
                }
                self.done = true;
                self.prompt_eval_ms = result.get("prompt_eval_ms").and_then(Value::as_u64);
            }
            other => {
                return Err(AppError::OllamaError(format!(
                    "jsonrpc stream unknown event method={} event={}",
                    method, other
                )));
            }
        }
        Ok(())
    }

    fn finish(self, method: &str) -> Result<Value> {
        if !self.done {
            return Err(AppError::OllamaError(format!(
                "jsonrpc stream ended before done method={}",
                method
            )));
        }
        Ok(json!({
            "text": self.output,
            "prompt_eval_ms": self.prompt_eval_ms,
        }))
    }
}

fn json_request_id_matches(id: &Value, expected: u64) -> bool {
    match id {
        Value::Number(n) => n.as_u64() == Some(expected),
        Value::String(s) => s.trim().parse::<u64>().ok() == Some(expected),
        _ => false,
    }
}

fn jsonrpc_envelope_result<'a>(
    value: &'a Value,
    method: &str,
    expected_id: u64,
    raw: &str,
) -> Result<&'a Value> {
    let jsonrpc_ok = value
        .get("jsonrpc")
        .and_then(|x| x.as_str())
        .map(|s| s == "2.0")
        .unwrap_or(false);
    if !jsonrpc_ok {
        return Err(AppError::OllamaError(format!(
            "jsonrpc invalid version method={} raw={}",
            method,
            body_preview(raw)
        )));
    }
    let Some(idv) = value.get("id") else {
        return Err(AppError::OllamaError(format!(
            "jsonrpc missing id method={} raw={}",
            method,
            body_preview(raw)
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
            body_preview(raw)
        )));
    }
    if let Some(err) = value.get("error") {
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
    value
        .get("result")
        .ok_or_else(|| AppError::OllamaError(format!("jsonrpc missing result method={}", method)))
}

fn parse_jsonrpc_result(text: &str, method: &str, expected_id: u64) -> Result<Value> {
    let trim = text.trim();
    if trim.is_empty() {
        return Err(AppError::OllamaError(format!(
            "jsonrpc empty_body method={}",
            method
        )));
    }
    let value: Value = serde_json::from_str(trim).map_err(|e| {
        AppError::OllamaError(format!(
            "jsonrpc parse method={} err={} raw={}",
            method,
            e,
            body_preview(trim)
        ))
    })?;
    jsonrpc_envelope_result(&value, method, expected_id, trim).cloned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

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

    #[test]
    fn stream_state_emits_tokens_and_requires_done() {
        let emitted = Mutex::new(String::new());
        let sink = |token: &str| {
            emitted.lock().expect("sink lock").push_str(token);
        };
        let mut state = JsonRpcStreamState::default();
        state
            .accept_line(
                br#"{"jsonrpc":"2.0","id":7,"result":{"event":"token","text":"mu"}}"#,
                "llm.generate_stream",
                7,
                &sink,
            )
            .expect("first token");
        state
            .accept_line(
                br#"{"jsonrpc":"2.0","id":7,"result":{"event":"token","text":"mu"}}"#,
                "llm.generate_stream",
                7,
                &sink,
            )
            .expect("second token");
        state
            .accept_line(
                br#"{"jsonrpc":"2.0","id":7,"result":{"event":"done","prompt_eval_ms":12}}"#,
                "llm.generate_stream",
                7,
                &sink,
            )
            .expect("done");

        let result = state.finish("llm.generate_stream").expect("result");
        assert_eq!(emitted.lock().expect("emitted lock").as_str(), "mumu");
        assert_eq!(result["text"], "mumu");
        assert_eq!(result["prompt_eval_ms"], 12);
    }

    #[test]
    fn stream_state_rejects_missing_done_and_tokens_after_done() {
        let sink = |_token: &str| {};
        let mut incomplete = JsonRpcStreamState::default();
        incomplete
            .accept_line(
                br#"{"jsonrpc":"2.0","id":1,"result":{"event":"token","text":"partial"}}"#,
                "llm.generate_stream",
                1,
                &sink,
            )
            .expect("partial token");
        assert!(incomplete.finish("llm.generate_stream").is_err());

        let mut completed = JsonRpcStreamState::default();
        completed
            .accept_line(
                br#"{"jsonrpc":"2.0","id":1,"result":{"event":"done"}}"#,
                "llm.generate_stream",
                1,
                &sink,
            )
            .expect("done");
        assert!(completed
            .accept_line(
                br#"{"jsonrpc":"2.0","id":1,"result":{"event":"token","text":"late"}}"#,
                "llm.generate_stream",
                1,
                &sink,
            )
            .is_err());
    }
}
