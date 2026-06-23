use super::DbManager;
use crate::error::{AppError, Result};

impl DbManager {
    /// Upsert persisted narrative hint for session namespace `srid`.
    ///
    /// # Errors
    ///
    /// Returns [`AppError::DatabaseError`] on SQL failure.
    pub async fn set_complex_emotion_hint(
        &self,
        srid: &str,
        narrative_hint: &str,
        updated_at: &str,
    ) -> Result<()> {
        sqlx::query(
            "INSERT INTO complex_emotion_hint (srid, narrative_hint, updated_at) VALUES (?, ?, ?)
             ON CONFLICT(srid) DO UPDATE SET narrative_hint = excluded.narrative_hint, updated_at = excluded.updated_at",
        )
        .bind(srid)
        .bind(narrative_hint)
        .bind(updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    /// Returns `(narrative_hint, updated_at)` when a row exists.
    ///
    /// # Errors
    ///
    /// Returns [`AppError::DatabaseError`] on SQL failure.
    pub async fn get_complex_emotion_hint(&self, srid: &str) -> Result<Option<(String, String)>> {
        let row: Option<(String, String)> = sqlx::query_as(
            "SELECT narrative_hint, updated_at FROM complex_emotion_hint WHERE srid = ?",
        )
        .bind(srid)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(row)
    }

    /// Remove persisted hint for `srid`.
    ///
    /// # Errors
    ///
    /// Returns [`AppError::DatabaseError`] on SQL failure.
    pub async fn delete_complex_emotion_hint(&self, srid: &str) -> Result<()> {
        sqlx::query("DELETE FROM complex_emotion_hint WHERE srid = ?")
            .bind(srid)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(())
    }
}
