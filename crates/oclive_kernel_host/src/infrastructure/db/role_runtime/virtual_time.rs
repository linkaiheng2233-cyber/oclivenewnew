//! [`DbManager`](super::super::DbManager) virtual-time fields on `role_runtime`.

#![allow(clippy::missing_errors_doc)]

use super::super::DbManager;
use crate::error::{AppError, Result};
use chrono::Utc;

impl DbManager {
    pub async fn get_virtual_time_ms(&self, role_id: &str) -> Result<Option<i64>> {
        sqlx::query_scalar::<_, i64>("SELECT virtual_time_ms FROM role_runtime WHERE role_id = ?")
            .bind(role_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))
    }

    pub async fn get_virtual_time_anchors(&self, role_id: &str) -> Result<(i64, i64, i64)> {
        let row: Option<(i64, i64, i64)> = sqlx::query_as(
            "SELECT virtual_time_anchor_real_ms, virtual_time_anchor_virtual_ms, virtual_time_ms
             FROM role_runtime WHERE role_id = ?",
        )
        .bind(role_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(row.unwrap_or((0, 0, 0)))
    }

    pub async fn set_virtual_time_anchors(
        &self,
        role_id: &str,
        anchor_real_ms: i64,
        anchor_virtual_ms: i64,
    ) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        let n = sqlx::query(
            "UPDATE role_runtime SET virtual_time_anchor_real_ms = ?, virtual_time_anchor_virtual_ms = ?, virtual_time_ms = ?, updated_at = ? WHERE role_id = ?",
        )
        .bind(anchor_real_ms)
        .bind(anchor_virtual_ms)
        .bind(anchor_virtual_ms)
        .bind(&now)
        .bind(role_id)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?
        .rows_affected();
        if n == 0 {
            return Err(AppError::RoleRuntimeNotReady);
        }
        Ok(())
    }

    pub async fn get_last_interaction_at(
        &self,
        role_id: &str,
    ) -> Result<Option<chrono::DateTime<Utc>>> {
        let raw: Option<String> =
            sqlx::query_scalar("SELECT last_interaction_at FROM role_runtime WHERE role_id = ?")
                .bind(role_id)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(raw.and_then(|s| {
            chrono::DateTime::parse_from_rfc3339(s.trim())
                .ok()
                .map(|d| d.with_timezone(&Utc))
        }))
    }

    pub async fn set_virtual_time_ms(&self, role_id: &str, ms: i64) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        let n = sqlx::query(
            "UPDATE role_runtime SET virtual_time_ms = ?, updated_at = ? WHERE role_id = ?",
        )
        .bind(ms)
        .bind(&now)
        .bind(role_id)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?
        .rows_affected();
        if n == 0 {
            return Err(AppError::RoleRuntimeNotReady);
        }
        Ok(())
    }

    pub async fn get_last_personality_evolution_virtual_ms(&self, role_id: &str) -> Result<i64> {
        let row: Option<(i64,)> = sqlx::query_as(
            "SELECT last_personality_evolution_virtual_ms FROM role_runtime WHERE role_id = ?",
        )
        .bind(role_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(row.map(|(v,)| v).unwrap_or(0))
    }

    pub async fn set_last_personality_evolution_virtual_ms(
        &self,
        role_id: &str,
        virtual_ms: i64,
    ) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        let n = sqlx::query(
            "UPDATE role_runtime SET last_personality_evolution_virtual_ms = ?, updated_at = ? WHERE role_id = ?",
        )
        .bind(virtual_ms)
        .bind(&now)
        .bind(role_id)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?
        .rows_affected();
        if n == 0 {
            return Err(AppError::RoleRuntimeNotReady);
        }
        Ok(())
    }
}
