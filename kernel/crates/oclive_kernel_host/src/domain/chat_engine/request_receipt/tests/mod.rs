#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
use super::*;
type DbManager = crate::infrastructure::db::DbManager;
use serde_json::json;
use sqlx::{sqlite::SqlitePoolOptions, Executor, SqlitePool};
use std::sync::atomic::{AtomicUsize, Ordering};

async fn fixture() -> (DbManager, SqlitePool) {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    pool.execute("CREATE TABLE chat_messages(id TEXT PRIMARY KEY); CREATE TABLE chat_sessions(session_id TEXT PRIMARY KEY);").await.unwrap();
    pool.execute(include_str!(
        "../../../../../migrations/040_chat_request_receipts.sql"
    ))
    .await
    .unwrap();
    (DbManager::new(pool.clone()), pool)
}

fn request() -> SendMessageRequest {
    SendMessageRequest {
        client_request_id: Some("ae45c9f0-5a98-4ac9-a103-112c1caa1526".into()),
        role_id: "role".into(),
        user_message: "same text".into(),
        ..Default::default()
    }
}

fn response() -> SendMessageResponse {
    serde_json::from_value(json!({"api_version":1,"schema":16,"presence_mode":"co_present",
        "relation_state":"friend","reply":"one authoritative reply","emotion":{"joy":0,"sadness":0,"anger":0,"fear":0,"surprise":0,"disgust":0,"neutral":1},
        "bot_emotion":"neutral","portrait_emotion":"neutral","favorability_delta":0,"favorability_current":50,
        "events":[],"scene_id":"default","offer_destination_picker":false,"timestamp":1})).unwrap()
}

async fn leave_running(db: &DbManager, req: &SendMessageRequest) {
    let (entered, started) = tokio::sync::oneshot::channel();
    let owner = execute(db, req, async {
        entered.send(()).unwrap();
        std::future::pending().await
    });
    tokio::pin!(owner);
    tokio::select! {
        result = &mut owner => panic!("owner unexpectedly completed: {result:?}"),
        result = started => result.unwrap(),
    }
    // Dropping the owner future after admission simulates caller cancellation.
}

#[tokio::test]
async fn completed_replay_survives_manager_recreation_and_never_polls_second_operation() {
    let (db, pool) = fixture().await;
    let req = request();
    let first = execute(&db, &req, async { Ok(response()) }).await.unwrap();
    drop(db);
    let reopened = DbManager::new(pool);
    let second = execute(&reopened, &req, async { panic!("replay must not execute") })
        .await
        .unwrap();
    assert_eq!(
        serde_json::to_value(first).unwrap(),
        serde_json::to_value(second).unwrap()
    );
}

#[tokio::test]
async fn concurrent_stream_plain_equivalent_waits_for_one_operation() {
    let (db, _) = fixture().await;
    let req = request();
    let calls = AtomicUsize::new(0);
    let (first, retry) = tokio::join!(
        execute(&db, &req, async {
            calls.fetch_add(1, Ordering::SeqCst);
            tokio::time::sleep(Duration::from_millis(30)).await;
            Ok(response())
        }),
        execute(&db, &req, async {
            calls.fetch_add(1, Ordering::SeqCst);
            Ok(response())
        }),
    );
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert_eq!(
        serde_json::to_value(first.unwrap()).unwrap(),
        serde_json::to_value(retry.unwrap()).unwrap()
    );
}

#[tokio::test]
async fn changed_payload_conflicts_even_while_pending_and_scopes_are_distinct() {
    let (db, _) = fixture().await;
    let req = request();
    leave_running(&db, &req).await;
    let mut changed = request();
    changed.user_message = "different".into();
    assert!(matches!(
        execute(&db, &changed, async { panic!("conflict must not execute") }).await,
        Err(AppError::ChatRequestConflict)
    ));
    changed.session_id = Some("another session".into());
    assert!(execute(&db, &changed, async { Ok(response()) })
        .await
        .is_ok());
}

#[tokio::test]
async fn cancelled_owner_is_never_taken_over() {
    let (db, pool) = fixture().await;
    let req = request();
    leave_running(&db, &req).await;
    let reopened = DbManager::new(pool);
    assert!(tokio::time::timeout(
        Duration::from_millis(150),
        execute(&reopened, &req, async {
            panic!("orphan must not execute again")
        })
    )
    .await
    .is_err());
}

#[tokio::test]
async fn execution_error_including_runtime_not_ready_becomes_unconfirmed_without_retry() {
    let (db, _) = fixture().await;
    let req = request();
    assert!(matches!(
        execute(&db, &req, async { Err(AppError::RoleRuntimeNotReady) }).await,
        Err(AppError::ChatRequestUnconfirmed)
    ));
    assert!(matches!(
        execute(&db, &req, async {
            panic!("failed execution must not retry")
        })
        .await,
        Err(AppError::ChatRequestUnconfirmed)
    ));
}

#[tokio::test]
async fn completion_write_failure_keeps_identity_consumed() {
    let (db, pool) = fixture().await;
    pool.execute("CREATE TRIGGER fail_completion BEFORE UPDATE ON chat_request_receipts WHEN NEW.status='completed' BEGIN SELECT RAISE(ABORT, 'injected receipt failure'); END;").await.unwrap();
    assert!(matches!(
        execute(&db, &request(), async { Ok(response()) }).await,
        Err(AppError::ChatRequestUnconfirmed)
    ));
    let status: String = sqlx::query_scalar("SELECT status FROM chat_request_receipts")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(status, "running");
}

#[tokio::test]
async fn deleted_history_cannot_be_replayed_or_reexecuted() {
    let (db, pool) = fixture().await;
    sqlx::query("INSERT INTO chat_messages VALUES ('u'), ('a')")
        .execute(&pool)
        .await
        .unwrap();
    let mut reply = response();
    reply.user_message_id = Some("u".into());
    reply.assistant_message_id = Some("a".into());
    execute(&db, &request(), async { Ok(reply) }).await.unwrap();
    sqlx::query("DELETE FROM chat_messages WHERE id='u'")
        .execute(&pool)
        .await
        .unwrap();
    assert!(matches!(
        execute(&db, &request(), async {
            panic!("deleted result must not regenerate")
        })
        .await,
        Err(AppError::ChatRequestUnconfirmed)
    ));
    let cached: Option<String> =
        sqlx::query_scalar("SELECT response_json FROM chat_request_receipts")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(cached.is_none());
}

#[tokio::test]
async fn deletion_before_receipt_commit_and_session_deletion_close_completion_races() {
    let (db, pool) = fixture().await;
    assert!(matches!(
        execute(&db, &request(), async {
            let mut reply = response();
            reply.user_message_id = Some("already-deleted".into());
            Ok(reply)
        })
        .await,
        Err(AppError::ChatRequestUnconfirmed)
    ));
    let mut req = request();
    req.client_request_id = Some(uuid::Uuid::new_v4().to_string());
    sqlx::query("INSERT INTO chat_sessions VALUES ('role')")
        .execute(&pool)
        .await
        .unwrap();
    assert!(matches!(
        execute(&db, &req, async {
            sqlx::query("DELETE FROM chat_sessions")
                .execute(&pool)
                .await
                .unwrap();
            Ok(response())
        })
        .await,
        Err(AppError::ChatRequestUnconfirmed)
    ));
}

#[tokio::test]
async fn legacy_no_id_and_new_ids_allow_intentional_identical_sends() {
    let (db, _) = fixture().await;
    let mut req = request();
    req.client_request_id = None;
    let calls = AtomicUsize::new(0);
    for _ in 0..2 {
        execute(&db, &req, async {
            calls.fetch_add(1, Ordering::SeqCst);
            Ok(response())
        })
        .await
        .unwrap();
    }
    for _ in 0..2 {
        req.client_request_id = Some(uuid::Uuid::new_v4().to_string());
        execute(&db, &req, async {
            calls.fetch_add(1, Ordering::SeqCst);
            Ok(response())
        })
        .await
        .unwrap();
    }
    assert_eq!(calls.load(Ordering::SeqCst), 4);
}

#[tokio::test]
async fn invalid_identity_is_rejected_before_admission() {
    let (db, pool) = fixture().await;
    let mut req = request();
    req.client_request_id = Some("invalid".into());
    assert!(matches!(
        execute(&db, &req, async { panic!("invalid identity") }).await,
        Err(AppError::InvalidParameter(_))
    ));
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM chat_request_receipts")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 0);
}

#[tokio::test]
async fn recovery_missing_or_missing_id_never_admits_a_request() {
    let (db, pool) = fixture().await;
    assert!(matches!(
        recover(&db, &request()).await,
        Err(AppError::ChatRequestUnconfirmed)
    ));
    let mut req = request();
    req.client_request_id = None;
    assert!(matches!(
        recover(&db, &req).await,
        Err(AppError::ChatRequestUnconfirmed)
    ));
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM chat_request_receipts")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 0);
    // A late first arrival can still execute exactly once; recovery did not claim it.
    execute(&db, &request(), async { Ok(response()) })
        .await
        .unwrap();
    assert!(recover(&db, &request()).await.is_ok());
}

#[tokio::test]
async fn recovery_waits_for_the_existing_owner_without_starting_an_operation() {
    let (db, pool) = fixture().await;
    let req = request();
    let (scope, fingerprint, namespace) =
        identity(&req, req.client_request_id.as_deref().unwrap()).unwrap();
    assert!(db
        .claim_chat_request(
            &scope,
            req.client_request_id.as_deref().unwrap(),
            &fingerprint,
            &namespace,
            (&req.role_id, "default")
        )
        .await
        .unwrap());
    let (completed, recovered) = tokio::join!(
        async {
            tokio::time::sleep(Duration::from_millis(20)).await;
            db.complete_chat_request(
                &scope,
                req.client_request_id.as_deref().unwrap(),
                &response(),
            )
            .await
        },
        recover(&db, &req)
    );
    completed.unwrap();
    assert_eq!(recovered.unwrap().reply, response().reply);
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM chat_request_receipts")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 1);
}

#[tokio::test]
async fn clearing_role_scene_or_session_invalidates_receipts_even_without_history_rows() {
    for kind in ["role", "scene", "session"] {
        let db = crate::infrastructure::test_db::mem_db_manager().await;
        let req = request();
        execute(&db, &req, async { Ok(response()) }).await.unwrap();
        let mut pending = request();
        pending.client_request_id = Some(uuid::Uuid::new_v4().to_string());
        leave_running(&db, &pending).await;
        match kind {
            "role" => db.delete_chat_data_for_manifest_role("role").await.unwrap(),
            "scene" => {
                db.delete_chat_data_for_role_scene("role", "default")
                    .await
                    .unwrap();
            }
            _ => db.delete_chat_session("role").await.unwrap(),
        }
        for req in [&req, &pending] {
            assert!(matches!(
                recover(&db, req).await,
                Err(AppError::ChatRequestUnconfirmed)
            ));
            assert!(matches!(
                execute(&db, req, async {
                    panic!("cleared identity must never execute")
                })
                .await,
                Err(AppError::ChatRequestUnconfirmed)
            ));
        }
    }
}
