//! Per-role request queue + kernel health supervisor (Phase 3).

use axum::body::Body;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use dashmap::DashMap;
use oclive_kernel_runtime::{resolve_api_port, RUNTIME_API_VERSION};
use serde_json::Value;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

/// Default listen port for the per-role scheduler (distinct from kernel [`DEFAULT_API_PORT`] 8420).
const DEFAULT_SCHEDULER_PORT: u16 = 8430;

const ENV_KERNEL_UPSTREAM: &str = "OCLIVE_KERNEL_UPSTREAM";
const ENV_SCHEDULER_PORT: &str = "OCLIVE_SCHEDULER_PORT";

fn upstream_base() -> String {
    std::env::var(ENV_KERNEL_UPSTREAM)
        .ok()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| format!("http://127.0.0.1:{}", resolve_api_port(None)))
}

fn listen_port() -> u16 {
    std::env::var(ENV_SCHEDULER_PORT)
        .ok()
        .and_then(|s| s.parse().ok())
        .filter(|p| *p > 0)
        .unwrap_or(DEFAULT_SCHEDULER_PORT)
}

fn parse_upstream_host_port(upstream: &str) -> Option<(String, u16)> {
    let trimmed = upstream.trim().trim_end_matches('/');
    let (scheme, rest) = trimmed
        .strip_prefix("https://")
        .map(|r| ("https", r))
        .or_else(|| trimmed.strip_prefix("http://").map(|r| ("http", r)))?;
    if let Some((host, port_str)) = rest.rsplit_once(':') {
        if !host.is_empty() && !host.contains(']') {
            let port = port_str.parse().ok()?;
            return Some((host.to_string(), port));
        }
    }
    let default_port = if scheme == "https" { 443 } else { 80 };
    Some((rest.to_string(), default_port))
}

fn listen_equals_upstream(listen_host: &str, listen_port: u16, upstream: &str) -> bool {
    let Some((up_host, up_port)) = parse_upstream_host_port(upstream) else {
        return false;
    };
    listen_host == up_host && listen_port == up_port
}

#[derive(Clone)]
struct SchedulerState {
    upstream: String,
    client: reqwest::Client,
    role_queues: Arc<DashMap<String, Arc<Mutex<()>>>>,
}

impl SchedulerState {
    fn role_lock(&self, role_id: &str) -> Arc<Mutex<()>> {
        self.role_queues
            .entry(role_id.to_string())
            .or_insert_with(|| Arc::new(Mutex::new(())))
            .clone()
    }

    fn extract_role_id(body: &[u8]) -> String {
        serde_json::from_slice::<Value>(body)
            .ok()
            .and_then(|v| {
                v.get("role_id")
                    .or_else(|| v.get("role_path"))
                    .and_then(|x| x.as_str())
                    .map(str::to_string)
            })
            .unwrap_or_else(|| "__global__".to_string())
    }
}

async fn health(State(st): State<Arc<SchedulerState>>) -> impl IntoResponse {
    let url = format!("{}/health", st.upstream);
    let upstream_ok = st
        .client
        .get(&url)
        .timeout(Duration::from_secs(3))
        .send()
        .await
        .map(|r| r.status().is_success())
        .unwrap_or(false);
    let body = serde_json::json!({
        "ok": upstream_ok,
        "runtime_api_version": RUNTIME_API_VERSION,
        "scheduler": "oclive-runtimed",
        "upstream": st.upstream,
    });
    (
        if upstream_ok {
            StatusCode::OK
        } else {
            StatusCode::SERVICE_UNAVAILABLE
        },
        axum::Json(body),
    )
}

async fn proxy(
    State(st): State<Arc<SchedulerState>>,
    req: axum::http::Request<Body>,
) -> impl IntoResponse {
    let method = req.method().clone();
    let uri = req.uri().path().to_string();
    let query = req
        .uri()
        .query()
        .map(|q| format!("?{q}"))
        .unwrap_or_default();
    let headers = req.headers().clone();
    let body_bytes = match axum::body::to_bytes(req.into_body(), 8 * 1024 * 1024).await {
        Ok(b) => b,
        Err(e) => {
            return (StatusCode::BAD_REQUEST, format!("read body: {e}")).into_response();
        }
    };

    let role_key = if uri == "/chat" && method == axum::http::Method::POST {
        SchedulerState::extract_role_id(&body_bytes)
    } else {
        "__global__".to_string()
    };

    let lock = st.role_lock(&role_key);
    let _guard = lock.lock().await;

    let url = format!("{}{}{}", st.upstream, uri, query);
    let mut builder = st.client.request(method, &url).body(body_bytes.to_vec());
    for (k, v) in headers.iter() {
        if k != axum::http::header::HOST && k != axum::http::header::CONTENT_LENGTH {
            builder = builder.header(k, v);
        }
    }
    match builder.send().await {
        Ok(res) => {
            let status = res.status();
            let bytes = res.bytes().await.unwrap_or_default();
            (status, bytes).into_response()
        }
        Err(e) => (StatusCode::BAD_GATEWAY, format!("upstream error: {e}")).into_response(),
    }
}

async fn supervise_upstream(st: Arc<SchedulerState>) {
    let mut backoff = Duration::from_secs(2);
    loop {
        let url = format!("{}/health", st.upstream);
        let ok = st
            .client
            .get(&url)
            .timeout(Duration::from_secs(3))
            .send()
            .await
            .map(|r| r.status().is_success())
            .unwrap_or(false);
        if ok {
            backoff = Duration::from_secs(2);
        } else {
            tracing::warn!(
                target: "oclive_runtimed",
                upstream = %st.upstream,
                "kernel upstream unhealthy; clients should retry attach/spawn"
            );
            backoff = (backoff * 2).min(Duration::from_secs(60));
        }
        tokio::time::sleep(backoff).await;
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let upstream = upstream_base();
    let port = listen_port();
    let listen_host = "127.0.0.1";
    if listen_equals_upstream(listen_host, port, &upstream) {
        tracing::error!(
            target: "oclive_runtimed",
            listen = %format!("{listen_host}:{port}"),
            %upstream,
            "scheduler listen address equals kernel upstream (self-proxy loop); \
             set {ENV_SCHEDULER_PORT} or {ENV_KERNEL_UPSTREAM} to different values"
        );
        std::process::exit(2);
    }
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(120))
        .build()
        .expect("http client");
    let st = Arc::new(SchedulerState {
        upstream: upstream.clone(),
        client,
        role_queues: Arc::new(DashMap::new()),
    });
    let sup = Arc::clone(&st);
    tokio::spawn(async move {
        supervise_upstream(sup).await;
    });

    let app = Router::new()
        .route("/health", get(health))
        .fallback(proxy)
        .with_state(st);

    let addr = format!("{listen_host}:{port}");
    tracing::info!(
        target: "oclive_runtimed",
        %addr,
        %upstream,
        "scheduler listening"
    );
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("bind scheduler port");
    axum::serve(listener, app).await.expect("serve");
}
