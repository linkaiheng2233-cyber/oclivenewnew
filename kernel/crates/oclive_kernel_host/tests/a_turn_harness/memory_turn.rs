//! M-V1: synthetic LTM -> real Host turn -> recorded Prompt; no real model or retrieval mock.
//! Only this run's freshly migrated SQLite is seeded. Normal product decay/access writes remain.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::{Row, SqlitePool};
use std::path::Path;

use crate::support::{verify_existing_chain, MetaFacts, RealFs, ROLE_ID, SCENE_ID};

pub const M1: &str = "m1";
pub const M2: &str = "m2";
pub const HIT_RUN: &str = "A-MV1-4b7ccc66-R1-M01-R0";
pub const MISS_RUN: &str = "A-MV1-4b7ccc66-R1-M02-R0";
pub const HIT_QUERY: &str = "风筝";
pub const MISS_QUERY: &str = "星舰";
pub const REPLY: &str = "我明白了，这就来。";
const OTHER_ROLE: &str = "mv1-other-role";
const SELECTED: &str = "Alice 不喜欢红色风筝，可能下周试飞蓝色风筝。MV1_SELECTED";
const DISTRACTOR: &str = "Bob 周日练习钢琴。MV1_DISTRACTOR";
const FOREIGN: &str = "另一个角色保存的风筝资料。MV1_FOREIGN";
const MEMORY_HEADING: &str =
    "关于用户的记忆（已按相关性排序；请勿在回复中复述编号、括号或「重要性」等系统字样）:";

pub fn is_memory(scenario: &str) -> bool {
    matches!(scenario, M1 | M2)
}

pub fn validate_mode(run_id: &str, scenario: &str, approval: &str) -> Result<(), String> {
    if approval.is_empty() && matches!((run_id, scenario), (HIT_RUN, M1) | (MISS_RUN, M2)) {
        Ok(())
    } else {
        Err("M-V1 requires the exact scenario/run_id pair and no live-model approval".into())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct MemoryRow {
    id: i64,
    role_id: String,
    content: String,
    importance: f64,
    weight: f64,
    created_at: String,
    accessed_at: Option<String>,
    scene_id: Option<String>,
    mention_count: i64,
    content_scope: String,
}

// Guard both the path and the existing file before opening SQLite; never create a DB here.
// Like the existing driver, this is not a guarantee against concurrent filesystem replacement.
fn check_db_path(path: &Path, root: &Path) -> Result<(), String> {
    if path != root.join("app-data/app.db") {
        return Err("memory fixture DB is not this scenario's app-data/app.db".into());
    }
    verify_existing_chain(&RealFs, &root.join("app-data"), "memory DB parent")?;
    let metadata = std::fs::symlink_metadata(path).map_err(|e| e.to_string())?;
    let facts = MetaFacts::from_std(&metadata);
    if !facts.is_file || facts.is_reparse {
        return Err("memory fixture DB must be an existing non-reparse file".into());
    }
    let canonical_parent =
        std::fs::canonicalize(root.join("app-data")).map_err(|e| e.to_string())?;
    if std::fs::canonicalize(path).map_err(|e| e.to_string())? != canonical_parent.join("app.db") {
        return Err("memory fixture DB canonical path mismatch".into());
    }
    Ok(())
}

async fn open_db(path: &Path, root: &Path, read_only: bool) -> Result<SqlitePool, String> {
    check_db_path(path, root)?;
    SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(
            SqliteConnectOptions::new()
                .filename(path)
                .create_if_missing(false)
                .read_only(read_only),
        )
        .await
        .map_err(|e| e.to_string())
}

async fn read_rows(pool: &SqlitePool) -> Result<Value, String> {
    let rows = sqlx::query(
        "SELECT id, role_id, content, importance, weight, created_at, accessed_at, scene_id, mention_count, content_scope FROM long_term_memory ORDER BY id",
    )
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;
    let decoded: Result<Vec<MemoryRow>, sqlx::Error> = rows
        .iter()
        .map(|r| {
            Ok(MemoryRow {
                id: r.try_get("id")?,
                role_id: r.try_get("role_id")?,
                content: r.try_get("content")?,
                importance: r.try_get("importance")?,
                weight: r.try_get("weight")?,
                created_at: r.try_get("created_at")?,
                accessed_at: r.try_get("accessed_at")?,
                scene_id: r.try_get("scene_id")?,
                mention_count: r.try_get("mention_count")?,
                content_scope: r.try_get("content_scope")?,
            })
        })
        .collect();
    serde_json::to_value(decoded.map_err(|e| e.to_string())?).map_err(|e| e.to_string())
}

/// The caller has already built the kernel and loaded ROLE_ID through its real public API.
pub async fn seed(path: &Path, root: &Path) -> Result<Value, String> {
    let pool = open_db(path, root, false).await?;
    let result = async {
        let mut tx = pool.begin().await.map_err(|e| e.to_string())?;
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM long_term_memory")
            .fetch_one(&mut *tx).await.map_err(|e| e.to_string())?;
        if count != 0 {
            return Err("refusing to seed a nonempty LTM table".into());
        }
        let loaded: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM role_runtime WHERE role_id = ?")
            .bind(ROLE_ID).fetch_one(&mut *tx).await.map_err(|e| e.to_string())?;
        if loaded != 1 {
            return Err("loaded test role has no unique runtime row".into());
        }
        sqlx::query("INSERT INTO role_runtime (role_id) VALUES (?)")
            .bind(OTHER_ROLE).execute(&mut *tx).await.map_err(|e| e.to_string())?;
        // A fixed age relative to setup avoids equal timestamps on coarse clocks, without sleep.
        let now = (chrono::Utc::now() - chrono::Duration::minutes(1)).to_rfc3339();
        // A higher-strength nonmatching candidate makes the hit case distinguish new selection
        // from the old all-candidate ranking. The third row tests role scope, not semantic relevance.
        for (role_id, content, importance) in [
            (ROLE_ID, SELECTED, 0.65),
            (ROLE_ID, DISTRACTOR, 0.95),
            (OTHER_ROLE, FOREIGN, 1.0),
        ] {
            sqlx::query("INSERT INTO long_term_memory (role_id, content, importance, weight, created_at, accessed_at, scene_id, mention_count, content_scope) VALUES (?, ?, ?, 1.0, ?, ?, ?, 1, 'ordinary')")
                .bind(role_id).bind(content).bind(importance).bind(&now).bind(&now).bind(SCENE_ID)
                .execute(&mut *tx).await.map_err(|e| e.to_string())?;
        }
        tx.commit().await.map_err(|e| e.to_string())?;
        let snapshot = read_rows(&pool).await?;
        validate_seed(&snapshot)?;
        Ok(snapshot)
    }.await;
    pool.close().await;
    result
}

pub async fn snapshot(path: &Path, root: &Path) -> Result<Value, String> {
    let pool = open_db(path, root, true).await?;
    let result = read_rows(&pool).await;
    pool.close().await;
    result
}

fn validate_seed(value: &Value) -> Result<Vec<MemoryRow>, String> {
    let rows: Vec<MemoryRow> = serde_json::from_value(value.clone()).map_err(|e| e.to_string())?;
    if rows.len() != 3 {
        return Err("expected exactly three seed rows".into());
    }
    for (row, (role, content, importance)) in rows.iter().zip([
        (ROLE_ID, SELECTED, 0.65),
        (ROLE_ID, DISTRACTOR, 0.95),
        (OTHER_ROLE, FOREIGN, 1.0),
    ]) {
        if row.id <= 0
            || row.role_id != role
            || row.content != content
            || row.importance != importance
            || row.weight != 1.0
            || row.mention_count != 1
            || row.content_scope != "ordinary"
            || row.scene_id.as_deref() != Some(SCENE_ID)
            || chrono::DateTime::parse_from_rfc3339(&row.created_at).is_err()
            || row.accessed_at.as_deref() != Some(row.created_at.as_str())
        {
            return Err(format!("invalid seed row: {row:?}"));
        }
    }
    if !rows.windows(2).all(|w| w[0].id < w[1].id) {
        return Err("seed IDs are not unique/increasing".into());
    }
    Ok(rows)
}

pub fn verify_rows(scenario: &str, before: &Value, after: &Value) -> Vec<String> {
    let before = match validate_seed(before) {
        Ok(rows) => rows,
        Err(e) => return vec![e],
    };
    let after: Vec<MemoryRow> = match serde_json::from_value(after.clone()) {
        Ok(rows) => rows,
        Err(e) => return vec![format!("invalid post rows: {e}")],
    };
    if !is_memory(scenario) || after.len() != before.len() {
        return vec!["unknown memory case or unexpected LTM row count".into()];
    }
    let mut failures = Vec::new();
    for (original, actual) in before.iter().zip(&after) {
        let mut allowed = original.clone();
        if original.role_id == ROLE_ID {
            // Existing Host decay and access stamping are allowed, not Base write authority.
            allowed.weight = actual.weight;
            allowed.accessed_at.clone_from(&actual.accessed_at);
            if !actual.weight.is_finite() || actual.weight <= 0.0 || actual.weight > original.weight
            {
                failures.push(format!("invalid decay weight for ID {}", original.id));
            }
            let touched = scenario == M2 || original.content == SELECTED;
            let valid_time = actual
                .accessed_at
                .as_deref()
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                .is_some();
            if !valid_time
                || (touched && actual.accessed_at == original.accessed_at)
                || (!touched && actual.accessed_at != original.accessed_at)
            {
                failures.push(format!("access stamp mismatch for ID {}", original.id));
            }
        }
        if &allowed != actual {
            failures.push(format!(
                "seed identity/content/metadata changed for ID {}",
                original.id
            ));
        }
    }
    failures
}

pub fn verify_prompts(scenario: &str, prompts: &[Value]) -> Vec<String> {
    if !is_memory(scenario) || prompts.len() != 1 {
        return vec!["memory case requires exactly one recorded main prompt".into()];
    }
    let Some(text) = prompts[0].get("prompt").and_then(Value::as_str) else {
        return vec!["memory prompt text is missing".into()];
    };
    let mut failures = Vec::new();
    if !text.contains(SELECTED) {
        failures.push(
            "selected memory did not survive verbatim, including subject/negation/uncertainty"
                .into(),
        );
    }
    if text.contains("MV1_FOREIGN") || text.contains(OTHER_ROLE) {
        failures.push("another role's memory leaked into the prompt".into());
    }
    if scenario == M1 {
        if text.contains("MV1_DISTRACTOR") {
            failures.push("nonmatching higher-strength memory was included in the hit turn".into());
        }
        if !text.contains(MEMORY_HEADING) && !text.contains("本轮被唤起的回忆") {
            failures.push("selected evidence lacks a product memory/recollection section".into());
        }
    } else {
        // No query overlap: existing Host fallback orders the two rows by strength.
        // Expected layout is written independently, not computed with the production builder.
        let expected = format!("{MEMORY_HEADING}\n1. Bob 周日练习钢琴。MV1_DISTRACTOR\n2. Alice 不喜欢红色风筝，可能下周试飞蓝色风筝。MV1_SELECTED\n");
        if !text.contains(&expected) {
            failures.push(
                "no-hit Host fallback memory block/order differs from the frozen expectation"
                    .into(),
            );
        }
    }
    failures
}

#[cfg(test)]
mod tests {
    use super::*;

    fn seeded_rows() -> Value {
        let when = "2026-09-22T00:00:00+00:00";
        let rows: Vec<Value> = [
            (ROLE_ID, SELECTED, 0.65),
            (ROLE_ID, DISTRACTOR, 0.95),
            (OTHER_ROLE, FOREIGN, 1.0),
        ]
        .into_iter()
        .enumerate()
        .map(|(i, (role, content, importance))| {
            json!({
                "id": i + 1, "role_id": role, "content": content, "importance": importance,
                "weight": 1.0, "created_at": when, "accessed_at": when,
                "scene_id": SCENE_ID, "mention_count": 1, "content_scope": "ordinary"
            })
        })
        .collect();
        json!(rows)
    }

    fn post_rows(scenario: &str) -> Value {
        let mut rows = seeded_rows();
        rows[0]["accessed_at"] = json!("2026-09-22T00:00:01+00:00");
        rows[0]["weight"] = json!(0.999);
        rows[1]["weight"] = json!(0.999);
        if scenario == M2 {
            rows[1]["accessed_at"] = json!("2026-09-22T00:00:01+00:00");
        }
        rows
    }

    #[test]
    fn m_v1_mode_is_closed_and_scenario_specific() {
        // Independent dispatch literals catch drift between approved IDs and constants.
        assert_eq!(HIT_RUN, "A-MV1-4b7ccc66-R1-M01-R0");
        assert_eq!(MISS_RUN, "A-MV1-4b7ccc66-R1-M02-R0");
        assert!(crate::support::validate_run_mode(HIT_RUN, M1, "").is_ok());
        assert!(crate::support::validate_run_mode(MISS_RUN, M2, "").is_ok());
        for (run, scenario, approval) in [
            (HIT_RUN, M2, ""),
            (MISS_RUN, M1, ""),
            ("A-other-run-000", M1, ""),
            ("A-MV1-4b7ccc66-M01-R0", M1, ""),
            ("A-MV1-4b7ccc66-M02-R0", M2, ""),
            (HIT_RUN, M1, "live"),
        ] {
            assert!(crate::support::validate_run_mode(run, scenario, approval).is_err());
        }
    }

    #[test]
    fn m_v1_prompt_checks_accept_both_product_evidence_locations() {
        for heading in [MEMORY_HEADING, "本轮被唤起的回忆\n回忆依据："] {
            assert!(
                verify_prompts(M1, &[json!({"prompt":format!("{heading}\n{SELECTED}")})])
                    .is_empty()
            );
        }
        let fallback = format!("{MEMORY_HEADING}\n1. Bob 周日练习钢琴。MV1_DISTRACTOR\n2. Alice 不喜欢红色风筝，可能下周试飞蓝色风筝。MV1_SELECTED\n");
        assert!(verify_prompts(M2, &[json!({"prompt": fallback})]).is_empty());
    }

    #[test]
    fn m_v1_prompt_checks_reject_missing_mutated_and_unselected_content() {
        let valid = format!("{MEMORY_HEADING}\n{SELECTED}");
        for text in [
            valid.replace("不喜欢", "喜欢"),
            valid.replace("可能", "确定"),
            format!("{valid}\n{DISTRACTOR}"),
            format!("{valid}\n{FOREIGN}"),
        ] {
            assert!(!verify_prompts(M1, &[json!({"prompt":text})]).is_empty());
        }
        assert!(!verify_prompts(M1, &[]).is_empty());
        assert!(!verify_prompts(M1, &[json!({})]).is_empty());
        assert!(!verify_prompts(M2, &[json!({"prompt":valid})]).is_empty());
    }

    #[test]
    fn m_v1_row_checks_allow_only_documented_host_writes() {
        for scenario in [M1, M2] {
            assert!(verify_rows(scenario, &seeded_rows(), &post_rows(scenario)).is_empty());
        }
        assert!(!verify_rows(M1, &seeded_rows(), &seeded_rows()).is_empty());
        let mut bad = post_rows(M1);
        bad[1]["accessed_at"] = json!("2026-09-22T00:00:01+00:00");
        assert!(!verify_rows(M1, &seeded_rows(), &bad).is_empty());
    }

    #[test]
    fn m_v1_row_checks_reject_wrong_identity_content_and_foreign_writes() {
        for (index, field, value) in [
            (0, "id", json!(99)),
            (0, "content", json!("fabricated")),
            (0, "role_id", json!(OTHER_ROLE)),
            (2, "weight", json!(0.5)),
        ] {
            let mut bad = post_rows(M1);
            bad[index][field] = value;
            assert!(!verify_rows(M1, &seeded_rows(), &bad).is_empty());
        }
        assert!(!verify_rows(M1, &json!([]), &post_rows(M1)).is_empty());
        assert!(!verify_rows(M1, &seeded_rows(), &json!([])).is_empty());
    }
}
