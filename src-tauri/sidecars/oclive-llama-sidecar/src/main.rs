use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    net::{SocketAddr, TcpListener as StdTcpListener},
    path::PathBuf,
    sync::Arc,
    time::Duration,
};
use tokio::{
    fs,
    io::AsyncWriteExt,
    net::TcpListener,
    process::Command,
    sync::Mutex,
    time::Instant,
};

#[derive(Clone)]
struct AppState {
    http: reqwest::Client,
    llama: Arc<LlamaSupervisor>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct DebugPluginConfig {
    /// Absolute or relative path to GGUF.
    model_path: Option<String>,
    /// Extra args forwarded to llama-server (optional).
    #[serde(default)]
    llama_args: Vec<String>,
}

const PLUGIN_ID: &str = "com.oclive.llama.local";
#[derive(Debug, Clone, Deserialize, serde::Serialize, Default)]
#[serde(rename_all = "camelCase")]
struct PluginConfigFile {
    /// Absolute path to GGUF (preferred), or path relative to `{app_data}`.
    #[serde(default)]
    model_path: Option<String>,
    /// Extra args forwarded to llama-server.
    #[serde(default)]
    llama_args: Option<String>,
}

fn app_data_dir() -> Option<PathBuf> {
    let v = std::env::var("OCLIVE_APP_DATA_DIR").ok()?;
    let t = v.trim();
    if t.is_empty() {
        None
    } else {
        Some(PathBuf::from(t))
    }
}

fn plugin_config_path() -> Option<PathBuf> {
    let ad = app_data_dir()?;
    Some(ad.join("plugin-data").join(PLUGIN_ID).join("config.json"))
}

fn models_dir() -> Option<PathBuf> {
    let ad = app_data_dir()?;
    Some(ad.join("models").join("gguf"))
}

async fn load_plugin_config_file() -> PluginConfigFile {
    let Some(p) = plugin_config_path() else {
        return PluginConfigFile::default();
    };
    let raw = fs::read_to_string(&p).await.unwrap_or_default();
    serde_json::from_str::<PluginConfigFile>(&raw).unwrap_or_default()
}

async fn save_plugin_config_file(cfg: &PluginConfigFile) -> Result<(), String> {
    let Some(p) = plugin_config_path() else {
        return Err("OCLIVE_APP_DATA_DIR not set; cannot persist config".to_string());
    };
    if let Some(parent) = p.parent() {
        fs::create_dir_all(parent)
            .await
            .map_err(|e| format!("create config dir: {}", e))?;
    }
    let raw = serde_json::to_string_pretty(cfg).map_err(|e| e.to_string())?;
    fs::write(&p, raw)
        .await
        .map_err(|e| format!("write config: {}", e))
}

fn load_debug_plugin_config() -> DebugPluginConfig {
    let raw = std::env::var("OCLIVE_DEBUG_PLUGIN_CONFIG").unwrap_or_default();
    let t = raw.trim();
    if t.is_empty() {
        return DebugPluginConfig::default();
    }
    serde_json::from_str::<DebugPluginConfig>(t).unwrap_or_default()
}

struct LlamaSupervisor {
    child: Mutex<Option<tokio::process::Child>>,
    base_url: Mutex<Option<String>>,
    config: Mutex<PluginConfigFile>,
}

impl LlamaSupervisor {
    async fn new() -> Self {
        let mut cfg = load_plugin_config_file().await;

        // Developer override: allow injecting a model path for `spawn_plugin_for_test`.
        let dbg = load_debug_plugin_config();
        if let Some(p) = dbg.model_path.as_ref().map(|s| s.trim()).filter(|s| !s.is_empty()) {
            cfg.model_path = Some(p.to_string());
        }
        if !dbg.llama_args.is_empty() {
            cfg.llama_args = Some(dbg.llama_args.join(" "));
        }

        Self {
            child: Mutex::new(None),
            base_url: Mutex::new(None),
            config: Mutex::new(cfg),
        }
    }

    async fn set_debug_config(&self, cfg: DebugPluginConfig) -> Result<(), String> {
        let mut cur = self.config.lock().await.clone();
        if let Some(p) = cfg.model_path.as_ref() {
            let t = p.trim();
            cur.model_path = if t.is_empty() { None } else { Some(t.to_string()) };
        }
        if !cfg.llama_args.is_empty() {
            cur.llama_args = Some(cfg.llama_args.join(" "));
        }
        save_plugin_config_file(&cur).await?;
        *self.config.lock().await = cur;
        self.stop().await;
        Ok(())
    }

    fn llama_server_exe_path() -> PathBuf {
        if cfg!(windows) {
            PathBuf::from("bin/llama-server.exe")
        } else {
            PathBuf::from("bin/llama-server")
        }
    }

    async fn is_running(&self) -> bool {
        let mut guard = self.child.lock().await;
        match guard.as_mut() {
            None => false,
            Some(ch) => match ch.try_wait() {
                Ok(Some(_)) => {
                    *guard = None;
                    false
                }
                Ok(None) => true,
                Err(_) => true,
            },
        }
    }

    async fn stop(&self) {
        let mut guard = self.child.lock().await;
        if let Some(mut ch) = guard.take() {
            let _ = ch.kill().await;
            let _ = ch.wait().await;
        }
        *self.base_url.lock().await = None;
    }

    fn pick_free_port() -> u16 {
        let l = StdTcpListener::bind("127.0.0.1:0").expect("bind free port");
        l.local_addr().expect("local_addr").port()
    }

    async fn wait_healthy(http: &reqwest::Client, base_url: &str) -> bool {
        let deadline = Instant::now() + Duration::from_secs(20);
        let url = format!("{}/v1/models", base_url.trim_end_matches('/'));
        while Instant::now() < deadline {
            let ok = http
                .get(&url)
                .timeout(Duration::from_millis(800))
                .send()
                .await
                .map(|r| r.status().is_success())
                .unwrap_or(false);
            if ok {
                return true;
            }
            tokio::time::sleep(Duration::from_millis(200)).await;
        }
        false
    }

    async fn ensure_running(&self, http: &reqwest::Client) -> Result<String, String> {
        if self.is_running().await {
            if let Some(u) = self.base_url.lock().await.clone() {
                return Ok(u);
            }
        }

        let cfg = self.config.lock().await.clone();
        let model_path = cfg
            .model_path
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .ok_or_else(|| "llama-server not configured: missing model_path".to_string())?;

        let exe = Self::llama_server_exe_path();
        if !exe.is_file() {
            return Err(format!("llama-server not found at {}", exe.display()));
        }

        let port = Self::pick_free_port();
        let base_url = format!("http://127.0.0.1:{}", port);

        let mut cmd = Command::new(exe);
        cmd.arg("--host")
            .arg("127.0.0.1")
            .arg("--port")
            .arg(port.to_string())
            .arg("--model")
            .arg(model_path);
        if let Some(extra) = cfg.llama_args.as_ref() {
            for a in extra.split_whitespace() {
                let t = a.trim();
                if !t.is_empty() {
                    cmd.arg(t);
                }
            }
        }
        cmd.stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::piped());

        let child = cmd.spawn().map_err(|e| format!("spawn llama-server: {}", e))?;
        *self.child.lock().await = Some(child);
        *self.base_url.lock().await = Some(base_url.clone());

        if !Self::wait_healthy(http, &base_url).await {
            self.stop().await;
            return Err("llama-server failed health check (v1/models)".to_string());
        }

        Ok(base_url)
    }
}

#[derive(Debug, Deserialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    id: Value,
    method: String,
    #[serde(default)]
    params: Value,
}

fn jsonrpc_ok(id: Value, result: Value) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": result
    })
}

fn jsonrpc_err(id: Value, code: i64, message: impl Into<String>) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": {
            "code": code,
            "message": message.into()
        }
    })
}

async fn healthz() -> &'static str {
    "ok"
}

async fn rpc_handler(
    State(state): State<Arc<AppState>>,
    Json(req): Json<JsonRpcRequest>,
) -> impl IntoResponse {
    if req.jsonrpc != "2.0" {
        return (StatusCode::OK, Json(jsonrpc_err(req.id, -32600, "invalid_request: jsonrpc must be 2.0")));
    }

    match req.method.as_str() {
        "rpc.discover" => (
            StatusCode::OK,
            Json(jsonrpc_ok(
                req.id,
                json!({
                    "methods": [
                      "llm.generate",
                      "llm.generate_tag",
                      "rpc.discover",
                      "llama.status",
                      "llama.stop",
                      "llama.set_debug_config",
                      "llama.list_models",
                      "llama.set_model",
                      "llama.download_model"
                      ,
                      "config_updated"
                    ]
                }),
            )),
        ),
        "llama.status" => {
            let running = state.llama.is_running().await;
            let base = state.llama.base_url.lock().await.clone().unwrap_or_default();
            (
                StatusCode::OK,
                Json(jsonrpc_ok(
                    req.id,
                    json!({
                      "running": running,
                      "base_url": base
                    }),
                )),
            )
        }
        "llama.stop" => {
            state.llama.stop().await;
            (StatusCode::OK, Json(jsonrpc_ok(req.id, json!({ "ok": true }))))
        }
        "llama.list_models" => {
            let dir = models_dir();
            let mut items: Vec<Value> = Vec::new();
            if let Some(d) = dir {
                if let Ok(mut rd) = std::fs::read_dir(&d) {
                    while let Some(Ok(ent)) = rd.next() {
                        let p = ent.path();
                        if p.is_file()
                            && p.extension()
                                .and_then(|x| x.to_str())
                                .map(|s| s.eq_ignore_ascii_case("gguf"))
                                .unwrap_or(false)
                        {
                            let name = p
                                .file_name()
                                .and_then(|x| x.to_str())
                                .unwrap_or("")
                                .to_string();
                            items.push(json!({
                              "name": name,
                              "path": p.to_string_lossy().to_string()
                            }));
                        }
                    }
                }
            }
            (StatusCode::OK, Json(jsonrpc_ok(req.id, json!({ "items": items }))))
        }
        "llama.set_model" => {
            let p = req
                .params
                .get("modelPath")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .trim()
                .to_string();
            if p.is_empty() {
                return (
                    StatusCode::OK,
                    Json(jsonrpc_err(req.id, -32602, "modelPath required")),
                );
            }
            let mut cfg = load_plugin_config_file().await;
            cfg.model_path = Some(p);
            match save_plugin_config_file(&cfg).await {
                Ok(()) => {
                    *state.llama.config.lock().await = cfg;
                    state.llama.stop().await;
                    (StatusCode::OK, Json(jsonrpc_ok(req.id, json!({ "ok": true }))))
                }
                Err(e) => (
                    StatusCode::OK,
                    Json(jsonrpc_err(req.id, -32603, format!("save config: {}", e))),
                ),
            }
        }
        "llama.download_model" => {
            let url = req
                .params
                .get("url")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .trim()
                .to_string();
            let sha = req
                .params
                .get("sha256")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .trim()
                .to_ascii_lowercase();
            if url.is_empty() || sha.is_empty() {
                return (
                    StatusCode::OK,
                    Json(jsonrpc_err(req.id, -32602, "url and sha256 required")),
                );
            }
            let file_name = req
                .params
                .get("fileName")
                .and_then(|v| v.as_str())
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| {
                    url.split('/')
                        .last()
                        .unwrap_or("model.gguf")
                        .split('?')
                        .next()
                        .unwrap_or("model.gguf")
                        .to_string()
                });
            let Some(dir) = models_dir() else {
                return (
                    StatusCode::OK,
                    Json(jsonrpc_err(
                        req.id,
                        -32603,
                        "OCLIVE_APP_DATA_DIR not set; cannot download",
                    )),
                );
            };
            if let Err(e) = fs::create_dir_all(&dir).await {
                return (
                    StatusCode::OK,
                    Json(jsonrpc_err(req.id, -32603, format!("create models dir: {}", e))),
                );
            }
            let dest = dir.join(&file_name);
            let tmp = dir.join(format!("{}.part", file_name));
            let res = async {
                let resp = state.http.get(&url).send().await.map_err(|e| e.to_string())?;
                if !resp.status().is_success() {
                    return Err(format!("http_status {}", resp.status()));
                }
                let mut hasher = Sha256::new();
                let mut f = fs::File::create(&tmp).await.map_err(|e| e.to_string())?;
                let mut stream = resp.bytes_stream();
                use futures_util::StreamExt;
                while let Some(chunk) = stream.next().await {
                    let bytes = chunk.map_err(|e| e.to_string())?;
                    hasher.update(&bytes);
                    f.write_all(&bytes).await.map_err(|e| e.to_string())?;
                }
                f.flush().await.map_err(|e| e.to_string())?;
                drop(f);
                let digest = hasher.finalize();
                let mut got = String::with_capacity(digest.len() * 2);
                for b in digest {
                    got.push_str(&format!("{:02x}", b));
                }
                if got != sha {
                    let _ = fs::remove_file(&tmp).await;
                    return Err(format!("sha256_mismatch expected={} got={}", sha, got));
                }
                if dest.exists() {
                    let _ = fs::remove_file(&dest).await;
                }
                fs::rename(&tmp, &dest).await.map_err(|e| e.to_string())?;
                Ok::<_, String>(dest.to_string_lossy().to_string())
            }
            .await;
            match res {
                Ok(path) => (
                    StatusCode::OK,
                    Json(jsonrpc_ok(req.id, json!({ "ok": true, "path": path }))),
                ),
                Err(e) => (StatusCode::OK, Json(jsonrpc_err(req.id, -32603, e))),
            }
        }
        "config_updated" => {
            let cfg_val = req.params.get("config").cloned().unwrap_or(Value::Null);
            let cfg: PluginConfigFile = serde_json::from_value(cfg_val).unwrap_or_default();
            // Persist to the same plugin-data config.json path if possible, so
            // this sidecar stays consistent even when started outside the host.
            let _ = save_plugin_config_file(&cfg).await;
            *state.llama.config.lock().await = cfg;
            state.llama.stop().await;
            (StatusCode::OK, Json(jsonrpc_ok(req.id, json!({ "ok": true }))))
        }
        "llama.set_debug_config" => {
            // For developer/debug panel use: update config for this process lifetime.
            let cfg: DebugPluginConfig = serde_json::from_value(req.params.clone()).unwrap_or_default();
            match state.llama.set_debug_config(cfg).await {
                Ok(()) => (StatusCode::OK, Json(jsonrpc_ok(req.id, json!({ "ok": true })))),
                Err(e) => (
                    StatusCode::OK,
                    Json(jsonrpc_err(req.id, -32603, format!("failed to set config: {}", e))),
                ),
            }
        }
        "llm.generate" => {
            let model = req
                .params
                .get("model")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            let prompt = req
                .params
                .get("prompt")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            // Prefer real llama-server if present/configured; fall back to stub for now.
            match state.llama.ensure_running(&state.http).await {
                Ok(base_url) => {
                    let url = format!("{}/v1/completions", base_url.trim_end_matches('/'));
                    let body = json!({
                      "model": model,
                      "prompt": prompt,
                      "temperature": 0.7,
                      "max_tokens": 512
                    });
                    let resp = state
                        .http
                        .post(url)
                        .json(&body)
                        .send()
                        .await;
                    let text = match resp {
                        Ok(r) => {
                            let v: Value = r.json().await.unwrap_or(Value::Null);
                            v.get("choices")
                                .and_then(|c| c.get(0))
                                .and_then(|c0| c0.get("text"))
                                .and_then(|t| t.as_str())
                                .map(|s| s.to_string())
                                .unwrap_or_else(|| format!("[llama-empty:{}] {}", model, prompt))
                        }
                        Err(e) => format!("[llama-error:{}] {}", model, e),
                    };
                    (StatusCode::OK, Json(jsonrpc_ok(req.id, json!({ "text": text }))))
                }
                Err(_e) => (
                    StatusCode::OK,
                    Json(jsonrpc_ok(
                        req.id,
                        json!({
                          "text": format!("[stub:{}] {}", model, prompt)
                        }),
                    )),
                ),
            }
        }
        "llm.generate_tag" => {
            let model = req
                .params
                .get("model")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            match state.llama.ensure_running(&state.http).await {
                Ok(base_url) => {
                    let url = format!("{}/v1/completions", base_url.trim_end_matches('/'));
                    let prompt = req
                        .params
                        .get("prompt")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    let body = json!({
                      "model": model,
                      "prompt": prompt,
                      "temperature": 0.0,
                      "max_tokens": 32
                    });
                    let resp = state.http.post(url).json(&body).send().await;
                    let text = match resp {
                        Ok(r) => {
                            let v: Value = r.json().await.unwrap_or(Value::Null);
                            v.get("choices")
                                .and_then(|c| c.get(0))
                                .and_then(|c0| c0.get("text"))
                                .and_then(|t| t.as_str())
                                .map(|s| s.trim().to_string())
                                .unwrap_or_else(|| "[llama-tag-empty] ok".to_string())
                        }
                        Err(_) => format!("[stub-tag:{}] ok", model),
                    };
                    (StatusCode::OK, Json(jsonrpc_ok(req.id, json!({ "text": text }))))
                }
                Err(_) => (
                    StatusCode::OK,
                    Json(jsonrpc_ok(
                        req.id,
                        json!({
                          "text": format!("[stub-tag:{}] ok", model)
                        }),
                    )),
                ),
            }
        }
        _ => (
            StatusCode::OK,
            Json(jsonrpc_err(req.id, -32601, format!("method_not_found: {}", req.method))),
        ),
    }
}

#[tokio::main]
async fn main() {
    let state = Arc::new(AppState {
        http: reqwest::Client::new(),
        llama: Arc::new(LlamaSupervisor::new().await),
    });
    let app = Router::new()
        .route("/healthz", get(healthz))
        .route("/rpc", post(rpc_handler))
        .with_state(state);

    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind 127.0.0.1:0");
    let addr: SocketAddr = listener.local_addr().expect("local_addr");

    // IMPORTANT: this stdout line is used by the host for the spawn handshake.
    println!("OCLIVE_READY http://127.0.0.1:{}/rpc", addr.port());

    axum::serve(listener, app).await.expect("serve");
}

