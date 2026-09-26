//! CP-INT B9：**真实 Host 进程重启同库** 的崩溃恢复（C1 已完成再崩溃 / C2 在途崩溃）。
//!
//! 与 B7-R2 的区别：这里启动的是**独立 Host 进程**（`oclive-kernel-server --api --port N`，即桌面
//! `.setup` 所 spawn 的同一形状）并使用**磁盘 SQLite**；回合中途由测试杀死该进程、按原库路径重启，
//! 再用生产恢复桥（`KernelHttpClient::recover_message_via_http`）验证同身份恢复。
//!
//! 预算（本批冻结）：Host 启动 **4** 次（C1/C2 各 2）、同时在线 loopback 端点 **2**（Host + 假 provider）、
//! 业务路由 **4**（stream×2 + recover×2）、假 provider 调用 **2**（C1 完成 1 / C2 中断 1）、
//! `/health` 探测**单独记账**、真实模型/语音/外部网络 **0**、Host run ID **0**。
//!
//! 明确不主张：C2 的 HTTP `UNCONFIRMED` **不代表**磁盘收据已改写；崩溃前未提交的助手正文不可恢复。

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::too_many_lines
)]

mod common;

use axum::body::{Body, Bytes};
use axum::extract::State;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::Router;
use futures_util::StreamExt;
use oclive_kernel_types::error::AppError;
use oclive_kernel_types::models::dto::SendMessageRequest;
use oclivenewnew_tauri::kernel_attach::KernelHttpClient;
use oclivenewnew_tauri::kernel_lifecycle::KernelConnection;
use serde_json::{json, Map, Value};
use sha2::{Digest, Sha256};
use sqlx::Row;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Semaphore;

// ---- 冻结常量 ----
const C1_ATTEMPT: &str = "B9-CRASH-C1-R3";
const C2_ATTEMPT: &str = "B9-CRASH-C2-R3";
const RUN_GATE_ENV: &str = "OCLIVE_B9_LIVE";
const HOST_BIN_ENV: &str = "OCLIVE_B9_HOST_BIN";
const API_TOKEN: &str = "b9-isolated-local-token";
const FACTS_REL: &str = ".cursor/plans/cp-int-b9-live-facts-r3.json";
const NATIVE_EXIT_REL: &str = ".cursor/plans/cp-int-b9-logs/L0-native-exit-r3.json";
/// 证据包按 attempt 分目录：R1 曾把同一目录覆盖成自己的库，属本夹具缺陷，R2 起修正。
const EVIDENCE_TAG: &str = "r3";
const ROLE_ID: &str = "a-probe-role";
const MODEL_ID: &str = "a-probe-model:latest";
const MESSAGE: &str = "聊聊清晨的风景吧。";
const FIRST_TOKEN: &str = "清晨的风";
const REST_TOKEN: &str = "轻轻吹过树梢。";
const FULL_REPLY: &str = "清晨的风轻轻吹过树梢。";
/// R3（新身份）：与 R0/R1/R2 的 UUID 完全不同。R3 只补记录 C2 的 provider 调用基线，
/// 场景断言与 R2 逐条等价。
const C1_TURN_ID: &str = "6b1f4d02-8a37-4c19-b5e2-70d94f1c3a86";
const C2_TURN_ID: &str = "d4a72e15-3b60-4f8c-9e21-58c0b7d6f4a3";
const ROUTE_BUDGET: usize = 4;
const HOST_START_BUDGET: usize = 4;
const PROVIDER_CALL_BUDGET: usize = 2;
const SCENARIO_DEADLINE: Duration = Duration::from_secs(180);
const RECOVER_DEADLINE_C2: Duration = Duration::from_secs(45);
const HEALTH_BUDGET: Duration = Duration::from_secs(30);
const STREAM_DEADLINE: Duration = Duration::from_secs(120);
const KILL_DEADLINE: Duration = Duration::from_secs(15);
const CLEANUP_DEADLINE: Duration = Duration::from_secs(20);

const MANIFEST_JSON: &str = r#"{
  "id": "a-probe-role",
  "name": "A Probe Role",
  "version": "0.0.1",
  "author": "harness",
  "description": "CP-INT B9 synthetic role (non-production, isolated)",
  "ollama_model": "a-probe-model:latest",
  "default_personality": [0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5],
  "evolution": { "personality_source": "vector" },
  "scenes": [],
  "user_relations": { "friend": { "prompt_hint": "harness" } },
  "default_relation": "friend",
  "dev_only": false
}
"#;
const SETTINGS_JSON: &str = r#"{
  "schema_version": 1,
  "plugin_backends": {
    "llm": "ollama",
    "agent": "none",
    "memory": "builtin",
    "emotion": "builtin",
    "event": "builtin",
    "prompt": "builtin"
  }
}
"#;
const CONFIG_JSON: &str = r#"{
  "chat_storage": { "mirror": false }
}
"#;

// ---------------- 假 provider（Ollama `/api/generate`，记录型 + 门闩） ----------------

struct ProviderInner {
    calls: Mutex<Vec<Value>>,
    gate: Arc<Semaphore>,
    hold_next: AtomicBool,
    probes: AtomicUsize,
}

#[derive(Clone)]
struct Provider {
    inner: Arc<ProviderInner>,
}

impl Provider {
    fn new() -> Self {
        Self {
            inner: Arc::new(ProviderInner {
                calls: Mutex::new(Vec::new()),
                gate: Arc::new(Semaphore::new(0)),
                hold_next: AtomicBool::new(false),
                probes: AtomicUsize::new(0),
            }),
        }
    }

    fn call_count(&self) -> usize {
        self.inner.calls.lock().unwrap().len()
    }

    fn calls(&self) -> Vec<Value> {
        self.inner.calls.lock().unwrap().clone()
    }

    fn hold_next_call(&self) {
        self.inner.hold_next.store(true, Ordering::SeqCst);
    }

    fn release_gate(&self) {
        self.inner.gate.add_permits(1);
    }

    fn facts(&self, port: u16) -> Value {
        json!({
            "port": port,
            "call_count": self.call_count(),
            "calls": self.calls(),
            "probe_count": self.inner.probes.load(Ordering::SeqCst),
        })
    }
}

fn ndjson_line(text: &str, done: bool) -> Bytes {
    let payload = json!({
        "model": MODEL_ID,
        "created_at": "2026-09-26T00:00:00Z",
        "response": text,
        "done": done,
    });
    Bytes::from(format!("{payload}\n"))
}

/// `/api/generate`：记录每次调用；首个被 `hold_next_call()` 标记的调用在**首帧之后**停在门闩上。
async fn provider_generate(State(provider): State<Provider>, body: Bytes) -> Response {
    let parsed: Value = serde_json::from_slice(&body).unwrap_or(Value::Null);
    {
        let mut calls = provider.inner.calls.lock().unwrap();
        let seq = calls.len() + 1;
        calls.push(json!({
            "seq": seq,
            "method": "POST",
            "path": "/api/generate",
            "model": parsed.get("model").cloned().unwrap_or(Value::Null),
            "stream": parsed.get("stream").cloned().unwrap_or(Value::Null),
            "prompt_bytes": parsed.get("prompt").and_then(Value::as_str).map(str::len),
            "at_ms": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0),
        }));
    }
    let gated = provider.inner.hold_next.swap(false, Ordering::SeqCst);
    let chunks = vec![
        ndjson_line(FIRST_TOKEN, false),
        ndjson_line(REST_TOKEN, false),
        ndjson_line("", true),
    ];
    let gate = if gated {
        Some(Arc::clone(&provider.inner.gate))
    } else {
        None
    };
    let stream =
        futures_util::stream::unfold((0usize, chunks, gate), |(index, chunks, gate)| async move {
            if index >= chunks.len() {
                return None;
            }
            if index == 1 {
                if let Some(permit_gate) = &gate {
                    let _ = permit_gate.acquire().await;
                }
            }
            let chunk = chunks[index].clone();
            Some((
                Ok::<Bytes, std::io::Error>(chunk),
                (index + 1, chunks, gate),
            ))
        });
    let body = Body::from_stream(stream);
    (
        [(axum::http::header::CONTENT_TYPE, "application/x-ndjson")],
        body,
    )
        .into_response()
}

async fn provider_tags(State(provider): State<Provider>) -> Response {
    provider.inner.probes.fetch_add(1, Ordering::SeqCst);
    axum::Json(json!({ "models": [{ "name": MODEL_ID, "model": MODEL_ID }] })).into_response()
}

async fn provider_version(State(provider): State<Provider>) -> Response {
    provider.inner.probes.fetch_add(1, Ordering::SeqCst);
    axum::Json(json!({ "version": "0.0.0-b9-fake" })).into_response()
}

async fn start_provider() -> Result<(String, Provider, tokio::task::JoinHandle<()>), String> {
    let provider = Provider::new();
    let app = Router::new()
        .route("/api/generate", post(provider_generate))
        .route("/api/tags", get(provider_tags))
        .route("/api/version", get(provider_version))
        .with_state(provider.clone());
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .map_err(|e| format!("provider bind: {e}"))?;
    let addr = listener
        .local_addr()
        .map_err(|e| format!("provider local_addr: {e}"))?;
    let handle = tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });
    Ok((format!("http://{addr}"), provider, handle))
}

// ---------------- 夹具目录 ----------------

struct Fixture {
    /// 目录**不**交给 RAII：本批要在归档 SQLite 证据之后，才用 `safe_remove_tree` 自行删除。
    root: PathBuf,
}

impl Fixture {
    fn new() -> Result<Self, String> {
        let dir = tempfile::TempDir::new().map_err(|e| format!("TempDir: {e}"))?;
        let root = dir.keep();
        for sub in ["app-data", "roles", "models", "logs", "cwd"] {
            std::fs::create_dir_all(root.join(sub)).map_err(|e| format!("mkdir {sub}: {e}"))?;
        }
        let role_dir = root.join("roles").join(ROLE_ID);
        std::fs::create_dir_all(&role_dir).map_err(|e| format!("mkdir role: {e}"))?;
        for (name, body) in [
            ("manifest.json", MANIFEST_JSON),
            ("settings.json", SETTINGS_JSON),
            ("config.json", CONFIG_JSON),
        ] {
            std::fs::write(role_dir.join(name), body).map_err(|e| format!("write {name}: {e}"))?;
        }
        Ok(Self { root })
    }

    fn app_data(&self) -> PathBuf {
        self.root.join("app-data")
    }

    fn db_path(&self) -> PathBuf {
        self.app_data().join("app.db")
    }

    fn env_pairs(&self, port: u16, provider_url: &str) -> Vec<(String, String)> {
        let mut pairs: Vec<(String, String)> = Vec::new();
        for key in [
            "SystemRoot",
            "windir",
            "SystemDrive",
            "PATHEXT",
            "NUMBER_OF_PROCESSORS",
        ] {
            if let Ok(value) = std::env::var(key) {
                if !value.trim().is_empty() {
                    pairs.push((key.to_string(), value));
                }
            }
        }
        let system_root = pairs
            .iter()
            .find(|(k, _)| k == "SystemRoot")
            .map(|(_, v)| v.clone())
            .unwrap_or_else(|| "C:\\Windows".to_string());
        pairs.push((
            "PATH".to_string(),
            format!("{system_root}\\System32;{system_root}"),
        ));
        for (key, sub) in [
            ("TEMP", "tmp"),
            ("TMP", "tmp"),
            ("APPDATA", "appdata"),
            ("LOCALAPPDATA", "localappdata"),
            ("USERPROFILE", "profile"),
            ("ProgramData", "programdata"),
        ] {
            pairs.push((key.to_string(), self.root.join(sub).display().to_string()));
        }
        pairs.push(("RUST_BACKTRACE".to_string(), "1".to_string()));
        pairs.push(("RUST_LOG".to_string(), "info,oclive_chat=debug".to_string()));
        pairs.push((
            "OCLIVE_APP_DATA".to_string(),
            self.app_data().display().to_string(),
        ));
        pairs.push((
            "OCLIVE_ROLES_DIR".to_string(),
            self.root.join("roles").display().to_string(),
        ));
        pairs.push(("OCLIVE_API_PORT".to_string(), port.to_string()));
        pairs.push(("OCLIVE_API_TOKEN".to_string(), API_TOKEN.to_string()));
        pairs.push(("OLLAMA_BASE_URL".to_string(), provider_url.to_string()));
        pairs.push(("OCLIVE_LLM_BACKEND".to_string(), "ollama".to_string()));
        pairs.push(("OCLIVE_SKIP_LLM_STARTUP_PROBE".to_string(), "1".to_string()));
        pairs.push(("OCLIVE_PORTRAIT_EMOTION_LLM".to_string(), "0".to_string()));
        pairs.push(("OCLIVE_EVENT_IMPACT_LLM".to_string(), "0".to_string()));
        pairs.push(("OCLIVE_OLLAMA_PRELOAD".to_string(), "0".to_string()));
        pairs.push((
            "OCLIVE_CHAT_STORAGE_BACKEND".to_string(),
            "sqlite".to_string(),
        ));
        pairs
    }
}

fn host_binary() -> Result<PathBuf, String> {
    if let Ok(explicit) = std::env::var(HOST_BIN_ENV) {
        let path = PathBuf::from(explicit.trim());
        if path.is_file() {
            return Ok(path);
        }
        return Err(format!("{HOST_BIN_ENV} is not a file: {}", path.display()));
    }
    let exe = std::env::current_exe().map_err(|e| format!("current_exe: {e}"))?;
    let debug_dir = exe
        .parent()
        .and_then(Path::parent)
        .ok_or("cannot derive target/debug from current_exe")?;
    let name = if cfg!(windows) {
        "oclive-kernel-server.exe"
    } else {
        "oclive-kernel-server"
    };
    let candidate = debug_dir.join(name);
    if candidate.is_file() {
        Ok(candidate)
    } else {
        Err(format!("host binary not found: {}", candidate.display()))
    }
}

fn identity_of(path: &Path) -> Result<Value, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    let digest = Sha256::digest(&bytes);
    Ok(json!({
        "path": path.display().to_string(),
        "bytes": bytes.len(),
        "sha256": format!("{digest:x}"),
    }))
}

/// 归档一个目录下的 SQLite 文件（含 WAL/SHM）到证据包，返回逐文件清单。
fn archive_sqlite(source_dir: &Path, dest_dir: &Path) -> Result<Value, String> {
    std::fs::create_dir_all(dest_dir).map_err(|e| format!("evidence dir: {e}"))?;
    let mut entries: Vec<Value> = Vec::new();
    let mut copied = 0usize;
    for name in ["app.db", "app.db-wal", "app.db-shm"] {
        let from = source_dir.join(name);
        if !from.is_file() {
            continue;
        }
        let to = dest_dir.join(name);
        std::fs::copy(&from, &to).map_err(|e| format!("copy {name}: {e}"))?;
        let identity = identity_of(&to)?;
        entries.push(json!({
            "file": name,
            "bytes": identity["bytes"],
            "sha256": identity["sha256"],
            "source": from.display().to_string(),
        }));
        copied += 1;
    }
    Ok(json!({"dest": dest_dir.display().to_string(), "files_copied": copied, "entries": entries}))
}

/// 只在通过绝对路径/归属/无 reparse 检查后删除本批目录；否则保留现状并如实报告。
fn safe_remove_tree(path: &Path) -> Value {
    let absolute = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    let temp_root =
        std::fs::canonicalize(std::env::temp_dir()).unwrap_or_else(|_| std::env::temp_dir());
    let under_temp = absolute.starts_with(&temp_root);
    let leaf_ok = absolute
        .file_name()
        .and_then(|n| n.to_str())
        .is_some_and(|n| n.starts_with(".tmp"));
    let mut reparse = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&absolute) {
        for entry in entries.flatten() {
            if let Ok(meta) = entry.metadata() {
                if meta.file_type().is_symlink() {
                    reparse.push(entry.path().display().to_string());
                }
            }
        }
    }
    if !under_temp || !leaf_ok || !reparse.is_empty() {
        return json!({
            "path": absolute.display().to_string(),
            "deleted": false,
            "reason": format!("preconditions failed: under_temp={under_temp} leaf_ok={leaf_ok} reparse={reparse:?}"),
            "absent_after": !absolute.exists(),
        });
    }
    let deleted = std::fs::remove_dir_all(&absolute).is_ok();
    json!({
        "path": absolute.display().to_string(),
        "deleted": deleted,
        "under_temp": under_temp,
        "reparse_points": reparse,
        "absent_after": !absolute.exists(),
    })
}

// ---------------- Host 进程 ----------------

struct HostRun {
    child: Child,
    pid: u32,
    port: u16,
    stdout_log: PathBuf,
    stderr_log: PathBuf,
    health: Value,
}

/// RAII 守卫：任何提前返回路径都必须杀掉本夹具启动的 Host，绝不泄漏子进程
/// （R0 的 401 早退泄漏过一次，导致端口冲突与 runner 管道被占）。
impl Drop for HostRun {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// 端口必须无人服务，否则说明本夹具之前的 Host 未退出（身份冲突）。
async fn wait_port_free(port: u16) -> Result<(), String> {
    let addr = format!("127.0.0.1:{port}");
    let deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < deadline {
        match tokio::time::timeout(Duration::from_millis(300), TcpStream::connect(&addr)).await {
            Ok(Err(_)) | Err(_) => return Ok(()),
            Ok(Ok(_stream)) => tokio::time::sleep(Duration::from_millis(100)).await,
        }
    }
    Err(format!(
        "port {port} is still being served by a foreign process"
    ))
}

/// 本夹具启动的 Host 必须真正创建**本次**的数据库；否则视为端口/进程身份冲突。
async fn wait_for_db(db_path: &Path) -> Result<(), String> {
    let deadline = Instant::now() + Duration::from_secs(20);
    while Instant::now() < deadline {
        if db_path.is_file() {
            return Ok(());
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    Err(format!(
        "host never created the run-specific database: {}",
        db_path.display()
    ))
}

/// 子进程 stdout 里必须出现它自己绑定该端口的日志行。
fn listen_line_present(stdout_log: &Path, port: u16) -> bool {
    match std::fs::read_to_string(stdout_log) {
        Ok(text) => text.contains(&format!("127.0.0.1:{port}")),
        Err(_) => false,
    }
}

async fn wait_health(base: &str, client: &reqwest::Client) -> Result<Value, String> {
    let started = Instant::now();
    let mut probes: Vec<Value> = Vec::new();
    let mut last = String::from("<none>");
    while started.elapsed() < HEALTH_BUDGET {
        match client
            .get(format!("{base}/health"))
            .timeout(Duration::from_secs(3))
            .send()
            .await
        {
            Ok(response) => {
                let status = response.status().as_u16();
                let body = response.text().await.unwrap_or_default();
                let body_head: String = body.chars().take(80).collect();
                probes.push(json!({"at_ms": started.elapsed().as_millis() as u64, "status": status, "body_head": body_head}));
                if status == 200 {
                    return Ok(json!({
                        "probe_count": probes.len(),
                        "elapsed_ms": started.elapsed().as_millis() as u64,
                        "probes": probes,
                    }));
                }
                last = format!("status {status}");
            }
            Err(e) => last = e.to_string(),
        }
        tokio::time::sleep(Duration::from_millis(250)).await;
    }
    Err(format!(
        "host /health never returned 200 within {HEALTH_BUDGET:?} (last={last}, probes={})",
        probes.len()
    ))
}

async fn start_host(
    fixture: &Fixture,
    binary: &Path,
    port: u16,
    provider_url: &str,
    client: &reqwest::Client,
    label: &str,
) -> Result<HostRun, String> {
    wait_port_free(port).await?;
    let stdout_log = fixture
        .root
        .join("logs")
        .join(format!("host-{label}.stdout.log"));
    let stderr_log = fixture
        .root
        .join("logs")
        .join(format!("host-{label}.stderr.log"));
    let stdout_file = std::fs::File::create(&stdout_log).map_err(|e| format!("stdout log: {e}"))?;
    let stderr_file = std::fs::File::create(&stderr_log).map_err(|e| format!("stderr log: {e}"))?;
    let mut cmd = Command::new(binary);
    cmd.args(["--api", "--port", &port.to_string()]);
    cmd.current_dir(fixture.root.join("cwd"));
    // stdin=null：既不继承 runner 的管道句柄（R0 曾因此让 cargo 永不返回），也不等待交互输入。
    cmd.stdin(Stdio::null());
    cmd.stdout(Stdio::from(stdout_file));
    cmd.stderr(Stdio::from(stderr_file));
    cmd.env_clear();
    for (key, value) in fixture.env_pairs(port, provider_url) {
        cmd.env(key, value);
    }
    let child = cmd.spawn().map_err(|e| format!("spawn host: {e}"))?;
    let pid = child.id();
    let base = format!("http://127.0.0.1:{port}");
    let health = match wait_health(&base, client).await {
        Ok(health) => health,
        Err(failure) => {
            let mut run = HostRun {
                child,
                pid,
                port,
                stdout_log,
                stderr_log,
                health: json!({"failure": failure}),
            };
            let _ = kill_host(&mut run).await;
            return Err(format!("host did not become healthy: {failure}"));
        }
    };
    let mut run = HostRun {
        child,
        pid,
        port,
        stdout_log,
        stderr_log,
        health,
    };
    // 身份核验：本次数据库确实出现，且该子进程自己打印了监听行——否则拒绝相信这个端口。
    if let Err(error) = wait_for_db(&fixture.db_path()).await {
        let _ = kill_host(&mut run).await;
        return Err(error);
    }
    if !listen_line_present(&run.stdout_log, port) {
        let _ = kill_host(&mut run).await;
        return Err(format!(
            "child stdout has no listen line for 127.0.0.1:{port}; refusing to trust a foreign listener"
        ));
    }
    Ok(run)
}

struct KillFacts {
    pid: u32,
    waited_ms: u128,
    exit_code: Option<i32>,
    port_closed: bool,
    survivor: bool,
}

async fn kill_host(run: &mut HostRun) -> KillFacts {
    let started = Instant::now();
    let _ = run.child.kill();
    let mut exit_code = None;
    let mut survivor = true;
    while started.elapsed() < KILL_DEADLINE {
        match run.child.try_wait() {
            Ok(Some(status)) => {
                exit_code = status.code();
                survivor = false;
                break;
            }
            Ok(None) => {}
            Err(_) => {}
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    // 端口不再服务：连接应被拒绝/超时。
    let addr = format!("127.0.0.1:{}", run.port);
    let mut port_closed = false;
    let deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < deadline {
        match tokio::time::timeout(Duration::from_millis(400), TcpStream::connect(&addr)).await {
            Ok(Err(_)) => {
                port_closed = true;
                break;
            }
            Err(_) => {
                port_closed = true;
                break;
            }
            Ok(Ok(_stream)) => tokio::time::sleep(Duration::from_millis(100)).await,
        }
    }
    KillFacts {
        pid: run.pid,
        waited_ms: started.elapsed().as_millis(),
        exit_code,
        port_closed,
        survivor,
    }
}

// ---------------- 只读取证 ----------------

async fn read_pool(db_path: &Path) -> Result<sqlx::SqlitePool, String> {
    sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(
            sqlx::sqlite::SqliteConnectOptions::new()
                .filename(db_path)
                .read_only(true),
        )
        .await
        .map_err(|e| format!("open read-only pool: {e}"))
}

async fn database_list(db_path: &Path) -> Result<Vec<String>, String> {
    let pool = read_pool(db_path).await?;
    let rows = sqlx::query("PRAGMA database_list")
        .fetch_all(&pool)
        .await
        .map_err(|e| format!("PRAGMA database_list: {e}"))?;
    pool.close().await;
    Ok(rows
        .iter()
        .map(|r| r.get::<String, _>("file"))
        .collect::<Vec<_>>())
}

async fn snapshot(db_path: &Path) -> Result<Value, String> {
    let pool = read_pool(db_path).await?;
    let messages = sqlx::query(
        "SELECT id, session_id, turn_index, sender, content FROM chat_messages ORDER BY turn_index, sender",
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| format!("chat_messages: {e}"))?;
    let sessions = sqlx::query(
        "SELECT session_id, role_id, scene_id, message_count FROM chat_sessions ORDER BY session_id",
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| format!("chat_sessions: {e}"))?;
    let receipts = sqlx::query(
        "SELECT request_id, status, payload_sha256, user_message_id, assistant_message_id FROM chat_request_receipts ORDER BY request_id",
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| format!("chat_request_receipts: {e}"))?;
    pool.close().await;
    Ok(json!({
        "messages": messages.iter().map(|r| json!({
            "id": r.get::<String, _>("id"),
            "session_id": r.get::<String, _>("session_id"),
            "turn_index": r.get::<i64, _>("turn_index"),
            "sender": r.get::<String, _>("sender"),
            "content": r.get::<String, _>("content"),
        })).collect::<Vec<_>>(),
        "sessions": sessions.iter().map(|r| json!({
            "session_id": r.get::<String, _>("session_id"),
            "role_id": r.get::<String, _>("role_id"),
            "scene_id": r.get::<Option<String>, _>("scene_id"),
            "message_count": r.get::<Option<i64>, _>("message_count"),
        })).collect::<Vec<_>>(),
        "receipts": receipts.iter().map(|r| json!({
            "request_id": r.get::<String, _>("request_id"),
            "status": r.get::<String, _>("status"),
            "payload_sha256": r.get::<String, _>("payload_sha256"),
            "user_message_id": r.get::<Option<String>, _>("user_message_id"),
            "assistant_message_id": r.get::<Option<String>, _>("assistant_message_id"),
        })).collect::<Vec<_>>(),
    }))
}

async fn receipt_status(db_path: &Path, request_id: &str) -> Result<Option<String>, String> {
    let pool = read_pool(db_path).await?;
    let row = sqlx::query("SELECT status FROM chat_request_receipts WHERE request_id = ?")
        .bind(request_id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| format!("receipt status: {e}"))?;
    pool.close().await;
    Ok(row.map(|r| r.get::<String, _>("status")))
}

// ---------------- 客户端辅助 ----------------

fn stream_body(role_path: &Path, turn_id: &str, message: &str) -> Value {
    json!({
        "client_request_id": turn_id,
        "role_path": role_path.to_string_lossy(),
        "message": message,
        "scene_id": "default",
        "session_id": null,
        "adult": null,
    })
}

fn recover_request(turn_id: &str, message: &str) -> SendMessageRequest {
    SendMessageRequest {
        client_request_id: Some(turn_id.to_string()),
        role_id: ROLE_ID.to_string(),
        user_message: message.to_string(),
        scene_id: Some("default".to_string()),
        session_id: None,
        include_raw_reply: None,
        adult: None,
    }
}

fn token_of(block: &str) -> Option<String> {
    let mut event = "message";
    let mut data = String::new();
    for line in block.split('\n') {
        if let Some(rest) = line.strip_prefix("event:") {
            event = rest.trim();
        } else if let Some(rest) = line.strip_prefix("data:") {
            data.push_str(rest.trim());
        }
    }
    if event != "token" || data.is_empty() {
        return None;
    }
    serde_json::from_str::<Value>(&data)
        .ok()
        .and_then(|v| v.get("token").and_then(|t| t.as_str()).map(str::to_string))
}

/// 读取到第一个完整 token 帧即返回；`until_done` 为真时读到唯一 `done` 帧。
async fn read_stream(
    client: &reqwest::Client,
    url: &str,
    body: &Value,
    until_done: bool,
) -> Result<(u16, Option<Value>, String, String), String> {
    let response = client
        .post(url)
        .header("accept", "text/event-stream")
        .json(body)
        .send()
        .await
        .map_err(|e| format!("stream send: {e}"))?;
    let status = response.status().as_u16();
    let mut stream = response.bytes_stream();
    let mut buffer = String::new();
    let mut raw = String::new();
    let mut first_token: Option<String> = None;
    let mut done_dto: Option<Value> = None;
    let started = Instant::now();
    while let Some(chunk) = stream.next().await {
        if started.elapsed() > STREAM_DEADLINE {
            return Err(format!(
                "stream read exceeded {STREAM_DEADLINE:?} (status={status}, first_token_seen={})",
                first_token.is_some()
            ));
        }
        let chunk = chunk.map_err(|e| format!("stream chunk: {e}"))?;
        let text = String::from_utf8_lossy(&chunk).to_string();
        raw.push_str(&text);
        buffer.push_str(&text);
        while let Some(sep) = buffer.find("\n\n") {
            let block = buffer[..sep].to_string();
            buffer = buffer[sep + 2..].to_string();
            if first_token.is_none() {
                if let Some(token) = token_of(&block) {
                    first_token = Some(token);
                }
            }
            if block.contains("event: done") || block.contains("event:done") {
                if let Some(data) = block
                    .split('\n')
                    .find_map(|l| l.trim().strip_prefix("data:").map(str::trim))
                {
                    if let Ok(parsed) = serde_json::from_str::<Value>(data) {
                        done_dto = Some(parsed.get("data").cloned().unwrap_or(parsed));
                    }
                }
            }
        }
        if !until_done && first_token.is_some() {
            break;
        }
        if until_done && done_dto.is_some() {
            break;
        }
    }
    drop(stream);
    Ok((status, done_dto, raw, first_token.unwrap_or_default()))
}

// ---------------- 场景 ----------------

async fn scenario_c1(
    binary: &Path,
    port: u16,
    provider_url: &str,
    provider: &Provider,
    facts: &mut Map<String, Value>,
) -> Result<Value, String> {
    let started = Instant::now();
    let fixture = Fixture::new()?;
    let mut out = Map::new();
    out.insert("attempt".into(), json!(C1_ATTEMPT));
    out.insert("turn_id".into(), json!(C1_TURN_ID));
    out.insert("root".into(), json!(fixture.root.display().to_string()));
    out.insert(
        "db_path".into(),
        json!(fixture.db_path().display().to_string()),
    );
    facts.insert(
        "c1_temp_root".into(),
        json!(fixture.root.display().to_string()),
    );

    std::env::set_var("OCLIVE_API_TOKEN", API_TOKEN);
    // 生产客户端：`KernelConnection::new` 读取 OCLIVE_API_TOKEN，`http_client()` 自带鉴权头。
    // R0 用裸 reqwest 客户端导致 /chat/stream 被 401 拒绝，这里是修复点。
    let conn = KernelConnection::new(format!("http://127.0.0.1:{port}"), port);
    let client = conn.http_client();

    let mut host = start_host(&fixture, binary, port, provider_url, &client, "c1-first").await?;
    out.insert(
        "host_start_1".into(),
        json!({
            "pid": host.pid, "port": host.port,
            "args": ["--api", "--port", host.port.to_string()],
            "stdout_log": host.stdout_log.display().to_string(),
            "stderr_log": host.stderr_log.display().to_string(),
            "health": host.health.clone(),
        }),
    );
    let database_before = database_list(&fixture.db_path()).await?;
    out.insert("database_list_before".into(), json!(database_before));

    let role_path = fixture.root.join("roles").join(ROLE_ID);
    let url = format!("http://127.0.0.1:{port}/chat/stream");
    let body = stream_body(&role_path, C1_TURN_ID, MESSAGE);
    let (status, done, raw, first) = read_stream(&client, &url, &body, true)
        .await
        .map_err(|e| format!("C1 stream: {e}"))?;
    out.insert("stream_status".into(), json!(status));
    out.insert("stream_first_token".into(), json!(first));
    out.insert("stream_raw_bytes".into(), json!(raw.len()));
    if status != 200 {
        return Err(format!("C1 stream status {status}"));
    }
    let done = done.ok_or("C1: no done frame")?;
    out.insert("done_dto".into(), done.clone());

    let receipt_before_kill = receipt_status(&fixture.db_path(), C1_TURN_ID).await?;
    let db_before_kill = snapshot(&fixture.db_path()).await?;
    out.insert("receipt_before_kill".into(), json!(receipt_before_kill));
    out.insert("db_before_kill".into(), db_before_kill.clone());
    let provider_calls_before = provider.call_count();
    out.insert(
        "provider_calls_before_kill".into(),
        json!(provider_calls_before),
    );
    if provider_calls_before != 1 {
        return Err(format!(
            "C1: expected exactly 1 provider call for the turn, got {provider_calls_before}"
        ));
    }

    let kill = kill_host(&mut host).await;
    out.insert(
        "kill_1".into(),
        json!({
            "pid": kill.pid, "waited_ms": kill.waited_ms, "exit_code": kill.exit_code,
            "port_closed": kill.port_closed, "survivor": kill.survivor,
        }),
    );
    if kill.survivor || !kill.port_closed {
        return Err(format!(
            "C1: host survived the kill or port still serving (survivor={}, port_closed={})",
            kill.survivor, kill.port_closed
        ));
    }
    out.insert(
        "kill_1_completed_ms".into(),
        json!(started.elapsed().as_millis() as u64),
    );

    let mut host2 = start_host(&fixture, binary, port, provider_url, &client, "c1-restart").await?;
    out.insert(
        "host_start_2".into(),
        json!({
            "pid": host2.pid, "port": host2.port, "reason": "restart on the same database path",
            "health": host2.health.clone(),
        }),
    );
    let database_after = database_list(&fixture.db_path()).await?;
    out.insert("database_list_after".into(), json!(database_after));

    let recovered =
        KernelHttpClient::recover_message_via_http(&conn, &recover_request(C1_TURN_ID, MESSAGE))
            .await
            .map_err(|e| format!("C1 recover: {e:?}"))?;
    out.insert(
        "recover_dto".into(),
        json!({
            "reply": recovered.reply,
            "user_message_id": recovered.user_message_id,
            "assistant_message_id": recovered.assistant_message_id,
            "scene_id": recovered.scene_id,
            "reply_is_fallback": recovered.reply_is_fallback,
        }),
    );
    if recovered.reply != FULL_REPLY {
        return Err(format!(
            "C1: recovered reply {:?} != {FULL_REPLY:?}",
            recovered.reply
        ));
    }
    let db_after = snapshot(&fixture.db_path()).await?;
    if db_after != db_before_kill {
        return Err("C1: recovery changed the database".into());
    }
    if provider.call_count() != provider_calls_before {
        return Err("C1: recovery triggered a new provider call".into());
    }
    let done_reply = done.get("reply").and_then(Value::as_str).unwrap_or("");
    if done_reply != recovered.reply {
        return Err("C1: recovered reply differs from the pre-crash done reply".into());
    }
    if done.get("assistant_message_id").and_then(Value::as_str)
        != recovered.assistant_message_id.as_deref()
    {
        return Err("C1: recovered assistant id differs from the pre-crash done id".into());
    }
    out.insert("db_after_recovery".into(), db_after);
    out.insert(
        "provider_calls_after_recovery".into(),
        json!(provider.call_count()),
    );
    out.insert(
        "scenario_elapsed_ms".into(),
        json!(started.elapsed().as_millis() as u64),
    );
    out.insert("ok".into(), json!(true));

    let kill2 = kill_host(&mut host2).await;
    out.insert(
        "kill_2".into(),
        json!({"pid": kill2.pid, "waited_ms": kill2.waited_ms, "exit_code": kill2.exit_code,
               "port_closed": kill2.port_closed, "survivor": kill2.survivor}),
    );
    Ok(Value::Object(out))
}

async fn scenario_c2(
    binary: &Path,
    port: u16,
    provider_url: &str,
    provider: &Provider,
    facts: &mut Map<String, Value>,
) -> Result<Value, String> {
    let started = Instant::now();
    let fixture = Fixture::new()?;
    let mut out = Map::new();
    out.insert("attempt".into(), json!(C2_ATTEMPT));
    out.insert("turn_id".into(), json!(C2_TURN_ID));
    out.insert("root".into(), json!(fixture.root.display().to_string()));
    out.insert(
        "db_path".into(),
        json!(fixture.db_path().display().to_string()),
    );
    facts.insert(
        "c2_temp_root".into(),
        json!(fixture.root.display().to_string()),
    );

    std::env::set_var("OCLIVE_API_TOKEN", API_TOKEN);
    // 与 C1 相同的生产客户端（自带 x-oclive-api-token 头）。
    let conn = KernelConnection::new(format!("http://127.0.0.1:{port}"), port);
    let client = conn.http_client();

    let calls_before = provider.call_count();
    provider.hold_next_call();
    let mut host = start_host(&fixture, binary, port, provider_url, &client, "c2-first").await?;
    out.insert(
        "host_start_1".into(),
        json!({"pid": host.pid, "port": host.port, "health": host.health.clone()}),
    );
    let database_before = database_list(&fixture.db_path()).await?;
    out.insert("database_list_before".into(), json!(database_before));

    let role_path = fixture.root.join("roles").join(ROLE_ID);
    let url = format!("http://127.0.0.1:{port}/chat/stream");
    let body = stream_body(&role_path, C2_TURN_ID, MESSAGE);
    let (status, _done, raw, first) = read_stream(&client, &url, &body, false)
        .await
        .map_err(|e| format!("C2 stream: {e}"))?;
    out.insert("stream_status".into(), json!(status));
    out.insert("stream_first_token".into(), json!(first));
    out.insert("stream_raw_bytes".into(), json!(raw.len()));
    out.insert("client_dropped_body".into(), json!(true));
    if status != 200 || first != FIRST_TOKEN {
        return Err(format!(
            "C2: expected status 200 and first token {FIRST_TOKEN:?}"
        ));
    }
    if provider.call_count() != calls_before + 1 {
        return Err("C2: the gated provider call was not the only new call".into());
    }
    // 记录"kill 前"的 provider 调用数：R2 只断言了 calls_before + 1 而没有落盘，
    // 使只读核验无法从事实文件复核"恢复未触发新生成"；R3 起该值即被记录又被断言。
    let provider_calls_before_kill = provider.call_count();
    out.insert(
        "provider_calls_before_kill".into(),
        json!(provider_calls_before_kill),
    );

    let receipt_before_kill = receipt_status(&fixture.db_path(), C2_TURN_ID).await?;
    let db_before_kill = snapshot(&fixture.db_path()).await?;
    out.insert("receipt_before_kill".into(), json!(receipt_before_kill));
    out.insert("db_before_kill".into(), db_before_kill.clone());
    if receipt_before_kill.as_deref() != Some("running") {
        return Err(format!(
            "C2: receipt was expected to be running before the kill, got {receipt_before_kill:?}"
        ));
    }

    let kill = kill_host(&mut host).await;
    out.insert(
        "kill_1".into(),
        json!({"pid": kill.pid, "waited_ms": kill.waited_ms, "exit_code": kill.exit_code,
               "port_closed": kill.port_closed, "survivor": kill.survivor}),
    );
    if kill.survivor || !kill.port_closed {
        return Err("C2: host survived the kill or port still serving".into());
    }
    // 释放门闩，让被中断的 provider 流结束（Host 已死，写入无接收方）。
    provider.release_gate();
    out.insert("gate_released_after_kill".into(), json!(true));

    let mut host2 = start_host(&fixture, binary, port, provider_url, &client, "c2-restart").await?;
    out.insert(
        "host_start_2".into(),
        json!({"pid": host2.pid, "port": host2.port, "reason": "restart on the same database path",
               "health": host2.health.clone()}),
    );
    let database_after = database_list(&fixture.db_path()).await?;
    out.insert("database_list_after".into(), json!(database_after));

    let recover_started = Instant::now();
    let outcome = tokio::time::timeout(
        RECOVER_DEADLINE_C2,
        KernelHttpClient::recover_message_via_http(&conn, &recover_request(C2_TURN_ID, MESSAGE)),
    )
    .await
    .map_err(|_| "C2: recover exceeded its own deadline".to_string())?;
    out.insert(
        "recover_result".into(),
        match &outcome {
            Ok(_) => json!("200_OK"),
            Err(AppError::ChatRequestUnconfirmed) => json!("CHAT_REQUEST_UNCONFIRMED"),
            Err(AppError::ChatRequestConflict) => json!("CHAT_REQUEST_CONFLICT"),
            Err(other) => json!(format!("{other:?}")),
        },
    );
    out.insert(
        "recover_elapsed_ms".into(),
        json!(recover_started.elapsed().as_millis() as u64),
    );
    match &outcome {
        Err(AppError::ChatRequestUnconfirmed) => {}
        other => {
            return Err(format!(
                "C2: mid-turn crash recovery must map to CHAT_REQUEST_UNCONFIRMED, got {other:?}"
            ))
        }
    }

    let db_after = snapshot(&fixture.db_path()).await?;
    let receipt_after = receipt_status(&fixture.db_path(), C2_TURN_ID).await?;
    out.insert("receipt_after_recovery".into(), json!(receipt_after));
    out.insert("db_after_recovery".into(), db_after.clone());
    out.insert(
        "provider_calls_after_recovery".into(),
        json!(provider.call_count()),
    );
    if provider.call_count() != provider_calls_before_kill {
        return Err("C2: recovery re-triggered generation".into());
    }
    let before_messages = db_before_kill["messages"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    let after_messages = db_after["messages"].as_array().cloned().unwrap_or_default();
    let assistant_after = after_messages
        .iter()
        .filter(|m| m["sender"] == "assistant")
        .count();
    out.insert(
        "assistant_rows_before_kill".into(),
        json!(before_messages
            .iter()
            .filter(|m| m["sender"] == "assistant")
            .count()),
    );
    out.insert(
        "assistant_rows_after_recovery".into(),
        json!(assistant_after),
    );
    if assistant_after != 0 {
        return Err("C2: an assistant row appeared after a mid-turn crash".into());
    }
    if after_messages.len() < before_messages.len() {
        return Err("C2: rows disappeared across the crash".into());
    }
    out.insert(
        "side_effect_note".into(),
        json!("terminal UNCONFIRMED does not rewrite the disk receipt; the receipt status is recorded verbatim both before and after"),
    );
    out.insert(
        "scenario_elapsed_ms".into(),
        json!(started.elapsed().as_millis() as u64),
    );
    out.insert("ok".into(), json!(true));

    let kill2 = kill_host(&mut host2).await;
    out.insert(
        "kill_2".into(),
        json!({"pid": kill2.pid, "waited_ms": kill2.waited_ms, "exit_code": kill2.exit_code,
               "port_closed": kill2.port_closed, "survivor": kill2.survivor}),
    );
    Ok(Value::Object(out))
}

// ---------------- 入口 ----------------

#[test]
fn b9_declared_budget_and_gate() {
    assert_eq!(ROUTE_BUDGET, 4, "stream x2 + recover x2");
    assert_eq!(
        HOST_START_BUDGET, 4,
        "C1 and C2 each start and restart the host"
    );
    assert_eq!(
        PROVIDER_CALL_BUDGET, 2,
        "C1 completes one call, C2 interrupts one"
    );
    assert_eq!(RUN_GATE_ENV, "OCLIVE_B9_LIVE");
    assert!(FACTS_REL.ends_with("cp-int-b9-live-facts-r3.json"));
    assert_eq!(C1_ATTEMPT, "B9-CRASH-C1-R3");
    assert_eq!(C2_ATTEMPT, "B9-CRASH-C2-R3");
    for id in [C1_TURN_ID, C2_TURN_ID] {
        let parsed = uuid::Uuid::parse_str(id).expect("canonical uuid");
        assert!(!parsed.is_nil());
        assert_eq!(parsed.to_string(), id);
    }
    assert_ne!(C1_TURN_ID, C2_TURN_ID);
    assert!(RECOVER_DEADLINE_C2 >= Duration::from_secs(40));
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "B9 live crash-restart: starts real Host processes and binds loopback ports; run with OCLIVE_B9_LIVE=1 cargo test -p oclivenewnew-tauri --test crash_restart_recovery -- --ignored --exact b9_real_host_restart_same_database --nocapture"]
async fn b9_real_host_restart_same_database() {
    match run_live().await {
        Ok(summary) => println!("B9_FACTS_JSON={summary}"),
        Err(failure) => panic!("{failure}"),
    }
}

async fn run_live() -> Result<String, String> {
    if std::env::var(RUN_GATE_ENV).as_deref() != Ok("1") {
        return Err(format!(
            "explicit run gate missing: set {RUN_GATE_ENV}=1 to allow one live crash-restart run"
        ));
    }
    let repo = common::monorepo_root();
    let facts_path = repo.join(FACTS_REL.replace('/', std::path::MAIN_SEPARATOR_STR));
    if facts_path.exists() {
        return Err(format!(
            "refusing to overwrite the previous attempt's facts: {}",
            facts_path.display()
        ));
    }
    let binary = host_binary()?;
    let binary_identity = identity_of(&binary)?;

    let mut facts: Map<String, Value> = Map::new();
    facts.insert(
        "batch".into(),
        json!("CP-INT B9 real host restart same database"),
    );
    facts.insert(
        "attempts".into(),
        json!({"c1": C1_ATTEMPT, "c2": C2_ATTEMPT}),
    );
    facts.insert("host_binary".into(), binary_identity.clone());
    facts.insert("facts_path".into(), json!(facts_path.display().to_string()));
    facts.insert(
        "budgets".into(),
        json!({
            "host_starts": HOST_START_BUDGET,
            "loopback_endpoints_concurrent_max": 2,
            "business_routes": ROUTE_BUDGET,
            "provider_calls": PROVIDER_CALL_BUDGET,
            "real_model_calls": 0,
            "host_run_ids": 0,
            "scenario_deadline_ms": SCENARIO_DEADLINE.as_millis() as u64,
            "c2_recover_deadline_ms": RECOVER_DEADLINE_C2.as_millis() as u64,
        }),
    );

    let (provider_url, provider, provider_task) = start_provider().await?;
    let provider_port: u16 = provider_url
        .rsplit(':')
        .next()
        .and_then(|p| p.parse().ok())
        .unwrap_or(0);

    // Host 端口：绑定 0 取空闲端口后立即释放，四个启动复用同一端口号。
    let host_port = {
        let probe = TcpListener::bind("127.0.0.1:0")
            .await
            .map_err(|e| format!("probe bind: {e}"))?;
        let port = probe.local_addr().map_err(|e| e.to_string())?.port();
        drop(probe);
        port
    };
    facts.insert(
        "endpoints".into(),
        json!({
            "provider": {"url": provider_url, "port": provider_port},
            "host": {"port": host_port, "note": "same port reused by all four host starts"},
            "concurrent_max": 2,
        }),
    );

    let c1 = tokio::time::timeout(
        SCENARIO_DEADLINE,
        scenario_c1(&binary, host_port, &provider_url, &provider, &mut facts),
    )
    .await;
    let c1_result = match c1 {
        Ok(inner) => inner,
        Err(_) => Err(format!(
            "C1 exceeded the scenario deadline {SCENARIO_DEADLINE:?}"
        )),
    };
    facts.insert(
        "c1".into(),
        match &c1_result {
            Ok(value) => value.clone(),
            Err(error) => json!({"attempt": C1_ATTEMPT, "ok": false, "failure": error}),
        },
    );

    let c2 = tokio::time::timeout(
        SCENARIO_DEADLINE,
        scenario_c2(&binary, host_port, &provider_url, &provider, &mut facts),
    )
    .await;
    let c2_result = match c2 {
        Ok(inner) => inner,
        Err(_) => Err(format!(
            "C2 exceeded the scenario deadline {SCENARIO_DEADLINE:?}"
        )),
    };
    facts.insert(
        "c2".into(),
        match &c2_result {
            Ok(value) => value.clone(),
            Err(error) => json!({"attempt": C2_ATTEMPT, "ok": false, "failure": error}),
        },
    );

    // 提供者与清理（有界；只停本夹具拥有的任务）。
    facts.insert("provider".into(), provider.facts(provider_port));
    provider_task.abort();
    let _ = tokio::time::timeout(Duration::from_secs(5), provider_task).await;
    std::env::remove_var("OCLIVE_API_TOKEN");

    let cleanup_started = Instant::now();
    let evidence_dir = repo.join(".cursor/plans/cp-int-b9-evidence");
    let mut archived: Vec<Value> = Vec::new();
    let mut removed: Vec<Value> = Vec::new();
    for (key, scenario) in [("c1_temp_root", "c1"), ("c2_temp_root", "c2")] {
        let Some(root) = facts.get(key).and_then(Value::as_str).map(PathBuf::from) else {
            continue;
        };
        let app_data = root.join("app-data");
        let dest = evidence_dir.join(EVIDENCE_TAG).join(scenario);
        match archive_sqlite(&app_data, &dest) {
            Ok(report) => archived.push(json!({"scenario": scenario, "report": report})),
            Err(error) => archived.push(json!({"scenario": scenario, "error": error})),
        }
        if let Some(logs) = facts
            .get(scenario)
            .and_then(|v| v.get("host_start_1"))
            .and_then(|v| v.get("stdout_log"))
            .and_then(Value::as_str)
        {
            let from = PathBuf::from(logs);
            if let Some(name) = from.file_name() {
                let to = dest.join(name);
                let _ = std::fs::copy(&from, &to);
            }
        }
        if let Some(logs) = facts
            .get(scenario)
            .and_then(|v| v.get("host_start_1"))
            .and_then(|v| v.get("stderr_log"))
            .and_then(Value::as_str)
        {
            let from = PathBuf::from(logs);
            if let Some(name) = from.file_name() {
                let to = dest.join(name);
                let _ = std::fs::copy(&from, &to);
            }
        }
        removed.push(json!({"scenario": scenario, "removal": safe_remove_tree(&root)}));
    }
    let evidence_entries = archived
        .iter()
        .filter_map(|entry| entry.get("report").and_then(|r| r.get("entries")).cloned())
        .flat_map(|entries| entries.as_array().cloned().unwrap_or_default())
        .collect::<Vec<Value>>();
    facts.insert(
        "cleanup".into(),
        json!({
            "deadline_ms": CLEANUP_DEADLINE.as_millis() as u64,
            "elapsed_ms": cleanup_started.elapsed().as_millis() as u64,
            "evidence_dir": evidence_dir.display().to_string(),
            "archived_sqlite": archived,
            "evidence_file_count": evidence_entries.len(),
            "temp_dir_removal": removed,
            "provider_task_aborted": true,
            "api_token_env_removed": std::env::var("OCLIVE_API_TOKEN").is_err(),
        }),
    );

    let both_ok = c1_result.is_ok() && c2_result.is_ok();
    facts.insert("failed".into(), json!(!both_ok));
    if let Err(error) = &c1_result {
        facts.insert("c1_failure".into(), json!(error));
    }
    if let Err(error) = &c2_result {
        facts.insert("c2_failure".into(), json!(error));
    }
    let written = serde_json::to_string_pretty(&Value::Object(facts.clone()))
        .map_err(|e| format!("facts encode: {e}"))?;
    std::fs::write(&facts_path, written.as_bytes()).map_err(|e| format!("facts write: {e}"))?;

    // native 退出件（供只读核验器使用）。
    let native_path = repo.join(NATIVE_EXIT_REL.replace('/', std::path::MAIN_SEPARATOR_STR));
    if let Some(parent) = native_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("native log dir: {e}"))?;
    }
    std::fs::write(
        &native_path,
        serde_json::to_string_pretty(&json!({
            "batch": "CP-INT B9",
            "attempts": {"c1": C1_ATTEMPT, "c2": C2_ATTEMPT},
            "green": both_ok,
            "facts_path": facts_path.display().to_string(),
            "facts_bytes": written.len(),
            "note": "written by the live test itself; the process exit code is recorded separately by the runner",
        }))
        .map_err(|e| e.to_string())?,
    )
    .map_err(|e| format!("native exit write: {e}"))?;

    if !both_ok {
        return Err(format!(
            "B9 live attempt FAILED: c1={:?} c2={:?}",
            c1_result.err(),
            c2_result.err()
        ));
    }
    Ok(format!(
        "{} bytes={} routes=4",
        facts_path.display(),
        written.len()
    ))
}
