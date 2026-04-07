//! 手动压测：chat turn 延迟分布（P50/P95/P99）
//!
//! 运行方式：
//! cargo test --test perf_chat_turns -- --ignored --nocapture

use oclivenewnew_tauri::domain::chat_engine::process_message;
use oclivenewnew_tauri::infrastructure::MockLlmClient;
use oclivenewnew_tauri::models::dto::SendMessageRequest;
use oclivenewnew_tauri::state::AppState;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

fn roles_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../roles")
}

fn percentile(sorted_ms: &[u128], p: f64) -> u128 {
    if sorted_ms.is_empty() {
        return 0;
    }
    let idx = ((sorted_ms.len() - 1) as f64 * p).round() as usize;
    sorted_ms[idx]
}

#[tokio::test]
#[ignore = "manual perf run only"]
async fn perf_chat_turn_distribution() {
    let llm = Arc::new(MockLlmClient {
        reply: "好的，我记住了。".to_string(),
    });
    let state = AppState::new_in_memory_with_llm(llm, roles_dir())
        .await
        .expect("state");

    let rounds = 200usize;
    let mut latencies = Vec::with_capacity(rounds);
    let mut failed = 0usize;

    for i in 0..rounds {
        let req = SendMessageRequest {
            role_id: "mumu".to_string(),
            user_message: format!("第{}轮对话，今天心情不错。", i),
            scene_id: None,
            session_id: None,
        };
        let started = Instant::now();
        let result = process_message(&state, &req).await;
        let elapsed = started.elapsed().as_millis();
        latencies.push(elapsed);
        if result.is_err() {
            failed += 1;
        }
    }

    latencies.sort_unstable();
    let p50 = percentile(&latencies, 0.50);
    let p95 = percentile(&latencies, 0.95);
    let p99 = percentile(&latencies, 0.99);
    let max = *latencies.last().unwrap_or(&0);

    println!(
        "perf_chat_turn_distribution rounds={} failed={} p50_ms={} p95_ms={} p99_ms={} max_ms={}",
        rounds, failed, p50, p95, p99, max
    );

    assert_eq!(failed, 0, "perf run should not have failures");
}
