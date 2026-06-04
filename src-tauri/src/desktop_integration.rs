//! Tauri-desktop-only hooks (plugin FS watcher, chat auto-cleanup scheduler).

use oclive_kernel_host::infrastructure::chat_storage::run_global_auto_cleanup;
use oclive_kernel_host::infrastructure::directory_plugins::{
    plugin_scan_container_roots, HostPluginsFile,
};
use oclive_kernel_host::state::{AppState, SharedAppState};
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use serde_json::json;
use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::time::Duration;
use tauri::{AppHandle, Manager};

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
    let Some(state) = app.try_state::<SharedAppState>() else {
        return;
    };
    run_global_auto_cleanup(state.inner()).await;
}

/// Watch plugin container dirs in developer mode; debounced rescan + `plugin:changed` emit.
pub fn start_plugin_fs_watcher(app: tauri::AppHandle, state: &AppState, roles_dir: PathBuf) {
    let app_data = state.directory_plugins.app_data_dir().to_path_buf();
    let host: HostPluginsFile = state.directory_plugins.host().clone();
    if !host.developer_effective() {
        return;
    }
    let roots = plugin_scan_container_roots(&roles_dir, &app_data, &host);
    if roots.is_empty() {
        tracing::info!(
            target: "oclive_plugin",
            "plugin fs watcher: no plugin container directories"
        );
        return;
    }

    let (tx, rx) = channel::<()>();
    let mut watcher = match RecommendedWatcher::new(
        move |res: Result<notify::Event, notify::Error>| {
            if res.is_ok() {
                let _ = tx.send(());
            }
        },
        Config::default(),
    ) {
        Ok(w) => w,
        Err(e) => {
            tracing::warn!(
                target: "oclive_plugin",
                "plugin fs watcher: create failed: {e}"
            );
            return;
        }
    };

    for r in &roots {
        if let Err(e) = watcher.watch(r, RecursiveMode::Recursive) {
            tracing::warn!(
                target: "oclive_plugin",
                "plugin fs watcher: watch {:?}: {e}",
                r
            );
        }
    }

    let runtime = state.directory_plugins.clone();
    let roles_for_rescan = roles_dir.clone();
    let app_emit = app.clone();
    let n_roots = roots.len();
    std::thread::spawn(move || {
        let _keep = watcher;
        while let Ok(()) = rx.recv() {
            std::thread::sleep(Duration::from_millis(500));
            while rx.try_recv().is_ok() {}

            runtime.rescan_plugin_roots(roles_for_rescan.as_path());
            let _ = app_emit.emit_all(
                "plugin:changed",
                json!({ "source": "fs", "containerRoots": n_roots }),
            );
        }
    });
}
