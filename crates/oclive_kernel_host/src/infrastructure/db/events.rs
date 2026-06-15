//! [`DbManager`](super::DbManager) methods for the `events` table.

#![allow(clippy::missing_errors_doc)]

use super::{DbManager, EventListRow};
use crate::error::{AppError, Result};
use crate::models::{Event, EventType};
use chrono::Utc;

impl DbManager {
    pub async fn save_event(&self, role_id: &str, event: &Event) -> Result<String> {
        let now = Utc::now();

        let result = sqlx::query(
            "INSERT INTO events (role_id, event_type, user_emotion, bot_emotion, created_at)
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(role_id)
        .bind(event.event_type.as_ref())
        .bind(&event.user_emotion)
        .bind(&event.bot_emotion)
        .bind(now.to_rfc3339())
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(result.last_insert_rowid().to_string())
    }

    pub async fn count_events(&self, role_id: &str) -> Result<i64> {
        let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM events WHERE role_id = ?")
            .bind(role_id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(row.0)
    }

    pub async fn list_events_paged(
        &self,
        role_id: &str,
        limit: i32,
        offset: i32,
    ) -> Result<Vec<EventListRow>> {
        let rows = sqlx::query_as::<
            _,
            (
                i64,
                String,
                String,
                Option<String>,
                Option<String>,
                Option<String>,
                String,
            ),
        >(
            "SELECT id, role_id, event_type, user_emotion, bot_emotion, resolution, created_at
             FROM events
             WHERE role_id = ?
             ORDER BY created_at DESC
             LIMIT ? OFFSET ?",
        )
        .bind(role_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(rows
            .into_iter()
            .map(
                |(id, role_id, event_type, user_emotion, bot_emotion, resolution, created_at)| {
                    EventListRow {
                        id,
                        role_id,
                        event_type,
                        user_emotion,
                        bot_emotion,
                        resolution,
                        created_at,
                    }
                },
            )
            .collect())
    }

    pub async fn insert_manual_event(
        &self,
        role_id: &str,
        event_type: &EventType,
        user_emotion: &str,
        bot_emotion: &str,
        resolution: Option<&str>,
    ) -> Result<(i64, String)> {
        let now = Utc::now().to_rfc3339();
        let result = sqlx::query(
            "INSERT INTO events (role_id, event_type, user_emotion, bot_emotion, resolution, created_at)
             VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(role_id)
        .bind(event_type.as_ref())
        .bind(user_emotion)
        .bind(bot_emotion)
        .bind(resolution)
        .bind(&now)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok((result.last_insert_rowid(), now))
    }

    pub async fn get_events(&self, role_id: &str, limit: i32) -> Result<Vec<Event>> {
        let rows = sqlx::query_as::<_, (String, String, String, String)>(
            "SELECT event_type, user_emotion, bot_emotion, resolution
             FROM events
             WHERE role_id = ?
             ORDER BY created_at DESC
             LIMIT ?",
        )
        .bind(role_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let events = rows
            .into_iter()
            .map(|(event_type, user_emotion, bot_emotion, _resolution)| {
                let event_type = match event_type.as_str() {
                    "Quarrel" => EventType::Quarrel,
                    "Apology" => EventType::Apology,
                    "Praise" => EventType::Praise,
                    "Complaint" => EventType::Complaint,
                    "Confession" => EventType::Confession,
                    "Joke" => EventType::Joke,
                    "Ignore" => EventType::Ignore,
                    _ => EventType::Ignore,
                };

                Event {
                    event_type,
                    user_emotion,
                    bot_emotion,
                }
            })
            .collect();

        Ok(events)
    }
}
