//! Pure JSON file chat storage (no SQLite chat tables).

use super::super::config::{
    resolve_max_messages_per_session, resolve_role_chat_storage_root, resolve_session_dir,
    sanitize_path_segment,
};
use super::super::db::highlight_snippet;
use super::super::mirror::{self, MirrorDocument, MirrorMessage};
use super::super::replay::{run_memory_replay, ReplayTaskRegistry};
use super::super::shared::{cap_limit, normalize_scene_id};
use super::super::stats::collect_file_chat_storage_stats;
use super::super::store_trait::ConversationStore;
use super::super::types::{
    AppendTurnResult, ChatSearchResult, ReplayProgress, ReplayResult, ReplayTarget, RoleStorageStat,
    SessionMeta, StoredMessage, TurnPersistInput,
};
use crate::error::{AppError, Result};
use crate::infrastructure::db::DbManager;
use chrono::Utc;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;
use uuid::Uuid;

pub struct FileConversationStore {
    db: Arc<DbManager>,
    app_data_dir: PathBuf,
    roles_dir: PathBuf,
    storage_root: PathBuf,
    replay_tasks: Arc<ReplayTaskRegistry>,
}

impl FileConversationStore {
    #[must_use]
    pub fn new(
        db: Arc<DbManager>,
        app_data_dir: PathBuf,
        roles_dir: PathBuf,
        storage_root: PathBuf,
        replay_tasks: Arc<ReplayTaskRegistry>,
    ) -> Self {
        Self {
            db,
            app_data_dir,
            roles_dir,
            storage_root,
            replay_tasks,
        }
    }

    fn role_storage_root(&self, role_id: &str, location: Option<&str>) -> PathBuf {
        resolve_role_chat_storage_root(
            &self.app_data_dir,
            &self.roles_dir,
            role_id,
            location,
        )
    }

    fn mirror_search_roots(&self) -> Vec<PathBuf> {
        let mut roots = vec![self.storage_root.clone()];
        if let Ok(entries) = std::fs::read_dir(&self.roles_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if !path.is_dir() {
                    continue;
                }
                let role_id = entry.file_name().to_string_lossy().to_string();
                let root = self.role_storage_root(&role_id, None);
                if root != self.storage_root && !roots.iter().any(|r| r == &root) {
                    roots.push(root);
                }
            }
        }
        roots
    }

    fn clone_store(&self) -> Arc<dyn ConversationStore> {
        Arc::new(FileConversationStore {
            db: Arc::clone(&self.db),
            app_data_dir: self.app_data_dir.clone(),
            roles_dir: self.roles_dir.clone(),
            storage_root: self.storage_root.clone(),
            replay_tasks: Arc::clone(&self.replay_tasks),
        })
    }

    fn session_meta_from_doc(doc: &MirrorDocument) -> SessionMeta {
        let snippet = doc
            .messages
            .last()
            .map(|m| m.content.clone())
            .unwrap_or_default();
        SessionMeta {
            session_id: doc.session_id.clone(),
            role_id: doc.role_id.clone(),
            scene_id: doc.scene_id.clone(),
            created_at: doc.created_at.clone(),
            updated_at: doc.updated_at.clone(),
            message_count: doc.messages.len() as i64,
            last_message_snippet: snippet,
        }
    }

    async fn sessions_from_json_dir(&self, dir: &Path) -> Result<Vec<SessionMeta>> {
        let mut out = Vec::new();
        if !dir.is_dir() {
            return Ok(out);
        }
        let mut read = fs::read_dir(dir).await.map_err(AppError::IoError)?;
        while let Some(entry) = read.next_entry().await.map_err(AppError::IoError)? {
            let p = entry.path();
            if p.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }
            if let Ok(raw) = fs::read_to_string(&p).await {
                if let Ok(doc) = serde_json::from_str::<MirrorDocument>(&raw) {
                    out.push(Self::session_meta_from_doc(&doc));
                }
            }
        }
        Ok(out)
    }

    async fn search_in_role_dir(
        &self,
        role_id: &str,
        query: &str,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<ChatSearchResult>> {
        if query.is_empty() {
            return Ok(Vec::new());
        }
        let query_lower = query.to_ascii_lowercase();
        let root = self.role_storage_root(role_id, None);
        let role_seg = sanitize_path_segment(role_id)?;
        let role_dir = root.join(role_seg);
        if !role_dir.is_dir() {
            return Ok(Vec::new());
        }
        let mut hits: Vec<(String, ChatSearchResult)> = Vec::new();
        let mut read = fs::read_dir(&role_dir).await.map_err(AppError::IoError)?;
        while let Some(entry) = read.next_entry().await.map_err(AppError::IoError)? {
            let scene_dir = entry.path();
            if !scene_dir.is_dir() {
                continue;
            }
            let mut scene_read = fs::read_dir(&scene_dir).await.map_err(AppError::IoError)?;
            while let Some(file_entry) = scene_read.next_entry().await.map_err(AppError::IoError)? {
                let p = file_entry.path();
                if p.extension().and_then(|e| e.to_str()) != Some("json") {
                    continue;
                }
                let Ok(raw) = fs::read_to_string(&p).await else {
                    continue;
                };
                let Ok(doc) = serde_json::from_str::<MirrorDocument>(&raw) else {
                    continue;
                };
                for (hit_idx, m) in doc.messages.iter().enumerate() {
                    if !m.content.to_ascii_lowercase().contains(&query_lower) {
                        continue;
                    }
                    let created_at = m.timestamp.clone();
                    let (context_before, context_after) = Self::search_context_messages(&doc, hit_idx);
                    hits.push((
                        created_at.clone(),
                        ChatSearchResult {
                            session_id: doc.session_id.clone(),
                            role_id: doc.role_id.clone(),
                            scene_id: doc.scene_id.clone(),
                            highlight_snippet: highlight_snippet(&m.content, query, 40),
                            message: StoredMessage {
                                id: m.id.clone(),
                                session_id: doc.session_id.clone(),
                                turn_index: m.turn_index.unwrap_or(0),
                                sender: m.sender.clone(),
                                content: m.content.clone(),
                                metadata: m.metadata.as_ref().map(|v| v.to_string()),
                                created_at,
                            },
                            context_before,
                            context_after,
                        },
                    ));
                }
            }
        }
        hits.sort_by(|a, b| b.0.cmp(&a.0));
        let off = offset as usize;
        Ok(hits
            .into_iter()
            .skip(off)
            .take(limit as usize)
            .map(|(_, r)| r)
            .collect())
    }

    async fn load_doc(&self, path: &Path) -> Result<MirrorDocument> {
        if !path.is_file() {
            return Err(AppError::InvalidParameter("chat session file not found".into()));
        }
        let raw = fs::read_to_string(path).await.map_err(AppError::IoError)?;
        serde_json::from_str(&raw).map_err(|e| AppError::InvalidParameter(e.to_string()))
    }

    async fn find_session_in_root(&self, root: &Path, session_id: &str) -> Result<PathBuf> {
        if !root.is_dir() {
            return Err(AppError::InvalidParameter(format!(
                "chat session not found: {session_id}"
            )));
        }
        let mut stack = vec![root.to_path_buf()];
        while let Some(dir) = stack.pop() {
            let mut read = fs::read_dir(&dir).await.map_err(AppError::IoError)?;
            while let Some(entry) = read.next_entry().await.map_err(AppError::IoError)? {
                let p = entry.path();
                if p.is_dir() {
                    stack.push(p);
                } else if p.extension().and_then(|e| e.to_str()) == Some("json") {
                    if let Ok(raw) = fs::read_to_string(&p).await {
                        if raw.contains(session_id) {
                            return Ok(p);
                        }
                    }
                }
            }
        }
        Err(AppError::InvalidParameter(format!(
            "chat session not found: {session_id}"
        )))
    }

    async fn find_session_path(&self, session_id: &str) -> Result<PathBuf> {
        for root in self.mirror_search_roots() {
            if let Ok(path) = self.find_session_in_root(&root, session_id).await {
                return Ok(path);
            }
        }
        Err(AppError::InvalidParameter(format!(
            "chat session not found: {session_id}"
        )))
    }

    fn doc_to_messages(doc: &MirrorDocument) -> Vec<StoredMessage> {
        doc.messages
            .iter()
            .map(|m| Self::mirror_message_to_stored(&doc.session_id, m))
            .collect()
    }

    fn mirror_message_to_stored(session_id: &str, m: &MirrorMessage) -> StoredMessage {
        StoredMessage {
            id: m.id.clone(),
            session_id: session_id.to_string(),
            turn_index: m.turn_index.unwrap_or(0),
            sender: m.sender.clone(),
            content: m.content.clone(),
            metadata: m.metadata.as_ref().map(|v| v.to_string()),
            created_at: m.timestamp.clone(),
        }
    }

    fn search_context_messages(
        doc: &MirrorDocument,
        hit_idx: usize,
    ) -> (Vec<StoredMessage>, Vec<StoredMessage>) {
        const CONTEXT_MSGS: usize = 4;
        let before: Vec<StoredMessage> = doc.messages[..hit_idx]
            .iter()
            .rev()
            .take(CONTEXT_MSGS)
            .rev()
            .map(|m| Self::mirror_message_to_stored(&doc.session_id, m))
            .collect();
        let after: Vec<StoredMessage> = doc.messages[hit_idx + 1..]
            .iter()
            .take(CONTEXT_MSGS)
            .map(|m| Self::mirror_message_to_stored(&doc.session_id, m))
            .collect();
        (before, after)
    }
}

#[async_trait::async_trait]
impl ConversationStore for FileConversationStore {
    async fn append_turn(&self, input: TurnPersistInput) -> Result<AppendTurnResult> {
        let scene_id = normalize_scene_id(&input.scene_id);
        let max = resolve_max_messages_per_session(input.max_messages_per_session);
        let root = self.role_storage_root(&input.role_id, Some(&input.chat_storage_location));
        let dir = resolve_session_dir(&root, &input.role_id, &scene_id)?;
        fs::create_dir_all(&dir).await.map_err(AppError::IoError)?;

        let session_row = super::super::db::SessionRow {
            session_id: input.session_id.clone(),
            role_id: input.role_id.clone(),
            scene_id: scene_id.clone(),
            created_at: Utc::now().to_rfc3339(),
            updated_at: Utc::now().to_rfc3339(),
            message_count: 0,
        };
        let path = mirror::mirror_path_for_session(&root, &session_row)?;
        let mut doc = if path.is_file() {
            self.load_doc(&path).await.unwrap_or_else(|_| {
                MirrorDocument::from_session_and_rows(&session_row, &[])
            })
        } else if let Ok(existing) = self.find_session_path(&input.session_id).await {
            self.load_doc(&existing).await.unwrap_or_else(|_| {
                MirrorDocument::from_session_and_rows(&session_row, &[])
            })
        } else {
            MirrorDocument::from_session_and_rows(&session_row, &[])
        };

        let user_ts = Utc::now().to_rfc3339();
        let assistant_ts = Utc::now().to_rfc3339();
        let user_id = Uuid::new_v4().to_string();
        let assistant_id = Uuid::new_v4().to_string();
        let turn_index = (doc.messages.len() / 2) as i32;
        doc.messages.push(MirrorMessage {
            id: user_id.clone(),
            sender: "user".into(),
            content: input.user_message,
            timestamp: user_ts.clone(),
            turn_index: Some(turn_index),
            metadata: None,
        });
        doc.messages.push(MirrorMessage {
            id: assistant_id.clone(),
            sender: "assistant".into(),
            content: input.assistant_reply,
            timestamp: assistant_ts.clone(),
            turn_index: Some(turn_index),
            metadata: None,
        });
        let cap = max.max(2) as usize;
        if doc.messages.len() > cap {
            let drop_n = doc.messages.len() - cap;
            doc.messages.drain(0..drop_n);
        }
        doc.updated_at = assistant_ts.clone();
        doc.session_id = input.session_id.clone();
        doc.role_id = input.role_id.clone();
        doc.scene_id = scene_id;

        let json = serde_json::to_string_pretty(&doc)
            .map_err(|e| AppError::InvalidParameter(e.to_string()))?;
        let tmp = path.with_extension("json.tmp");
        fs::write(&tmp, json).await.map_err(AppError::IoError)?;
        fs::rename(&tmp, &path).await.map_err(AppError::IoError)?;

        Ok(AppendTurnResult {
            user_message_id: user_id,
            assistant_message_id: assistant_id,
            user_message_timestamp: user_ts,
            assistant_message_timestamp: assistant_ts,
        })
    }

    async fn list_sessions(
        &self,
        role_id: &str,
        scene_id: &str,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<SessionMeta>> {
        let scene_id = normalize_scene_id(scene_id);
        let root = self.role_storage_root(role_id, None);
        let dir = resolve_session_dir(&root, role_id, &scene_id)?;
        if !dir.is_dir() {
            return Ok(Vec::new());
        }
        let mut out = Vec::new();
        let mut read = fs::read_dir(&dir).await.map_err(AppError::IoError)?;
        while let Some(entry) = read.next_entry().await.map_err(AppError::IoError)? {
            let p = entry.path();
            if p.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }
            if let Ok(raw) = fs::read_to_string(&p).await {
                if let Ok(doc) = serde_json::from_str::<MirrorDocument>(&raw) {
                    let snippet = doc
                        .messages
                        .last()
                        .map(|m| m.content.clone())
                        .unwrap_or_default();
                    out.push(SessionMeta {
                        session_id: doc.session_id,
                        role_id: doc.role_id,
                        scene_id: doc.scene_id,
                        created_at: doc.created_at,
                        updated_at: doc.updated_at,
                        message_count: doc.messages.len() as i64,
                        last_message_snippet: snippet,
                    });
                }
            }
        }
        out.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        let off = offset as usize;
        Ok(out.into_iter().skip(off).take(limit as usize).collect())
    }

    async fn fetch_messages(
        &self,
        session_id: &str,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<StoredMessage>> {
        let path = self.find_session_path(session_id).await?;
        let doc = self.load_doc(&path).await?;
        let msgs = Self::doc_to_messages(&doc);
        let off = offset as usize;
        let cap = cap_limit(limit) as usize;
        Ok(msgs.into_iter().skip(off).take(cap).collect())
    }

    async fn list_sessions_by_role(&self, role_id: &str) -> Result<Vec<SessionMeta>> {
        let root = self.role_storage_root(role_id, None);
        let role_seg = sanitize_path_segment(role_id)?;
        let role_dir = root.join(role_seg);
        if !role_dir.is_dir() {
            return Ok(Vec::new());
        }
        let mut out = Vec::new();
        let mut read = fs::read_dir(&role_dir).await.map_err(AppError::IoError)?;
        while let Some(entry) = read.next_entry().await.map_err(AppError::IoError)? {
            let p = entry.path();
            if !p.is_dir() {
                continue;
            }
            out.extend(self.sessions_from_json_dir(&p).await?);
        }
        out.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        Ok(out.into_iter().take(500).collect())
    }

    async fn replay_memory_extraction(
        &self,
        source: &str,
        target: &ReplayTarget,
        task_id: &str,
        progress: &ReplayProgress,
    ) -> Result<ReplayResult> {
        let _ = progress;
        run_memory_replay(
            Arc::clone(&self.db),
            self.clone_store(),
            source,
            target,
            task_id,
            Arc::clone(&self.replay_tasks),
        )
        .await
    }

    async fn search_messages(
        &self,
        query: &str,
        role_id: Option<&str>,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<ChatSearchResult>> {
        let rid = match role_id {
            Some(r) if !r.trim().is_empty() => r.trim(),
            _ => return Ok(Vec::new()),
        };
        let cap = limit.clamp(1, 100);
        self.search_in_role_dir(rid, query.trim(), cap, offset).await
    }

    async fn export_session(
        &self,
        session_id: &str,
        format: &str,
        _max_messages: i64,
        role_name: Option<&str>,
    ) -> Result<super::super::types::ChatExportResponse> {
        let path = self.find_session_path(session_id).await?;
        let doc = self.load_doc(&path).await?;
        let fmt = format.trim().to_ascii_lowercase();
        match fmt.as_str() {
            "json" => {
                let content = serde_json::to_string_pretty(&doc)
                    .map_err(|e| AppError::InvalidParameter(e.to_string()))?;
                Ok(super::super::types::ChatExportResponse {
                    content,
                    suggested_filename: format!("{session_id}-chat.json"),
                    mime_type: "application/json".into(),
                    content_encoding: None,
                })
            }
            "markdown" | "md" => {
                let mut body = String::new();
                body.push_str(&format!(
                    "# Chat export — {}\n\n",
                    role_name.unwrap_or(&doc.role_id)
                ));
                for m in &doc.messages {
                    body.push_str(&format!(
                        "**{}** ({}): {}\n\n",
                        m.sender, m.timestamp, m.content
                    ));
                }
                Ok(super::super::types::ChatExportResponse {
                    content: body,
                    suggested_filename: format!("{session_id}-chat.md"),
                    mime_type: "text/markdown".into(),
                    content_encoding: None,
                })
            }
            other => Err(AppError::InvalidParameter(format!(
                "unsupported export format: {other}"
            ))),
        }
    }

    fn backend_kind(&self) -> &'static str {
        "file"
    }

    async fn get_storage_stats(&self) -> Result<Vec<RoleStorageStat>> {
        collect_file_chat_storage_stats(
            &self.app_data_dir,
            &self.roles_dir,
            &self.storage_root,
        )
        .await
    }

    async fn supports_search(&self) -> bool {
        true
    }

    async fn supports_replay(&self) -> bool {
        true
    }

    async fn supports_cleanup(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::db::DbManager;
    use crate::infrastructure::test_db;
    use crate::infrastructure::chat_storage::replay::ReplayTaskRegistry;
    use std::sync::Arc;

    async fn store() -> FileConversationStore {
        let pool = test_db::connect_memory_migrated().await;
        let app_data = tempfile::tempdir().unwrap().path().to_path_buf();
        let roles_dir = app_data.join("roles");
        let _ = std::fs::create_dir_all(&roles_dir);
        let storage_root = app_data.join("chats");
        FileConversationStore::new(
            Arc::new(DbManager::new(pool)),
            app_data,
            roles_dir,
            storage_root,
            Arc::new(ReplayTaskRegistry::new()),
        )
    }

    #[tokio::test]
    async fn append_and_fetch_turn() {
        let store = store().await;
        store
            .append_turn(TurnPersistInput {
                session_id: "sess1".into(),
                role_id: "mumu".into(),
                scene_id: "default".into(),
                user_message: "hi".into(),
                assistant_reply: "hello".into(),
                reply_is_fallback: false,
                model_name: None,
                response_ms: 0,
                user_emotion: None,
                bot_emotion: None,
                max_messages_per_session: Some(10),
                auto_cleanup_config: Default::default(),
            chat_storage_location: "global".into(),
            })
            .await
            .expect("append");
        let msgs = store.fetch_messages("sess1", 10, 0).await.expect("fetch");
        assert_eq!(msgs.len(), 2);
    }

    #[tokio::test]
    async fn list_sessions_returns_meta() {
        let store = store().await;
        store
            .append_turn(TurnPersistInput {
                session_id: "sess2".into(),
                role_id: "mumu".into(),
                scene_id: "default".into(),
                user_message: "a".into(),
                assistant_reply: "b".into(),
                reply_is_fallback: false,
                model_name: None,
                response_ms: 0,
                user_emotion: None,
                bot_emotion: None,
                max_messages_per_session: None,
                auto_cleanup_config: Default::default(),
            chat_storage_location: "global".into(),
            })
            .await
            .expect("append");
        let sessions = store.list_sessions("mumu", "default", 10, 0).await.expect("list");
        assert_eq!(sessions.len(), 1);
    }

    #[tokio::test]
    async fn fifo_truncates_old_messages() {
        let store = store().await;
        for i in 0..5 {
            store
                .append_turn(TurnPersistInput {
                    session_id: "cap".into(),
                    role_id: "r".into(),
                    scene_id: "default".into(),
                    user_message: format!("u{i}"),
                    assistant_reply: format!("a{i}"),
                    reply_is_fallback: false,
                    model_name: None,
                    response_ms: 0,
                    user_emotion: None,
                    bot_emotion: None,
                    max_messages_per_session: Some(4),
                    auto_cleanup_config: Default::default(),
            chat_storage_location: "global".into(),
                })
                .await
                .expect("append");
        }
        let msgs = store.fetch_messages("cap", 20, 0).await.expect("fetch");
        assert!(msgs.len() <= 4);
    }
}
