//! One process-local queue per canonical mirror root, shared by all store instances.

use crate::error::{AppError, Result};
use parking_lot::Mutex;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock, Weak};
use tokio::sync::{Mutex as AsyncMutex, OwnedMutexGuard};

pub(crate) async fn lock_root(root: &Path) -> Result<OwnedMutexGuard<()>> {
    tokio::fs::create_dir_all(root).await?;
    let canonical = tokio::fs::canonicalize(root).await?;
    static QUEUES: OnceLock<Mutex<HashMap<PathBuf, Weak<AsyncMutex<()>>>>> = OnceLock::new();
    let lock = {
        let mut queues = QUEUES.get_or_init(Mutex::default).lock();
        queues.retain(|_, weak| weak.strong_count() > 0);
        match queues.get(&canonical).and_then(Weak::upgrade) {
            Some(lock) => lock,
            None => {
                let lock = Arc::new(AsyncMutex::new(()));
                queues.insert(canonical, Arc::downgrade(&lock));
                lock
            }
        }
    };
    Ok(lock.lock_owned().await)
}

/// The blocking worker retains the queue even if the async caller is cancelled.
pub(super) async fn run_locked_io(
    guard: OwnedMutexGuard<()>,
    operation: impl FnOnce() -> Result<()> + Send + 'static,
) -> Result<()> {
    tokio::task::spawn_blocking(move || {
        let _guard = guard;
        operation()
    })
    .await
    .map_err(|error| AppError::InvalidParameter(format!("chat mirror IO task failed: {error}")))?
}

pub(super) async fn delete_tree(root: &Path, tree: PathBuf) -> Result<()> {
    if !root.try_exists()? {
        return Ok(());
    }
    let guard = lock_root(root).await?;
    run_locked_io(guard, move || match std::fs::remove_dir_all(&tree) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.into()),
    })
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn cancelled_caller_keeps_queue_until_blocking_write_finishes() {
        let root = tempfile::tempdir().unwrap();
        let guard = lock_root(root.path()).await.unwrap();
        let (started, start) = tokio::sync::oneshot::channel();
        let (release, wait) = std::sync::mpsc::channel();
        let path = root.path().join("finished.txt");
        let job = tokio::spawn(run_locked_io(guard, move || {
            let _ = started.send(());
            wait.recv().unwrap();
            std::fs::write(path, "finished")?;
            Ok(())
        }));
        start.await.unwrap();
        job.abort();
        assert!(
            tokio::time::timeout(Duration::from_millis(20), lock_root(&root.path().join(".")))
                .await
                .is_err()
        );
        // Another root is not held by the first root's IO queue.
        let other = tempfile::tempdir().unwrap();
        drop(
            tokio::time::timeout(Duration::from_secs(2), lock_root(other.path()))
                .await
                .unwrap()
                .unwrap(),
        );
        release.send(()).unwrap();
        let _guard = tokio::time::timeout(Duration::from_secs(2), lock_root(root.path()))
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            std::fs::read(root.path().join("finished.txt")).unwrap(),
            b"finished"
        );
    }
}
