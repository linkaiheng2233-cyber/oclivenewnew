//! Test-only bounded gateway; no production client is replaced or modified.
use crate::semantic_cases::{Case, LEGACY};
use crate::support::artifacts;
use axum::body::{to_bytes, Body};
use axum::extract::State;
use axum::http::{Method, Request, Response, StatusCode};
use axum::Router;
use futures_util::StreamExt;
use parking_lot::Mutex;
use serde_json::{json, Value};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;

pub const MODEL: &str = "qwen2.5:7b";
pub const VERSION: &str = "0.34.2";
pub const DIGEST: &str = "845dbda0ea48ed749caafd9e6037047aa19acfcfd82e704d7ca97d631a0b697e";
const CURRENT_S01_VERSION: &str = "0.34.4";
const CURRENT_S01_DIGEST: &str = "b6d9ecfeea2fdfe83d6f5cafb3646d3838a0c3c48a958d2ec0a2e333b38b431f";
pub const UPSTREAM: &str = "http://127.0.0.1:11434";
#[cfg(test)]
const MESSAGE: &str = "我不喝咖啡，只喝茶。请用一句中文复述我的饮品偏好，不要添加原因或建议。";
const REQUEST_LIMIT: usize = 256 * 1024;
const RESPONSE_LIMIT: usize = 1024 * 1024;

pub fn pinned_client(timeout: Duration) -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .retry(reqwest::retry::never())
        .http1_only()
        .timeout(timeout)
        .build()
        .map_err(|e| e.to_string())
}

async fn bounded_body(response: reqwest::Response) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| e.to_string())?;
        if bytes.len().saturating_add(chunk.len()) > RESPONSE_LIMIT {
            return Err("upstream response exceeds 1 MiB".into());
        }
        bytes.extend_from_slice(&chunk);
    }
    Ok(bytes)
}

pub fn check_identity(version: &Value, tags: &Value) -> Result<Value, String> {
    check_identity_with(version, tags, VERSION, DIGEST)
}

fn check_identity_with(
    version: &Value,
    tags: &Value,
    expected_version: &str,
    expected_digest: &str,
) -> Result<Value, String> {
    if version.get("version").and_then(Value::as_str) != Some(expected_version) {
        return Err("Ollama version differs from frozen approval".into());
    }
    let matches: Vec<&Value> = tags
        .get("models")
        .and_then(Value::as_array)
        .ok_or("missing models array")?
        .iter()
        .filter(|m| m.get("name").and_then(Value::as_str) == Some(MODEL))
        .collect();
    let [model] = matches.as_slice() else {
        return Err("approved model must occur exactly once".into());
    };
    if model.get("digest").and_then(Value::as_str) != Some(expected_digest) {
        return Err("model digest differs from frozen approval".into());
    }
    Ok(json!({"version": expected_version, "model": model, "endpoint": UPSTREAM}))
}

pub async fn preflight(case: &Case) -> Result<Value, String> {
    let client = pinned_client(Duration::from_secs(5))?;
    let mut results = Vec::new();
    for path in ["version", "tags"] {
        let response = client
            .get(format!("{UPSTREAM}/api/{path}"))
            .send()
            .await
            .map_err(|e| format!("preflight {path}: {e}"))?;
        if response.status() != reqwest::StatusCode::OK {
            return Err(format!("preflight {path}: HTTP {}", response.status()));
        }
        let bytes = bounded_body(response).await?;
        results.push(serde_json::from_slice::<Value>(&bytes).map_err(|e| e.to_string())?);
    }
    let mut iter = results.iter();
    let (expected_version, expected_digest) = if case.run_id.starts_with("B-SV2-")
        || case.run_id.starts_with("B-SV3-")
        || case.run_id.starts_with("B-SV4-")
    {
        (CURRENT_S01_VERSION, CURRENT_S01_DIGEST)
    } else {
        (VERSION, DIGEST)
    };
    check_identity_with(
        iter.next().ok_or("version result absent")?,
        iter.next().ok_or("tags result absent")?,
        expected_version,
        expected_digest,
    )
}

fn check_request_for_case(
    method: &str,
    uri: &str,
    v: &Value,
    sequence: usize,
    case: &Case,
) -> Result<(), String> {
    if method != "POST" || uri != "/api/generate" {
        return Err("only POST /api/generate without query is allowed".into());
    }
    if !(1..=3).contains(&sequence) {
        return Err("forward-attempt budget exceeded".into());
    }
    if v.get("model").and_then(Value::as_str) != Some(MODEL)
        || v.get("stream").and_then(Value::as_bool) != Some(false)
    {
        return Err("model/stream mismatch".into());
    }
    let prompt = v
        .get("prompt")
        .and_then(Value::as_str)
        .ok_or("missing prompt")?;
    if !prompt.contains(case.message) {
        return Err("approved user message absent from prompt".into());
    }
    if sequence > 1 && !prompt.contains(crate::support::REPAIR_PROMPT_MARKER) {
        return Err("additional request lacks the product repair marker".into());
    }
    if sequence == 1 && prompt.contains(crate::support::REPAIR_PROMPT_MARKER) {
        return Err("first request is unexpectedly a repair".into());
    }
    let opts = v.get("options").ok_or("missing options")?;
    if opts.get("num_predict").and_then(Value::as_u64) != Some(256)
        || opts.get("num_ctx").and_then(Value::as_u64) != Some(4096)
        || v.get("keep_alive").and_then(Value::as_str) != Some("30m")
    {
        return Err("token/context/residency binding mismatch".into());
    }
    for (key, expected) in [("temperature", 0.2), ("top_p", 0.9)] {
        if !opts
            .get(key)
            .and_then(Value::as_f64)
            .is_some_and(|x| (x - expected).abs() < 0.00001)
        {
            return Err(format!("sampling binding mismatch: {key}"));
        }
    }
    Ok(())
}

pub fn check_response(value: &Value) -> Result<(), String> {
    if value.get("model").and_then(Value::as_str) != Some(MODEL)
        || value.get("done").and_then(Value::as_bool) != Some(true)
        || value.get("response").and_then(Value::as_str).is_none()
    {
        return Err("provider model/done/text does not meet this normal-run gate".into());
    }
    Ok(())
}

struct Ledger {
    case: &'static Case,
    enabled: bool,
    closed: bool,
    busy: bool,
    received: usize,
    forwarded: usize,
    events: Vec<Value>,
    violations: Vec<String>,
}

impl Default for Ledger {
    fn default() -> Self {
        Self {
            case: &LEGACY,
            enabled: false,
            closed: false,
            busy: false,
            received: 0,
            forwarded: 0,
            events: Vec::new(),
            violations: Vec::new(),
        }
    }
}

impl Ledger {
    /// Used by the actual handler before any upstream send; rejection is sticky.
    fn admit(&mut self, method: &str, uri: &str, value: &Value) -> Result<usize, String> {
        let sequence = self.forwarded + 1;
        let denied = if !self.enabled || self.closed || self.busy {
            Some("request outside approved phase, after failure, or concurrently".to_string())
        } else {
            check_request_for_case(method, uri, value, sequence, self.case).err()
        };
        if let Some(reason) = denied {
            self.closed = true;
            self.violations.push(reason.clone());
            return Err(reason);
        }
        self.busy = true;
        self.forwarded += 1;
        Ok(sequence)
    }
}

struct Gateway {
    client: reqwest::Client,
    ledger: Mutex<Ledger>,
    path: PathBuf,
}

impl Gateway {
    fn facts(ledger: &Ledger) -> Value {
        json!({"semantic_case": ledger.case.identity(), "received_http_requests": ledger.received,
            "forward_attempts": ledger.forwarded, "busy": ledger.busy,
            "closed": ledger.closed, "violations": ledger.violations, "events": ledger.events,
            "method_entry_count": "NOT_OBSERVED", "model_inference_count": "NOT_OBSERVED",
            "scope": "this test gateway only; not OS traffic or server inference count"})
    }

    fn persist(&self, ledger: &mut Ledger) -> Result<(), String> {
        if let Err(error) = artifacts::write_json(&self.path, &Self::facts(ledger)) {
            ledger.closed = true;
            ledger
                .violations
                .push(format!("ledger write failed: {error}"));
            return Err(error);
        }
        Ok(())
    }

    fn reject(&self, reason: String) -> Response<Body> {
        let mut ledger = self.ledger.lock();
        ledger.closed = true;
        ledger.violations.push(reason.clone());
        let _ = self.persist(&mut ledger);
        reply(StatusCode::BAD_GATEWAY, reason.into_bytes())
    }
}

fn reply(status: StatusCode, bytes: Vec<u8>) -> Response<Body> {
    let mut response = Response::new(Body::from(bytes));
    *response.status_mut() = status;
    response.headers_mut().insert(
        axum::http::header::CONTENT_TYPE,
        axum::http::HeaderValue::from_static("application/json"),
    );
    response
}

async fn forward(State(gateway): State<Arc<Gateway>>, request: Request<Body>) -> Response<Body> {
    let method = request.method().clone();
    let uri = request.uri().to_string();
    {
        let mut ledger = gateway.ledger.lock();
        ledger.received += 1;
    }
    let bytes = match to_bytes(request.into_body(), REQUEST_LIMIT).await {
        Ok(b) => b,
        Err(e) => return gateway.reject(format!("request body: {e}")),
    };
    let value: Value = match serde_json::from_slice(&bytes) {
        Ok(v) => v,
        Err(e) => return gateway.reject(format!("request JSON: {e}")),
    };
    let sequence = {
        let mut ledger = gateway.ledger.lock();
        let sequence = ledger.forwarded + 1;
        let admission = ledger.admit(method.as_str(), &uri, &value);
        let denied = admission.as_ref().err();
        ledger
            .events
            .push(json!({"phase":"received", "sequence":sequence,
            "method":method.as_str(),"uri":uri,"body":value,
            "body_sha256":artifacts::sha256_hex(&bytes),"denied":denied}));
        if admission.is_err() {
            let _ = gateway.persist(&mut ledger);
            return reply(
                StatusCode::BAD_GATEWAY,
                b"test gateway rejected request".to_vec(),
            );
        }
        if let Err(e) = gateway.persist(&mut ledger) {
            ledger.busy = false;
            return reply(StatusCode::BAD_GATEWAY, e.into_bytes());
        }
        sequence
    };
    let result = async {
        let response = gateway
            .client
            .request(Method::POST, format!("{UPSTREAM}/api/generate"))
            .header("content-type", "application/json")
            .body(bytes)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        let status = response.status();
        let bytes = bounded_body(response).await?;
        Ok::<_, String>((status, bytes))
    }
    .await;
    let mut ledger = gateway.ledger.lock();
    ledger.busy = false;
    match result {
        Ok((status, bytes)) => {
            let decoded = serde_json::from_slice::<Value>(&bytes);
            let failure = if status != reqwest::StatusCode::OK {
                Some(format!("upstream HTTP {status}"))
            } else {
                match &decoded {
                    Ok(v) => check_response(v).err(),
                    Err(e) => Some(format!("upstream JSON: {e}")),
                }
            };
            ledger.events.push(json!({"phase":"response","sequence":sequence,
                "status":status.as_u16(),"body":String::from_utf8_lossy(&bytes),"validation_error":failure}));
            if let Some(reason) = failure {
                ledger.closed = true;
                ledger.violations.push(reason);
            }
            if let Err(e) = gateway.persist(&mut ledger) {
                return reply(StatusCode::BAD_GATEWAY, e.into_bytes());
            }
            // Never send a redirect or Location to the production client. For a 200 response,
            // preserve even malformed/done=false content so its normal product checks run.
            let downstream = if status.is_redirection() {
                StatusCode::BAD_GATEWAY
            } else {
                status
            };
            reply(downstream, bytes)
        }
        Err(e) => {
            ledger.closed = true;
            ledger.violations.push(format!("upstream transfer: {e}"));
            ledger
                .events
                .push(json!({"phase":"transfer_error","sequence":sequence,"error":e}));
            let _ = gateway.persist(&mut ledger);
            reply(
                StatusCode::BAD_GATEWAY,
                b"test gateway transfer failed".to_vec(),
            )
        }
    }
}

pub struct Proxy {
    pub endpoint: String,
    gateway: Arc<Gateway>,
    stop: Option<oneshot::Sender<()>>,
    task: JoinHandle<Result<(), std::io::Error>>,
}

impl Proxy {
    pub async fn start(path: PathBuf, case: &'static Case) -> Result<Self, String> {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .map_err(|e| e.to_string())?;
        let endpoint = format!(
            "http://{}",
            listener.local_addr().map_err(|e| e.to_string())?
        );
        let gateway = Arc::new(Gateway {
            client: pinned_client(Duration::from_secs(45))?,
            ledger: Mutex::new(Ledger {
                case,
                ..Default::default()
            }),
            path,
        });
        gateway.persist(&mut gateway.ledger.lock())?;
        let app = Router::new()
            .fallback(forward)
            .with_state(Arc::clone(&gateway));
        let (stop, rx) = oneshot::channel();
        let task = tokio::spawn(async move {
            axum::serve(listener, app)
                .with_graceful_shutdown(async {
                    let _ = rx.await;
                })
                .await
        });
        Ok(Self {
            endpoint,
            gateway,
            stop: Some(stop),
            task,
        })
    }

    pub fn enable(&self) -> Result<(), String> {
        let mut ledger = self.gateway.ledger.lock();
        if ledger.received != 0 || ledger.closed {
            return Err("unexpected request before turn".into());
        }
        ledger.enabled = true;
        self.gateway.persist(&mut ledger)
    }

    pub fn snapshot(&self) -> Value {
        Gateway::facts(&self.gateway.ledger.lock())
    }

    pub fn disable(&self) {
        self.gateway.ledger.lock().enabled = false;
    }

    pub async fn stop(&mut self) -> Result<(), String> {
        self.disable();
        if let Some(tx) = self.stop.take() {
            let _ = tx.send(());
        }
        match tokio::time::timeout(Duration::from_secs(2), &mut self.task).await {
            Ok(Ok(Ok(()))) => Ok(()),
            Ok(result) => Err(format!("gateway shutdown: {result:?}")),
            Err(_) => {
                self.task.abort();
                Err("gateway shutdown deadline; aborted owned task, not Ollama".into())
            }
        }
    }
}

impl Drop for Proxy {
    fn drop(&mut self) {
        self.task.abort();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check_request(method: &str, uri: &str, v: &Value, sequence: usize) -> Result<(), String> {
        check_request_for_case(method, uri, v, sequence, &LEGACY)
    }

    #[test]
    fn b_sv1_real_admission_is_case_bound_and_budget_is_per_ledger() {
        for case in &crate::semantic_cases::CASES {
            let mut ledger = Ledger {
                case,
                enabled: true,
                ..Default::default()
            };
            assert_eq!(Gateway::facts(&ledger)["semantic_case"], case.identity());
            for seq in 1..=3 {
                let mut value = request(seq);
                value["prompt"] = json!(if seq == 1 {
                    case.message.to_string()
                } else {
                    format!("{}\n{}", crate::support::REPAIR_PROMPT_MARKER, case.message)
                });
                assert_eq!(ledger.admit("POST", "/api/generate", &value), Ok(seq));
                ledger.busy = false;
            }
            assert!(ledger.admit("POST", "/api/generate", &request(4)).is_err());
            assert_eq!(ledger.forwarded, 3);
            assert!(ledger.closed);
        }
    }

    #[test]
    fn b_sv1_other_sample_is_rejected_before_forward_and_stays_closed() {
        let cases = &crate::semantic_cases::CASES;
        for (i, case) in cases.iter().enumerate() {
            let mut ledger = Ledger {
                case,
                enabled: true,
                ..Default::default()
            };
            let mut wrong = request(1);
            wrong["prompt"] = json!(cases[(i + 1) % cases.len()].message);
            assert!(ledger.admit("POST", "/api/generate", &wrong).is_err());
            wrong["prompt"] = json!(case.message);
            assert!(ledger.admit("POST", "/api/generate", &wrong).is_err());
            assert_eq!(ledger.forwarded, 0);
        }
    }

    fn request(sequence: usize) -> Value {
        json!({"model":MODEL,"stream":false,"keep_alive":"30m",
            "prompt": if sequence == 1 { MESSAGE.to_string() } else { format!("{}\n{MESSAGE}",crate::support::REPAIR_PROMPT_MARKER) },
            "options":{"temperature":0.2,"top_p":0.9,"num_predict":256,"num_ctx":4096}})
    }

    #[test]
    fn b_p2_identity_is_pinned() {
        let version = json!({"version":VERSION});
        let tags = json!({"models":[{"name":MODEL,"digest":DIGEST}]});
        assert!(check_identity(&version, &tags).is_ok());
        assert!(check_identity(&json!({"version":"other"}), &tags).is_err());
        assert!(check_identity(
            &version,
            &json!({"models":[{"name":MODEL,"digest":"other"}]})
        )
        .is_err());
    }

    #[test]
    fn current_s01_identity_does_not_relabel_the_frozen_sv1_provider() {
        let version = json!({"version":CURRENT_S01_VERSION});
        let tags = json!({"models":[{"name":MODEL,"digest":CURRENT_S01_DIGEST}]});
        assert!(
            check_identity_with(&version, &tags, CURRENT_S01_VERSION, CURRENT_S01_DIGEST).is_ok()
        );
        assert!(check_identity(&version, &tags).is_err());
        assert!(check_identity_with(
            &json!({"version":VERSION}),
            &tags,
            CURRENT_S01_VERSION,
            CURRENT_S01_DIGEST
        )
        .is_err());
        assert!(check_identity_with(
            &version,
            &json!({"models":[{"name":MODEL,"digest":DIGEST}]}),
            CURRENT_S01_VERSION,
            CURRENT_S01_DIGEST
        )
        .is_err());
    }

    #[test]
    fn b_p2_request_gate_rejects_changes_and_extra_calls() {
        for seq in 1..=3 {
            assert!(check_request("POST", "/api/generate", &request(seq), seq).is_ok());
        }
        for (method, uri) in [
            ("GET", "/api/generate"),
            ("POST", "/api/generate?x=1"),
            ("POST", "/api/pull"),
        ] {
            assert!(check_request(method, uri, &request(1), 1).is_err());
        }
        assert!(check_request("POST", "/api/generate", &request(4), 4).is_err());
        assert!(check_request("POST", "/api/generate", &request(1), 2).is_err());
        for (key, value) in [
            ("model", json!("wrong")),
            ("stream", json!(true)),
            ("prompt", json!("other")),
            ("keep_alive", json!("0")),
        ] {
            let mut v = request(1);
            v[key] = value;
            assert!(check_request("POST", "/api/generate", &v, 1).is_err());
        }
        for (key, value) in [
            ("num_predict", json!(257)),
            ("num_ctx", json!(8192)),
            ("temperature", json!(0.8)),
            ("top_p", Value::Null),
        ] {
            let mut v = request(1);
            v["options"][key] = value;
            assert!(check_request("POST", "/api/generate", &v, 1).is_err());
        }
    }

    #[test]
    fn b_p2_response_gate_does_not_confuse_empty_with_incomplete() {
        assert!(check_response(&json!({"model":MODEL,"done":true,"response":""})).is_ok());
        assert!(check_response(&json!({"model":MODEL,"done":false,"response":"partial"})).is_err());
        assert!(check_response(&json!({"model":MODEL,"done":true})).is_err());
    }

    #[test]
    fn b_p2_actual_admission_is_sticky_and_bounds_forward_attempts() {
        let mut ledger = Ledger {
            enabled: true,
            ..Default::default()
        };
        for sequence in 1..=3 {
            assert_eq!(
                ledger.admit("POST", "/api/generate", &request(sequence)),
                Ok(sequence)
            );
            assert_eq!(ledger.forwarded, sequence);
            // Simulate completion, not another admission rule implementation.
            ledger.busy = false;
        }
        assert!(ledger.admit("POST", "/api/generate", &request(4)).is_err());
        assert_eq!(ledger.forwarded, 3);
        assert!(ledger.closed);
        assert!(ledger.admit("POST", "/api/generate", &request(1)).is_err());
        assert_eq!(ledger.forwarded, 3);
    }

    #[test]
    fn b_p2_actual_admission_rejects_disabled_concurrent_and_invalid_without_forward() {
        for mut ledger in [
            Ledger::default(),
            Ledger {
                enabled: true,
                busy: true,
                ..Default::default()
            },
            Ledger {
                enabled: true,
                closed: true,
                ..Default::default()
            },
        ] {
            assert!(ledger.admit("POST", "/api/generate", &request(1)).is_err());
            assert_eq!(ledger.forwarded, 0);
            assert!(ledger.closed);
        }
        let mut ledger = Ledger {
            enabled: true,
            ..Default::default()
        };
        assert!(ledger.admit("GET", "/api/pull", &request(1)).is_err());
        assert!(ledger.admit("POST", "/api/generate", &request(1)).is_err());
        assert_eq!(ledger.forwarded, 0);
    }
}
