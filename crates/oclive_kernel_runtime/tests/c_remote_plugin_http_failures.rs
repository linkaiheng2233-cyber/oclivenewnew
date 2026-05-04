//! 任务 C.1：远程插件 HTTP JSON-RPC 异常路径（超时 / HTTP 错误 / 畸形 JSON-RPC）。

use chrono::Utc;
use oclive_kernel_runtime::domain::complex_emotion::{
    default_complex_emotion_keyword_arc, ComplexEmotionInput, ComplexEmotionProvider,
};
use oclive_kernel_runtime::domain::memory_retrieval::{default_memory_slot_v1, MemoryRetrieval};
use oclive_kernel_runtime::domain::user_emotion_analyzer::UserEmotionAnalyzer;
use oclive_kernel_runtime::error::AppError;
use oclive_kernel_runtime::infrastructure::remote_plugin::{
    remote_plugin_call_async, RemoteComplexEmotionHttp, RemoteMemoryRetrievalHttp,
    RemotePluginHttpConfig, RemoteRpcChannel, RemoteUserEmotionAnalyzerHttp,
};
use oclive_kernel_runtime::models::Memory;
use serde_json::json;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread;
use std::time::Duration;

fn spawn_one_shot_http(
    status_line: &str,
    content_type: &str,
    body: &str,
) -> (String, thread::JoinHandle<()>) {
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

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn remote_jsonrpc_connect_timeout_maps_to_transport_error() {
    let (url, _h) = spawn_hanging_server();
    let cfg = RemotePluginHttpConfig {
        endpoint: url,
        timeout: Duration::from_millis(400),
        bearer_token: None,
    };
    let client = reqwest::Client::builder()
        .timeout(cfg.timeout)
        .connect_timeout(cfg.connect_timeout())
        .build()
        .unwrap();
    let err = remote_plugin_call_async(
        RemoteRpcChannel::Plugin,
        &client,
        &cfg.endpoint,
        "memory.rank",
        json!({"memories":[],"user_query":"x","limit":3}),
        None,
    )
    .await
    .expect_err("should timeout");
    let s = err.to_string();
    assert!(
        matches!(err, AppError::OllamaError(_)),
        "expected OllamaError carrier for remote transport, got {:?}",
        err
    );
    assert!(
        s.contains("remote_plugin") && (s.contains("timeout") || s.contains("kind=timeout")),
        "expected timeout classification in message, got {}",
        s
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn remote_jsonrpc_http_500_includes_status_and_body_preview() {
    let (url, h) = spawn_one_shot_http(
        "500 Internal Server Error",
        "application/json",
        r#"{"detail":"upstream boom"}"#,
    );
    let cfg = RemotePluginHttpConfig {
        endpoint: url.clone(),
        timeout: Duration::from_secs(5),
        bearer_token: None,
    };
    let client = reqwest::Client::builder()
        .timeout(cfg.timeout)
        .connect_timeout(cfg.connect_timeout())
        .build()
        .unwrap();
    let err = remote_plugin_call_async(
        RemoteRpcChannel::Plugin,
        &client,
        &cfg.endpoint,
        "memory.rank",
        json!({"memories":[],"user_query":"x","limit":3}),
        None,
    )
    .await
    .expect_err("http 500");
    let s = err.to_frontend_error();
    assert!(s.starts_with("[LLM_ERROR]"), "envelope: {}", s);
    assert!(
        s.contains("http_status") && s.contains("500"),
        "expected http_status 500 in {:?}",
        s
    );
    let _ = h.join();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn remote_jsonrpc_http_403_user_visible_envelope() {
    let (url, h) = spawn_one_shot_http("403 Forbidden", "text/plain", "no access");
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .connect_timeout(Duration::from_secs(2))
        .build()
        .unwrap();
    let err = remote_plugin_call_async(
        RemoteRpcChannel::Plugin,
        &client,
        &url,
        "emotion.analyze",
        json!({"text":"hi"}),
        None,
    )
    .await
    .expect_err("403");
    let s = err.to_frontend_error();
    assert!(s.starts_with("[LLM_ERROR]"));
    assert!(s.contains("403"), "{}", s);
    let _ = h.join();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn remote_jsonrpc_non_json_body_parse_error() {
    let (url, h) = spawn_one_shot_http("200 OK", "text/plain", "not json {");
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .connect_timeout(Duration::from_secs(2))
        .build()
        .unwrap();
    let err = remote_plugin_call_async(
        RemoteRpcChannel::Plugin,
        &client,
        &url,
        "memory.rank",
        json!({}),
        None,
    )
    .await
    .expect_err("parse");
    let msg = err.to_string();
    assert!(
        msg.contains("jsonrpc parse") || msg.contains("parse"),
        "got {}",
        msg
    );
    let _ = h.join();
}

#[test]
fn remote_memory_rank_falls_back_to_builtin_on_http_error() {
    let (url, h) = spawn_one_shot_http("502 Bad Gateway", "text/plain", "bad");
    let cfg = RemotePluginHttpConfig {
        endpoint: url,
        timeout: Duration::from_secs(5),
        bearer_token: None,
    };
    let remote = RemoteMemoryRetrievalHttp::new(cfg);
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
fn remote_emotion_analyze_falls_back_ok_on_http_error() {
    let (url, h) = spawn_one_shot_http("500 Internal Server Error", "text/plain", "x");
    let cfg = RemotePluginHttpConfig {
        endpoint: url,
        timeout: Duration::from_secs(5),
        bearer_token: None,
    };
    let r = RemoteUserEmotionAnalyzerHttp::new(cfg);
    let out = r.analyze("thanks").expect("fallback builtin");
    assert!(out.joy >= 0.0);
    let _ = h.join();
}

#[test]
fn remote_complex_emotion_marks_degraded_on_rpc_failure() {
    let (url, h) = spawn_one_shot_http("200 OK", "application/json", "{not valid json");
    let cfg = RemotePluginHttpConfig {
        endpoint: url,
        timeout: Duration::from_secs(5),
        bearer_token: None,
    };
    let remote = RemoteComplexEmotionHttp::new(cfg);
    let input = ComplexEmotionInput {
        role_id: "r".into(),
        scene_id: "s".into(),
        user_message: "都行".into(),
        bot_reply: "ok".into(),
        recent_dialogue_summary: None,
        previous_narrative_hint: String::new(),
        user_valence: None,
        user_dominance: None,
        previous_user_message: None,
    };
    let out = remote.resolve_turn(&input).expect("degraded ok");
    assert!(
        out.degraded_to_builtin,
        "expected degraded_to_builtin when remote body invalid"
    );
    let builtin = default_complex_emotion_keyword_arc();
    let base = builtin.resolve_turn(&input).expect("builtin");
    assert_eq!(out.source, base.source);
    let _ = h.join();
}
