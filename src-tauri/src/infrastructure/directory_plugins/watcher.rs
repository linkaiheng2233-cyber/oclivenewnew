//! 开发者模式：监听插件容器目录，防抖后重新扫描并发出 `plugin:changed`。

use super::runtime::{plugin_scan_container_roots, HostPluginsFile};
use crate::state::AppState;
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use serde_json::json;
use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::time::Duration;
use tauri::Manager;

/// 在独立线程中监听 `plugin_scan_container_roots` 返回的目录；失败仅打日志。
pub fn start_plugin_fs_watcher(app: tauri::AppHandle, state: &AppState, roles_dir: PathBuf) {
    let app_data = state.directory_plugins.app_data_dir().to_path_buf();
    let host: HostPluginsFile = state.directory_plugins.host().clone();
    if !host.developer_effective() {
        return;
    }
    let roots = plugin_scan_container_roots(&roles_dir, &app_data, &host);
    if roots.is_empty() {
        log::info!(
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
            log::warn!(
                target: "oclive_plugin",
                "plugin fs watcher: create failed: {}",
                e
            );
            return;
        }
    };

    for r in &roots {
        if let Err(e) = watcher.watch(r, RecursiveMode::Recursive) {
            log::warn!(
                target: "oclive_plugin",
                "plugin fs watcher: watch {:?}: {}",
                r,
                e
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
