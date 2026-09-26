//! B preparation: one real non-streaming turn, reusing the accepted A supervisor.
use crate::live_proxy::{self, Proxy, MODEL};
use crate::semantic_cases::Case;
use crate::support::{self, artifacts, OpsLedger, RealFs, ROLE_ID, SCENE_ID};
use serde_json::{json, Map, Value};
use std::path::Path;
use std::sync::atomic::AtomicUsize;
use std::time::{Duration, Instant};

fn blueprint() -> Value {
    json!({
        "schema_version":4,
        "meta":{
            "id":ROLE_ID,"name":"B Probe Role","version":"0.0.1","author":"harness",
            "description":"isolated real local model verification; no tools",
            "personality":[0.5,0.5,0.5,0.5,0.5,0.5,0.5],
            "ollama_model":MODEL,"evolution":{"personality_source":"vector"},
            "relations":{"friend":{"initial_favorability":50,"favor_multiplier":1}},
            "default_relation":"friend","scenes":[]
        },
        "slot_registry":{
            "llm":{"type":"llm","label":"LLM","backend":"ollama","position":0,"model":MODEL},
            "memory":{"type":"memory","label":"Memory","backend":"builtin","position":1},
            "emotion":{"type":"emotion","label":"Emotion","backend":"builtin","position":2},
            "event":{"type":"event","label":"Event","backend":"builtin","position":3},
            "prompt":{"type":"prompt","label":"Prompt","backend":"builtin","position":4},
            "agent":{"type":"agent","label":"Agent","backend":"none","position":5}
        },
        "runtime_config":{
            "inference_profile":{
                "generation":{"temperature":0.2,"top_p":0.9,"maximum_output_tokens":256},
                "context":{"preferred_tokens":4096}
            }
        }
    })
}

fn prepare_role(root: &Path) -> Result<Value, String> {
    let roles = root.join("roles");
    let dir = roles.join(ROLE_ID);
    support::create_new_dir_verified(&RealFs, &roles, &dir, "B role", &OpsLedger::default())?;
    let bp = blueprint();
    oclive_validation::validate_blueprint_v4_json(&bp.to_string(), Some(ROLE_ID))
        .map_err(|e| format!("blueprint validation: {e:?}"))?;
    artifacts::write_json(&dir.join("pipeline.ocblueprint"), &bp)?;
    artifacts::write_json(
        &dir.join("config.json"),
        &json!({"chat_storage":{"mirror":false}}),
    )?;
    let role = oclive_kernel_host::infrastructure::RoleStorage::new(&roles)
        .load_role(ROLE_ID)
        .map_err(|e| e.to_string())?;
    if role.id != ROLE_ID
        || role.ollama_model.as_deref() != Some(MODEL)
        || format!("{:?}", role.plugin_backends.llm) != "Ollama"
        || format!("{:?}", role.plugin_backends.agent) != "None"
        || !format!("{:?}", role.evolution_config.personality_source).contains("Vector")
        || role.scene_ids.len() > 1
    {
        return Err(
            "loaded role differs from the frozen model/backend/evolution configuration".into(),
        );
    }
    let actual = serde_json::to_value(&role.runtime_config).map_err(|e| e.to_string())?;
    if actual.get("inference_profile")
        != bp
            .get("runtime_config")
            .and_then(|r| r.get("inference_profile"))
    {
        // Optional serialized fields may be retained as null; compare the operative fields below.
        let profile = role
            .runtime_config
            .as_ref()
            .and_then(|r| r.inference_profile.as_ref())
            .ok_or("loaded inference profile missing")?;
        let generation = profile
            .generation
            .as_ref()
            .ok_or("generation profile missing")?;
        if generation.maximum_output_tokens != Some(256)
            || generation.temperature != Some(0.2)
            || generation.top_p != Some(0.9)
            || profile.context.as_ref().and_then(|c| c.preferred_tokens) != Some(4096)
        {
            return Err("loaded inference limits changed".into());
        }
    }
    Ok(
        json!({"blueprint":bp,"loaded_runtime_config":actual,"loaded_model":role.ollama_model,
        "backends":format!("{:?}",role.plugin_backends),"scene_ids":role.scene_ids.as_ref()}),
    )
}

fn response_failures(response: &Value) -> Vec<String> {
    let mut failures = Vec::new();
    if !response
        .get("reply")
        .and_then(Value::as_str)
        .is_some_and(|r| !r.trim().is_empty())
    {
        failures.push("reply missing or empty".into());
    }
    if response.get("reply_is_fallback").and_then(Value::as_bool) != Some(false) {
        failures.push("product returned fallback instead of a normal live reply".into());
    }
    if !response
        .get("llm_fallback_reason")
        .is_some_and(Value::is_null)
    {
        failures.push("unexpected or missing fallback reason field".into());
    }
    if response.get("chat_persist_failed").and_then(Value::as_bool) == Some(true) {
        failures.push("chat persistence failure".into());
    }
    failures
}

async fn run_kernel(
    root: &Path,
    proxy: &Proxy,
    case: &'static Case,
    facts: &mut Map<String, Value>,
    failures: &mut Vec<String>,
) {
    let role = match prepare_role(root) {
        Ok(v) => v,
        Err(e) => {
            failures.push(e);
            return;
        }
    };
    facts.insert("fixture".into(), role);
    let cfg =
        oclive_kernel_host::OcliveKernelConfig::new(root.join("app-data"), root.join("roles"));
    let db = cfg.database_path().to_path_buf();
    if db != root.join("app-data/app.db") {
        failures.push("database path differs from approved new tree".into());
        return;
    }
    facts.insert("database_path".into(), json!(db.display().to_string()));
    if let Err(e) =
        support::verify_existing_chain(&RealFs, &root.join("app-data"), "B database parent")
    {
        failures.push(format!("database path gate: {e}"));
        return;
    }
    let profile = oclive_kernel_host::domain::host_profile::HostProfile {
        event_impact_llm: false,
        skip_agent: true,
        ..Default::default()
    };
    if profile.llm_runtime.mode
        != oclive_kernel_host::domain::host_profile::LocalLlmRuntimeMode::Ollama
        || profile.theater.director_plugin.is_some()
    {
        failures.push("unexpected HostProfile default".into());
        return;
    }
    facts.insert(
        "host_profile".into(),
        json!({"mode":"ollama","event_impact_llm":false,"skip_agent":true}),
    );
    // This child executes only one test; no other thread reads these keys yet.
    std::env::set_var("OLLAMA_BASE_URL", &proxy.endpoint);
    facts.insert(
        "endpoint_at_client_construction".into(),
        json!(proxy.endpoint),
    );
    let kernel = match oclive_kernel_host::OcliveKernelBuilder::new(cfg)
        .with_host_profile(profile)
        .build()
        .await
    {
        Ok(k) => k,
        Err(e) => {
            failures.push(format!("build: {e}"));
            return;
        }
    };
    facts.insert(
        "endpoint_env_after_build".into(),
        json!(std::env::var("OLLAMA_BASE_URL").ok()),
    );
    let ready = match kernel.load_role(ROLE_ID).await {
        Ok(_) => match proxy.enable() {
            Ok(()) => true,
            Err(e) => {
                failures.push(e);
                false
            }
        },
        Err(e) => {
            failures.push(format!("load_role: {e}"));
            false
        }
    };
    let mut response = None;
    if ready {
        let req = turn_request(case);
        let started = Instant::now();
        match tokio::time::timeout(Duration::from_secs(150), kernel.process_message(&req)).await {
            Ok(Ok(r)) => {
                let v = crate::response_facts_of(&r);
                failures.extend(response_failures(&v));
                facts.insert("response".into(), v.clone());
                response = Some(v);
            }
            Ok(Err(e)) => failures.push(format!("process_message: {e}")),
            Err(_) => failures
                .push("turn deadline: remote inference state UNKNOWN; no automatic rerun".into()),
        }
        facts.insert("turn_ms".into(), json!(started.elapsed().as_millis()));
    }
    // Do not permit any new generation during shutdown, including after a turn timeout.
    proxy.disable();
    let attempts = AtomicUsize::new(0);
    let fin = crate::run_finalize(Duration::from_secs(30), &attempts, || kernel.shutdown()).await;
    facts.insert(
        "shutdown".into(),
        json!({"outcome":fin.outcome,"attempts":fin.attempts}),
    );
    if fin.outcome == "returned" && response.is_some() {
        let (v, errors) =
            crate::verify_sqlite(&db, support::B1, case.message, response.as_ref()).await;
        facts.insert("sqlite_verify".into(), v);
        failures.extend(errors);
    } else {
        facts.insert("sqlite_verify".into(), json!({"state":"not_executed"}));
        if fin.outcome != "returned" {
            failures.push("shutdown did not return; DB check skipped".into());
        }
    }
    match artifacts::scan_files(&root.join("chats")) {
        Ok(scan) if scan.state == artifacts::ScanState::Present && scan.file_count() == 0 => {
            facts.insert("chats_files".into(), json!([]));
        }
        Ok(scan) => failures.push(format!(
            "chats is not present and empty: {:?}",
            scan.relative_paths()
        )),
        Err(e) => failures.push(format!("chats scan: {e}")),
    }
}

fn turn_request(case: &Case) -> support::SendMessageRequest {
    support::SendMessageRequest {
        client_request_id: None,
        role_id: ROLE_ID.into(),
        user_message: case.message.into(),
        scene_id: Some(SCENE_ID.into()),
        session_id: None,
        include_raw_reply: Some(true),
        adult: None,
    }
}

pub async fn run_scenario(root: &Path, case: &'static Case) -> (Map<String, Value>, Vec<String>) {
    let mut facts = Map::new();
    let mut failures = Vec::new();
    facts.insert("user_message".into(), json!(case.message));
    facts.insert("semantic_case".into(), case.identity());
    facts.insert(
        "semantic_review".into(),
        json!({"status":"PENDING_CONTROLLER_REVIEW",
        "rubric":case.rubric,
        "note":"machine ok is structural only, not a model-quality verdict"}),
    );
    match live_proxy::preflight(case).await {
        Ok(v) => {
            facts.insert("preflight".into(), v);
        }
        Err(e) => {
            failures.push(e);
            return (facts, failures);
        }
    }
    let mut proxy = match Proxy::start(root.join("artifacts/proxy-ledger.json"), case).await {
        Ok(p) => p,
        Err(e) => {
            failures.push(e);
            return (facts, failures);
        }
    };
    run_kernel(root, &proxy, case, &mut facts, &mut failures).await;
    if let Err(e) = proxy.stop().await {
        failures.push(e);
    }
    let snapshot = proxy.snapshot();
    if snapshot.get("semantic_case") != Some(&case.identity()) {
        failures.push("gateway sample identity differs from the turn".into());
    }
    let count = snapshot
        .get("forward_attempts")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    if !(1..=3).contains(&count) {
        failures.push(format!("expected 1..=3 forward attempts, got {count}"));
    }
    let received = snapshot
        .get("received_http_requests")
        .and_then(Value::as_u64);
    let responses = snapshot
        .get("events")
        .and_then(Value::as_array)
        .map(|events| {
            events
                .iter()
                .filter(|event| event.get("phase").and_then(Value::as_str) == Some("response"))
                .count() as u64
        });
    if received != Some(count) || responses != Some(count) {
        failures.push(
            "request/forward/response ledger is incomplete or contains extra requests".into(),
        );
    }
    if snapshot.get("busy").and_then(Value::as_bool) != Some(false) {
        failures.push("transfer still in flight; server inference state UNKNOWN".into());
    }
    if let Some(violations) = snapshot.get("violations").and_then(Value::as_array) {
        for v in violations {
            failures.push(format!("gateway violation: {v}"));
        }
    } else {
        failures.push("gateway violations ledger missing".into());
    }
    facts.insert("gateway".into(), snapshot);
    facts.insert(
        "automatic_result_scope".into(),
        json!("STRUCTURE_ONLY; final semantic verdict belongs to controller"),
    );
    (facts, failures)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn b_sv1_turn_input_and_sqlite_verifier_use_each_selected_message() {
        for case in &crate::semantic_cases::CASES {
            let req = turn_request(case);
            assert_eq!(req.user_message, case.message);
            assert_eq!(req.role_id, ROLE_ID);
            assert_eq!(req.scene_id.as_deref(), Some(SCENE_ID));
            assert!(req.session_id.is_none());
            assert!(!req.user_message.contains(case.rubric));
            let response =
                json!({"reply":"回复","user_message_id":"u1","assistant_message_id":"a1"});
            let rows = vec![
                json!({"id":"u1","sender":"user","session_id":ROLE_ID,"content":req.user_message}),
                json!({"id":"a1","sender":"assistant","session_id":ROLE_ID,"content":"回复"}),
            ];
            let sessions = vec![
                json!({"session_id":ROLE_ID,"role_id":ROLE_ID,"scene_id":SCENE_ID,"message_count":2}),
            ];
            assert!(
                crate::verify_sqlite_rows(&rows, &sessions, Some(&response), case.message)
                    .is_empty()
            );
            assert!(!crate::verify_sqlite_rows(
                &rows,
                &sessions,
                Some(&response),
                "错误的另一条输入"
            )
            .is_empty());
        }
    }

    #[test]
    fn b_p2_blueprint_passes_real_schema_without_io() {
        let value = blueprint();
        let result =
            oclive_validation::validate_blueprint_v4_json(&value.to_string(), Some(ROLE_ID));
        assert!(result.is_ok(), "{result:?}");
        assert_eq!(
            value["runtime_config"]["inference_profile"]["generation"]["maximum_output_tokens"],
            256
        );
        assert_eq!(value["meta"]["ollama_model"], MODEL);
    }

    #[test]
    fn b_p2_mode_gate_preserves_a_and_requires_explicit_b() {
        assert!(support::validate_run_mode("A-test-run-1", support::S1, "").is_ok());
        assert!(
            support::validate_run_mode("B-test-run-1", support::B1, support::B_APPROVAL).is_ok()
        );
        for (id, scenario, approval) in [
            ("A-test-run-1", support::B1, support::B_APPROVAL),
            ("B-test-run-1", support::S1, support::B_APPROVAL),
            ("B-test-run-1", support::B1, ""),
        ] {
            assert!(support::validate_run_mode(id, scenario, approval).is_err());
        }
    }

    #[test]
    fn b_p2_structure_is_not_a_semantic_verdict() {
        let mut response = json!({"reply":"结构合法但语义可能错误","reply_is_fallback":false,"llm_fallback_reason":null,"chat_persist_failed":false});
        assert!(response_failures(&response).is_empty());
        response["reply"] = json!("");
        assert!(!response_failures(&response).is_empty());
        response["reply"] = json!("text");
        response["reply_is_fallback"] = json!(true);
        assert!(!response_failures(&response).is_empty());
    }
}
