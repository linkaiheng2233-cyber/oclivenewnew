//! SQLite `SQLITE_BUSY` / `database is locked` 的有限次指数退避重试（与 WAL 配合降低冲突）。

use sqlx::Error as SqlxError;
use std::future::Future;
use std::time::Duration;

#[must_use]
pub(crate) fn sqlite_err_is_busy_or_locked(e: &SqlxError) -> bool {
    match e {
        SqlxError::Database(db) => {
            if db.code().map(|c| c == "5").unwrap_or(false) {
                return true;
            }
            let msg = db.message().to_lowercase();
            msg.contains("database is locked") || msg.contains("busy")
        }
        _ => false,
    }
}

/// 初值失败后最多再试 **5** 次（共 6 次执行），间隔 10 → 20 → 40 → 80 → 160 ms（单次休眠封顶 **250** ms）。
pub(crate) async fn with_sqlite_busy_retry<F, Fut, T>(mut op: F) -> Result<T, SqlxError>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, SqlxError>> + Send,
{
    let mut attempt = 0u8;
    loop {
        match op().await {
            Ok(v) => return Ok(v),
            Err(e) if sqlite_err_is_busy_or_locked(&e) && attempt < 5 => {
                let shift = attempt.min(4);
                let ms = (10u64 << shift).min(250);
                tokio::time::sleep(Duration::from_millis(ms)).await;
                attempt += 1;
            }
            Err(e) => return Err(e),
        }
    }
}
