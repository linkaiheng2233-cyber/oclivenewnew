//! Background auto-cleanup: run at startup and every 24 hours.

use super::cleanup::AutoCleanupConfig;
use crate::state::AppState;
use std::time::Duration;
use tauri::{AppHandle, Manager};
use tracing::{info, warn};

const CLEANUP_INTERVAL: Duration = Duration::from_secs(24 * 60 * 60);

/// Spawn startup + periodic auto-cleanup for all roles with policy enabled.
pub fn spawn_auto_cleanup_scheduler(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        run_if_ready(&app).await;
        loop {
            tokio::time::sleep(CLEANUP_INTERVAL).await;
            run_if_ready(&app).await;
        }
    });
}

async fn run_if_ready(app: &AppHandle) {
    let Some(state) = app.try_state::<AppState>() else {
        return;
    };
    run_global_auto_cleanup(state.inner()).await;
}

async fn run_global_auto_cleanup(state: &AppState) {
    let role_ids = match state.db_manager.list_distinct_chat_role_ids().await {
        Ok(ids) => ids,
        Err(e) => {
            warn!(target: "oclive_chat_storage", error = %e, "list_distinct_chat_role_ids failed");
            return;
        }
    };
    for role_id in role_ids {
        let cfg = match state.load_role_cached_async(&role_id).await {
            Ok(role) => AutoCleanupConfig::from_role_config(&role.pack_chat_storage_config),
            Err(_) => AutoCleanupConfig::default(),
        };
        if !cfg.is_enabled() {
            continue;
        }
        match state
            .conversation_store
            .apply_auto_cleanup(&role_id, &cfg)
            .await
        {
            Ok(res) if res.sessions_deleted > 0 => {
                info!(
                    target: "oclive_chat_storage",
                    role_id = %role_id,
                    sessions_deleted = res.sessions_deleted,
                    bytes_freed = res.bytes_freed,
                    "scheduled auto_cleanup"
                );
            }
            Ok(_) => {}
            Err(e) => {
                warn!(
                    target: "oclive_chat_storage",
                    role_id = %role_id,
                    error = %e,
                    "scheduled auto_cleanup failed"
                );
            }
        }
    }
}
