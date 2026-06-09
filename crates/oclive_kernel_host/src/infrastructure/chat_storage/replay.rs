//! Memory replay from chat history into `long_term_memory` (merge, idempotent).

use super::shared::normalize_scene_id;
use super::store_trait::ConversationStore;
use super::types::{ReplayProgress, ReplayResult, ReplayTarget, StoredMessage};
use crate::error::{AppError, Result};
use crate::infrastructure::db::DbManager;
use crate::infrastructure::db::{merge_long_term_memory_line, MergeOutcome, TxOrPool};
use crate::infrastructure::policy_registry::{build_policy_sets_from_registry, PolicyRegistryFile};
use crate::models::{Event, EventType};
use dashmap::DashMap;
use oclive_kernel_types::PolicyContext;
use std::sync::Arc;
use std::time::{Duration, Instant};
use uuid::Uuid;

const COMPLETED_TASK_TTL: Duration = Duration::from_secs(600);

#[derive(Default)]
pub struct ReplayTaskRegistry {
    inner: DashMap<String, ReplayProgress>,
    completed_at: DashMap<String, Instant>,
}

impl ReplayTaskRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: DashMap::new(),
            completed_at: DashMap::new(),
        }
    }

    fn purge_stale_completed(&self) {
        let now = Instant::now();
        let stale: Vec<String> = self
            .completed_at
            .iter()
            .filter(|e| now.duration_since(*e.value()) > COMPLETED_TASK_TTL)
            .map(|e| e.key().clone())
            .collect();
        for id in stale {
            self.inner.remove(&id);
            self.completed_at.remove(&id);
        }
    }

    pub fn insert(&self, progress: ReplayProgress) {
        self.purge_stale_completed();
        self.inner.insert(progress.task_id.clone(), progress);
    }

    pub fn update<F>(&self, task_id: &str, f: F)
    where
        F: FnOnce(&mut ReplayProgress),
    {
        let mut mark_done = false;
        if let Some(mut entry) = self.inner.get_mut(task_id) {
            let was_done = entry.done;
            f(&mut entry);
            mark_done = !was_done && entry.done;
        }
        if mark_done {
            self.completed_at.insert(task_id.to_string(), Instant::now());
            self.purge_stale_completed();
        }
    }

    #[must_use]
    pub fn get(&self, task_id: &str) -> Option<ReplayProgress> {
        self.purge_stale_completed();
        let progress = self.inner.get(task_id).map(|v| v.clone())?;
        if progress.done {
            self.inner.remove(task_id);
            self.completed_at.remove(task_id);
        }
        Some(progress)
    }
}

fn pair_turns(messages: &[StoredMessage]) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let mut pending_user: Option<String> = None;
    for m in messages {
        if m.sender == "user" {
            pending_user = Some(m.content.clone());
        } else if m.sender == "assistant" {
            if let Some(u) = pending_user.take() {
                out.push((u, m.content.clone()));
            }
        }
    }
    out
}

async fn collect_session_ids(
    store: &Arc<dyn ConversationStore>,
    source: &str,
    target: &ReplayTarget,
) -> Result<Vec<(String, String)>> {
    let role_id = target.role_id.trim();
    let mut sessions: Vec<(String, String)> = Vec::new();
    match source {
        "session" => {
            let sid = target.session_id.as_deref().ok_or_else(|| {
                AppError::InvalidParameter("session replay requires session_id".into())
            })?;
            let scene = target
                .scene_id
                .clone()
                .unwrap_or_else(|| "default".to_string());
            sessions.push((sid.to_string(), scene));
        }
        "scene" => {
            let scene = normalize_scene_id(target.scene_id.as_deref().unwrap_or("default"));
            let rows = store.list_sessions(role_id, &scene, 500, 0).await?;
            for s in rows {
                sessions.push((s.session_id, s.scene_id));
            }
        }
        "role" => {
            let rows = store.list_sessions_by_role(role_id).await?;
            for s in rows {
                sessions.push((s.session_id, s.scene_id));
            }
        }
        other => {
            return Err(AppError::InvalidParameter(format!(
                "invalid replay source: {other}"
            )));
        }
    }
    Ok(sessions)
}

/// Run memory replay synchronously (caller may spawn).
pub async fn run_memory_replay(
    db: Arc<DbManager>,
    store: Arc<dyn ConversationStore>,
    source: &str,
    target: &ReplayTarget,
    task_id: &str,
    registry: Arc<ReplayTaskRegistry>,
) -> Result<ReplayResult> {
    let role_id = target.role_id.trim();
    let sessions = collect_session_ids(&store, source, target).await?;
    let total_turns_estimate: u32 = sessions.len() as u32 * 10;
    registry.update(task_id, |p| {
        p.total_turns = total_turns_estimate.max(1);
    });

    let policy_runtime = build_policy_sets_from_registry(PolicyRegistryFile::with_defaults());
    let memory_policy = policy_runtime.default_policy_set.memory.clone();
    let similarity = target
        .similarity_threshold
        .unwrap_or(0.6_f64)
        .clamp(0.1, 1.0);
    let mut result = ReplayResult::default();
    let mut processed = 0u32;

    for (session_id, scene_id) in sessions {
        let messages = store.fetch_messages(&session_id, u32::MAX, 0).await?;
        for (user, bot) in pair_turns(&messages) {
            processed += 1;
            let event = Event {
                event_type: EventType::Joke,
                user_emotion: String::new(),
                bot_emotion: String::new(),
            };
            let ctx = PolicyContext {
                role_id,
                user_message: user.as_str(),
                reply: bot.as_str(),
                event: &event,
                event_confidence: 0.5,
            };
            let memory_line = memory_policy.build_memory_entry(&ctx);
            if !memory_policy.should_persist(&ctx) {
                result.skipped_memories += 1;
                continue;
            }
            let importance = memory_policy.importance(&ctx);
            match merge_long_term_memory_line(
                TxOrPool::Pool(&db.pool),
                role_id,
                scene_id.as_str(),
                &memory_line,
                importance,
                similarity,
            )
            .await
            {
                Ok(MergeOutcome::New) => result.new_memories += 1,
                Ok(MergeOutcome::Updated) => result.updated_memories += 1,
                Ok(MergeOutcome::Skipped) => result.skipped_memories += 1,
                Err(e) => result.errors.push(e.to_string()),
            }
            let pct = ((processed as f64 / total_turns_estimate.max(1) as f64) * 100.0) as u8;
            registry.update(task_id, |p| {
                p.processed_turns = processed;
                p.percent = pct.min(99);
                p.new_memories = result.new_memories;
                p.updated_memories = result.updated_memories;
                p.skipped_memories = result.skipped_memories;
                p.errors = result.errors.clone();
            });
        }
    }

    result.total_turns = processed;
    registry.update(task_id, |p| {
        p.done = true;
        p.percent = 100;
        p.processed_turns = processed;
        p.new_memories = result.new_memories;
        p.updated_memories = result.updated_memories;
        p.skipped_memories = result.skipped_memories;
        p.errors = result.errors.clone();
    });
    Ok(result)
}

/// Start async replay; returns task id immediately.
pub fn spawn_memory_replay(
    db: Arc<DbManager>,
    store: Arc<dyn ConversationStore>,
    source: String,
    target: ReplayTarget,
    registry: Arc<ReplayTaskRegistry>,
) -> String {
    let task_id = Uuid::new_v4().to_string();
    registry.insert(ReplayProgress {
        task_id: task_id.clone(),
        percent: 0,
        processed_turns: 0,
        total_turns: 0,
        new_memories: 0,
        updated_memories: 0,
        skipped_memories: 0,
        done: false,
        errors: Vec::new(),
    });
    let tid = task_id.clone();
    tokio::spawn(async move {
        if let Err(e) = run_memory_replay(db, store, &source, &target, &tid, registry.clone()).await
        {
            registry.update(&tid, |p| {
                p.done = true;
                p.errors.push(e.to_string());
            });
        }
    });
    task_id
}
