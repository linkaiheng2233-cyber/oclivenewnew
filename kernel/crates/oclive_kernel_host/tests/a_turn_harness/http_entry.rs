//! CP-INT B2: real router + isolated SQLite, no TCP listener or real provider.
//! H4 characterizes the current retry hazard; a green harness is NOT product acceptance.

use crate::support::{self, artifacts, fixture, OpsLedger, RealFs, ROLE_ID};
use async_trait::async_trait;
use axum::body::{to_bytes, Body};
use axum::http::Request;
use futures_util::StreamExt;
use oclive_kernel_contracts::{LlmClient, LlmGenerateOpts, LlmGenerateOutcome, LlmTokenSink};
use oclive_kernel_host::state::{AppState, AppStateBuilder};
use oclive_kernel_types::{AppError, Result as KernelResult};
use serde_json::{json, Map, Value};
use sqlx::Row;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::Semaphore;
use tower::ServiceExt;

/// Reserved B2/B3-era identities: already consumed, so they must never be admitted again.
pub const CONSUMED_RUN_IDS: [&str; 4] = [
    "A-CPINTB2-4b7ccc66-H01-R0",
    "A-CPINTB2-4b7ccc66-H02-R0",
    "A-CPINTB2-4b7ccc66-H03-R0",
    "A-CPINTB2-4b7ccc66-H04-R0",
];

/// CP-INT B4/H03 preparation: the controller reserved these two identities. They are
/// recognised by the gate below but are NOT authorised to run in the preparation batch, and
/// `run_ids` stays empty in the candidate manifest until the controller allocates a budget.
pub const RESERVED_B4_RUN_IDS: [(&str, &str); 2] = [
    ("h2", "A-CPINTB4-f2a23853-H02-R0"),
    ("h3", "A-CPINTB4-f2a23853-H03-R0"),
];

pub const CASES: [(&str, &str); 4] = [
    ("h1", "A-CPINTB2-4b7ccc66-H01-R0"),
    ("h2", "A-CPINTB4-f2a23853-H02-R0"),
    ("h3", "A-CPINTB4-f2a23853-H03-R0"),
    ("h4", "A-CPINTB2-4b7ccc66-H04-R0"),
];

/// Run id currently bound to a scenario slot.
#[must_use]
pub fn scenario_run_id(scenario: &str) -> &'static str {
    CASES
        .iter()
        .find(|(slot, _)| *slot == scenario)
        .map_or("", |(_, run_id)| *run_id)
}
pub const MESSAGE: &str = "聊聊清晨的风景吧。";
const REPLY: &str = "清晨的风轻轻吹过树梢。";
const RETRY_REPLY: &str = "晨光照亮了窗边的叶子。";
const PREFIX: &str = "清晨的风";
const ERROR_MARKER: &str = "CP-INT-B2-PROVIDER-ERROR";

pub fn is_scenario(scenario: &str) -> bool {
    CASES.iter().any(|(s, _)| *s == scenario)
}

pub fn validate_mode(run_id: &str, scenario: &str, approval: &str) -> Result<(), String> {
    if approval.is_empty() && CASES.contains(&(scenario, run_id)) {
        Ok(())
    } else {
        Err("CP-INT B2 requires an exact frozen scenario/run ID and empty live approval".into())
    }
}

struct StreamRecorder {
    scenario: String,
    calls: Mutex<Vec<Value>>,
    violations: Mutex<Vec<String>>,
    release: Semaphore,
}

impl StreamRecorder {
    fn unexpected(&self, method: &str) -> AppError {
        self.violations.lock().unwrap().push(method.into());
        AppError::Unknown(format!("CP-INT-B2 unexpected {method}"))
    }

    fn record(&self, method: &str, model: &str, prompt: &str, opts: Option<&LlmGenerateOpts>) {
        if model != support::MODEL_ID || opts.is_none() {
            self.violations
                .lock()
                .unwrap()
                .push("model/options mismatch".into());
        }
        self.calls.lock().unwrap().push(json!({
            "method": method, "model": model, "prompt": prompt,
            "prompt_sha256": artifacts::sha256_hex(prompt.as_bytes()),
            "opts": opts.map(|o| json!({"keep_alive": o.keep_alive,
                "want_metrics": o.want_metrics, "temperature": o.temperature,
                "top_p": o.top_p, "max_output_tokens": o.max_output_tokens,
                "preferred_context_tokens": o.preferred_context_tokens})),
        }));
    }
}

#[async_trait]
impl LlmClient for StreamRecorder {
    async fn generate(&self, _: &str, _: &str) -> KernelResult<String> {
        Err(self.unexpected("generate"))
    }
    async fn generate_tag(&self, _: &str, _: &str) -> KernelResult<String> {
        Err(self.unexpected("generate_tag"))
    }
    async fn startup_probe(&self) -> KernelResult<()> {
        Err(self.unexpected("startup_probe"))
    }
    async fn generate_stream(&self, _: &str, _: &str, _: LlmTokenSink) -> KernelResult<String> {
        Err(self.unexpected("generate_stream"))
    }
    async fn generate_with_opts(
        &self,
        model: &str,
        prompt: &str,
        opts: Option<&LlmGenerateOpts>,
    ) -> KernelResult<LlmGenerateOutcome> {
        self.record("generate_with_opts", model, prompt, opts);
        if self.scenario != "h4" || self.calls.lock().unwrap().len() != 2 {
            return Err(self.unexpected("extra non-stream generation"));
        }
        Ok(LlmGenerateOutcome {
            reply: RETRY_REPLY.into(),
            prompt_eval_ms: None,
        })
    }
    async fn generate_stream_with_opts(
        &self,
        model: &str,
        prompt: &str,
        sink: LlmTokenSink,
        opts: Option<&LlmGenerateOpts>,
    ) -> KernelResult<LlmGenerateOutcome> {
        self.record("generate_stream_with_opts", model, prompt, opts);
        if self.calls.lock().unwrap().len() != 1 {
            return Err(self.unexpected("extra stream generation"));
        }
        if self.scenario == "h2" {
            return Err(AppError::OllamaError(ERROR_MARKER.into()));
        }
        sink(PREFIX);
        if self.scenario == "h3" {
            return Err(AppError::OllamaError(ERROR_MARKER.into()));
        }
        if self.scenario == "h4" {
            let permit = self
                .release
                .acquire()
                .await
                .map_err(|_| self.unexpected("release closed"))?;
            permit.forget();
        }
        sink("轻轻吹过树梢。");
        Ok(LlmGenerateOutcome {
            reply: REPLY.into(),
            prompt_eval_ms: None,
        })
    }
}

pub(super) async fn post(
    app: &axum::Router,
    uri: &str,
    body: &Value,
) -> Result<axum::response::Response, String> {
    app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(uri)
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_vec(body).map_err(|e| e.to_string())?,
                ))
                .map_err(|e| e.to_string())?,
        )
        .await
        .map_err(|e| e.to_string())
}

fn events(raw: &str) -> Result<Vec<(String, Value)>, String> {
    let mut parsed = Vec::new();
    for block in raw.split("\n\n") {
        let event = block
            .lines()
            .find_map(|l| l.strip_prefix("event:").map(str::trim));
        let data = block
            .lines()
            .find_map(|l| l.strip_prefix("data:").map(str::trim));
        if let (Some(event), Some(data)) = (event, data) {
            parsed.push((
                event.into(),
                serde_json::from_str(data).map_err(|e| e.to_string())?,
            ));
        }
    }
    Ok(parsed)
}

pub(super) async fn snapshot(root: &Path) -> Result<Value, String> {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(
            sqlx::sqlite::SqliteConnectOptions::new()
                .filename(root.join("app-data/app.db"))
                .read_only(true),
        )
        .await
        .map_err(|e| e.to_string())?;
    let rows = sqlx::query("SELECT id, session_id, turn_index, sender, content FROM chat_messages ORDER BY turn_index, sender")
        .fetch_all(&pool).await;
    let sessions = sqlx::query("SELECT session_id, role_id, scene_id, message_count FROM chat_sessions ORDER BY session_id")
        .fetch_all(&pool).await;
    pool.close().await;
    let rows = rows.map_err(|e| e.to_string())?;
    let sessions = sessions.map_err(|e| e.to_string())?;
    Ok(json!({
        "messages": rows.iter().map(|r| json!({"id":r.get::<String,_>("id"),
            "session_id":r.get::<String,_>("session_id"), "turn_index":r.get::<i64,_>("turn_index"),
            "sender":r.get::<String,_>("sender"), "content":r.get::<String,_>("content")})).collect::<Vec<_>>(),
        "sessions": sessions.iter().map(|r| json!({"session_id":r.get::<String,_>("session_id"),
            "role_id":r.get::<String,_>("role_id"), "scene_id":r.get::<Option<String>,_>("scene_id"),
            "message_count":r.get::<Option<i64>,_>("message_count")})).collect::<Vec<_>>()
    }))
}

fn require(ok: bool, message: &str) -> Result<(), String> {
    if ok {
        Ok(())
    } else {
        Err(message.into())
    }
}

pub(super) fn verify_pair(db: &Value, response: &Value) -> Result<(), String> {
    let failures = crate::verify_sqlite_rows(
        db["messages"].as_array().ok_or("messages missing")?,
        db["sessions"].as_array().ok_or("sessions missing")?,
        Some(response),
        MESSAGE,
    );
    require(
        failures.is_empty(),
        &format!("SQLite binding: {failures:?}"),
    )
}

async fn exercise(
    scenario: &str,
    root: &Path,
    state: Arc<AppState>,
    rec: &StreamRecorder,
    facts: &mut Map<String, Value>,
) -> Result<(), String> {
    let app = oclive_kernel_host::http_api::api_router(Arc::clone(&state));
    let request = json!({"role_path":fixture::role_dir(root), "message":MESSAGE,
        "scene_id":"default", "session_id":null, "adult":null});
    facts.insert("request".into(), request.clone());
    // A rejected input must not enter generation or create chat rows.
    let mut invalid = request.clone();
    invalid["message"] = json!("   ");
    let rejected = post(&app, "/chat/stream", &invalid).await?;
    require(
        rejected.status().as_u16() == 400,
        "empty input must be rejected",
    )?;
    let bytes = to_bytes(rejected.into_body(), 1_048_576)
        .await
        .map_err(|e| e.to_string())?;
    let error: Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    facts.insert("rejected_input".into(), error.clone());
    require(
        error["error"]["code"] == "EMPTY_MESSAGE",
        "wrong empty-input code",
    )?;
    require(
        rec.calls.lock().unwrap().is_empty(),
        "rejected request generated",
    )?;
    require(
        snapshot(root).await?["messages"]
            .as_array()
            .is_some_and(Vec::is_empty),
        "rejected input persisted",
    )?;

    let response = post(&app, "/chat/stream", &request).await?;
    require(response.status().as_u16() == 200, "stream HTTP status")?;
    require(
        response
            .headers()
            .get("content-type")
            .is_some_and(|v| v == "text/event-stream"),
        "SSE content type",
    )?;
    if scenario == "h4" {
        let mut body = response.into_body().into_data_stream();
        let frame = body
            .next()
            .await
            .ok_or("missing first frame")?
            .map_err(|e| e.to_string())?;
        let prefix = String::from_utf8(frame.to_vec()).map_err(|e| e.to_string())?;
        facts.insert("sse".into(), json!(prefix));
        let parsed = events(&prefix)?;
        require(
            parsed.len() == 1 && parsed[0].0 == "token" && parsed[0].1["token"] == PREFIX,
            "first token boundary",
        )?;
        // Drop the REAL Axum response consumer while provider generation is gated.
        drop(body);
        facts.insert("body_dropped_before_provider_release".into(), json!(true));
        rec.release.add_permits(1);
        let first = tokio::time::timeout(Duration::from_secs(10), async {
            loop {
                let db = snapshot(root).await?;
                if db["messages"].as_array().is_some_and(|r| r.len() == 2) {
                    return Ok::<_, String>(db);
                }
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        })
        .await
        .map_err(|_| "original turn did not persist within 10s")??;
        let completed = tokio::time::timeout(
            Duration::from_secs(10),
            state.turn_lock_for(ROLE_ID).lock_owned(),
        )
        .await
        .map_err(|_| "original turn did not release its lock")?;
        drop(completed);
        facts.insert("sqlite_before_retry".into(), first.clone());
        require(
            first["messages"]
                .as_array()
                .unwrap()
                .iter()
                .any(|r| r["sender"] == "assistant" && r["content"] == REPLY),
            "original assistant content",
        )?;
        let retry = post(&app, "/chat", &request).await?;
        require(retry.status().as_u16() == 200, "retry HTTP status")?;
        let bytes = to_bytes(retry.into_body(), 1_048_576)
            .await
            .map_err(|e| e.to_string())?;
        let final_response: Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
        facts.insert("response".into(), final_response.clone());
        let db = snapshot(root).await?;
        facts.insert("sqlite".into(), db.clone());
        require(
            final_response["reply"] == RETRY_REPLY && final_response["reply_is_fallback"] == false,
            "retry reply changed",
        )?;
        let rows = db["messages"].as_array().ok_or("messages missing")?;
        require(
            rows.len() == 4 && db["sessions"][0]["message_count"] == 4,
            "expected two persisted turns",
        )?;
        require(
            rows.iter()
                .filter(|r| r["sender"] == "user" && r["content"] == MESSAGE)
                .count()
                == 2,
            "duplicate input not preserved",
        )?;
        require(
            first["messages"]
                .as_array()
                .unwrap()
                .iter()
                .all(|r| rows.contains(r)),
            "original rows changed",
        )?;
        let new_rows: Vec<Value> = rows
            .iter()
            .filter(|r| !first["messages"].as_array().unwrap().contains(r))
            .cloned()
            .collect();
        let mut isolated = db.clone();
        isolated["messages"] = json!(new_rows);
        isolated["sessions"][0]["message_count"] = json!(2);
        verify_pair(&isolated, &final_response)?;
        facts.insert(
            "product_assessment".into(),
            json!("DUPLICATE_TURN_RISK_REPRODUCED_NOT_ACCEPTED"),
        );
    } else {
        let bytes = to_bytes(response.into_body(), 1_048_576)
            .await
            .map_err(|e| e.to_string())?;
        let raw = String::from_utf8(bytes.to_vec()).map_err(|e| e.to_string())?;
        facts.insert("sse".into(), json!(raw));
        let parsed = events(&raw)?;
        let done: Vec<_> = parsed.iter().filter(|(e, _)| e == "done").collect();
        require(
            done.len() == 1 && parsed.last().is_some_and(|(e, _)| e == "done"),
            "one terminal done required",
        )?;
        require(
            parsed.iter().all(|(e, _)| e == "token" || e == "done"),
            "unexpected SSE event",
        )?;
        let final_response = done[0].1.clone();
        facts.insert("response".into(), final_response.clone());
        let tokens: String = parsed
            .iter()
            .filter(|(e, _)| e == "token")
            .map(|(_, d)| d["token"].as_str().unwrap_or_default())
            .collect();
        facts.insert("tokens".into(), json!(tokens));
        if scenario == "h1" {
            require(
                final_response["reply"] == REPLY
                    && tokens == REPLY
                    && final_response["reply_is_fallback"] == false,
                "success token/final mismatch",
            )?;
        } else {
            require(
                final_response["reply_is_fallback"] == true,
                "provider error must produce Host fallback",
            )?;
            let reason: Value = serde_json::from_str(
                final_response["llm_fallback_reason"]
                    .as_str()
                    .ok_or("missing reason")?,
            )
            .map_err(|e| e.to_string())?;
            require(
                reason["code"] == "LLM_ERROR"
                    && reason["message"] == format!("Ollama error: {ERROR_MARKER}"),
                "typed fallback reason",
            )?;
            let reply = final_response["reply"]
                .as_str()
                .ok_or("missing fallback reply")?;
            require(!reply.is_empty(), "empty fallback")?;
            // CP-INT B4/H03 contract: a prefix the client already received must never be glued
            // to the fallback reply. The stale preview stays on the wire as-is, and the single
            // authoritative fallback travels only in the terminal `done` event and the row.
            if scenario == "h3" {
                require(
                    tokens == PREFIX,
                    "partial preview must not absorb the fallback",
                )?;
                require(
                    tokens != format!("{PREFIX}{reply}"),
                    "fallback must never be appended to the stale partial prefix",
                )?;
            } else {
                require(
                    tokens == reply,
                    "pre-first-token fallback must be the only streamed token",
                )?;
            }
        }
        let db = snapshot(root).await?;
        facts.insert("sqlite".into(), db.clone());
        verify_pair(&db, &final_response)?;
        if scenario == "h3" {
            let persisted: Vec<String> = db["messages"]
                .as_array()
                .ok_or("messages missing")?
                .iter()
                .filter(|row| row["sender"] == "assistant")
                .filter_map(|row| row["content"].as_str().map(str::to_string))
                .collect();
            let reply = final_response["reply"].as_str().ok_or("missing reply")?;
            require(
                persisted.len() == 1 && persisted[0] == reply,
                "persisted text must be the single authoritative fallback",
            )?;
            require(
                persisted[0] != tokens,
                "persisted text must not be the stale partial preview",
            )?;
        }
        facts.insert(
            "product_assessment".into(),
            json!(if scenario == "h3" {
                "PARTIAL_PREVIEW_KEPT_FALLBACK_WITHHELD_FROM_TOKENS"
            } else {
                "LIMITED_ROUTE_ASSERTIONS_PASSED"
            }),
        );
    }
    Ok(())
}

pub async fn run_scenario(scenario: &str, root: &Path) -> (Map<String, Value>, Vec<String>) {
    let mut facts = Map::new();
    let mut failures = Vec::new();
    let setup = (|| {
        support::create_new_dir_verified(
            &RealFs,
            &root.join("roles"),
            &fixture::role_dir(root),
            "http_role",
            &OpsLedger::default(),
        )?;
        fixture::write_role_pack(root)?;
        fixture::assert_role_pack_loads(root)
    })();
    match setup {
        Ok(role) => {
            facts.insert("fixture_role".into(), role);
        }
        Err(e) => return (facts, vec![e]),
    }
    let rec = Arc::new(StreamRecorder {
        scenario: scenario.into(),
        calls: Mutex::new(Vec::new()),
        violations: Mutex::new(Vec::new()),
        release: Semaphore::new(0),
    });
    let profile = oclive_kernel_host::domain::host_profile::HostProfile {
        event_impact_llm: false,
        skip_agent: true,
        ..Default::default()
    };
    let state = match AppStateBuilder::production(
        root.join("app-data/app.db"),
        root.join("roles"),
        root.join("app-data"),
    )
    .with_host_profile(profile)
    .with_llm_client(Arc::clone(&rec) as Arc<dyn LlmClient>)
    .build()
    .await
    {
        Ok(state) => Arc::new(state),
        Err(e) => return (facts, vec![format!("HTTP state build: {e}")]),
    };
    facts.insert(
        "assembly".into(),
        json!({"production_builder":true, "real_network":false,
        "tcp_listener":false, "trace_path":null, "ollama_present":state.ollama.is_some(),
        "performance_present":state.performance_llm.is_some()}),
    );
    if state.ollama.is_some() || state.performance_llm.is_some() {
        failures.push("unexpected real provider assembly".into());
    } else {
        match tokio::time::timeout(
            Duration::from_secs(45),
            exercise(scenario, root, Arc::clone(&state), &rec, &mut facts),
        )
        .await
        {
            Ok(Ok(())) => (),
            Ok(Err(e)) => failures.push(e),
            Err(_) => failures.push("HTTP exercise timeout".into()),
        }
    }
    // Match available public cleanup surfaces; NOT OcliveKernel::shutdown.
    rec.release.add_permits(1);
    state.directory_plugins.shutdown_all();
    facts.insert(
        "cleanup".into(),
        json!({"directory_shutdown_called":true, "production_pool_explicitly_closed":false,
            "oclive_kernel_shutdown_called":false, "trace_configured":false,
            "detached_tasks_joined":false, "owned_child_exit_required":true}),
    );
    // The production pool has no public close surface on AppState. Parent-owned
    // child exit is the isolation boundary; do not claim a kernel shutdown.
    drop(state);
    let calls = rec.calls.lock().unwrap().clone();
    let violations = rec.violations.lock().unwrap().clone();
    let expected: Vec<&str> = if scenario == "h4" {
        vec!["generate_stream_with_opts", "generate_with_opts"]
    } else {
        vec!["generate_stream_with_opts"]
    };
    if calls
        .iter()
        .map(|c| c["method"].as_str().unwrap_or_default())
        .collect::<Vec<_>>()
        != expected
        || !violations.is_empty()
    {
        failures.push("generation ledger mismatch".into());
    }
    facts.insert("calls".into(), json!(calls));
    facts.insert("violations".into(), json!(violations));
    match artifacts::scan_files(&root.join("chats")) {
        Ok(scan) => {
            let present = scan.state == artifacts::ScanState::Present;
            facts.insert(
                "chat_mirror".into(),
                json!({"present":present, "entries":scan.entries}),
            );
            if !present || !scan.entries.is_empty() {
                failures.push("chat mirror directory missing or unexpectedly populated".into());
            }
        }
        Err(e) => failures.push(e),
    }
    (facts, failures)
}

#[test]
fn h_v1_identity_pairs_are_exact_and_old_ids_rejected() {
    assert_eq!(
        CASES,
        [
            ("h1", "A-CPINTB2-4b7ccc66-H01-R0"),
            ("h2", "A-CPINTB4-f2a23853-H02-R0"),
            ("h3", "A-CPINTB4-f2a23853-H03-R0"),
            ("h4", "A-CPINTB2-4b7ccc66-H04-R0")
        ]
    );
    // The reserved B4 identities are recognised for the h2/h3 scenario slots.
    for (scenario, run_id) in RESERVED_B4_RUN_IDS {
        assert_eq!(scenario_run_id(scenario), run_id);
        assert!(validate_mode(run_id, scenario, "").is_ok());
        assert!(support::validate_run_mode(run_id, scenario, "").is_ok());
        assert!(validate_mode(run_id, scenario, "live").is_err());
    }
    for (scenario, id) in CASES {
        assert!(validate_mode(id, scenario, "").is_ok());
        assert!(validate_mode(id, scenario, "live").is_err());
        assert!(validate_mode("A-EV1-4b7ccc66-E01-R0", scenario, "").is_err());
        assert!(validate_mode(id, "h9", "").is_err());
        assert!(support::validate_run_mode(id, scenario, "").is_ok());
        for (other_scenario, other_id) in CASES {
            if other_scenario != scenario {
                assert!(validate_mode(other_id, scenario, "").is_err());
            }
        }
    }
    // Every consumed B2 identity is rejected for every scenario slot it is no longer bound
    // to, so the reserved B4 ids can never be silently replaced by an exhausted one.
    for consumed in CONSUMED_RUN_IDS {
        for (scenario, current) in CASES {
            if current == consumed {
                // h1/h4 still point at their original B2 identity.
                assert!(validate_mode(consumed, scenario, "").is_ok());
                continue;
            }
            assert!(
                validate_mode(consumed, scenario, "").is_err(),
                "consumed identity {consumed} must not be admitted for {scenario}"
            );
            assert!(support::validate_run_mode(consumed, scenario, "").is_err());
        }
    }
    // The h2/h3 slots no longer admit the consumed B2 ids they used to point at.
    for (scenario, run_id) in [
        ("h2", "A-CPINTB2-4b7ccc66-H02-R0"),
        ("h3", "A-CPINTB2-4b7ccc66-H03-R0"),
    ] {
        assert!(
            validate_mode(run_id, scenario, "").is_err(),
            "h2/h3 must reject the consumed B2 identity {run_id}"
        );
    }
    // Reserved ids belong only to their own slot.
    for (scenario, run_id) in RESERVED_B4_RUN_IDS {
        for (other, _) in CASES {
            if other != scenario {
                assert!(validate_mode(run_id, other, "").is_err());
            }
        }
    }
    // The reserved family resolves to the CPB4V1 tree root, consumed families keep CPB3V2.
    for (_scenario, run_id) in RESERVED_B4_RUN_IDS {
        assert_eq!(support::tree_prefix_for(run_id), "CPB4V1-");
        assert!(support::tree_root(run_id).ends_with(format!("CPB4V1-{run_id}")));
    }
    for run_id in ["A-CPINTB2-4b7ccc66-H01-R0", "A-CPINTB3-f2a23853-I01-R2"] {
        assert_eq!(support::tree_prefix_for(run_id), "CPB3V2-");
        assert!(support::tree_root(run_id).ends_with(format!("CPB3V2-{run_id}")));
    }
    for (driver, scenario) in [
        ("h1_driver_http_stream_success", "h1"),
        ("h2_driver_http_stream_error", "h2"),
        ("h3_driver_http_partial_error", "h3"),
        ("h4_driver_http_disconnect_retry", "h4"),
    ] {
        assert_eq!(support::scenario_of_driver(driver), scenario);
    }
}
