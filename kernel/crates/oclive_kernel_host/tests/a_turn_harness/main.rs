//! A-P2-R1 隔离回合 harness · 入口（R1–R3 返修版）。
//!
//! 所有 I/O 入口全部 `#[ignore]` 显式 opt-in；默认仅作纯内存验证。
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::field_reassign_with_default
)]

mod emotion_turn;
mod http_entry;
mod http_idempotency;
mod live;
mod live_proxy;
mod memory_turn;
mod semantic_cases;
mod support;

use serde_json::{json, Map, Value};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use support::artifacts::{self, FsReader};
use support::driver::{self, ChildControl, ChildPoll};
use support::recording_llm::{RecordingLlm, Step};
use support::{
    create_new_dir_verified, env, fixture, verify_existing_chain, FakeFs, MetaFacts, OpsLedger,
    RealFs, EMPTY_REPLY_REJECTION, MODEL_ID, REPAIR_PROMPT_MARKER, ROLE_ID, ROLE_NAME, S1, S2, S3,
    S3_REASON_CODE, S3_REASON_MESSAGE, SCENE_ID, SHUTDOWN_TIMEOUT_SECS,
};

// —— 父 driver（每场景一个被拥有的测试 child）——

#[test]
#[ignore = "A-P2 opt-in：必须由主控批准的 run_id 显式驱动"]
fn a1_driver_normal_turn() {
    driver::run_driver("a1_driver_normal_turn");
}

#[test]
#[ignore = "A-P2 opt-in：必须由主控批准的 run_id 显式驱动"]
fn a2_driver_empty_reply_repair() {
    driver::run_driver("a2_driver_empty_reply_repair");
}

#[test]
#[ignore = "A-P2 opt-in：必须由主控批准的 run_id 显式驱动"]
fn a3_driver_main_llm_err_fallback() {
    driver::run_driver("a3_driver_main_llm_err_fallback");
}

#[test]
#[ignore = "B opt-in: frozen local model, approval marker and a fresh B run_id required"]
fn b1_driver_live_turn() {
    driver::run_driver("b1_driver_live_turn");
}

#[test]
#[ignore = "M-V1 opt-in: frozen synthetic-memory run_id; no real model"]
fn m1_driver_selected_memory_turn() {
    driver::run_driver("m1_driver_selected_memory_turn");
}

#[test]
#[ignore = "M-V1 opt-in: frozen no-hit Host fallback run_id; no real model"]
fn m2_driver_no_hit_memory_turn() {
    driver::run_driver("m2_driver_no_hit_memory_turn");
}

#[test]
#[ignore = "E-V1 opt-in: frozen builtin-emotion turn; no real model"]
fn e1_driver_joy_emotion_turn() {
    driver::run_driver("e1_driver_joy_emotion_turn");
}

#[test]
#[ignore = "E-V1 opt-in: frozen no-clue turn; no real model"]
fn e2_driver_no_clue_emotion_turn() {
    driver::run_driver("e2_driver_no_clue_emotion_turn");
}

#[test]
#[ignore = "E-V1 opt-in: frozen negated-clue turn; no real model"]
fn e3_driver_negated_emotion_turn() {
    driver::run_driver("e3_driver_negated_emotion_turn");
}

#[test]
#[ignore = "E-V1 opt-in: frozen neutral-clue turn; no real model"]
fn e4_driver_neutral_clue_emotion_turn() {
    driver::run_driver("e4_driver_neutral_clue_emotion_turn");
}

#[test]
#[ignore = "CP-INT B2: isolated in-process HTTP/SSE; frozen run ID required"]
fn h1_driver_http_stream_success() {
    driver::run_driver("h1_driver_http_stream_success");
}

#[test]
#[ignore = "CP-INT B2: isolated pre-token provider failure"]
fn h2_driver_http_stream_error() {
    driver::run_driver("h2_driver_http_stream_error");
}

#[test]
#[ignore = "CP-INT B2: isolated partial-token provider failure"]
fn h3_driver_http_partial_error() {
    driver::run_driver("h3_driver_http_partial_error");
}

#[test]
#[ignore = "CP-INT B2: characterize dropped SSE body and explicit retry, not product acceptance"]
fn h4_driver_http_disconnect_retry() {
    driver::run_driver("h4_driver_http_disconnect_retry");
}

#[test]
#[ignore = "CP-INT B3: isolated same-turn identity; frozen run ID required"]
fn i1_driver_concurrent_identity() {
    driver::run_driver("i1_driver_concurrent_identity");
}

#[test]
#[ignore = "CP-INT B3: isolated same-turn identity; frozen run ID required"]
fn i2_driver_disconnect_recovery() {
    driver::run_driver("i2_driver_disconnect_recovery");
}

#[test]
#[ignore = "CP-INT B3: isolated same-turn identity; frozen run ID required"]
fn i3_driver_conflict_and_new_send() {
    driver::run_driver("i3_driver_conflict_and_new_send");
}

#[test]
#[ignore = "CP-INT B3: isolated same-turn identity; frozen run ID required"]
fn i4_driver_durable_recovery() {
    driver::run_driver("i4_driver_durable_recovery");
}

// —— 子（唯一入口；一次只跑一个场景）——

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "A-P2 opt-in：只能由父 driver 以 OCLIVE_A_CHILD=1 启动"]
async fn a_child_run_turn() {
    let (scenario, run_id, run_token) = driver::require_child_marker();
    let root = support::scenario_root(&run_id, &scenario);
    // 写日志/写角色/连 DB 之前复验（R1.2）。
    let root_canon = match driver::verify_existing_root(&root) {
        Ok(c) => c,
        Err(e) => panic!("[A-HARNESS] child 根复验失败（不进入日志/写盘）：{e}"),
    };
    for sub in ["logs", "roles", "app-data", "artifacts", "reports", "chats"] {
        if let Err(e) =
            verify_existing_chain(&RealFs, &root_canon.join(sub), &format!("child:{sub}"))
        {
            panic!("[A-HARNESS] 子入口目录复验失败（不进入日志/写盘）：{e}");
        }
    }
    let _log_guard = oclive_kernel_host::init_tracing_with_log_dir(Some(&root_canon));

    let env_before = env::env_facts();
    let (mut facts, failures) = if http_idempotency::is_scenario(&scenario) {
        http_idempotency::run_scenario(&scenario, &root_canon).await
    } else if http_entry::is_scenario(&scenario) {
        http_entry::run_scenario(&scenario, &root_canon).await
    } else if scenario == support::B1 {
        let case = semantic_cases::select(
            &run_id,
            &std::env::var(support::ENV_B_APPROVAL).unwrap_or_default(),
        )
        .expect("child marker already validated the frozen sample");
        live::run_scenario(&root_canon, case).await
    } else {
        run_scenario(&scenario, &root_canon).await
    };

    facts.insert("env_before".into(), env_before);
    facts.insert("env_after".into(), env::env_facts());
    facts.insert("env_safe_values".into(), env::safe_set_values());
    facts.insert("run_id".into(), json!(run_id));
    facts.insert("run_token".into(), json!(run_token));
    facts.insert(
        "scenario_root_canonical".into(),
        json!(root_canon.display().to_string()),
    );
    facts.insert("process_exit_called".into(), json!(false));
    let report = json!({
        "role": "child",
        "ok": failures.is_empty(),
        "scenario": scenario,
        "run_id": run_id,
        "run_token": run_token,
        "scenario_root_canonical": root_canon.display().to_string(),
        "failures": failures,
        "facts": Value::Object(facts),
        "generated_at": support::now_utc_stamp(),
    });
    if let Err(e) = artifacts::write_json(&root_canon.join("reports/child-report.json"), &report) {
        panic!("[A-HARNESS] 子报告写入失败：{e}");
    }
    let _ = artifacts::write_json(
        &root_canon.join("reports/child-failures.json"),
        &json!({ "failures": report.get("failures").cloned().unwrap_or(Value::Null) }),
    );
    if let Some(first) = report
        .get("failures")
        .and_then(Value::as_array)
        .and_then(|a| a.first())
        .and_then(Value::as_str)
    {
        panic!(
            "[A-HARNESS] 场景 {scenario} 判定失败（共 {} 条）：{first}",
            report
                .get("failures")
                .and_then(Value::as_array)
                .map_or(0, Vec::len)
        );
    }
}

fn script_for(scenario: &str) -> Vec<Step> {
    match scenario {
        "e1" | "e2" | "e3" | "e4" => vec![Step::Ok(emotion_turn::REPLY.to_string())],
        memory_turn::M1 | memory_turn::M2 => vec![Step::Ok(memory_turn::REPLY.to_string())],
        S1 => vec![Step::Ok(support::S1_REPLY.to_string())],
        S2 => vec![
            Step::Ok(support::S2_EMPTY_REPLY.to_string()),
            Step::Ok(support::S2_REPLY.to_string()),
        ],
        S3 => vec![Step::Err(support::S3_ERR_MARKER.to_string())],
        other => panic!("[A-HARNESS] unknown scenario {other}"),
    }
}

const CONTRACT_MARKERS: &[&str] = &[
    "【候选必须包含：",
    "【候选不得包含：",
    "【候选必须以：",
    "【候选不得等于：",
    "【短语次数：",
    "【安全回退：",
];

/// 收尾接缝（R3）：正常返回 / 超时两类，且**只执行一次**。
pub struct FinalizeOutcome {
    pub outcome: &'static str,
    pub attempts: u32,
}

pub async fn run_finalize<F, Fut>(
    timeout: Duration,
    attempts: &AtomicUsize,
    f: F,
) -> FinalizeOutcome
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = ()>,
{
    attempts.fetch_add(1, Ordering::SeqCst);
    let r = tokio::time::timeout(timeout, f()).await;
    FinalizeOutcome {
        outcome: if r.is_ok() { "returned" } else { "timeout" },
        attempts: attempts.load(Ordering::SeqCst) as u32,
    }
}

async fn run_scenario(scenario: &str, root: &Path) -> (Map<String, Value>, Vec<String>) {
    let mut facts: Map<String, Value> = Map::new();
    let mut failures: Vec<String> = Vec::new();
    let ledger = OpsLedger::default();
    let user_msg = support::user_message(scenario);
    facts.insert("scenario".into(), json!(scenario));
    facts.insert("user_message_constant".into(), json!(user_msg));
    facts.insert("model_constant".into(), json!(MODEL_ID));

    // 1) 夹具目录：先验后建，再写文件并断言（失败 ⇒ 报告，不进入 build）。
    let roles_dir = root.join("roles");
    if let Err(e) = create_new_dir_verified(
        &RealFs,
        &roles_dir,
        &fixture::role_dir(root),
        "role_pack_dir",
        &ledger,
    ) {
        failures.push(format!("夹具目录创建/核验失败：{e}"));
        facts.insert("ops_ledger".into(), json!({ "creates": ledger.creates() }));
        return (facts, failures);
    }
    if let Err(e) = fixture::write_role_pack(root) {
        failures.push(format!("夹具写入失败：{e}"));
        return (facts, failures);
    }
    match fixture::assert_role_pack_loads(root) {
        Ok(v) => {
            facts.insert("fixture_role".into(), v);
        }
        Err(e) => {
            failures.push(format!("夹具闸门失败（不进入 build）：{e}"));
            return (facts, failures);
        }
    }

    // 2) 配置与 DB 路径：build 之前核验（含目录先验）。
    let app_data = root.join("app-data");
    let cfg = oclive_kernel_host::OcliveKernelConfig::new(&app_data, &roles_dir);
    let db_path = cfg.database_path().to_path_buf();
    facts.insert("database_path".into(), json!(db_path.display().to_string()));
    if db_path != app_data.join("app.db") {
        failures.push(format!(
            "database_path 与 find_db_path 契约不符：{}",
            db_path.display()
        ));
    }
    if !db_path.starts_with(root) {
        failures.push(format!(
            "database_path 越出本次运行树：{}",
            db_path.display()
        ));
        return (facts, failures);
    }
    if let Err(e) = verify_existing_chain(&RealFs, &app_data, "db_parent") {
        failures.push(format!("DB 父目录先验失败（不 build）：{e}"));
        return (facts, failures);
    }

    // 3) 注入替身 + 显式 HostProfile。
    let rec = Arc::new(RecordingLlm::new(
        scenario,
        script_for(scenario),
        &root.join("artifacts"),
    ));
    let mut profile = oclive_kernel_host::domain::host_profile::HostProfile::default();
    profile.event_impact_llm = false;
    profile.skip_agent = true;
    profile.theater.director_plugin = None;
    facts.insert(
        "host_profile".into(),
        json!({
            "event_impact_llm": profile.event_impact_llm,
            "skip_agent": profile.skip_agent,
            "theater_director_plugin": profile.theater.director_plugin,
            "backends_ceiling_none": profile.backends_ceiling.is_none(),
        }),
    );

    let build_started = Instant::now();
    let kernel = match oclive_kernel_host::OcliveKernelBuilder::new(cfg)
        .with_host_profile(profile)
        .with_llm_client(Arc::clone(&rec) as Arc<dyn support::HostLlmClient>)
        .build()
        .await
    {
        Ok(k) => k,
        Err(e) => {
            failures.push(format!("OcliveKernelBuilder::build 失败: {e}"));
            facts.insert("shutdown".into(), json!({ "outcome": "not_built" }));
            return (facts, failures);
        }
    };
    facts.insert(
        "build_ms".into(),
        json!(build_started.elapsed().as_millis() as u64),
    );
    facts.insert("env_after_build".into(), env::env_facts());

    // 4) 夹具可装载性闸门；失败也必须走一次收尾（R3）。
    let mut load_ok = true;
    match kernel.load_role(ROLE_ID).await {
        Ok(_) => {
            facts.insert("load_role".into(), json!({ "ok": true }));
        }
        Err(e) => {
            load_ok = false;
            failures.push(format!("kernel.load_role 失败: {e}"));
            facts.insert(
                "load_role".into(),
                json!({ "ok": false, "error": e.to_string() }),
            );
        }
    }

    let mut response_facts: Option<Value> = None;
    // Test setup only: real builder/migrations/load_role have already completed.
    // Setup failure follows the existing single-shutdown path, without entering the turn.
    let mut memory_before = None;
    if load_ok && memory_turn::is_memory(scenario) {
        match memory_turn::seed(&db_path, root).await {
            Ok(snapshot) => {
                facts.insert("memory_before".into(), snapshot.clone());
                memory_before = Some(snapshot);
            }
            Err(error) => {
                load_ok = false;
                failures.push(format!("synthetic memory setup failed: {error}"));
            }
        }
    }
    if load_ok {
        let request = support::SendMessageRequest {
            client_request_id: None,
            role_id: ROLE_ID.to_string(),
            user_message: user_msg.to_string(),
            scene_id: Some(SCENE_ID.to_string()),
            session_id: None,
            include_raw_reply: Some(true),
            adult: None,
        };
        let turn_started = Instant::now();
        match kernel.process_message(&request).await {
            Ok(r) => {
                let v = response_facts_of(&r);
                facts.insert(
                    "turn_ms".into(),
                    json!(turn_started.elapsed().as_millis() as u64),
                );
                facts.insert("response".into(), v.clone());
                response_facts = Some(v);
            }
            Err(e) => {
                failures.push(format!("process_message 返回 Err: {e}"));
                facts.insert(
                    "turn_ms".into(),
                    json!(turn_started.elapsed().as_millis() as u64),
                );
            }
        }
    } else {
        // 未进入回合：仍需收尾（load_role 失败不得跳过 shutdown）。
        let attempts = AtomicUsize::new(0);
        let fin = run_finalize(
            Duration::from_secs(SHUTDOWN_TIMEOUT_SECS),
            &attempts,
            || kernel.shutdown(),
        )
        .await;
        facts.insert(
            "shutdown".into(),
            json!({ "outcome": fin.outcome, "attempts": fin.attempts, "note": "load_role 失败路径" }),
        );
        if fin.outcome != "returned" {
            failures.push(format!("shutdown 未正常返回（{}）", fin.outcome));
        }
        facts.insert("sqlite_verify".into(), json!({ "state": "not_executed" }));
        return (facts, failures);
    }

    // 5) 一次受限收尾（返回 `()`；区分正常返回与超时）。
    let attempts = AtomicUsize::new(0);
    let fin = run_finalize(
        Duration::from_secs(SHUTDOWN_TIMEOUT_SECS),
        &attempts,
        || kernel.shutdown(),
    )
    .await;
    facts.insert(
        "shutdown".into(),
        json!({
            "outcome": fin.outcome,
            "attempts": fin.attempts,
            "signature_note": "OcliveKernel::shutdown(self) -> ()（无 Err 可记录）",
        }),
    );

    // 6) 只读 SQLite 核验：仅在 shutdown 正常返回后执行（R3）。
    if fin.outcome == "returned" {
        let (sqlite_facts, sqlite_failures) =
            verify_sqlite(&db_path, scenario, user_msg, response_facts.as_ref()).await;
        facts.insert("sqlite_verify".into(), sqlite_facts);
        failures.extend(sqlite_failures);
        if let Some(before) = memory_before.as_ref() {
            match memory_turn::snapshot(&db_path, root).await {
                Ok(after) => {
                    failures.extend(memory_turn::verify_rows(scenario, before, &after));
                    facts.insert("memory_after".into(), after);
                }
                Err(error) => failures.push(format!("memory post-read failed: {error}")),
            }
        }
    } else {
        failures.push(format!(
            "shutdown 超时（{}）⇒ 不按“池已关闭”查 DB，该项标未执行",
            fin.outcome
        ));
        facts.insert(
            "sqlite_verify".into(),
            json!({ "state": "not_executed_after_shutdown_timeout" }),
        );
    }

    // 7) 范围内文件事实（枚举失败不得当作空目录）。
    match artifacts::scan_files(&root.join("chats")) {
        Ok(scan) => {
            facts.insert(
                "chats_scan_state".into(),
                json!(if scan.state == artifacts::ScanState::Absent {
                    "absent"
                } else {
                    "present"
                }),
            );
            facts.insert("chats_files".into(), json!(scan.relative_paths()));
            if scan.state == artifacts::ScanState::Absent {
                failures.push("chats 目录本片应预建，实际不存在（口径必须是“存在且为空”）".into());
            } else if scan.file_count() != 0 {
                failures.push(format!(
                    "mirror 已关闭（OCLIVE_CHAT_STORAGE_BACKEND=sqlite）但 chats 出现文件：{:?}",
                    scan.relative_paths()
                ));
            }
        }
        Err(e) => {
            failures.push(format!("chats 枚举失败：{e}"));
            facts.insert("chats_scan_state".into(), json!("failure"));
        }
    }

    // 8) 方法级账 + 严格校验（R2）。
    let calls = rec.calls();
    facts.insert("ledger".into(), rec.method_counts());
    facts.insert("calls".into(), json!(calls));
    facts.insert("capability_queries".into(), json!(rec.capability_queries()));
    failures.extend(check_calls(
        scenario,
        &calls,
        &rec.violations(),
        rec.capability_queries(),
    ));

    let prompts = match artifacts::read_jsonl(&root.join("artifacts/prompts.jsonl")) {
        Ok(v) => v,
        Err(e) => {
            failures.push(format!("prompt 记录不可用：{e}"));
            Vec::new()
        }
    };
    facts.insert(
        "prompt_summary".into(),
        json!(prompts
            .iter()
            .map(|p| json!({
                "seq": p.get("seq"),
                "method": p.get("method"),
                "bytes": p.get("prompt").and_then(Value::as_str).map(str::len),
                "sha256": p.get("prompt").and_then(Value::as_str).map(|s| artifacts::sha256_hex(s.as_bytes())),
            }))
            .collect::<Vec<_>>()),
    );
    failures.extend(verify_prompts(scenario, &prompts));
    if memory_turn::is_memory(scenario) {
        failures.extend(memory_turn::verify_prompts(scenario, &prompts));
    }

    if let Some(rf) = response_facts.as_ref() {
        failures.extend(verify_response_facts(scenario, rf));
        if emotion_turn::find(scenario).is_some() {
            failures.extend(emotion_turn::verify(
                scenario,
                &prompts,
                rf,
                facts.get("sqlite_verify").unwrap_or(&Value::Null),
            ));
        }
    }
    (facts, failures)
}

fn response_facts_of(r: &support::SendMessageResponse) -> Value {
    json!({
        "api_version": r.api_version,
        "schema": r.schema,
        "presence_mode": format!("{:?}", r.presence_mode),
        "reply": r.reply,
        "reply_bytes": r.reply.len(),
        "emotion": r.emotion,
        "bot_emotion": r.bot_emotion,
        "portrait_emotion": r.portrait_emotion,
        "favorability_delta": r.favorability_delta,
        "scene_id": r.scene_id,
        "events_len": r.events.len(),
        "knowledge_chunks_in_prompt": r.knowledge_chunks_in_prompt,
        "reply_is_fallback": r.reply_is_fallback,
        "llm_fallback_reason": r.llm_fallback_reason,
        "user_message_id": r.user_message_id,
        "assistant_message_id": r.assistant_message_id,
        "user_message_timestamp": r.user_message_timestamp,
        "assistant_message_timestamp": r.assistant_message_timestamp,
        "chat_persist_failed": r.chat_persist_failed,
        "reply_presentation_present": r.reply_presentation.is_some(),
        "timestamp": r.timestamp,
    })
}

// —— R2：严格验证器（真实场景与内存反例共用同一函数）——

/// 响应判据：S1/S2 完整回复等于独立常量；S3 解析 reason JSON 断言 code/message 字面量。
#[must_use]
pub fn verify_response_facts(scenario: &str, rf: &Value) -> Vec<String> {
    let mut f: Vec<String> = Vec::new();
    let reply = rf.get("reply").and_then(Value::as_str).unwrap_or_default();
    let fallback = rf.get("reply_is_fallback").and_then(Value::as_bool);
    let reason = rf
        .get("llm_fallback_reason")
        .cloned()
        .unwrap_or(Value::Null);
    match scenario {
        S1 | S2 | memory_turn::M1 | memory_turn::M2 | "e1" | "e2" | "e3" | "e4" => {
            if let Some(want) = support::expected_reply(scenario) {
                if reply != want {
                    f.push(format!("回复必须完整等于独立常量 {want:?}，实际 {reply:?}"));
                }
            }
            if fallback != Some(false) {
                f.push(format!("期望 reply_is_fallback=false，实际 {fallback:?}"));
            }
            if !reason.is_null() {
                f.push(format!("期望 llm_fallback_reason=None，实际 {reason}"));
            }
        }
        S3 => {
            if fallback != Some(true) {
                f.push(format!("S3 期望 reply_is_fallback=true，实际 {fallback:?}"));
            }
            if reply.trim().is_empty() {
                f.push("S3 兜底回复不得为空".into());
            }
            match reason
                .as_str()
                .and_then(|s| serde_json::from_str::<Value>(s).ok())
            {
                Some(j) => {
                    if j.get("code").and_then(Value::as_str) != Some(S3_REASON_CODE) {
                        f.push(format!("reason.code 期望 {S3_REASON_CODE}，实际 {j}"));
                    }
                    if j.get("message").and_then(Value::as_str) != Some(S3_REASON_MESSAGE) {
                        f.push(format!(
                            "reason.message 期望 {S3_REASON_MESSAGE:?}，实际 {j}"
                        ));
                    }
                }
                None => f.push(format!("reason 不是可解析 JSON：{reason}")),
            }
        }
        other => f.push(format!("未知场景 {other}")),
    }
    if rf.get("scene_id").and_then(Value::as_str) != Some(SCENE_ID) {
        f.push(format!(
            "scene_id 期望 {SCENE_ID}，实际 {}",
            rf.get("scene_id").cloned().unwrap_or(Value::Null)
        ));
    }
    f
}

/// prompt 判据：条数、用户消息、角色名、契约标记、S2 修复段（独立字面量）。
#[must_use]
pub fn verify_prompts(scenario: &str, prompts: &[Value]) -> Vec<String> {
    let mut f: Vec<String> = Vec::new();
    let user_msg = support::user_message(scenario);
    let expected = support::scripted_main_calls(scenario);
    if prompts.len() != expected {
        f.push(format!(
            "prompt 条数 {} 与期望 {expected} 不符",
            prompts.len()
        ));
    }
    for p in prompts {
        let text = p.get("prompt").and_then(Value::as_str).unwrap_or_default();
        if !text.contains(user_msg) {
            f.push("prompt 未包含本次合成用户消息常量".into());
        }
        if !text.contains(ROLE_NAME) {
            f.push("prompt 未包含合成角色名".into());
        }
        for marker in CONTRACT_MARKERS {
            if text.contains(marker) {
                f.push(format!("prompt 出现回复约束标记 {marker}"));
            }
        }
    }
    let texts: Vec<&str> = prompts
        .iter()
        .map(|p| p.get("prompt").and_then(Value::as_str).unwrap_or_default())
        .collect();
    match scenario {
        S2 => {
            if texts.len() == 2 {
                if texts[0].contains(REPAIR_PROMPT_MARKER) {
                    f.push("首次主生成 prompt 不应包含修复段标记".into());
                }
                if !texts[1].contains(REPAIR_PROMPT_MARKER) {
                    f.push(format!("修复 prompt 必须包含 {REPAIR_PROMPT_MARKER}"));
                }
                if !texts[1].contains(EMPTY_REPLY_REJECTION) {
                    f.push(format!(
                        "修复 prompt 必须包含拒收原因 {EMPTY_REPLY_REJECTION}"
                    ));
                }
                if !texts[1].contains(user_msg) {
                    f.push("修复 prompt 必须保留原用户消息".into());
                }
                if texts[0] == texts[1] {
                    f.push("修复 prompt 与首次主生成相同".into());
                }
            }
        }
        S1 => {
            // 单条主生成 prompt 不得出现修复段标记（保持与 S2 相反的判据方向）。
            for text in &texts {
                if text.contains(REPAIR_PROMPT_MARKER) {
                    f.push("S1 不应出现修复段标记".into());
                }
            }
        }
        _ => {}
    }
    f
}

/// 调用账与 opts 判据（键必须存在且为 null，不得以缺字段代替 null）。
#[must_use]
pub fn check_calls(
    scenario: &str,
    calls: &[Value],
    violations: &[String],
    capability_queries: usize,
) -> Vec<String> {
    let mut f: Vec<String> = Vec::new();
    let expected = support::scripted_main_calls(scenario);
    let main_calls: Vec<&Value> = calls
        .iter()
        .filter(|c| c.get("method").and_then(Value::as_str) == Some("generate_with_opts"))
        .collect();
    if main_calls.len() != expected {
        f.push(format!(
            "主生成计数期望 {expected}，实际 {}",
            main_calls.len()
        ));
    }
    let unexpected: Vec<&str> = calls
        .iter()
        .filter_map(|c| c.get("method").and_then(Value::as_str))
        .filter(|m| *m != "generate_with_opts")
        .collect();
    if !unexpected.is_empty() {
        f.push(format!("出现未预期生成类方法：{unexpected:?}"));
    }
    if !violations.is_empty() {
        f.push(format!("替身记录违规：{violations:?}"));
    }
    if capability_queries == 0 {
        f.push("未观测到 supports_prefix_cache 能力查询".into());
    }
    for c in main_calls {
        if c.get("model").and_then(Value::as_str) != Some(MODEL_ID) {
            f.push(format!(
                "主生成 model 期望 {MODEL_ID}，实际 {:?}",
                c.get("model")
            ));
        }
        let opts = c.get("opts").cloned().unwrap_or(Value::Null);
        if opts.get("some").and_then(Value::as_bool) != Some(true) {
            f.push(format!("opts 必须为 Some，实际 {opts}"));
            continue;
        }
        if opts.get("want_metrics").and_then(Value::as_bool) != Some(true) {
            f.push(format!("opts.want_metrics 期望 true，实际 {opts}"));
        }
        if opts.get("keep_alive").and_then(Value::as_str) != Some("30m") {
            f.push(format!("opts.keep_alive 期望 \"30m\"，实际 {opts}"));
        }
        for field in [
            "temperature",
            "top_p",
            "max_output_tokens",
            "preferred_context_tokens",
        ] {
            match opts.get(field) {
                Some(Value::Null) => {}
                Some(other) => f.push(format!("opts.{field} 期望 null，实际 {other}")),
                None => f.push(format!(
                    "opts.{field} 键缺失（不得以缺字段代替显式 null）：{opts}"
                )),
            }
        }
    }
    f
}

// —— R2：SQLite 行绑定判据（sender+id+session+content 四者对应）——

#[must_use]
pub fn verify_sqlite_rows(
    rows: &[Value],
    sessions: &[Value],
    response: Option<&Value>,
    user_msg: &str,
) -> Vec<String> {
    let mut f: Vec<String> = Vec::new();
    let reply = response
        .and_then(|r| r.get("reply"))
        .and_then(Value::as_str)
        .unwrap_or_default();
    if rows.len() != 2 {
        f.push(format!("chat_messages 期望 2 行，实际 {}", rows.len()));
    }
    let users: Vec<&Value> = rows
        .iter()
        .filter(|r| r.get("sender").and_then(Value::as_str) == Some("user"))
        .collect();
    let assistants: Vec<&Value> = rows
        .iter()
        .filter(|r| r.get("sender").and_then(Value::as_str) == Some("assistant"))
        .collect();
    if users.len() != 1 {
        f.push(format!("user 行期望恰好 1 行，实际 {}", users.len()));
    }
    if assistants.len() != 1 {
        f.push(format!(
            "assistant 行期望恰好 1 行，实际 {}",
            assistants.len()
        ));
    }
    if let (Some(u), Some(a)) = (users.first(), assistants.first()) {
        let uid = u.get("id").and_then(Value::as_str).unwrap_or_default();
        let aid = a.get("id").and_then(Value::as_str).unwrap_or_default();
        if uid.is_empty() || aid.is_empty() || uid == aid {
            f.push(format!(
                "user/assistant 行 ID 必须非空且不同：{uid} / {aid}"
            ));
        }
        if u.get("content").and_then(Value::as_str) != Some(user_msg) {
            f.push(format!(
                "user 行内容 ≠ 输入常量：{:?}",
                u.get("content").cloned().unwrap_or(Value::Null)
            ));
        }
        if a.get("content").and_then(Value::as_str) != Some(reply) {
            f.push(format!(
                "assistant 行内容 ≠ 响应 reply：{:?}",
                a.get("content").cloned().unwrap_or(Value::Null)
            ));
        }
        let rus = response
            .and_then(|r| r.get("user_message_id"))
            .and_then(Value::as_str);
        let ras = response
            .and_then(|r| r.get("assistant_message_id"))
            .and_then(Value::as_str);
        if rus != Some(uid) {
            f.push(format!("响应 user_message_id {rus:?} ≠ user 行 ID {uid}"));
        }
        if ras != Some(aid) {
            f.push(format!(
                "响应 assistant_message_id {ras:?} ≠ assistant 行 ID {aid}"
            ));
        }
        let usid = u
            .get("session_id")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let asid = a
            .get("session_id")
            .and_then(Value::as_str)
            .unwrap_or_default();
        if usid != ROLE_ID || asid != ROLE_ID {
            f.push(format!(
                "chat_messages.session_id 期望 {ROLE_ID}，实际 {usid} / {asid}"
            ));
        }
        if sessions.len() != 1 {
            f.push(format!("chat_sessions 期望 1 行，实际 {}", sessions.len()));
        } else {
            let s = &sessions[0];
            if s.get("session_id").and_then(Value::as_str) != Some(usid) {
                f.push(format!("chat_sessions.session_id 与消息会话不一致：{s}"));
            }
            if s.get("role_id").and_then(Value::as_str) != Some(ROLE_ID) {
                f.push(format!("chat_sessions.role_id 期望 {ROLE_ID}，实际 {s}"));
            }
            if s.get("scene_id").and_then(Value::as_str) != Some(SCENE_ID) {
                f.push(format!("chat_sessions.scene_id 期望 {SCENE_ID}，实际 {s}"));
            }
            if s.get("message_count").and_then(Value::as_i64) != Some(2) {
                f.push(format!("chat_sessions.message_count 期望 2，实际 {s}"));
            }
        }
    }
    f
}

async fn verify_sqlite(
    db_path: &Path,
    scenario: &str,
    user_msg: &str,
    response: Option<&Value>,
) -> (Value, Vec<String>) {
    use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
    use sqlx::Row;

    if !db_path.is_file() {
        return (
            json!({ "state": "db_file_missing", "db_path": db_path.display().to_string() }),
            vec![format!("DB 文件不存在：{}", db_path.display())],
        );
    }
    let opts = SqliteConnectOptions::new()
        .filename(db_path)
        .read_only(true);
    let pool = match SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(opts)
        .await
    {
        Ok(p) => p,
        Err(e) => {
            return (
                json!({ "state": "connect_failed", "db_path": db_path.display().to_string() }),
                vec![format!("只读连接失败 {}: {e}", db_path.display())],
            )
        }
    };
    let msgs = sqlx::query(
        "SELECT id, session_id, turn_index, sender, content, created_at, metadata FROM chat_messages ORDER BY turn_index, sender",
    )
    .fetch_all(&pool)
    .await;
    let sessions = sqlx::query(
        "SELECT session_id, role_id, scene_id, message_count FROM chat_sessions ORDER BY session_id",
    )
    .fetch_all(&pool)
    .await;
    pool.close().await;

    let rows: Vec<Value> = match &msgs {
        Ok(rs) => rs
            .iter()
            .map(|r| {
                json!({
                    "id": r.get::<String, _>("id"),
                    "session_id": r.get::<String, _>("session_id"),
                    "turn_index": r.get::<i64, _>("turn_index"),
                    "sender": r.get::<String, _>("sender"),
                    "content": r.get::<String, _>("content"),
                    "created_at": r.get::<Option<String>, _>("created_at"),
                    "metadata": r.get::<Option<String>, _>("metadata"),
                })
            })
            .collect(),
        Err(_) => Vec::new(),
    };
    let session_rows: Vec<Value> = match &sessions {
        Ok(rs) => rs
            .iter()
            .map(|r| {
                json!({
                    "session_id": r.get::<String, _>("session_id"),
                    "role_id": r.get::<String, _>("role_id"),
                    "scene_id": r.get::<Option<String>, _>("scene_id"),
                    "message_count": r.get::<Option<i64>, _>("message_count"),
                })
            })
            .collect(),
        Err(_) => Vec::new(),
    };

    let mut failures = verify_sqlite_rows(&rows, &session_rows, response, user_msg);
    if let Err(e) = &msgs {
        failures.push(format!("chat_messages 查询失败：{e}"));
    }
    if let Err(e) = &sessions {
        failures.push(format!("chat_sessions 查询失败：{e}"));
    }
    let query_error = msgs.is_err() || sessions.is_err();
    let facts = json!({
        "db_path": db_path.display().to_string(),
        "state": if query_error { "query_failed" } else if failures.is_empty() { "ok" } else { "assertion_failed" },
        "db_bytes": std::fs::metadata(db_path).map(|m| m.len()).ok(),
        "wal_present": PathBuf::from(format!("{}-wal", db_path.display())).exists(),
        "shm_present": PathBuf::from(format!("{}-shm", db_path.display())).exists(),
        "chat_messages": rows,
        "chat_sessions": session_rows,
        "scenario": scenario,
        "assertion_failures": failures,
    });
    // 失败时也把已取得的行作为 facts 交回（不再只返回拼接错误字符串）。
    (facts, failures)
}

// ══════════════════════════════════════════════════════════════
// R1–R3 纯内存回归（`a_p2_r1_` 前缀；不写盘、不连 DB、不 spawn 真实进程、不改全局环境）
// ══════════════════════════════════════════════════════════════

fn recovery_parent() -> PathBuf {
    Path::new(support::RECOVERY_BASE)
        .parent()
        .expect("recovery root has a parent")
        .to_path_buf()
}

fn fake_base_dirs(fs: &FakeFs) {
    fs.add_dir(&recovery_parent());
    fs.add_dir(Path::new(support::RECOVERY_BASE));
}

#[test]
fn a_p2_r1_path_probe_allows_valid_chain() {
    let fs = FakeFs::new();
    fake_base_dirs(&fs);
    let ledger = OpsLedger::default();
    let run_id = "A-20990101T0000-abc1234";
    let prepared = driver::prepare_run_tree_with(&fs, &ledger, run_id, S1).expect("合法链应通过");
    assert!(prepared.tree.starts_with(support::RECOVERY_BASE));
    assert!(prepared.root.starts_with(support::RECOVERY_BASE));
    assert!(prepared.root_canon.starts_with(support::RECOVERY_BASE));
    assert!(prepared.reports_dir.starts_with(support::RECOVERY_BASE));
    assert!(ledger.creates() > 0, "合法路径应发生创建");
    assert_eq!(ledger.spawns(), 0, "prepare 阶段不得 spawn");
}

#[test]
fn a_p2_r1_path_probe_rejects_out_of_scope() {
    let fs = FakeFs::new();
    fake_base_dirs(&fs);
    let other = recovery_parent().join("other");
    fs.add_dir(&other);
    let base = PathBuf::from(support::RECOVERY_BASE);
    let err =
        support::verify_existing_dir_under(&fs, &base, &other, "probe").expect_err("越界必须拒绝");
    assert!(err.contains("越出恢复区"), "{err}");

    // 规范化解引用越界（词法在范围内，规范化后落到范围外）⇒ 必须拒绝。
    let fs2 = FakeFs::new();
    fake_base_dirs(&fs2);
    let inside = PathBuf::from(support::RECOVERY_BASE).join("CPB3V2-A-20990101T0000-abc1234");
    fs2.add_dir(&inside);
    fs2.add_canon(&inside, &recovery_parent().join("outside/escaped"));
    let err2 = support::verify_existing_dir_under(&fs2, &base, &inside, "probe_canon")
        .expect_err("规范化越界必须拒绝");
    assert!(err2.contains("规范化后越出恢复区"), "{err2}");
}

#[test]
fn a_p2_r1_path_probe_rejects_reparse_ancestor_root_and_run() {
    // 祖先重解析
    let fs = FakeFs::new();
    fs.add_reparse(&recovery_parent());
    fs.add_dir(Path::new(support::RECOVERY_BASE));
    assert!(
        support::verify_recovery_base(&fs).is_err(),
        "祖先重解析必须拒绝"
    );
    // 恢复区根自身重解析
    let fs2 = FakeFs::new();
    fs2.add_dir(&recovery_parent());
    fs2.add_reparse(Path::new(support::RECOVERY_BASE));
    assert!(
        support::verify_recovery_base(&fs2).is_err(),
        "根重解析必须拒绝"
    );
    // run 组件重解析
    let fs3 = FakeFs::new();
    fake_base_dirs(&fs3);
    let ledger = OpsLedger::default();
    let run_id = "A-20990101T0000-abc1234";
    let tree = support::tree_root(run_id);
    fs3.add_reparse(&tree);
    assert!(
        driver::prepare_run_tree_with(&fs3, &ledger, run_id, S1).is_err(),
        "tree/run 重解析必须拒绝"
    );
}

#[test]
fn a_p2_r1_path_probe_metadata_error_is_failure() {
    let fs = FakeFs::new();
    fs.add_dir(&recovery_parent());
    fs.add_dir(Path::new(support::RECOVERY_BASE));
    fs.add_meta_error(Path::new(support::RECOVERY_BASE));
    let err = support::verify_recovery_base(&fs).expect_err("metadata 失败必须 Err");
    assert!(err.contains("metadata"), "{err}");
}

#[test]
fn a_p2_r1_reject_branch_creates_and_spawns_nothing() {
    let fs = FakeFs::new();
    fs.add_reparse(&recovery_parent());
    fs.add_dir(Path::new(support::RECOVERY_BASE));
    let ledger = OpsLedger::default();
    let out = driver::prepare_run_tree_with(&fs, &ledger, "A-20990101T0000-abc1234", S1);
    assert!(out.is_err(), "重解析祖先必须拒绝");
    assert_eq!(ledger.creates(), 0, "拒绝分支不得创建目录");
    assert_eq!(ledger.spawns(), 0, "拒绝分支不得 spawn");
    assert!(fs.created_dirs().is_empty(), "拒绝分支不得留下创建记录");
}

/// 可注入失败的只读替身。
struct FailReader {
    mode: u8, // 0=read_dir 失败, 1=坏 JSONL
}

impl FsReader for FailReader {
    fn read_dir(&self, dir: &Path) -> std::io::Result<Vec<PathBuf>> {
        if self.mode == 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "fake read_dir error",
            ));
        }
        let _ = dir;
        Ok(Vec::new())
    }

    fn metadata(&self, path: &Path) -> std::io::Result<MetaFacts> {
        let _ = path;
        Ok(MetaFacts {
            is_dir: true,
            is_file: false,
            is_reparse: false,
            bytes: 0,
        })
    }

    fn read(&self, path: &Path) -> std::io::Result<Vec<u8>> {
        let _ = path;
        Ok(b"{not json}\n".to_vec())
    }
}

#[test]
fn a_p2_r1_manifest_enumeration_failure_is_not_empty_success() {
    let reader = FailReader { mode: 0 };
    let err = artifacts::scan_files_with(&reader, &Path::new(support::RECOVERY_BASE).join("x"))
        .expect_err("枚举失败必须 Err，不得当作空目录");
    assert!(err.contains("目录枚举失败"), "{err}");
}

#[test]
fn a_p2_r1_jsonl_decode_failure_is_not_empty_success() {
    let reader = FailReader { mode: 1 };
    let err = artifacts::read_jsonl_with(&reader, Path::new("calls.jsonl"))
        .expect_err("坏 JSONL 必须 Err，不得静默跳过");
    assert!(err.contains("解码失败"), "{err}");
}

#[test]
fn a_p2_r1_response_verifier_rejects_appended_reply() {
    let rf = json!({
        "reply": format!("{}{}", support::S1_REPLY, "（附加错误文本）"),
        "reply_is_fallback": false,
        "llm_fallback_reason": Value::Null,
        "scene_id": SCENE_ID,
    });
    let f = verify_response_facts(S1, &rf);
    assert!(!f.is_empty(), "附加文本必须被拒绝：{f:?}");

    let ok = json!({
        "reply": support::S1_REPLY,
        "reply_is_fallback": false,
        "llm_fallback_reason": Value::Null,
        "scene_id": SCENE_ID,
    });
    assert!(verify_response_facts(S1, &ok).is_empty());
}

#[test]
fn a_p2_r1_response_verifier_rejects_wrong_reason() {
    let rf = json!({
        "reply": "兜底回复",
        "reply_is_fallback": true,
        "llm_fallback_reason": "{\"code\":\"LLM_ERROR\",\"message\":\"另一个非空原因\"}",
        "scene_id": SCENE_ID,
    });
    let f = verify_response_facts(S3, &rf);
    assert!(!f.is_empty(), "错误原因必须被拒绝：{f:?}");

    let ok = json!({
        "reply": "兜底回复",
        "reply_is_fallback": true,
        "llm_fallback_reason": json!({"code": S3_REASON_CODE, "message": S3_REASON_MESSAGE}).to_string(),
        "scene_id": SCENE_ID,
    });
    assert!(
        verify_response_facts(S3, &ok).is_empty(),
        "{:?}",
        verify_response_facts(S3, &ok)
    );
}

#[test]
fn a_p2_r1_sqlite_verifier_rejects_swapped_ids_and_session_mismatch() {
    let response = json!({
        "reply": "回复", "user_message_id": "u1", "assistant_message_id": "a1",
    });
    let user_msg = "输入";
    let rows_ok = vec![
        json!({"id":"u1","session_id":ROLE_ID,"sender":"user","content":user_msg}),
        json!({"id":"a1","session_id":ROLE_ID,"sender":"assistant","content":"回复"}),
    ];
    let sessions_ok =
        vec![json!({"session_id":ROLE_ID,"role_id":ROLE_ID,"scene_id":SCENE_ID,"message_count":2})];
    assert!(verify_sqlite_rows(&rows_ok, &sessions_ok, Some(&response), user_msg).is_empty());

    // ID 对调
    let swapped = json!({
        "reply": "回复", "user_message_id": "a1", "assistant_message_id": "u1",
    });
    assert!(
        !verify_sqlite_rows(&rows_ok, &sessions_ok, Some(&swapped), user_msg).is_empty(),
        "ID 对调必须被拒绝"
    );
    // 会话不符
    let sessions_bad =
        vec![json!({"session_id":"other","role_id":ROLE_ID,"scene_id":SCENE_ID,"message_count":2})];
    assert!(
        !verify_sqlite_rows(&rows_ok, &sessions_bad, Some(&response), user_msg).is_empty(),
        "会话不符必须被拒绝"
    );
}

#[test]
fn a_p2_r1_opts_verifier_rejects_missing_key() {
    let base = json!({
        "method": "generate_with_opts", "model": MODEL_ID,
        "opts": {"some": true, "want_metrics": true, "keep_alive": "30m",
                 "temperature": Value::Null, "top_p": Value::Null,
                 "max_output_tokens": Value::Null, "preferred_context_tokens": Value::Null},
    });
    assert!(check_calls(S1, std::slice::from_ref(&base), &[], 1).is_empty());

    let mut missing = base.clone();
    missing["opts"]
        .as_object_mut()
        .unwrap()
        .remove("preferred_context_tokens");
    let f = check_calls(S1, &[missing], &[], 1);
    assert!(!f.is_empty(), "缺键必须被拒绝（不得等同 null）：{f:?}");
}

#[test]
fn a_p2_r1_summary_verifier_rejects_11_passed_and_mixed() {
    let ok = "test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s\n";
    assert!(driver::parse_libtest_summary(ok).is_ok());
    let eleven = "test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s\n";
    assert!(driver::parse_libtest_summary(eleven).is_err());
    let mixed = "test result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s\n";
    assert!(driver::parse_libtest_summary(mixed).is_err());
    let two = format!("{ok}{ok}");
    assert!(
        driver::parse_libtest_summary(&two).is_err(),
        "两条摘要必须拒绝"
    );
}

struct FakeChild {
    /// 非阻塞查询行为序列；耗尽后沿用 `last`。
    steps: Vec<PollStep>,
    idx: usize,
    last: PollStep,
    poll_err_from: Option<usize>,
    poll_err_until: Option<usize>,
    poll_calls: u32,
    kill_calls: u32,
    kill_fails: bool,
}

#[derive(Clone, Copy, Debug)]
enum PollStep {
    Running,
    Exited(Option<i32>),
}

impl FakeChild {
    fn new(steps: Vec<PollStep>, last: PollStep) -> Self {
        Self {
            steps,
            idx: 0,
            last,
            poll_err_from: None,
            poll_err_until: None,
            poll_calls: 0,
            kill_calls: 0,
            kill_fails: false,
        }
    }
}

impl ChildControl for FakeChild {
    fn try_wait(&mut self) -> std::io::Result<ChildPoll> {
        self.poll_calls += 1;
        if self.poll_err_from.is_some_and(|from| {
            let within_until = self
                .poll_err_until
                .is_none_or(|until| self.poll_calls as usize <= until);
            self.poll_calls as usize >= from && within_until
        }) {
            return Err(std::io::Error::other("fake try_wait error"));
        }
        let step = self.steps.get(self.idx).copied().unwrap_or(self.last);
        self.idx += 1;
        Ok(match step {
            PollStep::Running => ChildPoll::Running,
            PollStep::Exited(code) => ChildPoll::Exited(code),
        })
    }

    fn kill(&mut self) -> std::io::Result<()> {
        self.kill_calls += 1;
        if self.kill_fails {
            return Err(std::io::Error::other("fake kill error"));
        }
        Ok(())
    }
}

/// 虚拟时钟：每次读取前进固定步长（不使用真实 sleep）。
fn vclock(step_ms: u64) -> impl FnMut() -> Duration {
    let mut t = 0u64;
    move || {
        t += step_ms;
        Duration::from_millis(t)
    }
}

#[test]
fn a_p2_r1_parent_wait_error_kills_and_reaps_once_with_fields() {
    // 查询错误 ⇒ 进入清理；清理期首轮即报退出 ⇒ 确认回收并记录退出码。
    let mut c = FakeChild::new(vec![PollStep::Running], PollStep::Exited(Some(7)));
    c.poll_err_from = Some(1);
    c.poll_err_until = Some(1);
    let mut sleeps = 0u32;
    let sup = driver::supervise(
        &mut c,
        Duration::from_secs(120),
        Duration::from_millis(2000),
        &mut vclock(10),
        &mut |_| sleeps += 1,
    );
    assert!(sup.last_poll_error.is_some(), "{sup:?}");
    assert!(sup.cleanup_attempted && c.kill_calls == 1, "必须清理一次");
    assert!(sup.cleanup_confirmed && sup.exit_code == Some(7), "{sup:?}");
    assert_eq!(sup.kill_calls, 1);
}

#[test]
fn a_p2_r1_parent_timeout_kills_exactly_once() {
    // 运行期限先到（虚拟时钟），清理期首轮即报退出 ⇒ kill 一次并确认回收。
    let mut c = FakeChild::new(
        vec![
            PollStep::Running,
            PollStep::Running,
            PollStep::Exited(Some(0)),
        ],
        PollStep::Running,
    );
    let sup = driver::supervise(
        &mut c,
        Duration::from_millis(500),
        Duration::from_millis(2000),
        &mut vclock(400),
        &mut |_| {},
    );
    assert!(sup.run_deadline_hit, "{sup:?}");
    assert_eq!(c.kill_calls, 1, "超时只终止一次");
    assert!(sup.cleanup_confirmed && sup.exited, "{sup:?}");
}

#[tokio::test]
async fn a_p2_r1_child_error_still_finalizes_once() {
    let attempts = AtomicUsize::new(0);
    let fin = run_finalize(Duration::from_secs(5), &attempts, || async {}).await;
    assert_eq!(fin.outcome, "returned");
    assert_eq!(fin.attempts, 1, "收尾必须有且仅有一次");

    let attempts2 = AtomicUsize::new(0);
    let fin2 = run_finalize(Duration::from_millis(20), &attempts2, || async {
        std::future::pending::<()>().await;
    })
    .await;
    assert_eq!(fin2.outcome, "timeout");
    assert_eq!(fin2.attempts, 1, "超时也不得重复收尾");

    // A-P2-R2 注记：上面的用例只覆盖“收尾 helper 正常/超时各执行一次”；
    // 下面补上“工作先返回 Err 再收尾”的控制边（不新搭子任务框架）。
    async fn work_fails() -> Result<(), String> {
        Err("synthetic work error".into())
    }
    let attempts3 = AtomicUsize::new(0);
    let work = work_fails().await;
    let fin3 = run_finalize(Duration::from_secs(5), &attempts3, || async {}).await;
    assert!(work.is_err(), "控制边必须先是 Err 工作");
    assert_eq!(fin3.attempts, 1, "Err 工作路径仍只收尾一次");
}

// ══════════════════════════════════════════════════════════════
// A-P2-R2 纯内存回归：拒绝零写入、有界清理、通过后才发布
// ══════════════════════════════════════════════════════════════

fn fake_tree_paths(run_id: &str, scenario: &str) -> (PathBuf, PathBuf, PathBuf) {
    let tree = support::tree_root(run_id);
    let run_dir = tree.join("run");
    let root = support::scenario_root(run_id, scenario);
    (tree, run_dir, root)
}

#[test]
fn a_p2_r2_reject_before_report_dir_writes_nothing() {
    let fs = FakeFs::new();
    fs.add_reparse(&recovery_parent());
    fs.add_dir(Path::new(support::RECOVERY_BASE));
    let ledger = OpsLedger::default();
    let sink = driver::CountingSink::new(true);
    let out = driver::prepare_run_tree_with(&fs, &ledger, "A-20990101T0000-abc1234", S1);
    assert!(out.is_err(), "祖先重解析必须拒绝");
    // driver 的实际失败连接：未拿到 PreparedTree ⇒ 不调用任何 writer。
    let wrote = driver::handle_failure(&sink, None, &Map::new(), "path rejected");
    assert!(wrote.is_none(), "拒绝阶段不得调用报告 writer");
    assert_eq!(sink.calls(), 0, "拒绝阶段报告写回调必须为 0");
    assert_eq!(ledger.creates(), 0);
    assert_eq!(ledger.spawns(), 0);
    assert!(fs.created_dirs().is_empty());
}

#[test]
fn a_p2_r2_metadata_permission_denied_rejects_tree_and_run() {
    // tree 组件 metadata PermissionDenied
    let run_id = "A-20990101T0000-abc1234";
    let (tree, run_dir, _) = fake_tree_paths(run_id, S1);
    let fs = FakeFs::new();
    fake_base_dirs(&fs);
    fs.add_meta_error(&tree);
    let ledger = OpsLedger::default();
    let sink = driver::CountingSink::new(true);
    assert!(
        driver::prepare_run_tree_with(&fs, &ledger, run_id, S1).is_err(),
        "tree metadata 非 NotFound 必须拒绝"
    );
    assert_eq!(ledger.creates(), 0, "非 NotFound 不得创建");
    assert_eq!(sink.calls(), 0);

    // run 组件 metadata PermissionDenied（tree 已存在且合法）
    let fs2 = FakeFs::new();
    fake_base_dirs(&fs2);
    fs2.add_dir(&tree);
    fs2.add_meta_error(&run_dir);
    let ledger2 = OpsLedger::default();
    let sink2 = driver::CountingSink::new(true);
    assert!(
        driver::prepare_run_tree_with(&fs2, &ledger2, run_id, S1).is_err(),
        "run metadata 非 NotFound 必须拒绝"
    );
    assert_eq!(ledger2.creates(), 0, "run 检查失败时不得创建");
    assert_eq!(sink2.calls(), 0);
}

#[test]
fn a_p2_r2_existing_scenario_rejected_old_bytes_untouched() {
    let run_id = "A-20990101T0000-abc1234";
    let (tree, run_dir, root) = fake_tree_paths(run_id, S1);
    let fs = FakeFs::new();
    fake_base_dirs(&fs);
    fs.add_dir(&tree);
    fs.add_dir(&run_dir);
    fs.add_dir(&root); // 旧场景（含旧报告）已存在
    let ledger = OpsLedger::default();
    let sink = driver::CountingSink::new(true);
    let err = driver::prepare_run_tree_with(&fs, &ledger, run_id, S1)
        .expect_err("已存在场景必须拒绝复用");
    assert!(err.contains("禁止复用与覆盖旧报告"), "{err}");
    assert_eq!(sink.calls(), 0, "不得向旧场景写报告");
    assert!(
        !fs.created_dirs().iter().any(|p| p == &root),
        "不得重建/覆盖已存在场景"
    );
}

#[test]
fn a_p2_r2_partial_creation_then_stop_without_report() {
    let run_id = "A-20990101T0000-abc1234";
    let (_, _, root) = fake_tree_paths(run_id, S1);
    let fs = FakeFs::new();
    fake_base_dirs(&fs);
    fs.add_dir(&root); // 场景已存在 ⇒ tree/run 会被创建后停止
    let ledger = OpsLedger::default();
    let sink = driver::CountingSink::new(true);
    assert!(driver::prepare_run_tree_with(&fs, &ledger, run_id, S1).is_err());
    assert!(
        ledger.creates() > 0,
        "该路径属于“部分创建后停止”，不是“拒绝前零写入”"
    );
    assert_eq!(sink.calls(), 0, "部分创建失败也不得写报告");
}

#[test]
fn a_p2_r2_normal_new_tree_allows_current_report() {
    let run_id = "A-20990101T0000-abc1234";
    let fs = FakeFs::new();
    fake_base_dirs(&fs);
    let ledger = OpsLedger::default();
    let prepared = driver::prepare_run_tree_with(&fs, &ledger, run_id, S1).expect("全新树应通过");
    assert!(prepared.reports_dir.ends_with("reports"));
    let sink = driver::CountingSink::new(true);
    // 失败处理：本次已验证目录允许写入
    let wrote = driver::handle_failure(&sink, Some(&prepared), &Map::new(), "synthetic");
    assert!(wrote.is_some(), "已验证报告目录应允许写入");
    assert_eq!(sink.calls(), 1);
    assert_eq!(sink.ok_true_writes(), 0, "失败状态不得写 ok=true");
    // 成功发布：仅在全通过时
    let published =
        driver::publish_if_all_checks_pass(&sink, &prepared, &Map::new(), &[]).expect("publish");
    assert!(published);
    assert_eq!(sink.ok_true_writes(), 1);
}

/// 联合检查上下文构造（测试私有，借用调用方数据）。
fn joint_ctx<'a>(
    run_token: &'a str,
    run_id: &'a str,
    root_canon: &'a Path,
    child_report: &'a Value,
    protected_same: bool,
    supervision_failed: bool,
) -> driver::JointContext<'a> {
    driver::JointContext {
        run_token,
        scenario: S1,
        run_id,
        root_canon,
        child_report,
        protected_same,
        survivors: &[],
        supervision_failed,
    }
}

#[test]
fn a_p2_r2_no_ok_true_write_when_joint_checks_fail() {
    let run_id = "A-20990101T0000-abc1234";
    let fs = FakeFs::new();
    fake_base_dirs(&fs);
    let ledger = OpsLedger::default();
    let prepared = driver::prepare_run_tree_with(&fs, &ledger, run_id, S1).expect("准备");
    let good = json!({
        "run_token": "tok", "scenario": S1, "run_id": run_id,
        "scenario_root_canonical": prepared.root_canon.display().to_string(), "ok": true,
    });
    let root_canon = prepared.root_canon.clone();
    // 正例：全通过
    let mut bad_child = good.clone();
    bad_child["ok"] = json!(false);
    let none = driver::joint_checks(&joint_ctx("tok", run_id, &root_canon, &good, true, false));
    assert!(none.is_empty(), "{none:?}");
    let f1 = driver::joint_checks(&joint_ctx("other", run_id, &root_canon, &good, true, false));
    assert!(f1.contains(&"run_token"), "{f1:?}");
    let f2 = driver::joint_checks(&joint_ctx(
        "tok",
        run_id,
        &root_canon,
        &bad_child,
        true,
        false,
    ));
    assert!(f2.contains(&"child_ok"), "{f2:?}");
    let f3 = driver::joint_checks(&joint_ctx("tok", run_id, &root_canon, &good, false, false));
    assert!(f3.contains(&"protected_same"), "{f3:?}");
    let f4 = driver::joint_checks(&joint_ctx("tok", run_id, &root_canon, &good, true, true));
    assert!(f4.contains(&"supervision_clean"), "{f4:?}");

    for failed in [f1, f2, f3, f4] {
        let sink = driver::CountingSink::new(true);
        let published = driver::publish_if_all_checks_pass(&sink, &prepared, &Map::new(), &failed)
            .expect("publish");
        assert!(!published, "未通过时不得发布成功");
        assert_eq!(
            sink.ok_true_writes(),
            0,
            "未通过时不得产生任何 ok=true 写入"
        );
        assert_eq!(sink.calls(), 1, "只允许发布失败状态");
    }
}

#[test]
fn a_p2_r2_cleanup_kill_fails_and_never_exits_is_bounded_failure() {
    let mut c = FakeChild::new(Vec::new(), PollStep::Running);
    c.kill_fails = true;
    let mut sleeps = 0u32;
    let sup = driver::supervise(
        &mut c,
        Duration::from_secs(120),
        Duration::from_millis(2000),
        &mut vclock(400),
        &mut |_| sleeps += 1,
    );
    assert!(sup.cleanup_attempted, "{sup:?}");
    assert_eq!(sup.kill_calls, 1, "kill 只尝试一次");
    assert!(sup.kill_error.is_some(), "保留 kill 失败原文");
    assert!(!sup.cleanup_confirmed, "不得假装回收成功");
    assert!(sup.blocked_reason.is_some(), "{sup:?}");
    assert!(sup.stage.contains("cleanup_unconfirmed"), "{sup:?}");
    assert!(sup.sleep_count >= 1 && sup.cleanup_poll_count >= 1);
    assert!(
        sup.cleanup_poll_count < 1000,
        "轮询必须有界：{}",
        sup.cleanup_poll_count
    );
}

#[test]
fn a_p2_r2_cleanup_kill_ok_but_no_exit_before_deadline_is_bounded() {
    let mut c = FakeChild::new(Vec::new(), PollStep::Running);
    let sup = driver::supervise(
        &mut c,
        Duration::from_secs(120),
        Duration::from_millis(500),
        &mut vclock(100),
        &mut |_| {},
    );
    assert_eq!(c.kill_calls, 1);
    assert!(sup.kill_error.is_none());
    assert!(sup.cleanup_unconfirmed(), "{sup:?}");
    assert_eq!(sup.exit_code, None);
    assert!(!sup.exited);
}

#[test]
fn a_p2_r2_consecutive_poll_errors_still_bounded() {
    // F2：清理期**连续**查询错误（从第 1 次查询起每次都 Err），虚拟时钟推进。
    let mut c = FakeChild::new(Vec::new(), PollStep::Running);
    c.poll_err_from = Some(1);
    let sup = driver::supervise(
        &mut c,
        Duration::from_secs(120),
        Duration::from_millis(300),
        &mut vclock(150),
        &mut |_| {},
    );
    // 实际发生多次错误查询（不是只注入一次）
    assert!(c.poll_calls >= 2, "必须真的连续查询：{}", c.poll_calls);
    assert!(sup.cleanup_attempted && sup.kill_calls == 1, "{sup:?}");
    assert!(sup.last_poll_error.is_some(), "原错误必须保存：{sup:?}");
    assert!(!sup.cleanup_confirmed, "持续查询错误不得假装回收成功");
    assert!(sup.blocked_reason.is_some(), "{sup:?}");
    assert!(sup.stage.contains("cleanup_query_error"), "{sup:?}");
    assert!(sup.poll_count < 1000, "轮询必须有界：{}", sup.poll_count);
    // F1 实际准入：即使“退出码=0”也不得改判成功；再经报告连接仍是失败且零 ok=true 写入。
    let reason = driver::supervision_failure(&sup).expect("必须判失败");
    assert!(reason.contains("查询错误"), "{reason}");
    let run_id = "A-20990101T0000-abc1234";
    let fs = FakeFs::new();
    fake_base_dirs(&fs);
    let ledger = OpsLedger::default();
    let prepared = driver::prepare_run_tree_with(&fs, &ledger, run_id, S1).expect("准备");
    let sink = driver::CountingSink::new(true);
    let failed = vec!["supervision_clean"];
    let published = driver::publish_if_all_checks_pass(&sink, &prepared, &Map::new(), &failed)
        .expect("publish");
    assert!(!published);
    assert_eq!(sink.ok_true_writes(), 0, "不得产生任何 ok=true 写入");
}

#[test]
fn a_p2_r2_normal_exit_is_not_killed_and_code_missing_is_not_running() {
    // 正常退出：不 kill
    let mut c = FakeChild::new(vec![PollStep::Exited(Some(0))], PollStep::Exited(Some(0)));
    let sup = driver::supervise(
        &mut c,
        Duration::from_secs(120),
        Duration::from_millis(2000),
        &mut vclock(10),
        &mut |_| {},
    );
    assert!(sup.exited && sup.exit_code == Some(0));
    assert_eq!(c.kill_calls, 0, "正常退出不得 kill");
    assert!(!sup.cleanup_attempted);

    // 退出但无 code：与“仍运行”明确区分
    let mut c2 = FakeChild::new(vec![PollStep::Exited(None)], PollStep::Exited(None));
    let sup2 = driver::supervise(
        &mut c2,
        Duration::from_secs(120),
        Duration::from_millis(2000),
        &mut vclock(10),
        &mut |_| {},
    );
    assert!(sup2.exited, "无 code 也属于已退出");
    assert_eq!(sup2.exit_code, None);
    assert_eq!(sup2.stage, "exited");
    assert_eq!(c2.kill_calls, 0);
}

// ══════════════════════════════════════════════════════════════
// A-P2-R3：F1 监督失败不得被成功收尾洗白 / F3 拒绝连接接线
// ══════════════════════════════════════════════════════════════

const R3_RUN_ID: &str = "A-20990101T0000-abc1234";

/// F1：四类监督失败即使随后观测到 `Exited(Some(0))` 也不得判成功；两类正常情形如实区分。
#[test]
fn a_p2_r3_supervision_failures_never_become_success() {
    // ① 运行期限到 → 清理期观测退出码 0
    let mut c1 = FakeChild::new(
        vec![
            PollStep::Running,
            PollStep::Running,
            PollStep::Exited(Some(0)),
        ],
        PollStep::Running,
    );
    let s1 = driver::supervise(
        &mut c1,
        Duration::from_millis(500),
        Duration::from_millis(2000),
        &mut vclock(400),
        &mut |_| {},
    );
    assert!(
        s1.run_deadline_hit && s1.exit_code == Some(0) && s1.exited,
        "{s1:?}"
    );
    let r1 = driver::supervision_failure(&s1).expect("超时后退出 0 仍必须判失败");
    assert!(r1.contains("运行期限到"), "{r1}");

    // ② 监督查询错误 → 清理期观测退出码 0
    let mut c2 = FakeChild::new(vec![PollStep::Exited(Some(0))], PollStep::Exited(Some(0)));
    c2.poll_err_from = Some(1);
    c2.poll_err_until = Some(1);
    let s2 = driver::supervise(
        &mut c2,
        Duration::from_secs(120),
        Duration::from_millis(2000),
        &mut vclock(10),
        &mut |_| {},
    );
    assert!(s2.exit_code == Some(0) && s2.cleanup_confirmed, "{s2:?}");
    let r2 = driver::supervision_failure(&s2).expect("查询错误后退出 0 仍必须判失败");
    assert!(r2.contains("查询错误"), "{r2}");

    // ③ 清理 kill 失败 → 随后观测退出码 0
    let mut c3 = FakeChild::new(vec![PollStep::Running], PollStep::Exited(Some(0)));
    c3.kill_fails = true;
    let s3 = driver::supervise(
        &mut c3,
        Duration::from_millis(100),
        Duration::from_millis(2000),
        &mut vclock(200),
        &mut |_| {},
    );
    assert_eq!(c3.kill_calls, 1, "kill 次数不得扩大");
    assert!(s3.kill_error.is_some() && s3.exit_code == Some(0), "{s3:?}");
    assert!(
        driver::supervision_failure(&s3).is_some(),
        "kill 失败必须保留为失败"
    );

    // ④ 持续未退出 → 清理未确认（BLOCKED）
    let mut c4 = FakeChild::new(Vec::new(), PollStep::Running);
    let s4 = driver::supervise(
        &mut c4,
        Duration::from_millis(100),
        Duration::from_millis(300),
        &mut vclock(150),
        &mut |_| {},
    );
    let r4 = driver::supervision_failure(&s4).expect("未确认回收必须判失败");
    assert!(r4.contains("BLOCKED"), "{r4}");

    // ⑤ 正常退出 0 → 监督层成功
    let mut c5 = FakeChild::new(vec![PollStep::Exited(Some(0))], PollStep::Exited(Some(0)));
    let s5 = driver::supervise(
        &mut c5,
        Duration::from_secs(120),
        Duration::from_millis(2000),
        &mut vclock(10),
        &mut |_| {},
    );
    assert!(driver::supervision_failure(&s5).is_none(), "{s5:?}");
    assert_eq!(c5.kill_calls, 0);

    // ⑥ 退出但无退出码 → 已退出但非成功
    let mut c6 = FakeChild::new(vec![PollStep::Exited(None)], PollStep::Exited(None));
    let s6 = driver::supervise(
        &mut c6,
        Duration::from_secs(120),
        Duration::from_millis(2000),
        &mut vclock(10),
        &mut |_| {},
    );
    let r6 = driver::supervision_failure(&s6).expect("无码不得判成功");
    assert!(r6.contains("退出码"), "{r6}");

    // 上述四类失败经“实际报告连接”不得写出任何 ok=true。
    let fs = FakeFs::new();
    fake_base_dirs(&fs);
    let ledger = OpsLedger::default();
    let prepared = driver::prepare_run_tree_with(&fs, &ledger, R3_RUN_ID, S1).expect("准备");
    for sup in [&s1, &s2, &s3, &s4] {
        assert!(driver::supervision_failure(sup).is_some());
        let sink = driver::CountingSink::new(true);
        let published = driver::publish_if_all_checks_pass(
            &sink,
            &prepared,
            &Map::new(),
            &["supervision_clean"],
        )
        .expect("publish");
        assert!(!published);
        assert_eq!(sink.ok_true_writes(), 0);
    }
}

/// F3：拒绝连接（driver 实际使用）在真实 ledger+sink 下零写入，且旧报告 sentinel 保持原值。
#[test]
fn a_p2_r3_reject_connection_zero_writes_and_old_bytes_kept() {
    let (tree, run_dir, root) = fake_tree_paths(R3_RUN_ID, S1);
    let sentinel_path = root
        .join("reports/parent-report.json")
        .display()
        .to_string();
    let sentinel = b"{\"ok\":true,\"old\":\"sentinel\"}".to_vec();

    // 用例：祖先重解析 / 恢复区根重解析 / 仅 run 重解析（tree 正常）/ tree metadata 权限错 /
    //       run metadata 权限错 / 旧 scenario 已存在
    let mut cases: Vec<(&str, FakeFs, usize)> = Vec::new();

    let c1 = FakeFs::new();
    c1.add_reparse(&recovery_parent());
    c1.add_dir(Path::new(support::RECOVERY_BASE));
    cases.push(("祖先重解析", c1, 0));

    let c2 = FakeFs::new();
    c2.add_dir(&recovery_parent());
    c2.add_reparse(Path::new(support::RECOVERY_BASE));
    cases.push(("恢复区根重解析", c2, 0));

    let c3 = FakeFs::new();
    fake_base_dirs(&c3);
    c3.add_dir(&tree);
    c3.add_reparse(&run_dir);
    cases.push(("仅 run 重解析", c3, 0));

    let c4 = FakeFs::new();
    fake_base_dirs(&c4);
    c4.add_meta_error(&tree);
    cases.push(("tree metadata 权限错", c4, 0));

    let c5 = FakeFs::new();
    fake_base_dirs(&c5);
    c5.add_dir(&tree);
    c5.add_meta_error(&run_dir);
    cases.push(("run metadata 权限错", c5, 0));

    let c6 = FakeFs::new();
    fake_base_dirs(&c6);
    c6.add_dir(&tree);
    c6.add_dir(&run_dir);
    c6.add_dir(&root);
    cases.push(("旧 scenario 已存在", c6, 0));

    for (label, fs, expected_creates) in cases {
        let ledger = OpsLedger::default();
        let sink = driver::CountingSink::new(true);
        sink.seed_bytes(&sentinel_path, &sentinel);
        let out = driver::prepare_or_reject(&fs, &ledger, R3_RUN_ID, S1, &sink);
        assert!(out.is_err(), "{label} 必须被拒绝");
        assert_eq!(sink.calls(), 0, "{label}: 报告写回调必须为 0");
        assert_eq!(ledger.creates(), expected_creates, "{label}: 创建计数");
        assert_eq!(
            sink.read_bytes(&sentinel_path).as_deref(),
            Some(sentinel.as_slice()),
            "{label}: 旧报告字节必须保持原值"
        );
    }

    // 部分创建后停止：tree/run 会被创建，仍然零写入且旧字节保持
    let fs7 = FakeFs::new();
    fake_base_dirs(&fs7);
    fs7.add_dir(&root);
    let ledger7 = OpsLedger::default();
    let sink7 = driver::CountingSink::new(true);
    sink7.seed_bytes(&sentinel_path, &sentinel);
    assert!(driver::prepare_or_reject(&fs7, &ledger7, R3_RUN_ID, S1, &sink7).is_err());
    assert!(
        ledger7.creates() > 0,
        "该路径确有目录创建（部分创建后停止）"
    );
    assert_eq!(sink7.calls(), 0, "部分创建失败仍不得写报告");
    assert_eq!(
        sink7.read_bytes(&sentinel_path).as_deref(),
        Some(sentinel.as_slice())
    );

    // 正常新树正例：允许本次写入（路径算法不改）
    let fs8 = FakeFs::new();
    fake_base_dirs(&fs8);
    let ledger8 = OpsLedger::default();
    let sink8 = driver::CountingSink::new(true);
    let prepared = driver::prepare_or_reject(&fs8, &ledger8, R3_RUN_ID, S1, &sink8).expect("新树");
    assert!(driver::handle_failure(&sink8, Some(&prepared), &Map::new(), "synthetic").is_some());
    assert_eq!(sink8.calls(), 1);
    assert_eq!(sink8.ok_true_writes(), 0);
}
