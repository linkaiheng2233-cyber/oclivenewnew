//! OOCP WebSocket 传输层（v0.1 最小实现）。
//!
//! 基于 axum WebSocket，挂载在现有 HTTP API 的 `/oocp` 路径。
//! 连接建立后发送 capabilities，然后进入请求-响应循环 + 事件推送。
//!
//! 鉴权：最小实现——客户端通过 `?token=` 查询参数或 `Authorization: Bearer <token>`
//! 传入共享令牌；令牌从环境变量 `OOCP_API_TOKEN` 读取，未设置则允许无鉴权连接。

use crate::domain::adapters::tauri_oocp_handler::TauriOocpHandler;
use crate::domain::core::oocp_handler::{dispatch_oocp_request, OocpHandled};
use crate::state::AppState;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Query, State};
use axum::http::HeaderMap;
use axum::response::Response;
use axum::routing::get;
use axum::Router;
use futures_util::stream::StreamExt;
use futures_util::SinkExt;
use std::sync::Arc;
use std::time::Duration;
use tokio::select;
use tokio::time::interval;

const PING_INTERVAL_SECS: u64 = 15;
const PING_TIMEOUT_SECS: u64 = 5;

/// 从环境变量读取 OOCP 共享令牌；未设置返回 `None`（允许无鉴权）。
fn oocp_api_token() -> Option<String> {
    std::env::var("OOCP_API_TOKEN")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

/// 验证客户端提供的令牌；若无服务端令牌则一律放行。
fn verify_token(maybe_server_token: &Option<String>, client_token: Option<&str>) -> bool {
    match maybe_server_token {
        None => true, // 未配置服务端令牌，允许所有连接
        Some(server_token) => match client_token {
            Some(ct) => subtle_compare(server_token, ct),
            None => false,
        },
    }
}

/// 常量时间比较，避免时序侧信道。
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

/// 从查询参数或 Header 中提取客户端提供的 token。
fn extract_client_token(headers: &HeaderMap, query_token: Option<&str>) -> Option<&str> {
    if let Some(t) = query_token.filter(|s| !s.is_empty()) {
        return Some(t);
    }
    headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
}

/// 查询参数类型，用于 `axum::extract::Query`。
#[derive(serde::Deserialize, Default)]
struct WsQuery {
    #[serde(default)]
    token: Option<String>,
}

/// WebSocket 升级 handler：验证鉴权后升级连接，驱动 OOCP 协议循环。
async fn oocp_ws_handler(
    ws: WebSocketUpgrade,
    State(app_state): State<Arc<AppState>>,
    Query(query): Query<WsQuery>,
    headers: HeaderMap,
) -> Result<Response, Response> {
    let server_token = oocp_api_token();
    let client_token = extract_client_token(&headers, query.token.as_deref());

    log::info!(
        target: "oclive_oocp_ws",
        "ws upgrade request auth_required={} client_has_token={}",
        server_token.is_some(),
        client_token.is_some()
    );

    if !verify_token(&server_token, client_token) {
        return Err((
            axum::http::StatusCode::UNAUTHORIZED,
            [(axum::http::header::WWW_AUTHENTICATE, "Bearer")],
            "OOCP 鉴权失败：token 不匹配",
        )
            .into_response());
    }

    Ok(ws.on_upgrade(move |socket| handle_oocp_socket(socket, app_state)))
}

/// 每个客户端连接的主循环。
async fn handle_oocp_socket(socket: WebSocket, app_state: Arc<AppState>) {
    let (mut writer, mut reader) = socket.split();
    let mut handler = TauriOocpHandler::new(app_state);

    // 阶段 0：发送 capabilities。
    {
        let caps = crate::domain::core::oocp_handler::get_capabilities();
        let payload = serde_json::to_string(&caps).unwrap_or_else(|_| "{}".to_string());
        if let Err(e) = writer.send(Message::Text(payload.into())).await {
            log::warn!(target: "oclive_oocp_ws", "failed to send capabilities: {}", e);
            return;
        }
        log::debug!(target: "oclive_oocp_ws", "capabilities sent");
    }

    let mut ping_tick = interval(Duration::from_secs(PING_INTERVAL_SECS));

    loop {
        select! {
            // ── 客户端消息 ──
            msg = reader.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        log::trace!(target: "oclive_oocp_ws", "rx text len={}", text.len());
                        let resp = handle_text_frame(&text, &mut handler).await;
                        if let Some(frame) = resp {
                            let json = serde_json::to_string(&frame).unwrap_or_else(|e| {
                                format!(r#"{{"type":"error","id":"0","error":{{"code":"INTERNAL","message":"serialize failed: {}"}}}}"#, e)
                            });
                            if let Err(e) = writer.send(Message::Text(json.into())).await {
                                log::warn!(target: "oclive_oocp_ws", "failed to send response: {}", e);
                                break;
                            }
                        }
                        // 推送缓冲事件。
                        for event in handler.drain_events() {
                            let json = serde_json::to_string(&event).unwrap_or_else(|e| {
                                format!(r#"{{"type":"event","event":"error","payload":{{"msg":"serialize: {}"}}}}"#, e)
                            });
                            if let Err(e) = writer.send(Message::Text(json.into())).await {
                                log::warn!(target: "oclive_oocp_ws", "failed to push event: {}", e);
                                break;
                            }
                        }
                    }
                    Some(Ok(Message::Binary(_))) => {
                        // v0.1 不支持 binary 帧；忽略。
                        log::debug!(target: "oclive_oocp_ws", "rx binary ignored");
                    }
                    Some(Ok(Message::Ping(data))) => {
                        if let Err(e) = writer.send(Message::Pong(data)).await {
                            log::warn!(target: "oclive_oocp_ws", "pong failed: {}", e);
                            break;
                        }
                    }
                    Some(Ok(Message::Pong(_))) => {
                        // 心跳响应；无需处理。
                    }
                    Some(Ok(Message::Close(frame))) => {
                        log::info!(target: "oclive_oocp_ws", "client close frame={:?}", frame);
                        break;
                    }
                    Some(Err(e)) => {
                        log::warn!(target: "oclive_oocp_ws", "read error: {}", e);
                        break;
                    }
                    None => {
                        log::debug!(target: "oclive_oocp_ws", "stream ended");
                        break;
                    }
                }
            }

            // ── 服务端 ping 保活 ──
            _ = ping_tick.tick() => {
                if let Err(e) = writer.send(Message::Ping(vec![])).await {
                    log::warn!(target: "oclive_oocp_ws", "ping failed: {}", e);
                    break;
                }
                // 启动短超时等待 Pong（非阻塞：若下轮未收到 Pong 则自然断开）。
                // 简化实现：下一轮 tick 前若未收到 Pong，tokio-tungstenite 层会自行处理。
                let _ = PING_TIMEOUT_SECS; // 留给后续增强
            }
        }
    }
}

/// 处理单帧 JSON 文本。`None` 表示无需回复（空帧 / capabilities 重协商）。
async fn handle_text_frame(
    text: &str,
    handler: &mut TauriOocpHandler,
) -> Option<serde_json::Value> {
    // 空帧视为 capabilities 重协商。
    if text.trim().is_empty() {
        let caps = crate::domain::core::oocp_handler::get_capabilities();
        return Some(serde_json::to_value(&caps).unwrap_or(serde_json::Value::Null));
    }

    match serde_json::from_str::<crate::models::oocp::OocpRequest>(text) {
        Ok(mut req) => {
            // 若 type 字段缺失，补上默认值。
            if req.msg_type.is_empty() {
                req.msg_type = "request".to_string();
            }
            // 容忍 type 不是 "request" 的帧（某些客户端会省略）。
            if req.msg_type != "request" && !req.method.is_empty() {
                req.msg_type = "request".to_string();
            }
            let result = dispatch_oocp_request(req, handler).await;
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
        Err(e) => {
            // 无法解析为 OocpRequest，返回错误。
            let err = crate::models::oocp::OocpError {
                msg_type: "error",
                id: serde_json::Value::String("0".to_string()),
                error: crate::models::oocp::OocpErrorBody {
                    code: crate::models::oocp::OocpErrorCode::InvalidParams
                        .as_str()
                        .to_string(),
                    message: format!("无法解析请求 JSON: {}", e),
                    data: serde_json::Value::Null,
                },
            };
            Some(serde_json::to_value(&err).unwrap_or(serde_json::Value::Null))
        }
    }
}

/// 构造包含 OOCP WebSocket 路由的 axum Router。
///
/// 使用此函数替代直接在 `http_api::api_router` 内添加路由，
/// 以便清晰分离关注点。调用方需 `.with_state(app_state)` 传入 AppState。
pub fn oocp_ws_router() -> Router<Arc<AppState>> {
    // 注意：WebSocketUpgrade 不需要显式传入 state：
    // axum 自动从 Router state 中提取所需类型。
    Router::new().route("/oocp", get(oocp_ws_handler))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subtle_compare_same_strings() {
        assert!(subtle_compare("hello", "hello"));
    }

    #[test]
    fn subtle_compare_different_case() {
        assert!(!subtle_compare("Hello", "hello"));
    }

    #[test]
    fn subtle_compare_different_length() {
        assert!(!subtle_compare("ab", "abc"));
    }

    #[test]
    fn subtle_compare_empty() {
        assert!(subtle_compare("", ""));
    }

    #[test]
    fn verify_token_no_server_token_allows_all() {
        assert!(verify_token(&None, None));
        assert!(verify_token(&None, Some("anything")));
    }

    #[test]
    fn verify_token_mismatch_denied() {
        assert!(!verify_token(&Some("secret".to_string()), Some("wrong")));
    }

    #[test]
    fn verify_token_match_allowed() {
        assert!(verify_token(&Some("secret".to_string()), Some("secret")));
    }

    #[test]
    fn verify_token_missing_client_denied() {
        assert!(!verify_token(&Some("secret".to_string()), None));
    }
}
