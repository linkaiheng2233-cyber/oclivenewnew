//! Durable admission receipts for ordinary user chat, before turn side effects.
use super::DbManager;
use crate::domain::ports::chat_request::{ChatRequestStore, Receipt};
use crate::error::{AppError, Result};
use crate::models::dto::SendMessageResponse;
use sqlx::Row;

#[async_trait::async_trait]
impl ChatRequestStore for DbManager {
    /// True exclusively for the caller whose atomic INSERT acquired admission.
    async fn claim_chat_request(
        &self,
        scope: &str,
        id: &str,
        fingerprint: &str,
        session_namespace: &str,
        role_scene: (&str, &str),
    ) -> Result<bool> {
        let result = sqlx::query(
            "INSERT INTO chat_request_receipts
             (scope_key, request_id, payload_sha256, session_namespace, role_id, scene_id, status)
             VALUES (?, ?, ?, ?, ?, ?, 'running') ON CONFLICT(scope_key, request_id) DO NOTHING",
        )
        .bind(scope)
        .bind(id)
        .bind(fingerprint)
        .bind(session_namespace)
        .bind(role_scene.0)
        .bind(role_scene.1)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(result.rows_affected() == 1)
    }

    async fn read_chat_request(&self, scope: &str, id: &str, fingerprint: &str) -> Result<Receipt> {
        let row = sqlx::query("SELECT payload_sha256, status, response_json FROM chat_request_receipts WHERE scope_key = ? AND request_id = ?")
            .bind(scope).bind(id).fetch_optional(&self.pool).await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?
            .ok_or(AppError::ChatRequestUnconfirmed)?;
        if row.get::<String, _>("payload_sha256") != fingerprint {
            return Err(AppError::ChatRequestConflict);
        }
        Ok(match row.get::<String, _>("status").as_str() {
            "running" => Receipt::Running,
            "completed" => row
                .get::<Option<String>, _>("response_json")
                .map_or(Receipt::Unconfirmed, Receipt::Completed),
            _ => Receipt::Unconfirmed,
        })
    }

    async fn complete_chat_request(
        &self,
        scope: &str,
        id: &str,
        response: &SendMessageResponse,
    ) -> Result<()> {
        let json = serde_json::to_string(response)?;
        // The existence predicates close the window between chat append, history
        // deletion, and receipt completion. UPDATE and predicates are one statement.
        let result = sqlx::query(
            "UPDATE chat_request_receipts SET status = 'completed', response_json = ?,
             user_message_id = ?, assistant_message_id = ?, scene_id = ?
             WHERE scope_key = ? AND request_id = ? AND status = 'running'
             AND (? IS NULL OR EXISTS (SELECT 1 FROM chat_messages WHERE id = ?))
             AND (? IS NULL OR EXISTS (SELECT 1 FROM chat_messages WHERE id = ?))",
        )
        .bind(json)
        .bind(&response.user_message_id)
        .bind(&response.assistant_message_id)
        .bind(&response.scene_id)
        .bind(scope)
        .bind(id)
        .bind(&response.user_message_id)
        .bind(&response.user_message_id)
        .bind(&response.assistant_message_id)
        .bind(&response.assistant_message_id)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        if result.rows_affected() != 1 {
            return Err(AppError::ChatRequestUnconfirmed);
        }
        Ok(())
    }

    async fn mark_chat_request_unconfirmed(&self, scope: &str, id: &str) -> Result<()> {
        sqlx::query("UPDATE chat_request_receipts SET status = 'unconfirmed', response_json = NULL WHERE scope_key = ? AND request_id = ? AND status = 'running'")
            .bind(scope).bind(id).execute(&self.pool).await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(())
    }
}
