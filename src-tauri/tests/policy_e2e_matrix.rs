//! 端到端策略矩阵回归：scene -> profile 绑定行为验证

use oclivenewnew_tauri::domain::chat_engine::process_message;
use oclivenewnew_tauri::infrastructure::MockLlmClient;
use oclivenewnew_tauri::models::dto::SendMessageRequest;
use oclivenewnew_tauri::state::AppState;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

fn roles_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../roles")
}

fn policy_file() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("config/policy.toml")
}

struct RunMetrics {
    memory_count: i64,
    avg_memory_importance: f64,
    avg_event_confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SnapshotMetrics {
    memory_count: i64,
    avg_memory_importance: f64,
    avg_event_confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MatrixSnapshot {
    scenes: BTreeMap<String, SnapshotMetrics>,
}

async fn run_scene(scene_id: &str) -> RunMetrics {
    let llm = Arc::new(MockLlmClient {
        reply: "嗯".to_string(),
    });
    let state = AppState::new_in_memory_with_llm_and_policy_file(
        llm,
        roles_dir(),
        Some(Path::new(&policy_file())),
    )
    .await
    .expect("state");
    assert!(
        state.scene_policy_count() > 0,
        "scene policy bindings should be loaded"
    );
    assert!(
        Arc::as_ptr(&state.policies_for_scene(Some(scene_id)))
            != Arc::as_ptr(&state.policies_for_scene(None)),
        "scene binding should exist for {}",
        scene_id
    );

    let role_id = "mumu".to_string();
    let inputs = vec!["你好", "你真好", "我累了", "你骗我", "x"];
    let mut confidence_sum = 0.0_f32;
    let mut rounds = 0_u32;

    for text in inputs {
        let req = SendMessageRequest {
            role_id: role_id.clone(),
            user_message: text.to_string(),
            scene_id: Some(scene_id.to_string()),
            session_id: None,
        };
        let res = process_message(&state, &req)
            .await
            .expect("process_message");
        confidence_sum += res.events.first().map(|e| e.confidence).unwrap_or(0.0);
        rounds += 1;
    }

    let memory_count = state
        .db_manager
        .count_memories(&role_id)
        .await
        .expect("count memories");
    let memories = state
        .db_manager
        .load_memories_paged(&role_id, 100, 0)
        .await
        .expect("load memories");
    let avg_memory_importance = if memories.is_empty() {
        0.0
    } else {
        memories.iter().map(|m| m.importance).sum::<f64>() / memories.len() as f64
    };
    let avg_event_confidence = if rounds == 0 {
        0.0
    } else {
        confidence_sum / rounds as f32
    };

    RunMetrics {
        memory_count,
        avg_memory_importance,
        avg_event_confidence,
    }
}

fn snapshot_file() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/snapshots/policy_e2e_matrix.json")
}

fn to_snapshot_metrics(m: &RunMetrics) -> SnapshotMetrics {
    SnapshotMetrics {
        memory_count: m.memory_count,
        avg_memory_importance: (m.avg_memory_importance * 10_000.0).round() / 10_000.0,
        avg_event_confidence: ((m.avg_event_confidence as f64) * 10_000.0).round() / 10_000.0,
    }
}

fn load_snapshot(path: &Path) -> MatrixSnapshot {
    let raw = fs::read_to_string(path).expect("snapshot file should exist");
    serde_json::from_str::<MatrixSnapshot>(&raw).expect("snapshot json should parse")
}

fn write_snapshot(path: &Path, snapshot: &MatrixSnapshot) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create snapshot dir");
    }
    let content = serde_json::to_string_pretty(snapshot).expect("serialize snapshot");
    fs::write(path, content).expect("write snapshot");
}

fn push_diff_line(
    diffs: &mut Vec<String>,
    key: &str,
    expected: impl ToString,
    actual: impl ToString,
) {
    diffs.push(format!(
        "{} expected={} actual={}",
        key,
        expected.to_string(),
        actual.to_string()
    ));
}

#[tokio::test]
async fn policy_e2e_matrix_home_vs_school() {
    let scene_ids = vec!["home", "school", "company", "park"];
    let mut result_map: BTreeMap<String, RunMetrics> = BTreeMap::new();
    for scene in &scene_ids {
        result_map.insert((*scene).to_string(), run_scene(scene).await);
    }
    let home = result_map.get("home").expect("home metrics");
    let school = result_map.get("school").expect("school metrics");
    let company = result_map.get("company").expect("company metrics");
    let park = result_map.get("park").expect("park metrics");

    // home -> conservative: 对单字 ignore 更保守，应更少落库
    // school -> exploratory: 过滤更宽松，应更多落库
    assert!(
        school.memory_count > home.memory_count,
        "exploratory should persist more memories than conservative"
    );
    assert!(
        company.memory_count > park.memory_count,
        "exploratory(company) should persist more memories than conservative(park)"
    );

    // exploratory 默认重要度更高（0.55）且置信度加成一致，平均重要度应更高
    assert!(
        school.avg_memory_importance > home.avg_memory_importance,
        "exploratory should have higher average importance"
    );
    assert!(
        company.avg_memory_importance > park.avg_memory_importance,
        "exploratory(company) should have higher average importance than conservative(park)"
    );

    // 事件策略相同，置信度均应在合法范围
    assert!((0.0..=1.0).contains(&home.avg_event_confidence));
    assert!((0.0..=1.0).contains(&school.avg_event_confidence));

    let current = MatrixSnapshot {
        scenes: result_map
            .iter()
            .map(|(k, v)| (k.clone(), to_snapshot_metrics(v)))
            .collect(),
    };
    let path = snapshot_file();
    let should_update = std::env::var("UPDATE_POLICY_SNAPSHOTS")
        .ok()
        .map(|v| matches!(v.to_ascii_lowercase().as_str(), "1" | "true" | "yes"))
        .unwrap_or(false);
    if should_update || !path.exists() {
        write_snapshot(&path, &current);
    }
    let expected = load_snapshot(&path);
    let mut diffs: Vec<String> = Vec::new();
    for scene in &scene_ids {
        let cur = current
            .scenes
            .get(*scene)
            .expect("current snapshot scene should exist");
        let exp = expected
            .scenes
            .get(*scene)
            .expect("expected snapshot scene should exist");
        if cur.memory_count != exp.memory_count {
            push_diff_line(
                &mut diffs,
                &format!("{}.memory_count", scene),
                exp.memory_count,
                cur.memory_count,
            );
        }
        if cur.avg_memory_importance != exp.avg_memory_importance {
            push_diff_line(
                &mut diffs,
                &format!("{}.avg_memory_importance", scene),
                exp.avg_memory_importance,
                cur.avg_memory_importance,
            );
        }
        if cur.avg_event_confidence != exp.avg_event_confidence {
            push_diff_line(
                &mut diffs,
                &format!("{}.avg_event_confidence", scene),
                exp.avg_event_confidence,
                cur.avg_event_confidence,
            );
        }
    }
    if current.scenes.len() != expected.scenes.len() {
        push_diff_line(
            &mut diffs,
            "scene_count",
            expected.scenes.len(),
            current.scenes.len(),
        );
    }
    assert!(
        diffs.is_empty(),
        "policy snapshot mismatch ({} items)\n{}\nSet UPDATE_POLICY_SNAPSHOTS=1 to accept.",
        diffs.len(),
        diffs.join("\n")
    );
}
