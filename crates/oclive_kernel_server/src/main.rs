use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::http::HeaderMap;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::Router;
use futures_util::{SinkExt, StreamExt};
use oclive_core::oocp::OocpRequest;
use oclive_core::oocp_handler::{
    dispatch_oocp_request, get_capabilities, MethodError, OocpHandled, OocpMethodHandler,
};
use serde_json::{json, Value};
use std::net::SocketAddr;
use std::sync::Arc;

#[derive(Clone)]
struct ServerConfig {
    auth_token: Option<String>,
}

#[derive(Clone)]
struct App {
    config: Arc<ServerConfig>,
}

fn read_token() -> Option<String> {
    std::env::var("OOCP_API_TOKEN")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

fn read_port() -> u16 {
    std::env::var("OOCP_API_PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(48888)
}

async fn health() -> &'static str {
    "ok"
}

fn extract_bearer(headers: &HeaderMap) -> Option<&str> {
    headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
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

async fn oocp_ws(
    ws: WebSocketUpgrade,
    State(app): State<App>,
    headers: HeaderMap,
) -> Result<Response, Response> {
    let server_token = &app.config.auth_token;
    if let Some(st) = server_token {
        let ct = extract_bearer(&headers);
        if ct.map(|t| subtle_compare(st, t)) != Some(true) {
            return Err((
                axum::http::StatusCode::UNAUTHORIZED,
                [(axum::http::header::WWW_AUTHENTICATE, "Bearer")],
                "OOCP auth failed",
            )
                .into_response());
        }
    }
    Ok(ws.on_upgrade(move |socket| handle_socket(socket, app)))
}

async fn handle_socket(socket: WebSocket, app: App) {
    let (mut writer, mut reader) = socket.split();

    let auth_required = app.config.auth_token.is_some();
    let caps = get_capabilities(auth_required, 8, 4096);
    if writer
        .send(Message::Text(
            serde_json::to_string(&caps).unwrap_or_else(|_| "{}".to_string()),
        ))
        .await
        .is_err()
    {
        return;
    }

    let mut handler = DummyKernelHandler::default();

    while let Some(msg) = reader.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                let req: Result<OocpRequest, _> = serde_json::from_str(&text);
                let frame = match req {
                    Ok(mut req) => {
                        if req.msg_type.is_empty() {
                            req.msg_type = "request".to_string();
                        }
                        match dispatch_oocp_request(req, &mut handler, &caps).await {
                            OocpHandled::Response(r) => serde_json::to_value(r).ok(),
                            OocpHandled::Error(e) => serde_json::to_value(e).ok(),
                            OocpHandled::Capabilities(c) => serde_json::to_value(c).ok(),
                        }
                    }
                    Err(e) => {
                        let err = json!({
                            "type": "error",
                            "id": Value::Null,
                            "error": { "code": "INVALID_PARAMS", "message": format!("invalid json: {}", e), "data": Value::Null }
                        });
                        Some(err)
                    }
                };

                if let Some(v) = frame {
                    let _ = writer.send(Message::Text(v.to_string())).await;
                }

                for ev in handler.drain_events() {
                    if let Ok(s) = serde_json::to_string(&ev) {
                        let _ = writer.send(Message::Text(s)).await;
                    }
                }
            }
            Ok(Message::Close(_)) | Err(_) => break,
            _ => {}
        }
    }
}

#[derive(Default)]
struct DummyKernelHandler {
    events: Vec<oclive_core::oocp::OocpEvent>,
}

impl DummyKernelHandler {
    fn drain_events(&mut self) -> Vec<oclive_core::oocp::OocpEvent> {
        std::mem::take(&mut self.events)
    }
}

fn err(code: oclive_core::oocp::OocpErrorCode, msg: impl Into<String>) -> MethodError {
    MethodError::new(code, msg)
}

impl OocpMethodHandler for DummyKernelHandler {
    async fn session_create(
        &mut self,
        role_id: &str,
        _session_id: Option<&str>,
        _scene_id: Option<&str>,
    ) -> Result<Value, MethodError> {
        Ok(json!({ "session_ns": format!("{role_id}__sess__default") }))
    }
    async fn session_destroy(&mut self, _session_ns: &str) -> Result<Value, MethodError> {
        Ok(json!({}))
    }
    async fn session_get_state(&mut self, _session_ns: &str) -> Result<Value, MethodError> {
        Ok(json!({}))
    }
    async fn session_switch_scene(
        &mut self,
        _session_ns: &str,
        _scene_id: &str,
    ) -> Result<Value, MethodError> {
        Ok(json!({}))
    }
    async fn session_switch_interaction_mode(
        &mut self,
        _session_ns: &str,
        _mode: &str,
    ) -> Result<Value, MethodError> {
        Ok(json!({}))
    }
    async fn session_export_chat_logs(
        &mut self,
        _session_ns: &str,
        _format: &str,
        _path: Option<&str>,
    ) -> Result<Value, MethodError> {
        Ok(json!({}))
    }
    async fn chat_send_message(
        &mut self,
        _session_ns: &str,
        _user_message: &str,
        _scene_id: Option<&str>,
    ) -> Result<Value, MethodError> {
        Err(err(
            oclive_core::oocp::OocpErrorCode::Internal,
            "kernel server not wired to full runtime yet",
        ))
    }
    async fn chat_generate_monologue(
        &mut self,
        _session_ns: &str,
        _context: Option<&str>,
    ) -> Result<Value, MethodError> {
        Ok(json!({ "monologue": "" }))
    }
    async fn role_list(&mut self) -> Result<Value, MethodError> {
        Ok(json!([]))
    }
    async fn role_get_info(&mut self, _role_id: &str) -> Result<Value, MethodError> {
        Ok(json!({}))
    }
    async fn role_set_remote_life(
        &mut self,
        _session_ns: &str,
        _enabled: bool,
    ) -> Result<Value, MethodError> {
        Ok(json!({}))
    }
    async fn time_get_state(&mut self) -> Result<Value, MethodError> {
        Ok(json!({}))
    }
    async fn time_jump(
        &mut self,
        _session_ns: &str,
        _target_time_ms: i64,
    ) -> Result<Value, MethodError> {
        Ok(json!({}))
    }
    async fn agent_call_mcp_tool(
        &mut self,
        _server_id: &str,
        _tool_name: &str,
        _arguments: Value,
    ) -> Result<Value, MethodError> {
        Err(err(
            oclive_core::oocp::OocpErrorCode::UnsupportedMethod,
            "not implemented",
        ))
    }
    fn push_event(&mut self, event: oclive_core::oocp::OocpEvent) {
        self.events.push(event);
    }
}

#[tokio::main]
async fn main() {
    let _ = env_logger::try_init();
    let port = read_port();
    let token = read_token();

    let app = App {
        config: Arc::new(ServerConfig { auth_token: token }),
    };

    let router = Router::new()
        .route("/health", get(health))
        .route("/oocp", get(oocp_ws))
        .with_state(app);

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    log::info!(target: "oclive_kernel_server", "listening http://{}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, router).await.unwrap();
}
