//! OOCP WebSocket transport (kernel runtime).

use crate::domain::adapters::runtime_oocp_handler::RuntimeOocpHandler;
use crate::domain::core::oocp_handler::{dispatch_oocp_request, OocpHandled};
use crate::state::KernelAppState;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Query, State};
use axum::http::HeaderMap;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::Router;
use futures_util::stream::StreamExt;
use futures_util::SinkExt;
use std::sync::Arc;
use std::time::Duration;
use tokio::select;
use tokio::time::interval;

const PING_INTERVAL_SECS: u64 = 15;

fn oocp_api_token() -> Option<String> {
    std::env::var("OOCP_API_TOKEN")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

fn verify_token(maybe_server_token: &Option<String>, client_token: Option<&str>) -> bool {
    match maybe_server_token {
        None => true,
        Some(server_token) => match client_token {
            Some(ct) => subtle_compare(server_token, ct),
            None => false,
        },
    }
}

fn subtle_compare(a: &str, b: &str) -> bool {
    let a = a.as_bytes();
    let b = b.as_bytes();
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

fn extract_client_token(headers: &HeaderMap, query_token: Option<&str>) -> Option<String> {
    if let Some(t) = query_token.filter(|s| !s.is_empty()) {
        return Some(t.to_string());
    }
    headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(|s| s.to_string())
}

#[derive(serde::Deserialize, Default)]
struct WsQuery {
    #[serde(default)]
    token: Option<String>,
}

async fn oocp_ws_handler(
    ws: WebSocketUpgrade,
    State(app_state): State<Arc<KernelAppState>>,
    Query(query): Query<WsQuery>,
    headers: HeaderMap,
) -> Result<Response, Response> {
    let server_token = oocp_api_token();
    let client_token = extract_client_token(&headers, query.token.as_deref());

    if !verify_token(&server_token, client_token.as_deref()) {
        return Err((
            axum::http::StatusCode::UNAUTHORIZED,
            [(axum::http::header::WWW_AUTHENTICATE, "Bearer")],
            "OOCP 鉴权失败：token 不匹配",
        )
            .into_response());
    }

    Ok(ws.on_upgrade(move |socket| handle_oocp_socket(socket, app_state)))
}

async fn handle_oocp_socket(socket: WebSocket, app_state: Arc<KernelAppState>) {
    let (mut writer, mut reader) = socket.split();
    let mut handler = RuntimeOocpHandler::new(app_state);

    let caps = crate::domain::core::oocp_handler::get_capabilities();
    let payload = serde_json::to_string(&caps).unwrap_or_else(|_| "{}".to_string());
    if writer.send(Message::Text(payload)).await.is_err() {
        return;
    }

    let mut ping_tick = interval(Duration::from_secs(PING_INTERVAL_SECS));

    loop {
        select! {
            msg = reader.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        let resp = handle_text_frame(&text, &mut handler, &caps).await;
                        if let Some(frame) = resp {
                            let json = serde_json::to_string(&frame).unwrap_or_else(|e| {
                                format!(r#"{{"type":"error","id":null,"error":{{"code":"INTERNAL","message":"serialize failed: {}"}}}}"#, e)
                            });
                            if writer.send(Message::Text(json)).await.is_err() {
                                break;
                            }
                        }
                        for event in handler.drain_events() {
                            let json = serde_json::to_string(&event).unwrap_or_else(|_| "{}".to_string());
                            if writer.send(Message::Text(json)).await.is_err() {
                                break;
                            }
                        }
                    }
                    Some(Ok(Message::Ping(data))) => {
                        let _ = writer.send(Message::Pong(data)).await;
                    }
                    Some(Ok(Message::Close(_))) => break,
                    Some(Ok(_)) => {}
                    Some(Err(_)) => break,
                    None => break,
                }
            }
            _ = ping_tick.tick() => {
                if writer.send(Message::Ping(vec![])).await.is_err() {
                    break;
                }
            }
        }
    }
}

async fn handle_text_frame(
    text: &str,
    handler: &mut RuntimeOocpHandler,
    caps: &crate::models::oocp::OocpCapabilities,
) -> Option<serde_json::Value> {
    if text.trim().is_empty() {
        return Some(serde_json::to_value(caps).unwrap_or(serde_json::Value::Null));
    }

    let runtime_caps = crate::domain::core::oocp_handler::get_capabilities();
    if text.len() > runtime_caps.limits.max_message_chars as usize {
        let err = crate::models::oocp::OocpError {
            msg_type: "error",
            id: serde_json::Value::Null,
            error: crate::models::oocp::OocpErrorBody {
                code: crate::models::oocp::OocpErrorCode::InvalidParams
                    .as_str()
                    .to_string(),
                message: format!(
                    "消息长度超过限制 ({} > {})",
                    text.len(),
                    runtime_caps.limits.max_message_chars
                ),
                data: serde_json::Value::Null,
            },
        };
        return Some(serde_json::to_value(&err).unwrap_or(serde_json::Value::Null));
    }

    match serde_json::from_str::<crate::models::oocp::OocpRequest>(text) {
        Ok(mut req) => {
            if req.msg_type.is_empty() {
                req.msg_type = "request".to_string();
            }
            if req.msg_type != "request" && !req.method.is_empty() {
                req.msg_type = "request".to_string();
            }
            let result = dispatch_oocp_request(req, handler, caps).await;
            match result {
                OocpHandled::Response(r) => {
                    Some(serde_json::to_value(&r).unwrap_or(serde_json::Value::Null))
                }
                OocpHandled::Error(e) => {
                    Some(serde_json::to_value(&e).unwrap_or(serde_json::Value::Null))
                }
                OocpHandled::Capabilities(c) => {
                    Some(serde_json::to_value(&c).unwrap_or(serde_json::Value::Null))
                }
            }
        }
        Err(_) => None,
    }
}

pub fn oocp_ws_router() -> Router<Arc<KernelAppState>> {
    Router::<Arc<KernelAppState>>::new().route("/oocp", get(oocp_ws_handler))
}
