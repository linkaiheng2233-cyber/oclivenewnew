//! CP-INT B7-R1：本地环回真实 TCP 同回合恢复（L1–L3）+ **受管清理与判定**。
//!
//! 与 B7 原版（head 13 已把它登记为不可变旧锚点）的差别，只在本文件的夹具与判定：
//!   * 场景 deadline 只包住**场景执行**，清理与事实写入无条件执行，不再被外层 `timeout` 取消；
//!   * 关闭顺序按 Host 既有 shutdown 口径：`directory_plugins.shutdown_all()` →
//!     `db_manager.close_pool().await`（等待 SQLite 池真正关闭）→ drop state → `TempDir::close()`；
//!   * facts 分离 `scenario_ok` 与 `cleanup`：场景成功但任一清理项失败时
//!     `failed=true` 且用例**非零退出**；场景自身失败时保留首因、清理错误单列；
//!   * 仍不宣称全部 detached task 已 join，只记录本次 join 结果与 survivor。
//!
//! 预算（本批授权）：1 个 `127.0.0.1:0` 监听器、L1–L3 共 4 条聊天路由、1 次记录型假生成、
//! 0 真实模型/音频/外部网络/Host run ID。attempt 标签 `B7-R1-LIVE-R0`（不是 Host run ID）。
//!
//! 边界：客户端只**丢弃响应体**，未观察 FIN/RST；不启 Tauri 进程、桌面服务、真实 provider。

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::too_many_lines
)]

mod common;

use async_trait::async_trait;
use axum::body::{to_bytes, Body};
use axum::extract::{Request as AxumRequest, State};
use axum::middleware::{self, Next};
use axum::response::Response as AxumResponse;
use futures_util::StreamExt;
use oclive_kernel_contracts::{LlmClient, LlmGenerateOpts, LlmGenerateOutcome, LlmTokenSink};
use oclive_kernel_host::domain::host_profile::HostProfile;
use oclive_kernel_host::infrastructure::RoleStorage;
use oclive_kernel_host::state::AppStateBuilder;
use oclive_kernel_types::error::AppError;
use oclive_kernel_types::models::dto::SendMessageRequest;
use oclivenewnew_tauri::kernel_attach::KernelHttpClient;
use oclivenewnew_tauri::kernel_lifecycle::KernelConnection;
use serde::Serialize;
use serde_json::{json, Map, Value};
use sqlx::Row;
use std::future::Future;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::sync::{oneshot, Semaphore};

// ---- 冻结的夹具与预算 ----
const ATTEMPT_LABEL: &str = "B7-R2-LIVE-R0";
const ROLE_ID: &str = "a-probe-role";
const MODEL_ID: &str = "a-probe-model:latest";
const SCENE_ID: &str = "default";
const MESSAGE: &str = "聊聊清晨的风景吧。";
const FIRST_TOKEN: &str = "清晨的风";
const REST_TOKEN: &str = "轻轻吹过树梢。";
const RECORDED_REPLY: &str = "清晨的风轻轻吹过树梢。";
/// L3 只改这一个语义字段（进入 Host 指纹），UUID 与 scope 仍复用 L1。
const L3_CHANGED_MESSAGE: &str = "聊聊清晨的风景吧。（L3 语义变更）";
/// 本批**全新**的 L1/L2 身份（不复用 B7 / B7-R1 的 UUID）。
const L1_TURN_ID: &str = "5d0b3f47-91ae-4c26-b8d3-46f19c7a2e58";
const L2_UNKNOWN_TURN_ID: &str = "a71c4e90-3b58-4d17-95fa-8c26d0b7e134";
const ROUTE_BUDGET: usize = 4;
const GENERATION_BUDGET: usize = 1;
/// 场景 deadline：只包住场景执行，不取消清理与事实写入。
const SCENARIO_DEADLINE: Duration = Duration::from_secs(60);
const GATE_WAIT: Duration = Duration::from_secs(20);
const RECEIPT_WAIT: Duration = Duration::from_secs(10);
/// 有界等待 server 优雅停机；超时后对本次拥有的 task 显式 abort 并再次有界等待。
const SHUTDOWN_WAIT: Duration = Duration::from_secs(10);
const ABORT_WAIT: Duration = Duration::from_secs(5);
/// 有界等待 `db_manager.close_pool()`（在 server 结束**之后**执行）。
const POOL_CLOSE_WAIT: Duration = Duration::from_secs(10);
const FACTS_REL: &str = ".cursor/plans/cp-int-b7-r2-live-facts.json";
const RUN_GATE_ENV: &str = "OCLIVE_B7_LIVE";

const MANIFEST_JSON: &str = r#"{
  "id": "a-probe-role",
  "name": "A Probe Role",
  "version": "0.0.1",
  "author": "harness",
  "description": "CP-INT B7-R1 synthetic role (non-production, isolated)",
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

/// 记录型 provider 替身：**唯一**允许的生成方法是 `generate_stream_with_opts`。
struct GatedRecorder {
    generations: Mutex<Vec<Value>>,
    violations: Mutex<Vec<String>>,
    probes: AtomicUsize,
    release: Semaphore,
}

impl GatedRecorder {
    fn new() -> Self {
        Self {
            generations: Mutex::new(Vec::new()),
            violations: Mutex::new(Vec::new()),
            probes: AtomicUsize::new(0),
            release: Semaphore::new(0),
        }
    }

    fn note(&self, what: impl Into<String>) {
        self.violations.lock().unwrap().push(what.into());
    }

    fn deny(&self, method: &str) -> AppError {
        self.note(format!("unexpected generation method: {method}"));
        AppError::Unknown(format!("CP-INT-B7R1-DENIED-{method}"))
    }

    fn generation_count(&self) -> usize {
        self.generations.lock().unwrap().len()
    }

    fn violation_list(&self) -> Vec<String> {
        self.violations.lock().unwrap().clone()
    }

    fn facts(&self) -> Value {
        json!({
            "attempts": self.generation_count(),
            "budget": GENERATION_BUDGET,
            "startup_probes": self.probes.load(Ordering::SeqCst),
            "violations": self.violation_list(),
        })
    }

    fn release_once(&self) {
        self.release.add_permits(1);
    }
}

#[async_trait]
impl LlmClient for GatedRecorder {
    async fn generate(&self, _: &str, _: &str) -> oclive_kernel_types::Result<String> {
        Err(self.deny("generate"))
    }

    async fn generate_tag(&self, _: &str, _: &str) -> oclive_kernel_types::Result<String> {
        Err(self.deny("generate_tag"))
    }

    async fn generate_stream(
        &self,
        _: &str,
        _: &str,
        _: LlmTokenSink,
    ) -> oclive_kernel_types::Result<String> {
        Err(self.deny("generate_stream"))
    }

    async fn generate_with_opts(
        &self,
        _: &str,
        _: &str,
        _: Option<&LlmGenerateOpts>,
    ) -> oclive_kernel_types::Result<LlmGenerateOutcome> {
        Err(self.deny("generate_with_opts"))
    }

    async fn generate_stream_with_opts(
        &self,
        model: &str,
        _prompt: &str,
        sink: LlmTokenSink,
        opts: Option<&LlmGenerateOpts>,
    ) -> oclive_kernel_types::Result<LlmGenerateOutcome> {
        {
            let mut gens = self.generations.lock().unwrap();
            if !gens.is_empty() {
                drop(gens);
                return Err(self.deny("generate_stream_with_opts#extra"));
            }
            if model != MODEL_ID || opts.is_none() {
                self.note(format!(
                    "generation model/opts mismatch: model={model} opts_present={}",
                    opts.is_some()
                ));
            }
            gens.push(json!({
                "method": "generate_stream_with_opts",
                "model": model,
                "opts_present": opts.is_some(),
            }));
        }
        sink(FIRST_TOKEN);
        match tokio::time::timeout(GATE_WAIT, self.release.acquire()).await {
            Ok(Ok(permit)) => permit.forget(),
            Ok(Err(_)) => {
                self.note("gate closed before release");
                return Err(AppError::Unknown("CP-INT-B7R1-GATE-CLOSED".into()));
            }
            Err(_) => {
                self.note("gate release timed out");
                return Err(AppError::Unknown("CP-INT-B7R1-GATE-TIMEOUT".into()));
            }
        }
        sink(REST_TOKEN);
        Ok(LlmGenerateOutcome {
            reply: RECORDED_REPLY.into(),
            prompt_eval_ms: None,
        })
    }

    async fn startup_probe(&self) -> oclive_kernel_types::Result<()> {
        self.probes.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }

    fn supports_prefix_cache(&self) -> bool {
        false
    }
}

/// Host 入口的**全量**请求账；请求体读取后原样重建再交给生产处理器。
#[derive(Default)]
struct RequestLedger {
    entries: Mutex<Vec<Value>>,
}

impl RequestLedger {
    fn push(&self, entry: Value) {
        let mut entries = self.entries.lock().unwrap();
        let seq = entries.len() + 1;
        let mut entry = entry;
        entry["seq"] = json!(seq);
        entries.push(entry);
    }

    fn entries(&self) -> Vec<Value> {
        self.entries.lock().unwrap().clone()
    }

    fn facts(&self) -> Value {
        routes_facts(&self.entries())
    }

    /// 摘要用的 route 总数：**类型化**读取，绝不索引 `Value`。
    fn route_total(&self) -> usize {
        self.entries.lock().unwrap().len()
    }
}

async fn record_request(
    State(ledger): State<Arc<RequestLedger>>,
    request: AxumRequest,
    next: Next,
) -> AxumResponse {
    let (parts, body) = request.into_parts();
    let method = parts.method.to_string();
    let path = parts.uri.path().to_string();
    let accept = parts
        .headers
        .get("accept")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();
    let bytes = to_bytes(body, 256 * 1024).await.unwrap_or_default();
    let parsed: Option<Value> = serde_json::from_slice(&bytes).ok();
    let field = |key: &str| -> Value {
        parsed
            .as_ref()
            .and_then(|v| v.get(key))
            .cloned()
            .unwrap_or(Value::Null)
    };
    let identity = json!({
        "client_request_id": field("client_request_id"),
        "role_id": field("role_id"),
        "role_path": field("role_path"),
        "message": field("message"),
        "user_message": field("user_message"),
        "scene_id": field("scene_id"),
        "session_id": field("session_id"),
        "include_raw_reply": field("include_raw_reply"),
        "adult": field("adult"),
    });
    let rebuilt = AxumRequest::from_parts(parts, Body::from(bytes.clone()));
    let response = next.run(rebuilt).await;
    ledger.push(json!({
        "method": method,
        "path": path,
        "status": response.status().as_u16(),
        "accept": accept,
        "body_bytes": bytes.len(),
        "identity": identity,
    }));
    response
}

/// 受管清理事实：每一项单独记录；`failed=false` 本身**不再**代表清理通过。
///
/// `order` 记录**实际执行顺序**（B7-R2 起要求先停服并 join、后关池）。
#[derive(Debug, Clone, PartialEq, Serialize)]
struct CleanupFacts {
    order: Vec<String>,
    directory_plugins_shutdown_all: bool,
    shutdown_signal_sent: bool,
    server_joined: bool,
    server_note: String,
    join_timed_out: bool,
    server_aborted: bool,
    /// abort 之后的有界等待结果：`Some(false)` 表示任务已结束，`Some(true)` 表示仍有存活。
    server_alive_after_abort: Option<bool>,
    pool_close_awaited: bool,
    pool_close_error: Option<String>,
    tempdir_close_ok: bool,
    tempdir_error: Option<String>,
    tempdir_absent_after_close: bool,
    behavior_env_removed: bool,
    /// server 结束**之后**读取的强引用数（含本次清理自身持有的那一份）。
    arc_strong_count_after_server: usize,
    /// 夹具自身已知引用数（本次清理作用域持有的唯一 `Arc`，=1）。
    fixture_owned_refs: usize,
    /// 扣除夹具自身引用后仍无法解释的引用；只描述事实，不推断来源。
    unexplained_refs: Vec<String>,
}

impl CleanupFacts {
    fn all_ok() -> Self {
        Self {
            order: vec![
                "directory_plugins_shutdown_all".into(),
                "graceful_shutdown_signal".into(),
                "server_join".into(),
                "arc_count".into(),
                "pool_close".into(),
                "drop_state".into(),
                "tempdir_close".into(),
                "env_restore".into(),
            ],
            directory_plugins_shutdown_all: true,
            shutdown_signal_sent: true,
            server_joined: true,
            server_note: "joined".into(),
            join_timed_out: false,
            server_aborted: false,
            server_alive_after_abort: None,
            pool_close_awaited: true,
            pool_close_error: None,
            tempdir_close_ok: true,
            tempdir_error: None,
            tempdir_absent_after_close: true,
            behavior_env_removed: true,
            arc_strong_count_after_server: 1,
            fixture_owned_refs: 1,
            unexplained_refs: Vec::new(),
        }
    }

    fn ok(&self) -> Result<(), String> {
        let mut problems: Vec<String> = Vec::new();
        if !self.directory_plugins_shutdown_all {
            problems.push("directory plugins were not shut down".into());
        }
        if !self.shutdown_signal_sent {
            problems.push("server graceful shutdown signal was not sent".into());
        }
        if !self.server_joined {
            problems.push(format!("http server did not join: {}", self.server_note));
        }
        if self.server_alive_after_abort == Some(true) {
            problems.push("server task was still alive after a bounded abort wait".into());
        }
        if !self.pool_close_awaited {
            problems.push("sqlite pool close was not awaited".into());
        }
        if let Some(error) = &self.pool_close_error {
            problems.push(format!("sqlite pool close failed: {error}"));
        }
        if !self.tempdir_close_ok {
            problems.push(format!(
                "TempDir::close failed: {}",
                self.tempdir_error
                    .clone()
                    .unwrap_or_else(|| "unknown".into())
            ));
        }
        if !self.tempdir_absent_after_close {
            problems.push("TempDir path still exists after close".into());
        }
        if !self.behavior_env_removed {
            problems.push("behaviour env vars were not restored".into());
        }
        for item in &self.unexplained_refs {
            problems.push(format!("unexplained AppState reference: {item}"));
        }
        if problems.is_empty() {
            Ok(())
        } else {
            Err(problems.join("; "))
        }
    }
}

/// 判定：场景结果与清理结果**分开**，且清理错误不覆盖场景首因。
#[derive(Debug, Clone, PartialEq, Serialize)]
struct Verdict {
    scenario_ok: bool,
    cleanup_ok: bool,
    failed: bool,
    primary: Option<String>,
    secondary: Vec<String>,
}

fn verdict_for(scenario_ok: bool, scenario_error: Option<&str>, cleanup: &CleanupFacts) -> Verdict {
    let cleanup_result = cleanup.ok();
    let mut secondary: Vec<String> = Vec::new();
    if let Err(error) = &cleanup_result {
        secondary.push(error.clone());
    }
    for item in &cleanup.unexplained_refs {
        secondary.push(format!("unexplained reference: {item}"));
    }
    if cleanup.server_alive_after_abort == Some(true) {
        secondary.push("server task still alive after abort".into());
    }
    let primary = if scenario_ok {
        None
    } else {
        Some(
            scenario_error
                .unwrap_or("scenario failed without a message")
                .to_string(),
        )
    };
    Verdict {
        scenario_ok,
        cleanup_ok: cleanup_result.is_ok(),
        failed: primary.is_some() || cleanup_result.is_err(),
        primary,
        secondary,
    }
}

/// 摘要字符串：字节数与 route 总数都由**显式返回值/类型化读取**提供，
/// 绝不索引已被写入闭包扩充过的 `facts` map（B7-R1 `no entry found for key` 的根因）。
fn summary_line(facts_path: &Path, bytes: u64, routes_total: usize) -> String {
    format!(
        "{} bytes={bytes} routes={routes_total}",
        facts_path.display()
    )
}

/// 把判定转成用例退出语义：任何失败都必须让用例非零退出。
fn exit_verdict(verdict: &Verdict) -> Result<(), String> {
    if !verdict.failed {
        return Ok(());
    }
    let mut message = format!("{ATTEMPT_LABEL} live attempt FAILED");
    if let Some(primary) = &verdict.primary {
        message.push_str(&format!("; scenario: {primary}"));
    }
    for item in &verdict.secondary {
        message.push_str(&format!("; cleanup: {item}"));
    }
    Err(message)
}

/// 受管编排：deadline 只包住场景；清理与事实写入**无条件**执行。
async fn run_managed<S, C, W>(
    deadline: Duration,
    scenario: S,
    cleanup: C,
    facts: &mut Map<String, Value>,
    write_facts: W,
) -> Result<(Verdict, u64), String>
where
    S: Future<Output = Result<Value, String>>,
    C: Future<Output = CleanupFacts>,
    W: Fn(&Map<String, Value>) -> Result<u64, String>,
{
    let started = std::time::Instant::now();
    let (scenario_ok, scenario_value, scenario_error, deadline_exceeded) =
        match tokio::time::timeout(deadline, scenario).await {
            Ok(Ok(value)) => (true, Some(value), None, false),
            Ok(Err(error)) => (false, None, Some(error), false),
            Err(_) => (
                false,
                None,
                Some(format!("scenario deadline {deadline:?} exceeded")),
                true,
            ),
        };
    facts.insert("scenario_ok".into(), json!(scenario_ok));
    facts.insert(
        "scenario_deadline_exceeded".into(),
        json!(deadline_exceeded),
    );
    facts.insert(
        "scenario_elapsed_ms".into(),
        json!(started.elapsed().as_millis() as u64),
    );
    if let Some(value) = scenario_value {
        facts.insert("scenario".into(), value);
    }
    if let Some(error) = &scenario_error {
        facts.insert("scenario_error".into(), json!(error));
    }

    // 清理无条件执行：deadline 与场景失败都不取消它，也不跳过事实写入。
    let cleanup_facts = cleanup.await;
    facts.insert(
        "cleanup".into(),
        serde_json::to_value(&cleanup_facts).unwrap_or(Value::Null),
    );
    let verdict = verdict_for(scenario_ok, scenario_error.as_deref(), &cleanup_facts);
    facts.insert("cleanup_ok".into(), json!(verdict.cleanup_ok));
    facts.insert("verdict_primary".into(), json!(verdict.primary));
    facts.insert("verdict_secondary".into(), json!(verdict.secondary));
    facts.insert("failed".into(), json!(verdict.failed));
    let bytes = write_facts(facts)?;
    Ok((verdict, bytes))
}

// ---------------- 不绑定端口的最小回归 ----------------

#[test]
fn b7_r2_declared_budget_and_gate() {
    assert_eq!(ROUTE_BUDGET, 4, "1 stream + 3 recover");
    assert_eq!(GENERATION_BUDGET, 1);
    assert_eq!(RUN_GATE_ENV, "OCLIVE_B7_LIVE");
    assert_eq!(ATTEMPT_LABEL, "B7-R2-LIVE-R0");
    assert!(FACTS_REL.ends_with("cp-int-b7-r2-live-facts.json"));
    for id in [L1_TURN_ID, L2_UNKNOWN_TURN_ID] {
        let parsed = uuid::Uuid::parse_str(id).expect("canonical uuid");
        assert!(!parsed.is_nil());
        assert_eq!(parsed.to_string(), id);
    }
    assert_ne!(L1_TURN_ID, L2_UNKNOWN_TURN_ID);
}

#[test]
fn cleanup_failure_fails_the_attempt_even_when_the_scenario_succeeded() {
    let cleanup = CleanupFacts {
        tempdir_close_ok: false,
        tempdir_error: Some("os error 32".into()),
        tempdir_absent_after_close: false,
        ..CleanupFacts::all_ok()
    };
    let verdict = verdict_for(true, None, &cleanup);
    assert!(verdict.scenario_ok, "scenario itself succeeded");
    assert!(!verdict.cleanup_ok, "cleanup must not be reported as OK");
    assert!(verdict.failed, "overall verdict must fail");
    assert!(verdict.primary.is_none(), "no scenario-side first cause");
    assert!(
        verdict
            .secondary
            .iter()
            .any(|item| item.contains("TempDir::close failed")),
        "cleanup cause must be recorded: {:?}",
        verdict.secondary
    );
    assert!(
        verdict
            .secondary
            .iter()
            .any(|item| item.contains("still exists")),
        "residual path must be recorded"
    );
    assert!(exit_verdict(&verdict).is_err(), "must exit non-zero");
}

#[test]
fn pool_or_server_failure_also_fails_the_attempt() {
    for (label, cleanup) in [
        (
            "pool not awaited",
            CleanupFacts {
                pool_close_awaited: false,
                ..CleanupFacts::all_ok()
            },
        ),
        (
            "pool close error",
            CleanupFacts {
                pool_close_error: Some("pool close timed out".into()),
                ..CleanupFacts::all_ok()
            },
        ),
        (
            "server join failure",
            CleanupFacts {
                server_joined: false,
                server_note: "join timed out".into(),
                join_timed_out: true,
                ..CleanupFacts::all_ok()
            },
        ),
        (
            "server still alive after abort",
            CleanupFacts {
                server_joined: false,
                server_note: "join timed out".into(),
                join_timed_out: true,
                server_aborted: true,
                server_alive_after_abort: Some(true),
                ..CleanupFacts::all_ok()
            },
        ),
        (
            "shutdown signal not sent",
            CleanupFacts {
                shutdown_signal_sent: false,
                ..CleanupFacts::all_ok()
            },
        ),
        (
            "directory plugins not shut down",
            CleanupFacts {
                directory_plugins_shutdown_all: false,
                ..CleanupFacts::all_ok()
            },
        ),
        (
            "env not restored",
            CleanupFacts {
                behavior_env_removed: false,
                ..CleanupFacts::all_ok()
            },
        ),
        (
            "unexplained AppState reference",
            CleanupFacts {
                arc_strong_count_after_server: 2,
                unexplained_refs: vec!["1 AppState strong ref beyond the fixture handle".into()],
                ..CleanupFacts::all_ok()
            },
        ),
    ] {
        let verdict = verdict_for(true, None, &cleanup);
        assert!(
            verdict.failed,
            "cleanup failure must fail the attempt ({label}): {cleanup:?}"
        );
        assert!(!verdict.cleanup_ok, "cleanup must not be OK ({label})");
        assert!(
            exit_verdict(&verdict).is_err(),
            "must exit non-zero ({label})"
        );
    }
}

#[test]
fn scenario_failure_and_cleanup_failure_keep_both_causes() {
    // 首因：场景失败；次因：清理里两类真实失败（server join + pool close）。
    let cleanup = CleanupFacts {
        server_joined: false,
        server_note: "join timed out".into(),
        join_timed_out: true,
        pool_close_error: Some("pool close failed after join timeout".into()),
        ..CleanupFacts::all_ok()
    };
    let verdict = verdict_for(false, Some("scenario boom"), &cleanup);
    assert_eq!(verdict.primary.as_deref(), Some("scenario boom"));
    assert!(
        verdict
            .secondary
            .iter()
            .any(|item| item.contains("http server did not join")),
        "server cause must stay visible next to the first cause"
    );
    assert!(
        verdict
            .secondary
            .iter()
            .any(|item| item.contains("sqlite pool close failed")),
        "pool cause must stay visible next to the first cause"
    );
    assert!(verdict.failed);
    let message = exit_verdict(&verdict).unwrap_err();
    assert!(message.contains("scenario boom"));
    assert!(message.contains("did not join") && message.contains("pool close failed"));
}

#[test]
fn summary_line_never_indexes_a_persisted_section() {
    // 复现 B7-R1 缺陷场景：写入闭包把 routes 段加入**克隆**后落盘，内存 map 里没有 routes。
    let mut facts: Map<String, Value> = Map::new();
    facts.insert("scenario_ok".into(), json!(true));
    facts.insert("failed".into(), json!(false));
    assert!(
        !facts.contains_key("routes"),
        "the in-memory map must not contain the writer-added section"
    );
    let line = summary_line(Path::new("cp-int-b7-r2-live-facts.json"), 1234, 4);
    assert!(
        line.contains("routes=4"),
        "route total must come from a typed source: {line}"
    );
    assert!(line.contains("bytes=1234"));
}

#[tokio::test]
async fn managed_run_summary_survives_a_writer_that_adds_sections() {
    let mut facts: Map<String, Value> = Map::new();
    let writes = Arc::new(Mutex::new(0usize));
    let write_counter = Arc::clone(&writes);

    let scenario = async { Ok(json!({"ok": true})) };
    let cleanup = async { CleanupFacts::all_ok() };

    let (verdict, bytes) = run_managed(
        Duration::from_secs(1),
        scenario,
        cleanup,
        &mut facts,
        |captured| {
            let mut all = captured.clone();
            all.insert("routes".into(), json!({"total": 4}));
            all.insert("generation".into(), json!({"attempts": 1}));
            *write_counter.lock().unwrap() += 1;
            Ok(serde_json::to_string(&all).unwrap().len() as u64)
        },
    )
    .await
    .expect("managed run must write facts");

    assert_eq!(*writes.lock().unwrap(), 1);
    assert_eq!(facts["failed"], json!(false));
    assert!(!facts.contains_key("routes"));
    assert!(exit_verdict(&verdict).is_ok());
    // 摘要走显式返回的字节数与 route 总数，**不再**索引内存 map。
    let line = summary_line(Path::new("f.json"), bytes, 4);
    assert!(line.contains("routes=4") && line.contains(&format!("bytes={bytes}")));
}

#[tokio::test]
async fn scenario_deadline_still_runs_cleanup_and_writes_facts() {
    let cleanup_runs = Arc::new(AtomicUsize::new(0));
    let cleanup_counter = Arc::clone(&cleanup_runs);
    let writes = Arc::new(Mutex::new(0usize));
    let write_counter = Arc::clone(&writes);
    let mut facts: Map<String, Value> = Map::new();

    let scenario = async {
        tokio::time::sleep(Duration::from_millis(500)).await;
        Ok(json!({"never": true}))
    };
    let cleanup = async move {
        cleanup_counter.fetch_add(1, Ordering::SeqCst);
        // 清理允许晚于 deadline 完成：deadline 不得取消清理。
        tokio::time::sleep(Duration::from_millis(20)).await;
        CleanupFacts::all_ok()
    };

    let (verdict, _bytes) = run_managed(
        Duration::from_millis(10),
        scenario,
        cleanup,
        &mut facts,
        |captured| {
            *write_counter.lock().unwrap() += 1;
            assert!(
                captured.contains_key("cleanup"),
                "facts must include the cleanup section on the deadline path"
            );
            Ok(serde_json::to_string(captured).unwrap().len() as u64)
        },
    )
    .await
    .expect("managed run must still write facts");

    assert_eq!(
        cleanup_runs.load(Ordering::SeqCst),
        1,
        "cleanup must run after the scenario deadline"
    );
    assert_eq!(*writes.lock().unwrap(), 1, "facts written exactly once");
    assert_eq!(facts["scenario_deadline_exceeded"], json!(true));
    assert_eq!(facts["scenario_ok"], json!(false));
    assert_eq!(facts["cleanup_ok"], json!(true));
    assert_eq!(
        facts["failed"],
        json!(true),
        "a deadline is a failed attempt"
    );
    assert!(verdict.failed && verdict.primary.is_some());
}

// ---------------- live 用例（显式运行门；常规 cargo test 不绑定端口） ----------------

#[tokio::test(flavor = "multi_thread")]
#[ignore = "B7-R2 live loopback: binds 127.0.0.1:0 once; run explicitly with OCLIVE_B7_LIVE=1 cargo test -p oclivenewnew-tauri --test cross_transport_loopback_live -- --ignored --exact b7_r2_live_loopback_managed_cleanup_and_native_exit --nocapture"]
async fn b7_r2_live_loopback_managed_cleanup_and_native_exit() {
    match run_live().await {
        Ok(summary) => println!("B7R2_FACTS_JSON={summary}"),
        Err(failure) => panic!("{failure}"),
    }
}

fn facts_write(path: &Path, facts: &Map<String, Value>) -> Result<u64, String> {
    if path.exists() {
        return Err(format!(
            "refusing to overwrite an existing facts file: {}",
            path.display()
        ));
    }
    let body = serde_json::to_string_pretty(&Value::Object(facts.clone()))
        .map_err(|e| format!("facts encode: {e}"))?;
    std::fs::write(path, body.as_bytes()).map_err(|e| format!("facts write: {e}"))?;
    Ok(body.len() as u64)
}

fn role_dir(root: &Path) -> PathBuf {
    root.join("roles").join(ROLE_ID)
}

fn write_role_pack(root: &Path) -> Result<(), String> {
    let dir = role_dir(root);
    std::fs::create_dir_all(&dir).map_err(|e| format!("create role dir: {e}"))?;
    for (name, body) in [
        ("manifest.json", MANIFEST_JSON),
        ("settings.json", SETTINGS_JSON),
        ("config.json", CONFIG_JSON),
    ] {
        std::fs::write(dir.join(name), body).map_err(|e| format!("write {name}: {e}"))?;
    }
    Ok(())
}

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

async fn snapshot(db_path: &Path) -> Result<Value, String> {
    let pool = read_pool(db_path).await?;
    let messages = sqlx::query(
        "SELECT id, session_id, turn_index, sender, content FROM chat_messages ORDER BY turn_index, sender",
    )
    .fetch_all(&pool)
    .await;
    let sessions = sqlx::query(
        "SELECT session_id, role_id, scene_id, message_count FROM chat_sessions ORDER BY session_id",
    )
    .fetch_all(&pool)
    .await;
    let receipts = sqlx::query(
        "SELECT request_id, status, payload_sha256, user_message_id, assistant_message_id FROM chat_request_receipts ORDER BY request_id",
    )
    .fetch_all(&pool)
    .await;
    pool.close().await;
    let messages = messages.map_err(|e| format!("chat_messages: {e}"))?;
    let sessions = sessions.map_err(|e| format!("chat_sessions: {e}"))?;
    let receipts = receipts.map_err(|e| format!("chat_request_receipts: {e}"))?;
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

async fn wait_receipt_completed(
    db_path: &Path,
    request_id: &str,
) -> Result<(String, u128), String> {
    let started = std::time::Instant::now();
    let mut last = String::from("<absent>");
    while started.elapsed() < RECEIPT_WAIT {
        match receipt_status(db_path, request_id).await? {
            Some(status) => {
                last = status.clone();
                if status == "completed" {
                    return Ok((status, started.elapsed().as_millis()));
                }
            }
            None => last = "<absent>".to_string(),
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    Err(format!(
        "receipt {request_id} did not reach completed within {RECEIPT_WAIT:?} (last={last})"
    ))
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

fn recover_request(turn_id: &str, user_message: &str) -> SendMessageRequest {
    SendMessageRequest {
        client_request_id: Some(turn_id.to_string()),
        role_id: ROLE_ID.to_string(),
        user_message: user_message.to_string(),
        scene_id: Some(SCENE_ID.to_string()),
        session_id: None,
        include_raw_reply: None,
        adult: None,
    }
}

fn routes_facts(entries: &[Value]) -> Value {
    let labels: Vec<String> = entries
        .iter()
        .map(|e| {
            format!(
                "{} {}",
                e["method"].as_str().unwrap_or("?"),
                e["path"].as_str().unwrap_or("?")
            )
        })
        .collect();
    let count = |label: &str| labels.iter().filter(|l| l.as_str() == label).count();
    let others: Vec<String> = labels
        .iter()
        .filter(|l| !matches!(l.as_str(), "POST /chat/stream" | "POST /chat/recover"))
        .cloned()
        .collect();
    json!({
        "entries": entries,
        "labels": labels,
        "total": entries.len(),
        "budget": ROUTE_BUDGET,
        "stream_count": count("POST /chat/stream"),
        "recover_count": count("POST /chat/recover"),
        "chat_route_count": count("POST /chat"),
        "other_requests": others,
    })
}

// ---------------- live 编排 ----------------

async fn run_live() -> Result<String, String> {
    let repo = common::monorepo_root();
    let facts_path = repo.join(FACTS_REL.replace('/', std::path::MAIN_SEPARATOR_STR));
    if std::env::var(RUN_GATE_ENV).as_deref() != Ok("1") {
        return Err(format!(
            "explicit run gate missing: set {RUN_GATE_ENV}=1 to allow one live loopback run"
        ));
    }
    if facts_path.exists() {
        return Err(format!(
            "refusing to overwrite the previous attempt's facts: {}",
            facts_path.display()
        ));
    }

    let mut facts: Map<String, Value> = Map::new();
    facts.insert("batch".into(), json!("CP-INT B7-R2"));
    facts.insert("attempt_label".into(), json!(ATTEMPT_LABEL));
    facts.insert(
        "test".into(),
        json!("b7_r2_live_loopback_managed_cleanup_and_native_exit"),
    );
    facts.insert("route_budget".into(), json!(ROUTE_BUDGET));
    facts.insert("generation_budget".into(), json!(GENERATION_BUDGET));
    facts.insert("facts_path".into(), json!(facts_path.display().to_string()));
    facts.insert(
        "scenario_deadline_ms".into(),
        json!(SCENARIO_DEADLINE.as_millis() as u64),
    );

    // TempDir 先验明：本次新建、位于系统 Temp 之下。
    let dir = tempfile::TempDir::new().map_err(|e| format!("TempDir: {e}"))?;
    let root = dir.path().to_path_buf();
    let temp_root = std::env::temp_dir();
    let fresh = !root.join("app-data").exists() && !root.join("roles").exists();
    facts.insert(
        "temp_dir".into(),
        json!({
            "path": root.display().to_string(),
            "fresh_isolated": fresh,
            "under_system_temp": root.starts_with(&temp_root),
            "system_temp": temp_root.display().to_string(),
            "recursive_delete_by_test": false,
        }),
    );
    if !fresh || !root.starts_with(&temp_root) {
        return Err(format!(
            "TempDir is not a fresh isolated directory under the system temp root: {}",
            root.display()
        ));
    }
    std::fs::create_dir_all(root.join("app-data")).map_err(|e| format!("app-data dir: {e}"))?;
    write_role_pack(&root)?;
    let fixture_facts = {
        let storage = RoleStorage::new(root.join("roles"));
        let role = storage
            .load_role(ROLE_ID)
            .map_err(|e| format!("probe role pack must load: {e}"))?;
        json!({
            "role_id": role.id,
            "ollama_model": role.ollama_model,
            "scene_ids": role.scene_ids.iter().cloned().collect::<Vec<_>>(),
        })
    };
    facts.insert("role_fixture".into(), fixture_facts);
    let db_path = root.join("app-data").join("app.db");

    let behavior: [(&str, &str); 5] = [
        ("OCLIVE_SKIP_LLM_STARTUP_PROBE", "1"),
        ("OCLIVE_PORTRAIT_EMOTION_LLM", "0"),
        ("OCLIVE_EVENT_IMPACT_LLM", "0"),
        ("OCLIVE_OLLAMA_PRELOAD", "0"),
        ("OCLIVE_CHAT_STORAGE_BACKEND", "sqlite"),
    ];
    for (key, value) in behavior {
        std::env::set_var(key, value);
    }
    facts.insert(
        "behavior_env_set".into(),
        json!(behavior
            .iter()
            .map(|(k, v)| json!({"key": k, "value": v}))
            .collect::<Vec<_>>()),
    );

    let recorder = Arc::new(GatedRecorder::new());
    let state = Arc::new(
        AppStateBuilder::production(db_path.clone(), root.join("roles"), root.join("app-data"))
            .with_host_profile(HostProfile {
                event_impact_llm: false,
                skip_agent: true,
                skip_complex_emotion: true,
                ..Default::default()
            })
            .with_llm_client(Arc::clone(&recorder) as Arc<dyn LlmClient>)
            .build()
            .await
            .map_err(|e| format!("AppStateBuilder::production: {e}"))?,
    );
    facts.insert(
        "app_state".into(),
        json!({
            "production_builder": true,
            "injected_llm_double": true,
            "real_ollama_client_present": state.ollama.is_some(),
            "real_performance_llm_present": state.performance_llm.is_some(),
        }),
    );
    if state.ollama.is_some() || state.performance_llm.is_some() {
        return Err("a real provider client exists in this process; refusing to continue".into());
    }

    let ledger = Arc::new(RequestLedger::default());
    // router 取得一份 Arc 克隆（随 server 任务结束而释放）；**不再**为清理另留第二份克隆，
    // 外层唯一的 state 所有权稍后整体移入 cleanup future。
    let router = oclive_kernel_host::http_api::api_router(Arc::clone(&state)).layer(
        middleware::from_fn_with_state(Arc::clone(&ledger), record_request),
    );
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .map_err(|e| format!("bind 127.0.0.1:0: {e}"))?;
    let addr = listener
        .local_addr()
        .map_err(|e| format!("local_addr: {e}"))?;
    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
    let mut server = tokio::spawn(async move {
        axum::serve(listener, router)
            .with_graceful_shutdown(async {
                let _ = shutdown_rx.await;
            })
            .await
    });
    facts.insert(
        "listener".into(),
        json!({
            "bind": "127.0.0.1:0",
            "addr": addr.to_string(),
            "ephemeral_port": addr.port() != 0,
            "production_router": true,
            "client_connects": 4,
        }),
    );
    let conn = KernelConnection::new(format!("http://{addr}"), addr.port());
    let base = format!("http://{addr}");
    let role_path = role_dir(&root);

    // 受管清理：与场景解耦，deadline/失败都不取消它；外层**唯一**的 state 所有权移入此处。
    // 顺序（B7-R2 冻结）：停目录插件 → 发送优雅停机 → 有界 join（超时则显式 abort + 有界等待）
    // → 读取 Arc 计数 → **然后**才有界等待池关闭 → drop state → TempDir::close → 环境恢复。
    let cleanup = {
        let root = root.clone();
        let behavior = behavior.to_vec();
        async move {
            let mut cleanup = CleanupFacts {
                order: Vec::new(),
                directory_plugins_shutdown_all: false,
                shutdown_signal_sent: false,
                server_joined: false,
                server_note: "not attempted".into(),
                join_timed_out: false,
                server_aborted: false,
                server_alive_after_abort: None,
                pool_close_awaited: false,
                pool_close_error: None,
                tempdir_close_ok: false,
                tempdir_error: None,
                tempdir_absent_after_close: false,
                behavior_env_removed: false,
                arc_strong_count_after_server: 0,
                fixture_owned_refs: 1,
                unexplained_refs: Vec::new(),
            };

            cleanup.order.push("directory_plugins_shutdown_all".into());
            state.directory_plugins.shutdown_all();
            cleanup.directory_plugins_shutdown_all = true;

            cleanup.order.push("graceful_shutdown_signal".into());
            cleanup.shutdown_signal_sent = shutdown_tx.send(()).is_ok();

            cleanup.order.push("server_join".into());
            match tokio::time::timeout(SHUTDOWN_WAIT, &mut server).await {
                Ok(Ok(Ok(()))) => {
                    cleanup.server_joined = true;
                    cleanup.server_note = "joined".into();
                }
                Ok(Ok(Err(e))) => cleanup.server_note = format!("serve error: {e}"),
                Ok(Err(e)) => cleanup.server_note = format!("join error: {e}"),
                Err(_) => {
                    cleanup.join_timed_out = true;
                    cleanup.server_note = format!("join timed out after {SHUTDOWN_WAIT:?}");
                    // 有界停止本次拥有的 server task，并如实记录其是否仍在存活。
                    cleanup.order.push("server_abort".into());
                    server.abort();
                    cleanup.server_aborted = true;
                    let alive = tokio::time::timeout(ABORT_WAIT, &mut server).await.is_err();
                    cleanup.server_alive_after_abort = Some(alive);
                    cleanup.server_note.push_str(&format!(
                        "; abort waited {ABORT_WAIT:?}, alive_after_abort={alive}"
                    ));
                }
            }

            // Arc 计数：此时夹具自身已知引用只有本清理作用域持有的这一份。
            cleanup.order.push("arc_count".into());
            let strong = Arc::strong_count(&state);
            cleanup.arc_strong_count_after_server = strong;
            let unexplained = strong.saturating_sub(cleanup.fixture_owned_refs);
            if unexplained > 0 {
                cleanup.unexplained_refs.push(format!(
                    "{unexplained} AppState strong ref(s) beyond the fixture's own cleanup handle (source not inferred)"
                ));
            }

            // server 结束**之后**才等待池关闭。
            cleanup.order.push("pool_close".into());
            match tokio::time::timeout(POOL_CLOSE_WAIT, state.db_manager.close_pool()).await {
                Ok(()) => cleanup.pool_close_awaited = true,
                Err(_) => {
                    cleanup.pool_close_error =
                        Some(format!("pool close timed out after {POOL_CLOSE_WAIT:?}"));
                }
            }

            cleanup.order.push("drop_state".into());
            drop(state);

            cleanup.order.push("tempdir_close".into());
            cleanup.tempdir_absent_after_close = !root.exists();
            match dir.close() {
                Ok(()) => cleanup.tempdir_close_ok = true,
                Err(e) => cleanup.tempdir_error = Some(e.to_string()),
            }
            cleanup.tempdir_absent_after_close = !root.exists();

            cleanup.order.push("env_restore".into());
            for (key, _) in &behavior {
                std::env::remove_var(key);
            }
            cleanup.behavior_env_removed = behavior.iter().all(|(k, _)| std::env::var(k).is_err());
            cleanup
        }
    };

    let scenario = run_scenarios(
        &base,
        &db_path,
        &role_path,
        &conn,
        Arc::clone(&recorder),
        Arc::clone(&ledger),
    );

    let recorder_for_facts = Arc::clone(&recorder);
    let ledger_for_facts = Arc::clone(&ledger);
    let writer_path = facts_path.clone();
    let writer = move |captured: &Map<String, Value>| -> Result<u64, String> {
        let mut all = captured.clone();
        all.insert("generation".into(), recorder_for_facts.facts());
        all.insert("routes".into(), ledger_for_facts.facts());
        facts_write(&writer_path, &all)
    };

    let (verdict, bytes) =
        run_managed(SCENARIO_DEADLINE, scenario, cleanup, &mut facts, writer).await?;
    exit_verdict(&verdict)?;
    // 摘要只使用显式返回值与类型化读取（ledger.route_total()），不再索引 facts 的 Value。
    Ok(summary_line(&facts_path, bytes, ledger.route_total()))
}

async fn run_scenarios(
    base: &str,
    db_path: &Path,
    role_path: &Path,
    conn: &KernelConnection,
    recorder: Arc<GatedRecorder>,
    ledger: Arc<RequestLedger>,
) -> Result<Value, String> {
    let mut out = Map::new();
    let http = reqwest::Client::builder()
        .no_proxy()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| format!("reqwest client: {e}"))?;

    // ================= L1 =================
    let stream_wire = json!({
        "client_request_id": L1_TURN_ID,
        "role_path": role_path.to_string_lossy(),
        "message": MESSAGE,
        "scene_id": SCENE_ID,
        "session_id": null,
        "adult": null,
    });
    out.insert("l1_stream_request_body".into(), stream_wire.clone());
    let response = http
        .post(format!("{base}/chat/stream"))
        .header("accept", "text/event-stream")
        .json(&stream_wire)
        .send()
        .await
        .map_err(|e| format!("L1 stream send: {e}"))?;
    let status = response.status().as_u16();
    if status != 200 {
        return Err(format!("L1 stream status {status}"));
    }
    let mut stream = response.bytes_stream();
    let mut buffer = String::new();
    let mut raw_observed = String::new();
    let mut observed_token: Option<String> = None;
    let mut observed_block: Option<String> = None;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| format!("L1 stream chunk: {e}"))?;
        let text = String::from_utf8_lossy(&chunk).to_string();
        raw_observed.push_str(&text);
        buffer.push_str(&text);
        while let Some(sep) = buffer.find("\n\n") {
            let block = buffer[..sep].to_string();
            buffer = buffer[sep + 2..].to_string();
            if let Some(token) = token_of(&block) {
                observed_token = Some(token);
                observed_block = Some(block);
                break;
            }
        }
        if observed_token.is_some() {
            break;
        }
    }
    drop(stream);
    let observed_token = observed_token.ok_or("L1: no complete token frame was observed")?;
    if observed_token != FIRST_TOKEN {
        return Err(format!(
            "L1: observed token {observed_token:?} != recorded first token {FIRST_TOKEN:?}"
        ));
    }
    out.insert("l1_stream_status".into(), json!(status));
    out.insert("l1_observed_token".into(), json!(observed_token));
    out.insert("l1_observed_block".into(), json!(observed_block));
    out.insert("l1_raw_observed".into(), json!(raw_observed));
    out.insert("l1_client_received_done".into(), json!(false));
    out.insert(
        "l1_client_disconnect_evidence".into(),
        json!("response body discarded while the turn was still gated; no TCP-level FIN/RST was observed"),
    );

    let while_gated = receipt_status(db_path, L1_TURN_ID).await?;
    out.insert("l1_receipt_status_while_gated".into(), json!(while_gated));
    if while_gated.as_deref() != Some("running") {
        return Err(format!(
            "L1: receipt was expected to be running while the provider is gated, got {while_gated:?}"
        ));
    }

    recorder.release_once();
    let (receipt_status_final, waited_ms) = wait_receipt_completed(db_path, L1_TURN_ID).await?;
    out.insert(
        "l1_receipt_status_after_release".into(),
        json!(receipt_status_final),
    );
    out.insert("l1_receipt_wait_ms".into(), json!(waited_ms));

    let db_after_turn = snapshot(db_path).await?;
    out.insert("l1_db_after_turn".into(), db_after_turn.clone());

    let recovered =
        KernelHttpClient::recover_message_via_http(conn, &recover_request(L1_TURN_ID, MESSAGE))
            .await
            .map_err(|e| format!("L1 recover_message_via_http: {e:?}"))?;

    let messages = db_after_turn["messages"]
        .as_array()
        .ok_or("L1: messages missing")?;
    let user_row = messages
        .iter()
        .find(|m| m["sender"] == "user")
        .ok_or("L1: user row missing")?;
    let assistant_row = messages
        .iter()
        .find(|m| m["sender"] == "assistant")
        .ok_or("L1: assistant row missing")?;
    let assistant_text = assistant_row["content"]
        .as_str()
        .ok_or("L1: assistant content missing")?
        .to_string();
    let user_id = user_row["id"].as_str().unwrap_or("").to_string();
    let assistant_id = assistant_row["id"].as_str().unwrap_or("").to_string();

    if messages.len() != 2 {
        return Err(format!(
            "L1: expected exactly 2 message rows, got {}",
            messages.len()
        ));
    }
    if recovered.user_message_id.as_deref() != Some(user_id.as_str()) {
        return Err(format!(
            "L1: recovered user_message_id {:?} != row id {user_id}",
            recovered.user_message_id
        ));
    }
    if recovered.assistant_message_id.as_deref() != Some(assistant_id.as_str()) {
        return Err(format!(
            "L1: recovered assistant_message_id {:?} != row id {assistant_id}",
            recovered.assistant_message_id
        ));
    }
    if recovered.reply != assistant_text {
        return Err("L1: recovered reply differs from the assistant row".into());
    }
    if !assistant_text.starts_with(FIRST_TOKEN) {
        return Err("L1: assistant row does not start with the observed first token".into());
    }
    if recovered.reply_is_fallback {
        return Err("L1: recovered reply is a fallback".into());
    }
    if recorder.generation_count() != GENERATION_BUDGET {
        return Err(format!(
            "L1: generation count {} != budget {GENERATION_BUDGET}",
            recorder.generation_count()
        ));
    }
    let receipts = db_after_turn["receipts"]
        .as_array()
        .ok_or("L1: receipts missing")?;
    if receipts.len() != 1 {
        return Err(format!(
            "L1: expected exactly 1 receipt, got {}",
            receipts.len()
        ));
    }
    if receipts[0]["request_id"] != L1_TURN_ID || receipts[0]["status"] != "completed" {
        return Err(format!("L1: unexpected receipt {}", receipts[0]));
    }
    out.insert(
        "l1_recovered".into(),
        json!({
            "reply": recovered.reply,
            "user_message_id": recovered.user_message_id,
            "assistant_message_id": recovered.assistant_message_id,
            "reply_is_fallback": recovered.reply_is_fallback,
            "scene_id": recovered.scene_id,
            "recorded_provider_reply": RECORDED_REPLY,
        }),
    );

    let db_after_recover = snapshot(db_path).await?;
    if db_after_recover != db_after_turn {
        return Err("L1: recovery changed the database".into());
    }
    out.insert("l1_db_after_recover".into(), db_after_recover.clone());

    // ================= L2 =================
    let l2 = KernelHttpClient::recover_message_via_http(
        conn,
        &recover_request(L2_UNKNOWN_TURN_ID, MESSAGE),
    )
    .await;
    match &l2 {
        Err(AppError::ChatRequestUnconfirmed) => {}
        other => {
            return Err(format!(
                "L2: unknown UUID must map to CHAT_REQUEST_UNCONFIRMED, got {other:?}"
            ))
        }
    }
    out.insert("l2_result".into(), json!("CHAT_REQUEST_UNCONFIRMED"));
    let db_after_l2 = snapshot(db_path).await?;
    if db_after_l2 != db_after_turn {
        return Err("L2: rejected recovery changed the database".into());
    }
    if recorder.generation_count() != GENERATION_BUDGET {
        return Err("L2: rejected recovery generated".into());
    }

    // ================= L3 =================
    let l3 = KernelHttpClient::recover_message_via_http(
        conn,
        &recover_request(L1_TURN_ID, L3_CHANGED_MESSAGE),
    )
    .await;
    match &l3 {
        Err(AppError::ChatRequestConflict) => {}
        other => {
            return Err(format!(
                "L3: same identity with a different payload must map to CHAT_REQUEST_CONFLICT, got {other:?}"
            ))
        }
    }
    out.insert("l3_result".into(), json!("CHAT_REQUEST_CONFLICT"));
    let db_after_l3 = snapshot(db_path).await?;
    if db_after_l3 != db_after_turn {
        return Err("L3: conflicted recovery changed the database".into());
    }
    if recorder.generation_count() != GENERATION_BUDGET {
        return Err("L3: conflicted recovery generated".into());
    }
    out.insert("l3_db_final".into(), db_after_l3);

    // ---- 路由预算与身份 ----
    let entries = ledger.entries();
    let routes = routes_facts(&entries);
    if routes["total"].as_u64() != Some(ROUTE_BUDGET as u64)
        || routes["stream_count"].as_u64() != Some(1)
        || routes["recover_count"].as_u64() != Some(3)
        || routes["chat_route_count"].as_u64() != Some(0)
        || routes["other_requests"].as_array().map_or(1, Vec::len) != 0
    {
        return Err(format!("route budget deviation: {routes}"));
    }
    let entry_ids: Vec<Value> = entries
        .iter()
        .map(|e| e["identity"]["client_request_id"].clone())
        .collect();
    if entry_ids
        != vec![
            json!(L1_TURN_ID),
            json!(L1_TURN_ID),
            json!(L2_UNKNOWN_TURN_ID),
            json!(L1_TURN_ID),
        ]
    {
        return Err(format!("route identity sequence mismatch: {entry_ids:?}"));
    }
    let entry_messages: Vec<Value> = entries
        .iter()
        .map(|e| {
            let id = &e["identity"];
            if id["message"].is_null() {
                id["user_message"].clone()
            } else {
                id["message"].clone()
            }
        })
        .collect();
    if entry_messages
        != vec![
            json!(MESSAGE),
            json!(MESSAGE),
            json!(MESSAGE),
            json!(L3_CHANGED_MESSAGE),
        ]
    {
        return Err(format!(
            "route payload sequence mismatch: {entry_messages:?}"
        ));
    }
    out.insert("l1_l2_l3_route_evidence".into(), routes);
    out.insert(
        "server_side_stream_identity".into(),
        entries[0]["identity"].clone(),
    );
    out.insert(
        "server_side_recover_identities".into(),
        json!(entries[1..]
            .iter()
            .map(|e| e["identity"].clone())
            .collect::<Vec<_>>()),
    );

    if !recorder.violation_list().is_empty() {
        return Err(format!(
            "provider double recorded violations: {:?}",
            recorder.violation_list()
        ));
    }
    Ok(Value::Object(out))
}
