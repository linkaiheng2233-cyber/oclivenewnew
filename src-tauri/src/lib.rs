#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]

pub mod api;
pub mod domain;
pub mod env_flags;

/// Host error bridge: core types live in `oclive_kernel_runtime`.
pub mod error {
    pub use oclive_kernel_runtime::error::*;

    /// Map kernel [`AppError`] to Tauri invoke failure (orphan-safe helper).
    #[must_use]
    pub fn to_invoke_error(err: AppError) -> tauri::InvokeError {
        tauri::InvokeError::from(err.to_kernel_json())
    }
}

pub mod http_api;
pub mod infrastructure;
pub mod models;
pub mod state;
pub mod utils;

/// Initialize `tracing` (stdout; optional rolling file when `log_dir` or `OCLIVE_LOG_DIR` is set).
/// When `OCLIVE_LOG_FORMAT=json`, stdout/file use JSON lines.
///
/// Returns a file-appender guard when rolling logs are enabled; keep it alive for the process lifetime.
#[must_use]
pub fn init_tracing() -> Option<tracing_appender::non_blocking::WorkerGuard> {
    let log_dir = std::env::var("OCLIVE_LOG_DIR")
        .ok()
        .map(PathBuf::from)
        .filter(|p| !p.as_os_str().is_empty());
    init_tracing_with_log_dir(log_dir.as_deref())
}

/// Like [`init_tracing`] but always writes to `log_dir/logs/` when `Some`.
#[must_use]
pub fn init_tracing_with_log_dir(
    log_dir: Option<&Path>,
) -> Option<tracing_appender::non_blocking::WorkerGuard> {
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;
    use tracing_subscriber::{EnvFilter, Layer};

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let use_json = std::env::var("OCLIVE_LOG_FORMAT")
        .map(|v| v.eq_ignore_ascii_case("json"))
        .unwrap_or(false);

    let stdout_layer = if use_json {
        tracing_subscriber::fmt::layer()
            .json()
            .with_target(true)
            .boxed()
    } else {
        tracing_subscriber::fmt::layer().with_target(true).boxed()
    };

    let mut layers = vec![stdout_layer];
    let mut file_guard = None;

    if let Some(dir) = log_dir {
        let logs = dir.join("logs");
        if std::fs::create_dir_all(&logs).is_ok() {
            let Ok(file_appender) = tracing_appender::rolling::RollingFileAppender::builder()
                .rotation(tracing_appender::rolling::Rotation::DAILY)
                .filename_prefix("oclive")
                .filename_suffix("log")
                .max_log_files(7)
                .build(&logs)
            else {
                eprintln!("failed to build rolling log appender");
                return file_guard;
            };
            let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
            file_guard = Some(guard);
            let file_layer = if use_json {
                tracing_subscriber::fmt::layer()
                    .json()
                    .with_target(true)
                    .with_writer(non_blocking)
                    .boxed()
            } else {
                tracing_subscriber::fmt::layer()
                    .with_target(true)
                    .with_writer(non_blocking)
                    .boxed()
            };
            layers.push(file_layer);
        }
    }

    if let Err(e) = tracing_subscriber::registry()
        .with(filter)
        .with(layers)
        .try_init()
    {
        eprintln!("tracing subscriber init failed: {e}");
    }

    file_guard
}

fn sanitize_plugin_id_for_log(plugin_id: &str) -> String {
    plugin_id
        .chars()
        .filter(|c| !c.is_control())
        .take(64)
        .collect()
}

use std::fs;
use std::path::{Path, PathBuf};
use tauri::http::{Request, Response, ResponseBuilder};
use tauri::{AppHandle, Manager};

use crate::infrastructure::deep_link::seed_pending_install_urls_from_args;
use crate::infrastructure::directory_plugins::{resolve_plugin_asset_path, start_plugin_fs_watcher};
use crate::infrastructure::plugin_protocol::{
    inject_plugin_bridge_script, mime_for_plugin_asset, plugin_asset_from_request_uri,
};
use crate::state::AppState;

fn serve_ocliveplugin_asset(
    app: &AppHandle,
    request: &Request,
) -> Result<Response, Box<dyn std::error::Error>> {
    let state = app
        .try_state::<state::AppState>()
        .ok_or_else(|| Box::<dyn std::error::Error>::from("app state not ready"))?;
    let uri = request.uri().to_string();
    let Some((plugin_id, rel)) = plugin_asset_from_request_uri(&uri) else {
        return ResponseBuilder::new()
            .status(404)
            .mimetype("text/plain; charset=utf-8")
            .body(b"unknown uri".to_vec());
    };
    if state
        .directory_plugins
        .plugin_state_snapshot()
        .is_plugin_disabled(plugin_id.trim())
    {
        return ResponseBuilder::new()
            .status(403)
            .mimetype("text/plain; charset=utf-8")
            .body(b"plugin disabled".to_vec());
    }
    let roots = state.directory_plugins.plugin_roots.read();
    let Some(entry) = roots.get(&plugin_id) else {
        return ResponseBuilder::new()
            .status(404)
            .mimetype("text/plain; charset=utf-8")
            .body(
                format!(
                    "unknown plugin_id={}",
                    sanitize_plugin_id_for_log(plugin_id.as_str())
                )
                .into_bytes(),
            );
    };
    let root = &entry.root;
    let path_norm = match resolve_plugin_asset_path(entry, &rel) {
        Ok(p) => p,
        Err(e) if e == "path escapes plugin directory" => {
            return ResponseBuilder::new()
                .status(403)
                .mimetype("text/plain; charset=utf-8")
                .body(b"forbidden".to_vec());
        }
        Err(_) => {
            return ResponseBuilder::new()
                .status(404)
                .mimetype("text/plain; charset=utf-8")
                .body(b"not found".to_vec());
        }
    };
    let mut data = match fs::read(&path_norm) {
        Ok(b) => b,
        Err(_) => {
            return ResponseBuilder::new()
                .status(404)
                .mimetype("text/plain; charset=utf-8")
                .body(b"not found".to_vec());
        }
    };
    if mime_for_plugin_asset(&rel).starts_with("text/html") {
        if let Ok(manifest) = state.directory_plugins.load_manifest_cached(&plugin_id, root) {
            if let Ok(html) = String::from_utf8(std::mem::take(&mut data)) {
                let injected = inject_plugin_bridge_script(&html, &plugin_id, &rel, manifest.as_ref());
                data = injected.into_bytes();
            }
        }
    }
    ResponseBuilder::new()
        .status(200)
        .mimetype(mime_for_plugin_asset(&rel))
        .body(data)
}

/// 独立 HTTP API 入口（`--api` 子进程）；无 Tauri 窗口与 IPC。
pub fn run_api_server(port: u16) {
    let worker_threads = std::thread::available_parallelism()
        .map(|n| (n.get() / 2).max(2))
        .unwrap_or(2);
    let rt = match tokio::runtime::Builder::new_multi_thread()
        .worker_threads(worker_threads)
        .enable_all()
        .build()
    {
        Ok(rt) => rt,
        Err(e) => {
            eprintln!("failed to build tokio runtime: {}", e);
            std::process::exit(1);
        }
    };
    let r = rt.block_on(http_api::serve_api(port));
    if let Err(e) = r {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 与 `tauri.conf.json` 中 `bundle.identifier` 对齐；`tauri-plugin-deep-link` 须在 setup 前注册。
    #[cfg(desktop)]
    tauri_plugin_deep_link::prepare("com.oclivenewnew.app");
    tauri::Builder::default()
        .register_uri_scheme_protocol("ocliveplugin", |app, request| {
            serve_ocliveplugin_asset(app, request)
        })
        .setup(|app| -> Result<(), Box<dyn std::error::Error>> {
            #[cfg(desktop)]
            {
                let app_h = app.handle().clone();
                if let Err(e) = tauri_plugin_deep_link::register("oclive", move |url: String| {
                    tracing::info!(target: "oclive_deep_link", "oclive deep link: {}", url);
                    seed_pending_install_urls_from_args(std::iter::once(url));
                    let _ = app_h.emit_all(
                        "protocol:pending_install",
                        serde_json::json!({ "reason": "deep-link" }),
                    );
                }) {
                    tracing::warn!(
                        target: "oclive_deep_link",
                        "register oclive:// handler failed: {}",
                        e
                    );
                }
            }
            seed_pending_install_urls_from_args(std::env::args());
            let app_dir = app.path_resolver().app_data_dir().ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "failed to resolve app_data_dir (Tauri path_resolver returned None)",
                )
            })?;
            fs::create_dir_all(&app_dir).map_err(|e| {
                std::io::Error::other(format!("create app_data_dir {}: {}", app_dir.display(), e))
            })?;
            let db_path = app_dir.join("app.db");
            let roles_dir = state::resolve_roles_dir(app.path_resolver().resource_dir().as_deref());
            let roles_for_watcher = roles_dir.clone();
            let app_state = tauri::async_runtime::block_on(async {
                state::AppState::new(&db_path, Some(roles_dir), &app_dir).await
            })
            .map_err(|e| -> Box<dyn std::error::Error> { Box::new(e) })?;

            app.manage(app_state);
            let roles_bg = roles_for_watcher.clone();
            let directory_plugins = app.state::<AppState>().directory_plugins.clone();
            tauri::async_runtime::spawn(async move {
                directory_plugins.rescan_plugin_roots(roles_bg.as_path());
            });
            let hk = crate::infrastructure::hotkey_bindings::HotkeyBindingsFile::load(
                app.state::<AppState>().directory_plugins.app_data_dir(),
            );
            if let Err(e) = crate::api::hotkeys::apply_global_hotkeys(&app.handle(), &hk) {
                tracing::warn!(target: "oclive_hotkey", "initial global shortcuts: {}", e);
            }
            start_plugin_fs_watcher(
                app.handle().clone(),
                &app.state::<AppState>(),
                roles_for_watcher,
            );
            Ok(())
        })
        // Tauri invoke commands — grouped by domain (see `src-tauri/src/api/`).
        .invoke_handler(tauri::generate_handler![
            // ── agent / MCP ──
            api::agent::list_mcp_servers,
            api::agent::list_mcp_tools,
            api::agent::call_mcp_tool,
            api::agent::get_agent_debug_traces,
            api::agent::clear_agent_debug_traces,

            // ── high-risk capabilities ──
            api::high_risk::grant_high_risk_capability,
            api::high_risk::list_high_risk_grants,
            api::high_risk::revoke_high_risk_capability,

            // ── diagnostics ──
            api::diagnostics::run_environment_diagnostics,
            api::llm_settings::get_llm_user_settings,
            api::llm_settings::list_ollama_models,
            api::llm_settings::save_llm_user_settings,
            api::llm_settings::probe_cloud_llm,
            api::llm_settings::scan_local_model_files,
            api::llm_settings::open_path_in_file_manager,
            api::llm_settings::import_gguf_to_ollama,

            // ── app settings ──
            api::settings::get_remote_fallback_app_settings,
            api::settings::set_remote_fallback_to_builtin,

            // ── chat ──
            api::chat::send_message,
            api::chat::list_chat_sessions,
            api::chat::fetch_chat_messages,
            api::chat::rebuild_chat_mirror,
            api::chat::migrate_indexeddb_to_backend,
            api::chat::get_chat_storage_stats,
            api::chat::delete_role_chats,
            api::chat::delete_scene_chats,

            // ── role / session / slot registry ──
            api::role::load_role,
            api::role::get_role_info,
            api::role::list_roles,
            api::role::switch_role,
            api::role::relation::set_user_relation,
            api::role::relation::set_scene_user_relation,
            api::role::relation::clear_scene_user_relation,
            api::role::evolution::set_evolution_factor,
            api::role::evolution::set_remote_life_enabled,
            api::role::evolution::set_role_interaction_mode,
            api::role::slot_session::set_session_plugin_backend,
            api::role::slot_session::set_session_slot_override,
            api::role::slot_session::clear_session_slot_override,
            api::role::slot_session::clear_all_session_slot_overrides,
            api::role::slot_session::save_role_slot_registry,
            api::role::slot_session::apply_author_suggested_plugin_backends,
            api::role::slot_session::get_plugin_resolution_debug,
            api::role::resolve_role_asset_path,
            api::role::read_role_asset_bytes,
            api::role::expert::list_blueprint_includes,
            api::role::expert::get_expert_routing,
            api::role::expert::save_expert_routing,

            // ── desktop filesystem (replaces `@tauri-apps/api/fs` IPC) ──
            api::desktop_fs::write_user_text_file,

            // ── role pack import/export ──
            api::role_pack::export_role_pack_command,
            api::role_pack::peek_role_pack_command,
            api::role_pack::import_role_pack_command,

            // ── scene / presence ──
            api::scene::switch_scene,
            api::scene::set_user_presence_scene,

            // ── virtual time ──
            api::time::get_time_state,
            api::time::jump_time,

            // ── monologue ──
            api::monologue::generate_monologue,

            // ── chat export ──
            api::export::export_chat_logs,

            // ── memory / events ──
            api::memory::query_memories,
            api::event::query_events,
            api::event::create_event,

            // ── policy plugins ──
            api::policy::reload_policy_plugins,

            // ── directory plugins (runtime + catalog) ──
            api::directory_plugin::get_directory_plugin_bootstrap,
            api::directory_plugin::read_plugin_asset_text,
            api::directory_plugin::is_host_event_subscribed,
            api::directory_plugin::get_directory_plugin_catalog,
            api::directory_plugin::get_plugin_state,
            api::directory_plugin::save_plugin_state,
            api::directory_plugin::save_global_plugin_state,
            api::directory_plugin::reset_plugin_state_to_role_default,
            api::directory_plugin::directory_plugin_invoke,

            // ── global hotkeys ──
            api::hotkeys::get_hotkey_bindings,
            api::hotkeys::save_hotkey_bindings,

            // ── plugin scaffold / pack ──
            api::plugin_scaffold::create_plugin_scaffold,
            api::plugin_pack::pack_plugin,

            // ── plugin debug / test runner ──
            api::plugin_debug::spawn_plugin_for_test,
            api::plugin_debug::kill_plugin_process,
            api::plugin_debug::list_plugin_processes,
            api::plugin_debug::get_plugin_logs,
            api::plugin_debug::clear_plugin_logs,
            api::plugin_debug::test_plugin_method,
            api::plugin_debug::discover_plugin_methods,

            // ── plugin HTML bridge ──
            api::plugin_bridge::plugin_bridge_invoke,

            // ── plugin install / update (local zip) ──
            api::plugin_update::check_plugin_updates,
            api::plugin_update::extract_plugin_zip,
            api::plugin_update::install_plugin_from_zip,

            // ── plugin market / index ──
            api::plugin_index::sync_plugin_index_command,
            api::plugin_index::get_cached_plugin_index,
            api::plugin_index::install_plugin_from_market,
            api::plugin_index::install_plugin_from_git,
            api::plugin_index::update_plugin_from_market,
            api::plugin_index::uninstall_plugin_from_market,
            api::plugin_index::batch_update_plugins,
            api::plugin_index::batch_uninstall_plugins,
            api::plugin_index::consume_pending_protocol_installs,

            // ── plugin settings UI ──
            api::plugin_config::get_plugin_settings_ui,
            api::plugin_config::set_plugin_settings_config,
        ])
        .run(tauri::generate_context!())
        .unwrap_or_else(|e| {
            eprintln!("error while running tauri application: {}", e);
            std::process::exit(1);
        });
}
