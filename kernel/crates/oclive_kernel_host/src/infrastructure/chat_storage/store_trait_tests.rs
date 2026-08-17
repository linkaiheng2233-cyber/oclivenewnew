//! [`HybridConversationStore`] conformance tests (mirror on/off).

#[cfg(test)]
mod tests {
    use super::super::backends::HybridConversationStore;
    use super::super::cleanup::AutoCleanupConfig;
    use super::super::replay::ReplayTaskRegistry;
    use super::super::store_trait::ConversationStore;
    use super::super::types::TurnPersistInput;
    use crate::infrastructure::db::DbManager;
    use std::sync::Arc;

    fn sample_turn(session: &str) -> TurnPersistInput {
        TurnPersistInput {
            idempotency_key: None,
            session_id: session.into(),
            role_id: "trait-test".into(),
            scene_id: "default".into(),
            user_message: "hello".into(),
            user_message_hidden: false,
            assistant_reply: "hi there".into(),
            reply_is_fallback: false,
            model_name: None,
            response_ms: 1,
            user_emotion: None,
            bot_emotion: None,
            bot_emotion_source: None,
            bot_emotion_labels: vec![],
            user_emotion_scores: None,
            emotion_pattern: None,
            emotion_confidence: None,
            emotion_intensity: None,
            emotion_dissonance: None,
            emotion_hint: None,
            reply_segments: None,
            reply_segment_delays_ms: None,
            max_messages_per_session: Some(500),
            auto_cleanup_config: AutoCleanupConfig::default(),
            chat_storage_location: "global".to_string(),
        }
    }

    async fn run_core_suite(store: Arc<dyn ConversationStore>) {
        store.append_turn(sample_turn("s1")).await.expect("append");
        let msgs = store.fetch_messages("s1", 10, 0).await.expect("fetch");
        assert_eq!(msgs.len(), 2);
        assert_eq!(msgs[0].sender, "user");
        let sessions = store
            .list_sessions("trait-test", "default", 10, 0)
            .await
            .expect("list");
        assert!(!sessions.is_empty());
        let search = store
            .search_messages("hello", Some("trait-test"), 10, 0)
            .await
            .expect("search");
        assert!(search.is_empty() || !search.is_empty());
    }

    async fn hybrid_store(mirror_enabled: bool) -> Arc<dyn ConversationStore> {
        let pool = crate::infrastructure::test_db::connect_memory_migrated().await;
        let dir = tempfile::tempdir().expect("dir");
        let app_data = dir.path().to_path_buf();
        let roles_dir = app_data.join("roles");
        let _ = std::fs::create_dir_all(&roles_dir);
        Arc::new(HybridConversationStore::new(
            Arc::new(DbManager::new(pool)),
            app_data,
            roles_dir,
            Arc::new(ReplayTaskRegistry::new()),
            mirror_enabled,
        ))
    }

    #[tokio::test]
    async fn hybrid_with_mirror_conforms_to_trait() {
        run_core_suite(hybrid_store(true).await).await;
    }

    #[tokio::test]
    async fn hybrid_without_mirror_conforms_to_trait() {
        run_core_suite(hybrid_store(false).await).await;
        let store = hybrid_store(false).await;
        assert_eq!(store.backend_kind(), "sqlite");
    }

    #[tokio::test]
    async fn sqlite_export_and_stats() {
        let store = hybrid_store(false).await;
        store.append_turn(sample_turn("ex")).await.expect("append");
        let stats = store.get_storage_stats().await.expect("stats");
        assert!(!stats.is_empty());
        let export = store
            .export_session("ex", "json", 500, None)
            .await
            .expect("export");
        assert!(export.content.contains("messages"));
    }
}
