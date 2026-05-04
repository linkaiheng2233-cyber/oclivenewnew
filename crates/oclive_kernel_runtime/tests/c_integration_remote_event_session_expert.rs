//! 集成补强：远程 `event.estimate` 回退、记忆侧车畸形 JSON-RPC 结果、专家图来源优先级、多会话并发读隔离。

use chrono::Utc;
use oclive_kernel_runtime::domain::expert_models_admin::{
    expert_models_get_effective, expert_models_set_role_default, expert_models_set_session_override,
};
use oclive_kernel_runtime::domain::memory_retrieval::{default_memory_slot_v1, MemoryRetrieval};
use oclive_kernel_runtime::domain::user_emotion_analyzer::UserEmotionAnalyzer;
use oclive_kernel_runtime::domain::role_lifecycle::load_role;
use oclive_kernel_runtime::domain::scene_commands::switch_scene;
use oclive_kernel_runtime::domain::chat_engine::process_message;
#[cfg(feature = "default-event-providers")]
use oclive_kernel_runtime::domain::event_estimator::BuiltinEventEstimator;
use oclive_kernel_runtime::infrastructure::llm::MockLlmClient;
#[cfg(feature = "default-event-providers")]
use oclive_kernel_runtime::infrastructure::remote_plugin::{
    RemoteEventEstimatorHttp, RemotePluginHttpConfig,
};
use oclive_kernel_runtime::infrastructure::remote_plugin::{
    remote_plugin_call_async, RemoteMemoryRetrievalHttp, RemotePluginHttpConfig as MemCfg,
    RemoteRpcChannel, RemoteUserEmotionAnalyzerHttp,
};
use oclive_kernel_runtime::models::dto::{
    ExpertModelsGetEffectiveRequest, ExpertModelsSetRoleDefaultRequest,
    ExpertModelsSetSessionOverrideRequest, SendMessageRequest, SwitchSceneRequest,
};
use oclive_kernel_runtime::models::expert_models::{ExpertConfigSource, ExpertGraph, ExpertNode};
use oclive_kernel_runtime::models::Memory;
use oclive_kernel_runtime::state::KernelAppState;
#[cfg(feature = "default-event-providers")]
use oclive_kernel_core::event_estimator::EventEstimator;
#[cfg(feature = "default-event-providers")]
use oclive_kernel_runtime::models::{Emotion, PersonalitySource, PersonalityVector};
use serde_json::json;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

fn spawn_one_shot_http(status_line: &str, content_type: &str, body: &str) -> (String, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
    let port = listener.local_addr().expect("addr").port();
    let status = status_line.to_string();
    let ct = content_type.to_string();
    let body = body.to_string();
    let h = thread::spawn(move || {
        let Ok((mut stream, _)) = listener.accept() else {
            return;
        };
        let mut buf = [0u8; 24_576];
        let _ = stream.read(&mut buf);
        let resp = format!(
            "HTTP/1.1 {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            status,
            ct,
            body.len(),
            body
        );
        let _ = stream.write_all(resp.as_bytes());
    });
    (format!("http://127.0.0.1:{}/rpc", port), h)
}

fn spawn_hanging_server() -> (String, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
    let port = listener.local_addr().expect("addr").port();
    let h = thread::spawn(move || {
        let Ok((mut stream, _)) = listener.accept() else {
            return;
        };
        let mut buf = [0u8; 24_576];
        let _ = stream.read(&mut buf);
        thread::sleep(Duration::from_secs(30));
        let _ = stream.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\n{}");
    });
    (format!("http://127.0.0.1:{}/rpc", port), h)
}

fn workspace_shimeng_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../roles/shimeng")
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let from = entry.path();
        let to = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_recursive(&from, &to)?;
        } else {
            if let Some(p) = to.parent() {
                std::fs::create_dir_all(p)?;
            }
            std::fs::copy(&from, &to)?;
        }
    }
    Ok(())
}

fn roles_dir_with_patched_shimeng_clone(role_dir_name: &str) -> tempfile::TempDir {
    let src = workspace_shimeng_dir();
    assert!(
        src.join("manifest.json").is_file(),
        "expected roles/shimeng in repo (got {:?})",
        src
    );
    let tmp = tempfile::tempdir().expect("tempdir");
    let dest = tmp.path().join(role_dir_name);
    copy_dir_recursive(&src, &dest).expect("copy shimeng tree");
    let manifest_path = dest.join("manifest.json");
    let raw = std::fs::read_to_string(&manifest_path).expect("read manifest");
    let mut v: serde_json::Value = serde_json::from_str(&raw).expect("manifest json");
    v["id"] = serde_json::Value::String(role_dir_name.to_string());
    std::fs::write(
        &manifest_path,
        serde_json::to_string_pretty(&v).expect("manifest serialize"),
    )
    .expect("write manifest");
    tmp
}

fn mock_llm() -> Arc<dyn oclive_kernel_runtime::infrastructure::llm::LlmClient> {
    Arc::new(MockLlmClient {
        reply: "c_int_ok".into(),
    })
}

fn sample_memories() -> Vec<Memory> {
    vec![Memory {
        id: "m1".into(),
        role_id: "r".into(),
        content: "alpha".into(),
        importance: 1.0,
        weight: 1.0,
        created_at: Utc::now(),
        scene_id: None,
    }]
}

#[cfg(feature = "default-event-providers")]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn remote_event_estimate_falls_back_to_builtin_on_http_502() {
    struct EnvUnsetGuard {
        key: &'static str,
        prev: Option<String>,
    }
    impl EnvUnsetGuard {
        fn set(key: &'static str, value: &str) -> Self {
            let prev = std::env::var(key).ok();
            std::env::set_var(key, value);
            Self { key, prev }
        }
    }
    impl Drop for EnvUnsetGuard {
        fn drop(&mut self) {
            match &self.prev {
                Some(v) => std::env::set_var(self.key, v),
                None => std::env::remove_var(self.key),
            }
        }
    }
    let _env = EnvUnsetGuard::set("OCLIVE_EVENT_IMPACT_LLM", "0");

    let (url, h) = spawn_one_shot_http("502 Bad Gateway", "text/plain", "bad");
    let remote = RemoteEventEstimatorHttp::new(RemotePluginHttpConfig {
        endpoint: url,
        timeout: Duration::from_secs(5),
        bearer_token: None,
    });
    let llm: Arc<dyn oclive_kernel_runtime::infrastructure::llm::LlmClient> =
        Arc::new(MockLlmClient {
            reply: String::new(),
        });
    let p = PersonalityVector::zero();
    let user_emotion = Emotion::Neutral;
    let recent_turns: &[(String, String)] = &[];
    let recent_events: &[oclive_kernel_runtime::models::Event] = &[];
    let r = remote
        .estimate(
            &llm,
            "m",
            "ping",
            &user_emotion,
            &p,
            PersonalitySource::Vector,
            recent_turns,
            recent_events,
            None,
        )
        .await
        .expect("remote fallback ok");
    let b = BuiltinEventEstimator
        .estimate(
            &llm,
            "m",
            "ping",
            &user_emotion,
            &p,
            PersonalitySource::Vector,
            recent_turns,
            recent_events,
            None,
        )
        .await
        .expect("builtin");
    assert_eq!(r.event_type, b.event_type);
    assert!((r.impact_factor - b.impact_factor).abs() < 1e-9);
    let _ = h.join();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn remote_emotion_analyze_jsonrpc_timeout_classified_like_memory() {
    let (url, _h) = spawn_hanging_server();
    let client = reqwest::Client::builder()
        .timeout(Duration::from_millis(400))
        .connect_timeout(Duration::from_secs(2))
        .build()
        .unwrap();
    let err = remote_plugin_call_async(
        RemoteRpcChannel::Plugin,
        &client,
        &url,
        "emotion.analyze",
        json!({"text": "hi"}),
        None,
    )
    .await
    .expect_err("timeout");
    let s = err.to_string();
    assert!(
        s.contains("remote_plugin") && (s.contains("timeout") || s.contains("kind=timeout")),
        "{}",
        s
    );
}

#[test]
fn remote_memory_rank_falls_back_when_jsonrpc_ok_but_result_shape_invalid() {
    let (url, h) = spawn_one_shot_http(
        "200 OK",
        "application/json",
        r#"{"jsonrpc":"2.0","id":1,"result":{"ordered_ids":"not_an_array"}}"#,
    );
    let remote = RemoteMemoryRetrievalHttp::new(MemCfg {
        endpoint: url,
        timeout: Duration::from_secs(5),
        bearer_token: None,
    });
    let builtin = default_memory_slot_v1();
    let memories = sample_memories();
    let input_a = oclive_kernel_runtime::domain::memory_retrieval::MemoryRetrievalInput {
        memories: &memories,
        user_query: "alpha",
        scene_id: None,
        limit: 4,
    };
    let input_b = oclive_kernel_runtime::domain::memory_retrieval::MemoryRetrievalInput {
        memories: &memories,
        user_query: "alpha",
        scene_id: None,
        limit: 4,
    };
    let a = remote.rank_memories(input_a);
    let b = MemoryRetrieval::rank_memories(builtin.as_ref(), input_b);
    assert_eq!(a.len(), b.len());
    assert_eq!(a[0].id, b[0].id);
    let _ = h.join();
}

#[test]
fn remote_user_emotion_http_survives_jsonrpc_error_field() {
    let (url, h) = spawn_one_shot_http(
        "200 OK",
        "application/json",
        r#"{"jsonrpc":"2.0","id":1,"error":{"code":-32603,"message":"boom"}}"#,
    );
    let r = RemoteUserEmotionAnalyzerHttp::new(MemCfg {
        endpoint: url,
        timeout: Duration::from_secs(5),
        bearer_token: None,
    });
    let out = r.analyze("hello").expect("builtin fallback");
    assert!(out.joy >= 0.0);
    let _ = h.join();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn expert_effective_is_pack_default_when_no_db_overrides() {
    let tmp = roles_dir_with_patched_shimeng_clone("c_int_pack_def");
    let roles_root = tmp.path().to_path_buf();
    let state = KernelAppState::new_in_memory_with_llm(mock_llm(), roles_root)
        .await
        .expect("state");
    load_role(&state, "c_int_pack_def", false)
        .await
        .expect("load");

    let eff = expert_models_get_effective(
        &state,
        &ExpertModelsGetEffectiveRequest {
            role_id: "c_int_pack_def".into(),
            session_id: None,
        },
    )
    .await
    .expect("get effective");
    assert_eq!(eff.graph_source, ExpertConfigSource::PackDefault);
    assert_eq!(eff.graph.version, 1);
    assert!(eff.graph.nodes.is_empty());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn expert_effective_role_default_beats_pack_when_session_unset() {
    let tmp = roles_dir_with_patched_shimeng_clone("c_int_role_pri");
    let roles_root = tmp.path().to_path_buf();
    let state = KernelAppState::new_in_memory_with_llm(mock_llm(), roles_root)
        .await
        .expect("state");
    load_role(&state, "c_int_role_pri", false)
        .await
        .expect("load");

    let g = ExpertGraph {
        version: 42,
        nodes: vec![ExpertNode::BaseModel {
            id: "b".into(),
            gguf_path: "/tmp/only_for_source_test.gguf".into(),
            ui: None,
        }],
        edges: vec![],
    };
    expert_models_set_role_default(
        &state,
        &ExpertModelsSetRoleDefaultRequest {
            role_id: "c_int_role_pri".into(),
            graph: g.clone(),
            prompt_style: None,
        },
    )
    .await
    .expect("set role default");

    let eff = expert_models_get_effective(
        &state,
        &ExpertModelsGetEffectiveRequest {
            role_id: "c_int_role_pri".into(),
            session_id: None,
        },
    )
    .await
    .expect("get effective");
    assert_eq!(eff.graph_source, ExpertConfigSource::RoleDefault);
    assert_eq!(eff.graph.version, 42);

    expert_models_set_session_override(
        &state,
        &ExpertModelsSetSessionOverrideRequest {
            role_id: "c_int_role_pri".into(),
            session_id: Some("s1".into()),
            graph: ExpertGraph {
                version: 99,
                ..Default::default()
            },
            prompt_style: None,
        },
    )
    .await
    .expect("set session override");

    let eff_s1 = expert_models_get_effective(
        &state,
        &ExpertModelsGetEffectiveRequest {
            role_id: "c_int_role_pri".into(),
            session_id: Some("s1".into()),
        },
    )
    .await
    .expect("get effective s1");
    assert_eq!(eff_s1.graph_source, ExpertConfigSource::SessionOverride);
    assert_eq!(eff_s1.graph.version, 99);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn expert_effective_concurrent_reads_keep_session_isolation() {
    let tmp = roles_dir_with_patched_shimeng_clone("c_int_conc");
    let roles_root = tmp.path().to_path_buf();
    let state = KernelAppState::new_in_memory_with_llm(mock_llm(), roles_root)
        .await
        .expect("state");
    load_role(&state, "c_int_conc", false)
        .await
        .expect("load");

    expert_models_set_session_override(
        &state,
        &ExpertModelsSetSessionOverrideRequest {
            role_id: "c_int_conc".into(),
            session_id: Some("ax".into()),
            graph: ExpertGraph {
                version: 10,
                ..Default::default()
            },
            prompt_style: None,
        },
    )
    .await
    .expect("sess ax");
    expert_models_set_session_override(
        &state,
        &ExpertModelsSetSessionOverrideRequest {
            role_id: "c_int_conc".into(),
            session_id: Some("by".into()),
            graph: ExpertGraph {
                version: 20,
                ..Default::default()
            },
            prompt_style: None,
        },
    )
    .await
    .expect("sess by");

    let st = Arc::new(state);
    let s1 = st.clone();
    let s2 = st.clone();
    let req_ax = ExpertModelsGetEffectiveRequest {
        role_id: "c_int_conc".into(),
        session_id: Some("ax".into()),
    };
    let req_by = ExpertModelsGetEffectiveRequest {
        role_id: "c_int_conc".into(),
        session_id: Some("by".into()),
    };
    let (a, b) = tokio::join!(
        expert_models_get_effective(s1.as_ref(), &req_ax),
        expert_models_get_effective(s2.as_ref(), &req_by),
    );
    let ea = a.expect("a");
    let eb = b.expect("b");
    assert_eq!(ea.graph.version, 10);
    assert_eq!(eb.graph.version, 20);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn switch_scene_together_then_message_keeps_scene_in_reply() {
    let tmp = roles_dir_with_patched_shimeng_clone("c_int_scene_msg");
    let roles_root = tmp.path().to_path_buf();
    let state = KernelAppState::new_in_memory_with_llm(mock_llm(), roles_root)
        .await
        .expect("state");
    load_role(&state, "c_int_scene_msg", false)
        .await
        .expect("load");

    switch_scene(
        &state,
        &SwitchSceneRequest {
            role_id: "c_int_scene_msg".into(),
            scene_id: "school".into(),
            together: true,
        },
    )
    .await
    .expect("switch");

    let res = process_message(
        &state,
        &SendMessageRequest {
            role_id: "c_int_scene_msg".into(),
            user_message: "课间聊聊".into(),
            scene_id: Some("school".into()),
            session_id: Some("sess_scene".into()),
        },
    )
    .await
    .expect("msg");

    assert_eq!(res.scene_id, "school");
}
