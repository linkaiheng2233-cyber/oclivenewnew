//! Transcript identity is independent of the role's memory/relationship namespace.

use crate::error::{AppError, Result};
use crate::models::dto::SendMessageRequest;

pub(super) fn validate(req: &SendMessageRequest) -> Result<()> {
    let Some(raw) = req.chat_thread_id.as_deref() else {
        return Ok(());
    };
    validate_id(raw)?;
    if req.adult.is_some() {
        return Err(AppError::InvalidParameter(
            "transcript-only threads do not support staged adult interactions".into(),
        ));
    }
    Ok(())
}

pub(super) fn validate_id(raw: &str) -> Result<()> {
    let id = uuid::Uuid::parse_str(raw).map_err(|_| {
        AppError::InvalidParameter("chat_thread_id must be a canonical UUID".into())
    })?;
    if id.is_nil() || id.to_string() != raw {
        return Err(AppError::InvalidParameter(
            "chat_thread_id must be a canonical non-nil UUID".into(),
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
    use crate::domain::chat_engine::{process_message, process_message_stream};
    use crate::domain::host_profile::load_host_profile_file;
    use crate::state::AppState;
    use crate::state::AppStateBuilder;
    use oclive_kernel_contracts::LlmClient;
    use parking_lot::Mutex;
    use std::sync::Arc;

    #[derive(Default)]
    struct MemoryPromptProbe {
        prompts: Mutex<Vec<String>>,
        reply: Mutex<Option<String>>,
    }

    #[async_trait::async_trait]
    impl LlmClient for MemoryPromptProbe {
        async fn generate(&self, _model: &str, prompt: &str) -> crate::error::Result<String> {
            self.prompts.lock().push(prompt.to_string());
            Ok(self
                .reply
                .lock()
                .clone()
                .unwrap_or_else(|| "收到，我会根据已有的信息回答。".into()))
        }

        async fn generate_tag(&self, model: &str, prompt: &str) -> crate::error::Result<String> {
            crate::infrastructure::MockLlmClient {
                reply: "neutral".into(),
            }
            .generate_tag(model, prompt)
            .await
        }
    }

    #[tokio::test]
    async fn desktop_verified_memory_recall_corrects_plain_and_streamed_replies() {
        let temp = tempfile::tempdir().unwrap();
        let db_path = temp.path().join("app.db");
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
        let roles = oclive_kernel_runtime::chat_pro_roles_dir(std::slice::from_ref(&root)).unwrap();
        let profile = load_host_profile_file(
            &root.join("distros/desktop-tauri/resources/distro-profiles/desktop.oclive.toml"),
        )
        .unwrap();
        let probe = Arc::new(MemoryPromptProbe::default());
        let state = AppStateBuilder::in_memory_test(probe.clone(), &roles, None)
            .with_test_db_path(&db_path)
            .with_app_data_dir(temp.path())
            .with_host_profile(profile.clone())
            .build()
            .await
            .unwrap();
        let teaching_thread = uuid::Uuid::new_v4().to_string();
        for message in [
            "请记住，我的项目代号是银湾9246",
            "请记住，我常喝热的无糖玄米茶",
        ] {
            process_message(
                &state,
                &SendMessageRequest {
                    role_id: "mumu".into(),
                    user_message: message.into(),
                    scene_id: Some("home".into()),
                    chat_thread_id: Some(teaching_thread.clone()),
                    client_request_id: Some(uuid::Uuid::new_v4().to_string()),
                    ..Default::default()
                },
            )
            .await
            .unwrap();
        }
        let saved = state
            .db_manager
            .load_memories_for_context("mumu", 10, false)
            .await
            .unwrap();
        for value in ["银湾9246", "热的无糖玄米茶"] {
            assert!(
                saved.iter().any(|memory| memory.content.contains(value)),
                "{value}"
            );
        }
        drop(state);

        let state = AppStateBuilder::in_memory_test(probe.clone(), &roles, None)
            .with_test_db_path(&db_path)
            .with_app_data_dir(temp.path())
            .with_host_profile(profile)
            .build()
            .await
            .unwrap();
        let expected = "你之前告诉过我，你的项目代号是银湾9246，你常喝的饮品是热的无糖玄米茶。";
        let req = || SendMessageRequest {
            role_id: "mumu".into(),
            user_message: "你记得我的项目代号和常喝的饮品吗？".into(),
            scene_id: Some("home".into()),
            chat_thread_id: Some(uuid::Uuid::new_v4().to_string()),
            client_request_id: Some(uuid::Uuid::new_v4().to_string()),
            ..Default::default()
        };
        *probe.reply.lock() = Some("我的项目代号是银湾9246，你常喝无糖玄米茶。".into());
        let plain = process_message(&state, &req()).await.unwrap();
        assert_eq!(plain.reply, expected);
        assert!(!plain.reply_is_fallback);

        let tokens = Arc::new(Mutex::new(String::new()));
        let captured = Arc::clone(&tokens);
        let stream_req = req();
        let streamed = process_message_stream(
            &state,
            &stream_req,
            Arc::new(move |token| captured.lock().push_str(token)),
        )
        .await
        .unwrap();
        assert_eq!(streamed.reply, expected);
        assert_eq!(tokens.lock().as_str(), expected);
        assert!(!streamed.reply_is_fallback);
        let stored = state
            .conversation_store
            .fetch_messages(
                &storage_id("mumu", stream_req.chat_thread_id.as_deref()),
                10,
                0,
            )
            .await
            .unwrap();
        assert_eq!(stored.len(), 2);
        assert_eq!(stored[1].content, expected);

        let character_reply = "当然记得啦，你的项目代号是银湾9246，你常喝热的无糖玄米茶。";
        *probe.reply.lock() = Some(character_reply.into());
        let reply = process_message(&state, &req()).await.unwrap();
        assert_eq!(reply.reply, character_reply);
        assert!(probe.prompts.lock().iter().any(|prompt| {
            prompt.contains("【本轮回忆依据】") && prompt.ends_with("只输出本轮回复。")
        }));
    }

    #[tokio::test]
    async fn desktop_explicit_memory_reaches_new_transcript_prompt_after_reopen() {
        let temp = tempfile::tempdir().unwrap();
        let db_path = temp.path().join("app.db");
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
        let roles = oclive_kernel_runtime::chat_pro_roles_dir(std::slice::from_ref(&root)).unwrap();
        let profile = load_host_profile_file(
            &root.join("distros/desktop-tauri/resources/distro-profiles/desktop.oclive.toml"),
        )
        .unwrap();
        let probe = Arc::new(MemoryPromptProbe::default());
        let state = AppStateBuilder::in_memory_test(probe.clone(), &roles, None)
            .with_test_db_path(&db_path)
            .with_app_data_dir(temp.path())
            .with_host_profile(profile.clone())
            .build()
            .await
            .unwrap();
        let first_thread = uuid::Uuid::new_v4().to_string();
        for message in [
            "请记住我们的虚构验收代号是松岚7319，我偏好温的无糖麦茶。以后新对话也请记住。",
            "再确认一下，松岚7319是验收代号，我喜欢温的无糖麦茶。",
            "最后再记一下：验收代号松岚7319，饮品偏好是温的无糖麦茶。",
        ] {
            process_message(
                &state,
                &SendMessageRequest {
                    role_id: "mumu".into(),
                    user_message: message.into(),
                    scene_id: Some("home".into()),
                    chat_thread_id: Some(first_thread.clone()),
                    client_request_id: Some(uuid::Uuid::new_v4().to_string()),
                    ..Default::default()
                },
            )
            .await
            .unwrap();
        }
        let memories = state
            .db_manager
            .load_memories_for_context("mumu", 10, false)
            .await
            .unwrap();
        assert!(
            memories
                .iter()
                .any(|m| m.content.contains("松岚7319") && m.content.contains("温的无糖麦茶")),
            "an explicit remember request must survive the desktop Fast/Deep persistence gate"
        );
        // Move the facts outside the recent-context window. Otherwise STM can
        // hide a regression in LTM retrieval and prompt injection.
        for index in 0..6 {
            process_message(
                &state,
                &SendMessageRequest {
                    role_id: "mumu".into(),
                    user_message: format!("这是第{index}次普通闲聊，聊聊今天的日常。"),
                    scene_id: Some("home".into()),
                    chat_thread_id: Some(first_thread.clone()),
                    client_request_id: Some(uuid::Uuid::new_v4().to_string()),
                    ..Default::default()
                },
            )
            .await
            .unwrap();
        }
        let recent = state
            .db_manager
            .list_short_term_recent_turns("mumu", 6, false)
            .await
            .unwrap();
        assert_eq!(recent.len(), 6);
        assert!(recent
            .iter()
            .all(|(user, reply)| !user.contains("松岚7319") && !reply.contains("松岚7319")));
        drop(state);

        let reopened = AppStateBuilder::in_memory_test(probe.clone(), &roles, None)
            .with_test_db_path(&db_path)
            .with_app_data_dir(temp.path())
            .with_host_profile(profile)
            .build()
            .await
            .unwrap();
        probe.prompts.lock().clear();
        let new_thread = uuid::Uuid::new_v4().to_string();
        let query = "之前告诉你的验收代号和我偏好的饮品是什么？";
        process_message(
            &reopened,
            &SendMessageRequest {
                role_id: "mumu".into(),
                user_message: query.into(),
                scene_id: Some("home".into()),
                chat_thread_id: Some(new_thread.clone()),
                client_request_id: Some(uuid::Uuid::new_v4().to_string()),
                ..Default::default()
            },
        )
        .await
        .unwrap();
        assert!(probe.prompts.lock().iter().any(|p| p.contains(query) && p.contains("松岚7319") && p.contains("温的无糖麦茶")),
            "a fresh transcript must receive the saved user facts without being re-taught the answers");
        assert_eq!(
            reopened
                .db_manager
                .get_chat_message_count(&storage_id("mumu", Some(&first_thread)))
                .await
                .unwrap(),
            18
        );
        assert_eq!(
            reopened
                .db_manager
                .get_chat_message_count(&storage_id("mumu", Some(&new_thread)))
                .await
                .unwrap(),
            2
        );
        assert!(reopened
            .db_manager
            .get_role_runtime_snapshot(&storage_id("mumu", Some(&new_thread)))
            .await
            .unwrap()
            .is_none());
    }

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
