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
            let file_appender = tracing_appender::rolling::daily(&logs, "oclive.log");
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

    let _ = tracing_subscriber::registry()
        .with(filter)
        .with(layers)
        .try_init();

    file_guard
}

use std::fs;
use std::path::{Path, PathBuf};
use tauri::http::{Request, Response, ResponseBuilder};
use tauri::{AppHandle, Manager};

use crate::infrastructure::deep_link::seed_pending_install_urls_from_args;
use crate::infrastructure::directory_plugins::{start_plugin_fs_watcher, OclivePluginManifest};
use crate::state::AppState;

/// 在 HTML 中注入 `window.OclivePluginBridge`；manifest 含 `bridge` 且资产路径匹配时启用。
fn inject_plugin_bridge_script(
    html: &str,
    plugin_id: &str,
    asset_rel: &str,
    manifest: &OclivePluginManifest,
) -> String {
    if !manifest.should_inject_bridge(asset_rel) {
        return html.to_string();
    }
    let Some(b) = manifest.bridge_for_asset_rel(asset_rel) else {
        return html.to_string();
    };
    let inv = serde_json::to_string(&b.invoke).unwrap_or_else(|_| "[]".to_string());
    let ev = serde_json::to_string(&b.events).unwrap_or_else(|_| "[]".to_string());
    let pid = serde_json::to_string(plugin_id).unwrap_or_else(|_| "\"\"".to_string());
    let arel = serde_json::to_string(asset_rel).unwrap_or_else(|_| "\"\"".to_string());
    static BRIDGE_CORE: &str = include_str!("../assets/plugin-bridge.iife.js");
    let script = format!(
        "<script>{core}window.__oclivSetupPluginBridge({pid},{arel},{inv},{ev});</script>",
        core = BRIDGE_CORE,
        pid = pid,
        arel = arel,
        inv = inv,
        ev = ev
    );
    let lower = html.to_ascii_lowercase();
    if let Some(idx) = lower.rfind("</body>") {
        let mut out = String::with_capacity(html.len() + script.len());
        out.push_str(&html[..idx]);
        out.push_str(&script);
        out.push_str(&html[idx..]);
        out
    } else {
        format!("{html}{script}")
    }
}

fn mime_for_plugin_asset(rel: &str) -> &'static str {
    let ext = Path::new(rel)
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    match ext.as_str() {
        "html" | "htm" => "text/html; charset=utf-8",
        "js" | "mjs" => "text/javascript; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "json" => "application/json; charset=utf-8",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "ico" => "image/x-icon",
        "wasm" => "application/wasm",
        "woff2" => "font/woff2",
        "woff" => "font/woff",
        "ttf" => "font/ttf",
        _ => "application/octet-stream",
    }
}

fn plugin_asset_from_request_uri(uri: &str) -> Option<(String, String)> {
    let lower = uri.to_ascii_lowercase();
    let marker = "ocliveplugin.localhost/";
    let idx = lower.find(marker)?;
    let after = uri.get(idx + marker.len()..)?;
    let path_only = after.split(['?', '#']).next()?;
    let mut parts = path_only.split('/').filter(|s| !s.is_empty());
    let plugin_id = parts.next()?.to_string();
    let rest: Vec<&str> = parts.collect();
    if rest.contains(&"..") {
        return None;
    }
    let rel = rest.join("/");
    if rel.is_empty() {
        return None;
    }
    Some((plugin_id, rel))
}

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
    let Some(root) = roots.get(&plugin_id) else {
        return ResponseBuilder::new()
            .status(404)
            .mimetype("text/plain; charset=utf-8")
            .body(format!("unknown plugin_id={}", plugin_id).into_bytes());
    };
    let path = root.join(&rel);
    let root_norm = root.canonicalize().unwrap_or_else(|_| root.clone());
    let mut data = match fs::read(&path) {
        Ok(b) => b,
        Err(_) => {
            return ResponseBuilder::new()
                .status(404)
                .mimetype("text/plain; charset=utf-8")
                .body(b"not found".to_vec());
        }
    };
    let path_norm = path.canonicalize().unwrap_or(path.clone());
    if !path_norm.starts_with(&root_norm) {
        return ResponseBuilder::new()
            .status(403)
            .mimetype("text/plain; charset=utf-8")
            .body(b"forbidden".to_vec());
    }
    if mime_for_plugin_asset(&rel).starts_with("text/html") {
        if let Ok(manifest) = OclivePluginManifest::load_from_dir(root) {
            if let Ok(s) = String::from_utf8(data.clone()) {
                let injected = inject_plugin_bridge_script(&s, &plugin_id, &rel, &manifest);
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
    let rt = match tokio::runtime::Builder::new_multi_thread()
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

            // ── app settings ──
            api::settings::get_remote_fallback_app_settings,
            api::settings::set_remote_fallback_to_builtin,

            // ── chat ──
            api::chat::send_message,

            // ── role / session / slot registry ──
            api::role::load_role,
            api::role::get_role_info,
            api::role::list_roles,
            api::role::switch_role,
            api::role::set_user_relation,
            api::role::set_scene_user_relation,
            api::role::clear_scene_user_relation,
            api::role::set_evolution_factor,
            api::role::set_remote_life_enabled,
            api::role::set_role_interaction_mode,
            api::role::set_session_plugin_backend,
            api::role::set_session_slot_override,
            api::role::clear_session_slot_override,
            api::role::clear_all_session_slot_overrides,
            api::role::save_role_slot_registry,
            api::role::apply_author_suggested_plugin_backends,
            api::role::get_plugin_resolution_debug,
            api::role::resolve_role_asset_path,
            api::role::read_role_asset_bytes,

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
