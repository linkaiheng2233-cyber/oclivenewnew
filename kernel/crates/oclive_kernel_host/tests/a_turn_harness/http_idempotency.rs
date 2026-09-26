//! CP-INT B3: real router + isolated SQLite, no TCP listener or real provider.
//! Same-turn admission and recover-only routes; no real provider or TCP.

use super::http_entry::{post, snapshot, verify_pair};
use crate::support::{self, artifacts, fixture, OpsLedger, RealFs, ROLE_ID};
use async_trait::async_trait;
use axum::body::to_bytes;
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

pub const CASES: [(&str, &str); 4] = [
    ("i1", "A-CPINTB3-f2a23853-I01-R2"),
    ("i2", "A-CPINTB3-f2a23853-I02-R1"),
    ("i3", "A-CPINTB3-f2a23853-I03-R1"),
    ("i4", "A-CPINTB3-f2a23853-I04-R1"),
];

/// Every B3 identity already consumed by a first scenario launch. None may be admitted again.
pub const CONSUMED_RUN_IDS: [&str; 5] = [
    "A-CPINTB3-f2a23853-I01-R0",
    "A-CPINTB3-f2a23853-I01-R1",
    "A-CPINTB3-f2a23853-I02-R0",
    "A-CPINTB3-f2a23853-I03-R0",
    "A-CPINTB3-f2a23853-I04-R0",
];
pub const MESSAGE: &str = "聊聊清晨的风景吧。";
const REPLY: &str = "清晨的风轻轻吹过树梢。";
/// Second ordinary turn in the i3 scenario; must differ from [REPLY] so the product's
/// anti-repetition rule cannot turn the intentional turn into a fallback.
const INTENTIONAL_REPLY: &str = "晨光照亮了窗边的叶子。";
const PREFIX: &str = "清晨的风";

pub fn is_scenario(scenario: &str) -> bool {
    CASES.iter().any(|(s, _)| *s == scenario)
}

pub fn validate_mode(run_id: &str, scenario: &str, approval: &str) -> Result<(), String> {
    if approval.is_empty() && CASES.contains(&(scenario, run_id)) {
        Ok(())
    } else {
        Err("CP-INT B3 requires an exact frozen scenario/run ID and empty live approval".into())
    }
}

/// Which generation method a scenario is allowed to use.
#[derive(Clone, Copy, PartialEq, Eq)]
enum StreamMode {
    None,
    Plain,
    Stream,
}

/// Shared admission state. All accounting lives under one lock so "attempts = admitted + denied"
/// holds atomically even when the product issues concurrent calls.
struct Admission {
    attempts: Vec<Value>,
    admitted_outputs: Vec<Value>,
    denied: Vec<Value>,
}

struct StreamRecorder {
    calls: Mutex<Vec<Value>>,
    violations: Mutex<Vec<String>>,
    release: Semaphore,
    /// Bounded reply queue. Once exhausted the call is **denied**; it never repeats the last entry.
    script: Mutex<std::collections::VecDeque<&'static str>>,
    /// Maximum number of generation calls that may produce output in this scenario.
    budget: usize,
    allowed: StreamMode,
    admission: Mutex<Admission>,
}

impl Admission {
    fn zero() -> Self {
        Self {
            attempts: Vec::new(),
            admitted_outputs: Vec::new(),
            denied: Vec::new(),
        }
    }
}

impl StreamRecorder {
    fn new(script: Vec<&'static str>, budget: usize, allowed: StreamMode) -> Self {
        Self {
            calls: Mutex::new(Vec::new()),
            violations: Mutex::new(Vec::new()),
            release: Semaphore::new(0),
            script: Mutex::new(script.into()),
            budget,
            allowed,
            admission: Mutex::new(Admission::zero()),
        }
    }

    fn unexpected(&self, method: &str) -> AppError {
        self.violations.lock().unwrap().push(method.into());
        AppError::Unknown(format!("CP-INT-B2 unexpected {method}"))
    }

    /// Unified denial accounting for a generation attempt this double must refuse.
    ///
    /// Every **generation** method funnels through here or through [`Self::admit`], so
    /// `attempts = admitted + denied` holds for the whole double rather than only for the one
    /// allowed method. The denial is recorded and returned **before** any token or reply exists.
    /// `startup_probe` is deliberately *not* a generation and keeps its own separate rejection.
    fn deny_generation(&self, method: &str, reason: &str) -> AppError {
        let mut admission = self.admission.lock().unwrap();
        let sequence = admission.attempts.len() + 1;
        admission
            .attempts
            .push(json!({"sequence": sequence, "method": method}));
        admission
            .denied
            .push(json!({"sequence": sequence, "method": method, "reason": reason}));
        drop(admission);
        self.violations
            .lock()
            .unwrap()
            .push(format!("{method} denied: {reason}"));
        AppError::Unknown(format!("CP-INT B3 denied generation: {reason}"))
    }

    /// Atomically admit one generation attempt and hand back its scripted reply.
    ///
    /// Denial happens **before** any token or reply is produced: a wrong method, an exhausted
    /// script, or a budget overrun yields `Err` and is recorded as a denied attempt.
    fn admit(
        &self,
        method: &str,
        model: &str,
        prompt: &str,
        opts: Option<&LlmGenerateOpts>,
    ) -> KernelResult<&'static str> {
        let mut admission = self.admission.lock().unwrap();
        let sequence = admission.attempts.len() + 1;
        admission
            .attempts
            .push(json!({"sequence": sequence, "method": method}));

        let method_ok = match self.allowed {
            StreamMode::None => false,
            StreamMode::Plain => method == "generate_with_opts",
            StreamMode::Stream => method == "generate_stream_with_opts",
        };
        let reason = if !method_ok {
            Some("method not allowed for this scenario")
        } else if sequence > self.budget {
            Some("generation budget exceeded")
        } else if self.script.lock().unwrap().is_empty() {
            Some("scripted replies exhausted")
        } else {
            None
        };
        if let Some(reason) = reason {
            admission
                .denied
                .push(json!({"sequence": sequence, "method": method, "reason": reason}));
            self.violations
                .lock()
                .unwrap()
                .push(format!("{method} denied: {reason}"));
            return Err(AppError::Unknown(format!(
                "CP-INT B3 denied generation: {reason}"
            )));
        }

        let reply = self
            .script
            .lock()
            .unwrap()
            .pop_front()
            .expect("script non-empty was checked under the admission lock");
        admission
            .admitted_outputs
            .push(json!({"sequence": sequence, "method": method, "reply": reply}));
        drop(admission);

        if model != support::MODEL_ID || opts.is_none() {
            self.violations
                .lock()
                .unwrap()
                .push("model/options mismatch".into());
        }
        self.calls.lock().unwrap().push(json!({
            "method": method, "model": model, "prompt": prompt, "reply": reply,
            "prompt_sha256": artifacts::sha256_hex(prompt.as_bytes()),
            "opts": opts.map(|o| json!({"keep_alive": o.keep_alive,
                "want_metrics": o.want_metrics, "temperature": o.temperature,
                "top_p": o.top_p, "max_output_tokens": o.max_output_tokens,
                "preferred_context_tokens": o.preferred_context_tokens})),
        }));
        Ok(reply)
    }

    fn admission_facts(&self) -> Value {
        let admission = self.admission.lock().unwrap();
        json!({
            "generation_attempts": admission.attempts.len(),
            "admitted_outputs": admission.admitted_outputs.len(),
            "denied_attempts": admission.denied.len(),
            "attempt_log": admission.attempts,
            "admitted_log": admission.admitted_outputs,
            "denied_log": admission.denied,
        })
    }
}

#[async_trait]
impl LlmClient for StreamRecorder {
    async fn generate(&self, _: &str, _: &str) -> KernelResult<String> {
        // A generation attempt on a legacy method is denied through the shared admission
        // accounting, so it can never produce text while escaping attempts/denied counts.
        Err(self.deny_generation(
            "generate",
            "legacy generate is not the identified-turn method",
        ))
    }
    async fn generate_tag(&self, _: &str, _: &str) -> KernelResult<String> {
        Err(self.deny_generation(
            "generate_tag",
            "tag generation is not the identified-turn method",
        ))
    }
    async fn startup_probe(&self) -> KernelResult<()> {
        // Not a generation: startup probing has its own rejection and takes no admission slot.
        Err(self.unexpected("startup_probe"))
    }
    async fn generate_stream(&self, _: &str, _: &str, _sink: LlmTokenSink) -> KernelResult<String> {
        // Legacy streaming is denied *before* the sink is touched: no token may be emitted.
        Err(self.deny_generation(
            "generate_stream",
            "legacy streaming is not the identified-turn method",
        ))
    }
    async fn generate_with_opts(
        &self,
        model: &str,
        prompt: &str,
        opts: Option<&LlmGenerateOpts>,
    ) -> KernelResult<LlmGenerateOutcome> {
        let reply = self.admit("generate_with_opts", model, prompt, opts)?;
        Ok(LlmGenerateOutcome {
            reply: reply.to_string(),
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
        let reply = self.admit("generate_stream_with_opts", model, prompt, opts)?;
        if reply == REPLY {
            // Two-frame form for the gate-blocking scenarios: the first frame proves tokens flow
            // before the provider is released.
            sink(PREFIX);
            let permit = self
                .release
                .acquire()
                .await
                .map_err(|_| self.unexpected("release closed"))?;
            permit.forget();
            sink("轻轻吹过树梢。");
        } else {
            sink(reply);
        }
        Ok(LlmGenerateOutcome {
            reply: reply.to_string(),
            prompt_eval_ms: None,
        })
    }
}

fn require(ok: bool, message: &str) -> Result<(), String> {
    if ok {
        Ok(())
    } else {
        Err(message.into())
    }
}

/// Uniform stop-loss for the generation admission budget.
///
/// Checked **before every adjacent route dispatch**, not only at selected boundaries: once any
/// generation call has been denied (wrong method, exhausted script, budget overrun) or any other
/// admission violation has been recorded, the remaining business dispatch for this scenario must
/// not happen. In a healthy run this is a no-op, so route order and counts are unchanged.
fn assert_no_admission_violation(rec: &StreamRecorder, context: &str) -> Result<(), String> {
    let violations = rec.violations.lock().unwrap();
    if violations.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "generation admission violation {context}: {}",
            violations.join("; ")
        ))
    }
}

async fn json_response(response: axum::response::Response, status: u16) -> Result<Value, String> {
    require(
        response.status().as_u16() == status,
        "unexpected HTTP status",
    )?;
    let bytes = to_bytes(response.into_body(), 1_048_576)
        .await
        .map_err(|e| e.to_string())?;
    serde_json::from_slice(&bytes).map_err(|e| e.to_string())
}

/// Round-trips one complete top-level JSON payload through [`SendMessageResponse`] and
/// re-serializes it, so two payloads can be compared field-by-field with the *same* numeric
/// representation on both sides. Both `/chat` (flattened `ChatApiResponse`) and `/chat/recover`
/// (bare DTO) are accepted: `serde` ignores unknown transport-level keys such as
/// `personality_source` / `session_id`.
fn dto_json(payload: &Value) -> Result<Value, String> {
    let dto: oclive_kernel_types::models::SendMessageResponse =
        serde_json::from_value(payload.clone()).map_err(|e| e.to_string())?;
    serde_json::to_value(dto).map_err(|e| e.to_string())
}

async fn receipts(root: &Path) -> Result<Value, String> {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(
            sqlx::sqlite::SqliteConnectOptions::new()
                .filename(root.join("app-data/app.db"))
                .read_only(true),
        )
        .await
        .map_err(|e| e.to_string())?;
    let rows = sqlx::query("SELECT request_id, status, payload_sha256, user_message_id, assistant_message_id FROM chat_request_receipts ORDER BY request_id")
        .fetch_all(&pool).await;
    pool.close().await;
    Ok(json!(rows
        .map_err(|e| e.to_string())?
        .iter()
        .map(|r| json!({
            "request_id":r.get::<String,_>("request_id"),"status":r.get::<String,_>("status"),
            "payload_sha256":r.get::<String,_>("payload_sha256"),
            "user_message_id":r.get::<Option<String>,_>("user_message_id"),
            "assistant_message_id":r.get::<Option<String>,_>("assistant_message_id")
        }))
        .collect::<Vec<_>>()))
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
        "scene_id":"default", "session_id":null, "adult":null,
        "client_request_id":"de979898-69ef-4c2d-97b6-22218ca346dc"});
    let recovery = json!({"role_id":ROLE_ID, "user_message":MESSAGE,
        "scene_id":"default", "session_id":null, "adult":null,
        "client_request_id":request["client_request_id"]});
    facts.insert("request".into(), request.clone());
    facts.insert("recovery_request".into(), recovery.clone());
    let response;
    if scenario == "i1" || scenario == "i2" {
        let stream = post(&app, "/chat/stream", &request).await?;
        require(stream.status().as_u16() == 200, "stream status")?;
        let mut body = stream.into_body().into_data_stream();
        let frame = body
            .next()
            .await
            .ok_or("missing first token")?
            .map_err(|e| e.to_string())?;
        let mut raw = String::from_utf8(frame.to_vec()).map_err(|e| e.to_string())?;
        require(
            raw.contains(PREFIX) && !raw.contains("event: done"),
            "provider gate boundary",
        )?;
        let retained = if scenario == "i2" {
            drop(body);
            None
        } else {
            Some(body)
        };
        facts.insert(
            "body_dropped_before_provider_release".into(),
            json!(scenario == "i2"),
        );
        let (uri, payload) = if scenario == "i1" {
            ("/chat", &request)
        } else {
            ("/chat/recover", &recovery)
        };
        assert_no_admission_violation(rec, "before the concurrent duplicate/recovery route")?;
        let waiting = post(&app, uri, payload);
        tokio::pin!(waiting);
        tokio::select! {
            result = &mut waiting => return Err(format!("duplicate/recovery returned before release: {:?}", result.map(|r| r.status()))),
            () = tokio::time::sleep(Duration::from_millis(200)) => (),
        }
        require(
            rec.calls.lock().unwrap().len() == 1,
            "duplicate entered provider before release",
        )?;
        facts.insert("waiter_pending_before_release".into(), json!(true));
        facts.insert("sqlite_while_provider_gated".into(), snapshot(root).await?);
        rec.release.add_permits(1);
        response = json_response(waiting.await?, 200).await?;
        if let Some(mut body) = retained {
            while let Some(frame) = body.next().await {
                raw.push_str(
                    &String::from_utf8(frame.map_err(|e| e.to_string())?.to_vec())
                        .map_err(|e| e.to_string())?,
                );
            }
            let done: Vec<Value> = raw
                .split("\n\n")
                .filter(|b| b.lines().any(|l| l.trim() == "event: done"))
                .filter_map(|b| {
                    b.lines()
                        .find_map(|l| l.strip_prefix("data:").map(str::trim))
                })
                .map(serde_json::from_str)
                .collect::<Result<_, _>>()
                .map_err(|e| e.to_string())?;
            require(done.len() == 1, "one terminal done required")?;
            let dto: oclive_kernel_types::models::SendMessageResponse =
                serde_json::from_value(response.clone()).map_err(|e| e.to_string())?;
            let stream_dto: oclive_kernel_types::models::SendMessageResponse =
                serde_json::from_value(done[0].clone()).map_err(|e| e.to_string())?;
            require(
                serde_json::to_value(stream_dto).map_err(|e| e.to_string())?
                    == serde_json::to_value(dto).map_err(|e| e.to_string())?,
                "stream/plain authoritative DTO mismatch",
            )?;
        }
        facts.insert("sse".into(), json!(raw));
    } else {
        response = json_response(post(&app, "/chat", &request).await?, 200).await?;
    }
    require(
        response["reply"] == REPLY && response["reply_is_fallback"] == false,
        "original reply changed",
    )?;
    let before = snapshot(root).await?;
    verify_pair(&before, &response)?;
    facts.insert("sqlite_original".into(), before.clone());
    // A budget/method/script violation must stop the remaining business dispatch for this scenario.
    assert_no_admission_violation(rec, "before recovery")?;
    let recovered = if scenario == "i4" {
        // Reassembly against the same durable DB, without loading a role runtime.
        // This is not a process-crash simulation; parent-owned child still bounds cleanup.
        assert_no_admission_violation(rec, "before the reassembly route")?;
        drop(app);
        // Reassembly must recover without producing any generation at all.
        let fresh_rec = Arc::new(StreamRecorder::new(Vec::new(), 0, StreamMode::None));
        let fresh = Arc::new(
            AppStateBuilder::production(
                root.join("app-data/app.db"),
                root.join("roles"),
                root.join("app-data"),
            )
            .with_host_profile(oclive_kernel_host::domain::host_profile::HostProfile {
                event_impact_llm: false,
                skip_agent: true,
                ..Default::default()
            })
            .with_llm_client(Arc::clone(&fresh_rec) as Arc<dyn LlmClient>)
            .build()
            .await
            .map_err(|e| e.to_string())?,
        );
        let rebuilt = oclive_kernel_host::http_api::api_router(Arc::clone(&fresh));
        let result = json_response(post(&rebuilt, "/chat/recover", &recovery).await?, 200).await;
        fresh.directory_plugins.shutdown_all();
        require(
            fresh.ollama.is_none() && fresh.performance_llm.is_none(),
            "real provider in rebuilt host",
        )?;
        require(
            fresh_rec.calls.lock().unwrap().is_empty()
                && fresh_rec.violations.lock().unwrap().is_empty(),
            "recovery generated in rebuilt host",
        )?;
        facts.insert("reassembly".into(), json!({"same_database":true,"role_runtime_loaded":false,"generation_calls":0,"process_restart":false,"directory_shutdown_called":true}));
        result?
    } else {
        assert_no_admission_violation(rec, "before the recovery route")?;
        let value = json_response(post(&app, "/chat/recover", &recovery).await?, 200).await?;
        if scenario == "i3" {
            let mut conflict = request.clone();
            conflict["message"] = json!("different payload");
            assert_no_admission_violation(rec, "before the conflict route")?;
            let rejected = json_response(post(&app, "/chat", &conflict).await?, 409).await?;
            require(
                rejected["error"]["code"] == "CHAT_REQUEST_CONFLICT",
                "payload conflict code",
            )?;
            facts.insert("conflict".into(), rejected);
            let mut unknown = recovery.clone();
            unknown["client_request_id"] = json!("24cc1153-7f3c-4f98-810f-cbc6dd94edcc");
            // Checked after the previous route returned and before dispatching the next one: an
            // admission violation (wrong method, exhausted script, budget overrun) stops dispatch.
            assert_no_admission_violation(rec, "after the conflict route")?;
            let absent = json_response(post(&app, "/chat/recover", &unknown).await?, 409).await?;
            require(
                absent["error"]["code"] == "CHAT_REQUEST_UNCONFIRMED",
                "missing receipt code",
            )?;
            facts.insert("missing_receipt".into(), absent);
            unknown.as_object_mut().unwrap().remove("client_request_id");
            assert_no_admission_violation(rec, "before the missing-identity route")?;
            let no_id = json_response(post(&app, "/chat/recover", &unknown).await?, 409).await?;
            require(
                no_id["error"]["code"] == "CHAT_REQUEST_UNCONFIRMED",
                "missing identity code",
            )?;
            facts.insert("missing_identity".into(), no_id);
            require(
                rec.calls.lock().unwrap().len() == 1,
                "rejected/recovery requests generated",
            )?;
            require(
                snapshot(root).await? == before,
                "rejected/recovery requests changed chat DB",
            )?;
            let mut intentional = request.clone();
            intentional["client_request_id"] = json!("b00e74ba-10fd-4f3d-a857-1eddf6078d54");
            // Uniform stop-loss: the intentional turn is the last route allowed to generate in this
            // scenario, so it is guarded like every other adjacent dispatch.
            assert_no_admission_violation(rec, "before the intentional turn")?;
            let second = json_response(post(&app, "/chat", &intentional).await?, 200).await?;
            // Capture the intentional-turn inputs before any judgement, so a failure keeps its evidence.
            let all = snapshot(root).await?;
            facts.insert("intentional_response".into(), second.clone());
            facts.insert("sqlite_final".into(), all.clone());
            facts.insert("receipts_final".into(), receipts(root).await?);
            facts.insert("admission".into(), rec.admission_facts());
            require(
                second["reply"] == INTENTIONAL_REPLY && second["reply_is_fallback"] == false,
                "intentional turn reply",
            )?;
            let rows = all["messages"].as_array().ok_or("messages missing")?;
            require(
                rows.len() == 4
                    && before["messages"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .all(|r| rows.contains(r)),
                "intentional turn must preserve original and add exactly two rows",
            )?;
            let mut pair = all.clone();
            pair["messages"] = json!(rows
                .iter()
                .filter(|r| !before["messages"].as_array().unwrap().contains(r))
                .collect::<Vec<_>>());
            pair["sessions"][0]["message_count"] = json!(2);
            verify_pair(&pair, &second)?;
        }
        value
    };
    // Capture both raw bodies *before* any DTO comparison so a mismatch keeps its inputs.
    facts.insert("response".into(), response.clone());
    facts.insert("recovered_response".into(), recovered.clone());
    facts.insert("sqlite_final".into(), snapshot(root).await?);
    facts.insert("receipts_final".into(), receipts(root).await?);
    facts.insert("admission".into(), rec.admission_facts());
    // Both sides are complete top-level JSON payloads:
    // - `/chat` answers with `ChatApiResponse`, whose `#[serde(flatten)] data` spreads the
    //   `SendMessageResponse` fields to the top level (so there is no nested `data` object).
    // - `/chat/recover` answers with the bare `SendMessageResponse`.
    // Comparing one side as a typed DTO and the other as a raw `Value` is not sound: `events[0].confidence`
    // is `f32`, and the raw JSON text `0.35` does not equal `to_value(f32)`'s `0.3499999940395355`.
    // Deserialize both into the same DTO and compare with the same serialization on both sides.
    require(
        dto_json(&recovered)? == dto_json(&response)?,
        "recovery changed authoritative DTO",
    )?;
    let db = snapshot(root).await?;
    if scenario != "i3" {
        require(db == before, "duplicate/recovery changed chat DB")?;
    }
    let ledger = receipts(root).await?;
    let rows = ledger.as_array().ok_or("receipt rows missing")?;
    require(
        rows.len() == if scenario == "i3" { 2 } else { 1 },
        "receipt admission count",
    )?;
    require(
        rows.iter().all(|r| {
            r["status"] == "completed"
                && r["user_message_id"].is_string()
                && r["assistant_message_id"].is_string()
        }),
        "receipt completion binding",
    )?;
    facts.insert("receipts".into(), ledger);
    facts.insert("response".into(), response);
    facts.insert("recovered_response".into(), recovered);
    facts.insert("sqlite".into(), db);
    facts.insert(
        "product_assessment".into(),
        json!("IDENTIFIED_TURN_LIMITED_ASSERTIONS_PASSED"),
    );
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
    // Bounded per-scenario generation budget. The product rejects a main reply that repeats the
    // immediately preceding assistant turn, so the two-ordinary-turn scenario (i3) needs two
    // distinct scripted replies; every other scenario has exactly one turn.
    let (script, budget, allowed) = if scenario == "i3" {
        (vec![REPLY, INTENTIONAL_REPLY], 2, StreamMode::Plain)
    } else if scenario == "i4" {
        (vec![REPLY], 1, StreamMode::Plain)
    } else {
        (vec![REPLY], 1, StreamMode::Stream)
    };
    let rec = Arc::new(StreamRecorder::new(script, budget, allowed));
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
    let expected: Vec<&str> = if scenario == "i3" {
        vec!["generate_with_opts", "generate_with_opts"]
    } else if scenario == "i4" {
        vec!["generate_with_opts"]
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
fn i_v1_identity_pairs_are_exact_and_old_ids_rejected() {
    for (scenario, id) in CASES {
        assert!(validate_mode(id, scenario, "").is_ok());
        assert!(support::validate_run_mode(id, scenario, "").is_ok());
        assert!(validate_mode(id, scenario, "live").is_err());
        assert!(validate_mode("A-CPINTB2-4b7ccc66-H04-R0", scenario, "").is_err());
        for (other, other_id) in CASES {
            if other != scenario {
                assert!(validate_mode(other_id, scenario, "").is_err());
            }
        }
    }
    // Every consumed B3 identity must be rejected, and every current identity accepted.
    for consumed in CONSUMED_RUN_IDS {
        for (scenario, _) in CASES {
            assert!(
                validate_mode(consumed, scenario, "").is_err(),
                "consumed identity {consumed} must not be admitted"
            );
            assert!(support::validate_run_mode(consumed, scenario, "").is_err());
        }
    }
    for (_scenario, id) in CASES {
        assert!(
            !CONSUMED_RUN_IDS.contains(&id),
            "current identity {id} must not also be a consumed identity"
        );
    }
    for (driver, scenario) in [
        ("i1_driver_concurrent_identity", "i1"),
        ("i2_driver_disconnect_recovery", "i2"),
        ("i3_driver_conflict_and_new_send", "i3"),
        ("i4_driver_durable_recovery", "i4"),
    ] {
        assert_eq!(support::scenario_of_driver(driver), scenario);
    }
}

/// Pure in-memory regressions for the bounded recording double (no host, DB, route or generation).
#[cfg(test)]
mod bounded_double_regression {
    use super::{StreamMode, StreamRecorder, INTENTIONAL_REPLY, REPLY};
    use crate::support;
    use oclive_kernel_contracts::{LlmClient, LlmGenerateOpts, LlmTokenSink};
    use std::sync::{Arc, Mutex};

    fn opts() -> Option<&'static LlmGenerateOpts> {
        static OPTS: std::sync::OnceLock<LlmGenerateOpts> = std::sync::OnceLock::new();
        Some(OPTS.get_or_init(LlmGenerateOpts::interactive))
    }

    fn admit_plain(rec: &StreamRecorder) -> Result<String, String> {
        rec.admit("generate_with_opts", support::MODEL_ID, "p", opts())
            .map(str::to_string)
            .map_err(|e| e.to_string())
    }

    #[test]
    fn i_v1_script_is_a_finite_queue_that_never_repeats_its_last_entry() {
        let rec = StreamRecorder::new(vec![REPLY, INTENTIONAL_REPLY], 2, StreamMode::Plain);
        assert_eq!(admit_plain(&rec).unwrap(), REPLY);
        assert_eq!(admit_plain(&rec).unwrap(), INTENTIONAL_REPLY);
        // Exhausted: the third attempt is denied and produces no reply.
        let denied = admit_plain(&rec);
        assert!(denied.is_err(), "an exhausted script must deny, not repeat");
        let facts = rec.admission_facts();
        assert_eq!(facts["generation_attempts"], 3);
        assert_eq!(facts["admitted_outputs"], 2);
        assert_eq!(facts["denied_attempts"], 1);
        assert!(
            rec.calls.lock().unwrap().len() == 2,
            "denied call is not recorded as a call"
        );
        assert!(!rec.violations.lock().unwrap().is_empty());
    }

    #[test]
    fn i_v1_zero_budget_denies_every_attempt() {
        let rec = StreamRecorder::new(vec![REPLY], 0, StreamMode::Plain);
        assert!(admit_plain(&rec).is_err());
        let facts = rec.admission_facts();
        assert_eq!(facts["generation_attempts"], 1);
        assert_eq!(facts["admitted_outputs"], 0);
        assert_eq!(facts["denied_attempts"], 1);
        assert!(rec.calls.lock().unwrap().is_empty());
    }

    #[test]
    fn i_v1_empty_script_denies_without_producing_a_reply() {
        let rec = StreamRecorder::new(Vec::new(), 1, StreamMode::Plain);
        assert!(admit_plain(&rec).is_err());
        assert_eq!(rec.admission_facts()["admitted_outputs"], 0);
    }

    #[test]
    fn i_v1_wrong_method_is_denied_and_never_returns_text() {
        let plain_only = StreamRecorder::new(vec![REPLY], 1, StreamMode::Plain);
        let denied = plain_only.admit("generate_stream_with_opts", support::MODEL_ID, "p", opts());
        assert!(
            denied.is_err(),
            "stream call must be denied in a plain-only scenario"
        );
        assert_eq!(plain_only.admission_facts()["admitted_outputs"], 0);

        let stream_only = StreamRecorder::new(vec![REPLY], 1, StreamMode::Stream);
        assert!(
            admit_plain(&stream_only).is_err(),
            "plain call must be denied in a stream-only scenario"
        );
        assert_eq!(stream_only.admission_facts()["admitted_outputs"], 0);

        let none = StreamRecorder::new(vec![REPLY], 1, StreamMode::None);
        assert!(
            admit_plain(&none).is_err(),
            "a no-generation scenario must deny every call"
        );
    }

    #[test]
    fn i_v1_admission_accounting_is_consistent_and_records_order_and_reasons() {
        let rec = StreamRecorder::new(vec![REPLY], 1, StreamMode::Plain);
        admit_plain(&rec).unwrap();
        let _ = admit_plain(&rec);
        let _ = rec.admit("generate_stream_with_opts", support::MODEL_ID, "p", opts());
        let facts = rec.admission_facts();
        let attempts = facts["generation_attempts"].as_u64().unwrap();
        let admitted = facts["admitted_outputs"].as_u64().unwrap();
        let denied = facts["denied_attempts"].as_u64().unwrap();
        assert_eq!(attempts, admitted + denied, "attempts = admitted + denied");
        let log = facts["attempt_log"].as_array().unwrap();
        assert_eq!(log.len() as u64, attempts);
        assert_eq!(log[0]["sequence"], 1);
        assert_eq!(log[1]["sequence"], 2);
        assert_eq!(log[2]["sequence"], 3);
        let denied_log = facts["denied_log"].as_array().unwrap();
        assert!(denied_log
            .iter()
            .all(|d| d["reason"].as_str().is_some_and(|r| !r.is_empty())));
    }

    #[test]
    fn i_v1_scenario_budgets_match_the_documented_plan() {
        // Mirrors the scenario table: I01/I02 one stream, I03 two plain, I04 one plain plus a
        // reassembly recorder that may not generate at all.
        for scenario in ["i1", "i2", "i4"] {
            let (budget, mode) = if scenario == "i4" {
                (1usize, StreamMode::Plain)
            } else {
                (1usize, StreamMode::Stream)
            };
            let rec = StreamRecorder::new(vec![REPLY], budget, mode);
            assert_eq!(rec.budget, 1, "{scenario} allows exactly one generation");
            assert_eq!(admit_plain(&rec).is_ok(), scenario == "i4");
        }
        let i3 = StreamRecorder::new(vec![REPLY, INTENTIONAL_REPLY], 2, StreamMode::Plain);
        assert_eq!(i3.budget, 2, "i3 allows exactly two plain generations");
    }

    /// Collects every token a sink receives, so "no token on denial" is directly observable.
    fn token_sink() -> (LlmTokenSink, Arc<Mutex<Vec<String>>>) {
        let hits = Arc::new(Mutex::new(Vec::<String>::new()));
        let inner = Arc::clone(&hits);
        let sink: LlmTokenSink = Arc::new(move |token: &str| {
            inner.lock().unwrap().push(token.to_string());
        });
        (sink, hits)
    }

    /// Real concurrency: several tasks contend for a bounded budget on a multi-thread runtime.
    /// The admission lock must make the accounting atomic — at most `budget` outputs, no repeated
    /// sequence, and `attempts = admitted + denied` for every worker count and budget.
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn i_v1_concurrent_admission_is_atomic_and_bounded() {
        const SCRIPT: [&str; 3] = ["甲一", "乙二", "丙三"];
        for (budget, workers) in [(3usize, 8usize), (1usize, 6usize), (0usize, 5usize)] {
            let rec = Arc::new(StreamRecorder::new(
                SCRIPT.to_vec(),
                budget,
                StreamMode::Plain,
            ));
            let barrier = Arc::new(tokio::sync::Barrier::new(workers));
            let mut handles = Vec::with_capacity(workers);
            for _ in 0..workers {
                let rec = Arc::clone(&rec);
                let barrier = Arc::clone(&barrier);
                handles.push(tokio::spawn(async move {
                    // Line every worker up first so the calls really overlap under contention.
                    barrier.wait().await;
                    rec.admit("generate_with_opts", support::MODEL_ID, "p", opts())
                        .is_ok()
                }));
            }
            let mut admitted = 0usize;
            let mut denied = 0usize;
            for handle in handles {
                if handle.await.unwrap() {
                    admitted += 1;
                } else {
                    denied += 1;
                }
            }

            let expected_admitted = budget.min(SCRIPT.len());
            assert_eq!(
                admitted, expected_admitted,
                "budget {budget}: at most the budget may be admitted"
            );
            assert_eq!(denied, workers - expected_admitted);

            let facts = rec.admission_facts();
            let attempts = facts["generation_attempts"].as_u64().unwrap();
            let admitted_logged = facts["admitted_outputs"].as_u64().unwrap();
            let denied_logged = facts["denied_attempts"].as_u64().unwrap();
            assert_eq!(attempts, workers as u64);
            assert_eq!(admitted_logged, expected_admitted as u64);
            assert_eq!(denied_logged, (workers - expected_admitted) as u64);
            assert_eq!(attempts, admitted_logged + denied_logged);

            let mut sequences: Vec<u64> = facts["attempt_log"]
                .as_array()
                .unwrap()
                .iter()
                .map(|entry| entry["sequence"].as_u64().unwrap())
                .collect();
            sequences.sort_unstable();
            assert_eq!(
                sequences,
                (1..=workers as u64).collect::<Vec<_>>(),
                "budget {budget}: attempt sequences must be unique and complete"
            );

            let mut admitted_sequences: Vec<u64> = facts["admitted_log"]
                .as_array()
                .unwrap()
                .iter()
                .map(|entry| entry["sequence"].as_u64().unwrap())
                .collect();
            admitted_sequences.sort_unstable();
            admitted_sequences.dedup();
            assert_eq!(admitted_sequences.len(), expected_admitted);
            assert!(
                rec.calls.lock().unwrap().len() == expected_admitted,
                "a denied call must never be recorded as a call"
            );
        }
    }

    /// Every generation method shares one denial accounting, so a legacy call can neither produce
    /// text nor escape `attempts = admitted + denied`. `startup_probe` is not a generation and is
    /// rejected separately without consuming an admission slot.
    #[tokio::test]
    async fn i_v1_all_generation_methods_share_one_denial_accounting() {
        let rec = StreamRecorder::new(vec![REPLY], 1, StreamMode::Plain);
        let (sink, sink_hits) = token_sink();

        assert!(LlmClient::generate(&rec, support::MODEL_ID, "p")
            .await
            .is_err());
        assert!(LlmClient::generate_tag(&rec, support::MODEL_ID, "p")
            .await
            .is_err());
        assert!(
            LlmClient::generate_stream(&rec, support::MODEL_ID, "p", Arc::clone(&sink))
                .await
                .is_err()
        );

        let facts = rec.admission_facts();
        assert_eq!(facts["generation_attempts"], 3);
        assert_eq!(facts["admitted_outputs"], 0);
        assert_eq!(facts["denied_attempts"], 3);
        assert_eq!(
            facts["generation_attempts"].as_u64().unwrap(),
            facts["admitted_outputs"].as_u64().unwrap()
                + facts["denied_attempts"].as_u64().unwrap()
        );
        assert!(
            rec.calls.lock().unwrap().is_empty(),
            "a refused legacy method is not a call"
        );
        assert!(
            sink_hits.lock().unwrap().is_empty(),
            "a denied generation must not emit a token"
        );
        let denied_log = facts["denied_log"].as_array().unwrap().clone();
        for method in ["generate", "generate_tag", "generate_stream"] {
            assert!(
                denied_log
                    .iter()
                    .any(|d| d["method"].as_str() == Some(method)),
                "{method} must be recorded in the denial log"
            );
        }
        assert!(denied_log
            .iter()
            .all(|d| d["reason"].as_str().is_some_and(|r| !r.is_empty())));

        let before = rec.admission_facts()["generation_attempts"]
            .as_u64()
            .unwrap();
        assert!(LlmClient::startup_probe(&rec).await.is_err());
        assert_eq!(
            rec.admission_facts()["generation_attempts"]
                .as_u64()
                .unwrap(),
            before,
            "startup_probe is not a generation and takes no admission slot"
        );
        assert!(rec
            .violations
            .lock()
            .unwrap()
            .iter()
            .any(|v| v.contains("startup_probe")));
    }

    /// The stream method is the one that normally speaks to a sink; its denial path must be
    /// silent — no token, no reply, and the denial is still accounted.
    #[tokio::test]
    async fn i_v1_denied_stream_generation_emits_no_token_and_no_reply() {
        let rec = StreamRecorder::new(vec![REPLY], 0, StreamMode::Stream);
        let (sink, sink_hits) = token_sink();
        let outcome = LlmClient::generate_stream_with_opts(
            &rec,
            support::MODEL_ID,
            "p",
            Arc::clone(&sink),
            opts(),
        )
        .await;
        assert!(outcome.is_err(), "a zero budget must deny the stream call");
        assert!(
            sink_hits.lock().unwrap().is_empty(),
            "the sink must receive nothing when the generation is denied"
        );
        let facts = rec.admission_facts();
        assert_eq!(facts["generation_attempts"], 1);
        assert_eq!(facts["admitted_outputs"], 0);
        assert_eq!(facts["denied_attempts"], 1);
        assert!(rec.calls.lock().unwrap().is_empty());
    }
}

/// Pure in-memory regressions for the authoritative-DTO comparison (no host, DB, route or generation).
#[cfg(test)]
mod dto_compare_regression {
    use super::dto_json;
    use serde_json::{json, Value};

    /// Realistic flattened `/chat` payload: `ChatApiResponse` spreads the DTO fields to the top
    /// level, so there is **no** nested `data` object. `confidence` is written the way both the SSE
    /// `done` event and the persisted receipt carry it: the literal JSON text `0.35`.
    fn flat_http_payload() -> Value {
        json!({
            "api_version": 1,
            "schema": 16,
            "presence_mode": "co_present",
            "display_metrics": {"favor": 50.0, "relation_summary": "Acquaintance",
                "traits": [0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5]},
            "relation_state": "Acquaintance",
            "reply": "清晨的风轻轻吹过树梢。",
            "emotion": {"joy": 0.0, "sadness": 0.0, "anger": 0.0, "fear": 0.0,
                "surprise": 0.0, "disgust": 0.0, "neutral": 1.0},
            "bot_emotion": "neutral",
            "portrait_emotion": "neutral",
            "favorability_delta": 0.0,
            "favorability_current": 50.0,
            "events": [{"event_type": "Ignore", "confidence": 0.35}],
            "scene_id": "default",
            "offer_destination_picker": false,
            "offer_together_travel": false,
            "reply_is_fallback": false,
            "knowledge_chunks_in_prompt": 0,
            "timestamp": 1790267772297i64,
            "user_message_id": "f4406832-e88a-48a5-9661-fcd61bd9b76a",
            "assistant_message_id": "0411e3b1-b4d3-466b-861a-e645cd7644cb",
            "personality_source": "vector"
        })
    }

    /// Bare `/chat/recover` payload: the DTO without the transport-level extra key.
    fn bare_dto_payload() -> Value {
        let mut v = flat_http_payload();
        v.as_object_mut().unwrap().remove("personality_source");
        v
    }

    /// The original defect: comparing a raw JSON `Value` against `to_value(f32)` is not sound.
    /// `events[0].confidence` is `f32`; the literal text `0.35` is not the widened f32.
    #[test]
    fn i_v1_single_sided_value_comparison_is_unsound_for_f32() {
        let raw = bare_dto_payload();
        let widened = dto_json(&raw).unwrap();
        assert_eq!(
            widened["events"][0]["confidence"].as_f64().unwrap(),
            f64::from(0.35_f32),
            "typed round-trip must widen the f32 exactly"
        );
        assert_ne!(
            raw["events"][0]["confidence"], widened["events"][0]["confidence"],
            "raw wire text and widened f32 must differ (this is why the old comparison failed)"
        );
        assert_ne!(raw, widened, "the old one-sided comparison must not pass");
    }

    /// The fix: both complete payloads round-trip through the same DTO type, then compare equal.
    #[test]
    fn i_v1_typed_comparison_accepts_flattened_and_bare_payloads() {
        assert_eq!(
            dto_json(&flat_http_payload()).unwrap(),
            dto_json(&bare_dto_payload()).unwrap()
        );
    }

    #[test]
    fn i_v1_typed_comparison_detects_reply_and_message_id_changes() {
        let base = bare_dto_payload();
        for (key, mutated) in [
            ("reply", json!("清晨的风轻轻吹过树梢。 ")),
            (
                "user_message_id",
                json!("00000000-0000-4000-8000-000000000000"),
            ),
            (
                "assistant_message_id",
                json!("00000000-0000-4000-8000-000000000001"),
            ),
        ] {
            let mut other = flat_http_payload();
            other[key] = mutated;
            assert_ne!(
                dto_json(&base).unwrap(),
                dto_json(&other).unwrap(),
                "changing {key} must still fail the comparison"
            );
        }
    }

    /// A f32 value that is *distinguishable* after widening must still be caught, so no field of
    /// `events` is silently ignored.
    #[test]
    fn i_v1_typed_comparison_detects_distinguishable_f32_confidence() {
        let mut other = flat_http_payload();
        other["events"][0]["confidence"] = json!(0.5);
        assert_ne!(
            dto_json(&bare_dto_payload()).unwrap(),
            dto_json(&other).unwrap(),
            "a distinguishable confidence change must still fail"
        );
    }

    /// Missing required fields, or a wrong nested `{data: ...}` shape, are not valid top-level DTOs.
    #[test]
    fn i_v1_missing_field_and_nested_data_shape_are_rejected() {
        let mut missing = bare_dto_payload();
        missing.as_object_mut().unwrap().remove("reply");
        assert!(
            dto_json(&missing).is_err(),
            "a payload missing a required DTO field must not compare as valid"
        );

        let nested = json!({"data": bare_dto_payload()});
        assert!(
            dto_json(&nested).is_err(),
            "a nested {{data: ...}} wrapper is not a valid top-level DTO payload"
        );
    }
}
