//! Transcript identity is independent of the role's memory/relationship namespace.

use crate::error::{AppError, Result};
use crate::models::dto::SendMessageRequest;

pub(super) fn validate(req: &SendMessageRequest) -> Result<()> {
    let Some(raw) = req.chat_thread_id.as_deref() else {
        return Ok(());
    };
    let id = uuid::Uuid::parse_str(raw).map_err(|_| {
        AppError::InvalidParameter("chat_thread_id must be a canonical UUID".into())
    })?;
    if id.is_nil() || id.to_string() != raw {
        return Err(AppError::InvalidParameter(
            "chat_thread_id must be a canonical non-nil UUID".into(),
        ));
    }
    if req.adult.is_some() {
        return Err(AppError::InvalidParameter(
            "transcript-only threads do not support staged adult interactions".into(),
        ));
    }
    Ok(())
}

pub(crate) fn storage_id(state_namespace: &str, thread: Option<&str>) -> String {
    match thread {
        // ':' cannot occur in a validated portable role ID. UUID-first also keeps
        // the mirror's filename prefix portable and distinct across new threads.
        Some(id) => format!("{id}:chat:{state_namespace}"),
        None => state_namespace.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::chat_engine::process_message;
    use crate::state::AppState;
    use std::sync::Arc;
    #[tokio::test]
    async fn transcript_selection_survives_database_reopen_without_splitting_role_state() {
        let temp = tempfile::tempdir().unwrap();
        let db_path = temp.path().join("app.db");
        let thread = "b1a27229-4f57-4456-b37f-fbc41b026879";
        let storage_id = format!("{thread}:chat:mumu");
        let req = SendMessageRequest {
            role_id: "mumu".into(),
            user_message: "first turn".into(),
            scene_id: Some("home".into()),
            chat_thread_id: Some(thread.into()),
            client_request_id: Some(uuid::Uuid::new_v4().to_string()),
            ..Default::default()
        };
        let first = AppState::new_file_test_with_llm(
            &db_path,
            Arc::new(crate::infrastructure::MockLlmClient {
                reply: "first reply".into(),
            }),
            oclive_kernel_runtime::chat_pro_roles_dir(&[std::path::PathBuf::from(env!(
                "CARGO_MANIFEST_DIR"
            ))])
            .expect("project roles"),
            temp.path(),
        )
        .await
        .unwrap();
        let before = process_message(&first, &req).await.unwrap();
        let runtime = first
            .db_manager
            .get_role_runtime_snapshot("mumu")
            .await
            .unwrap()
            .unwrap();
        drop(first);
        let reopened = AppState::new_file_test_with_llm(
            &db_path,
            Arc::new(crate::infrastructure::MockLlmClient {
                reply: "must not regenerate".into(),
            }),
            oclive_kernel_runtime::chat_pro_roles_dir(&[std::path::PathBuf::from(env!(
                "CARGO_MANIFEST_DIR"
            ))])
            .expect("project roles"),
            temp.path(),
        )
        .await
        .unwrap();
        let replay = process_message(&reopened, &req).await.unwrap();
        assert_eq!(before.assistant_message_id, replay.assistant_message_id);
        assert_eq!(
            reopened
                .db_manager
                .get_chat_message_count(&storage_id)
                .await
                .unwrap(),
            2
        );
        let after = reopened
            .db_manager
            .get_role_runtime_snapshot("mumu")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(runtime.relation_state, after.relation_state);
        assert_eq!(runtime.favorability, after.favorability);
        assert!(reopened
            .db_manager
            .get_role_runtime_snapshot(&storage_id)
            .await
            .unwrap()
            .is_none());
    }

    use super::*;

    #[test]
    fn transcript_threads_do_not_change_memory_namespace() {
        let role = "mumu";
        let a = "b1a27229-4f57-4456-b37f-fbc41b026871";
        let b = "b1a27229-4f57-4456-b37f-fbc41b026872";
        assert_ne!(storage_id(role, Some(a)), storage_id(role, Some(b)));
        assert_eq!(storage_id(role, None), role);
        assert_ne!(storage_id(role, Some(a)), storage_id("other", Some(a)));
        assert_eq!(super::super::conversation_state_role_id(role, None), role);
    }

    #[test]
    fn malformed_thread_ids_are_rejected() {
        for value in [
            "",
            "../other",
            "00000000-0000-0000-0000-000000000000",
            "B1A27229-4F57-4456-B37F-FBC41B026871",
        ] {
            let req = SendMessageRequest {
                chat_thread_id: Some(value.into()),
                ..Default::default()
            };
            assert!(validate(&req).is_err());
        }
        assert!(validate(&SendMessageRequest::default()).is_ok());
    }
}
