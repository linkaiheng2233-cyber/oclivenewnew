#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]
#![allow(clippy::missing_errors_doc, clippy::missing_panics_doc)]

//! Tauri desktop shell: IPC commands, deep links, kernel attach/lifecycle.
//! Headless HTTP kernel lives in [`oclive_kernel_host`].

pub mod api;
pub mod desktop_host;
pub mod desktop_integration;
pub mod installation_repair;
pub mod kernel_attach;
pub mod kernel_lifecycle;

pub use oclive_kernel_host::{
    command_error, env_flags, http_api, init_tracing, init_tracing_with_log_dir, run_api_server,
};

/// Host error bridge: core types live in `oclive_kernel_types`.
pub mod error {
    pub use oclive_kernel_types::error::*;

    /// Map kernel [`AppError`] to a serializable invoke failure payload (kernel JSON string).
    #[must_use]
    pub fn to_invoke_error(err: AppError) -> String {
        err.to_kernel_json()
    }
}

fn sanitize_plugin_id_for_log(plugin_id: &str) -> String {
    plugin_id
        .chars()
        .filter(|c| !c.is_control())
        .take(64)
        .collect()
}

use std::borrow::Cow;
use std::fs;
use tauri::http::{Request, Response};
use tauri::{AppHandle, Emitter, Manager};

use crate::desktop_integration::start_plugin_fs_watcher;
use oclive_kernel_host::infrastructure::deep_link::seed_pending_install_urls_from_args;
use oclive_kernel_host::infrastructure::directory_plugins::find_plugin_asset_path;
use oclive_kernel_host::infrastructure::plugin_protocol::{
    inject_plugin_bridge_script, mime_for_plugin_asset, plugin_asset_from_request_uri,
};
use oclive_kernel_host::state;

fn http_text(status: u16, body: impl Into<Vec<u8>>, mime: &str) -> Response<Cow<'static, [u8]>> {
    Response::builder()
        .status(status)
        .header("Content-Type", mime)
        .body(Cow::Owned(body.into()))
        .unwrap_or_else(|_| {
            Response::builder()
                .status(500)
                .body(Cow::Borrowed(b"response build failed" as &[u8]))
                .expect("fallback response")
        })
}

fn serve_ocliveplugin_asset(
    app: &AppHandle,
    request: &Request<Vec<u8>>,
) -> Response<Cow<'static, [u8]>> {
    let Some(state) = app.try_state::<state::SharedAppState>() else {
        return http_text(
            503,
            b"app state not ready".to_vec(),
            "text/plain; charset=utf-8",
        );
    };
    let uri = request.uri().to_string();
    let Some((plugin_id, rel)) = plugin_asset_from_request_uri(&uri) else {
        let safe_uri = uri
            .split(['?', '#'])
            .next()
            .unwrap_or_default()
            .chars()
            .filter(|c| !c.is_control())
            .take(256)
            .collect::<String>();
        tracing::warn!(
            target: "oclive_plugin",
            error_code = "PLUGIN_ASSET_URI_INVALID",
            request_uri = %safe_uri,
            "plugin asset request URI rejected"
        );
        return http_text(
            404,
            b"PLUGIN_ASSET_URI_INVALID: see oclive_plugin logs".to_vec(),
            "text/plain; charset=utf-8",
        );
    };
    if state
        .directory_plugins
        .plugin_state_snapshot()
        .is_plugin_disabled(plugin_id.trim())
    {
        return http_text(
            403,
            b"plugin disabled".to_vec(),
            "text/plain; charset=utf-8",
        );
    }
    let roots = state.directory_plugins.plugin_roots.read();
    let Some(entry) = roots.get(&plugin_id) else {
        return http_text(
            404,
            format!(
                "unknown plugin_id={}",
                sanitize_plugin_id_for_log(plugin_id.as_str())
            )
            .into_bytes(),
            "text/plain; charset=utf-8",
        );
    };
    let root = &entry.root;
    let path_norm = match find_plugin_asset_path(entry, &rel) {
        Ok(p) => p,
        Err(e) if e == "path escapes plugin directory" => {
            return http_text(403, b"forbidden".to_vec(), "text/plain; charset=utf-8");
        }
        Err(_) => {
            return http_text(404, b"not found".to_vec(), "text/plain; charset=utf-8");
        }
    };
    let mut data = match fs::read(&path_norm) {
        Ok(b) => b,
        Err(_) => {
            return http_text(404, b"not found".to_vec(), "text/plain; charset=utf-8");
        }
    };
    if mime_for_plugin_asset(&rel).starts_with("text/html") {
        if let Ok(manifest) = state
            .directory_plugins
            .load_manifest_cached(&plugin_id, root)
        {
            if let Ok(html) = String::from_utf8(std::mem::take(&mut data)) {
                let injected =
                    inject_plugin_bridge_script(&html, &plugin_id, &rel, manifest.as_ref());
                data = injected.into_bytes();
            }
        }
    }
    http_text(200, data, mime_for_plugin_asset(&rel))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut builder = tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init());

    #[cfg(desktop)]
    {
        builder = builder.plugin(tauri_plugin_global_shortcut::Builder::new().build());
    }

    #[cfg(all(desktop, feature = "desktop"))]
    {
        builder = builder.plugin(tauri_plugin_deep_link::init());
    }

    builder
        .register_uri_scheme_protocol("ocliveplugin", |ctx, request| {
            serve_ocliveplugin_asset(ctx.app_handle(), &request)
        })
        .setup(|app| -> Result<(), Box<dyn std::error::Error>> {
            #[cfg(all(desktop, feature = "desktop"))]
            {
                use tauri_plugin_deep_link::DeepLinkExt;
                let app_h = app.handle().clone();
                app.deep_link().on_open_url(move |event| {
                    for url in event.urls() {
                        let url_s = url.to_string();
                        tracing::info!(target: "oclive_deep_link", "oclive deep link: {}", url_s);
                        seed_pending_install_urls_from_args(std::iter::once(url_s));
                    }
                    let _ = app_h.emit(
                        "protocol:pending_install",
                        serde_json::json!({ "reason": "deep-link" }),
                    );
                });
                if let Err(e) = app.deep_link().register("oclive") {
                    tracing::warn!(
                        target: "oclive_deep_link",
                        "register oclive:// handler failed: {}",
                        e
                    );
                }
            }
            seed_pending_install_urls_from_args(std::env::args());
            let resource_dir = app.path().resource_dir().ok();
            let roles_dir = state::find_roles_dir(resource_dir.as_deref());
            let roles_for_watcher = roles_dir.clone();
            let (app_state, kernel_conn, _api_port) = desktop_host::bootstrap_desktop_blocking(
                resource_dir.clone(),
            )
            .map_err(|e| -> Box<dyn std::error::Error> {
                tracing::error!(
                    target: "oclive_desktop",
                    error = %e,
                    "desktop bootstrap failed"
                );
                e
            })?;
            app.manage(kernel_conn.clone());
            app.manage(app_state.clone());
            {
                let shell = app_state.clone();
                let app_handle = app.handle().clone();
                shell.set_affect_metrics_sink(Some(std::sync::Arc::new(move |ev| {
                    let _ = app_handle.emit("affect:metricsChanged", &ev);
                })));
            }
            desktop_host::finish_desktop_setup(
                app.handle(),
                kernel_conn,
                roles_for_watcher.clone(),
                resource_dir,
            );
            let directory_plugins = app
                .state::<state::SharedAppState>()
                .directory_plugins
                .clone();
            tauri::async_runtime::spawn(async move {
                directory_plugins.ensure_plugin_roots_scanned();
            });
            let hk = oclive_kernel_host::infrastructure::hotkey_bindings::HotkeyBindingsFile::load(
                app.state::<state::SharedAppState>()
                    .directory_plugins
                    .app_data_dir(),
            );
            if let Err(e) = crate::api::hotkeys::apply_global_hotkeys(app.handle(), &hk) {
                tracing::warn!(target: "oclive_hotkey", "initial global shortcuts: {}", e);
            }
            start_plugin_fs_watcher(
                app.handle().clone(),
                app.state::<state::SharedAppState>().as_ref(),
                roles_for_watcher,
            );
            crate::desktop_integration::spawn_auto_cleanup_scheduler(app.handle().clone());
            Ok(())
        })
        // Tauri invoke commands — grouped by domain (see `distros/desktop-tauri/src/api/`).
        .invoke_handler(tauri::generate_handler![
            // ?? agent / MCP ??
            api::agent::list_mcp_servers,
            api::agent::list_mcp_tools,
            api::agent::call_mcp_tool,
            api::agent::get_agent_debug_traces,
            api::agent::clear_agent_debug_traces,
            // ?? high-risk capabilities ??
            api::high_risk::grant_high_risk_capability,
            api::high_risk::list_high_risk_grants,
            api::high_risk::revoke_high_risk_capability,
            // ?? diagnostics ??
            api::diagnostics::run_environment_diagnostics,
            api::diagnostics::run_environment_repair,
            api::kernel::get_kernel_connection_status,
            api::kernel::get_kernel_diagnostics,
            api::kernel::reconnect_kernel,
            api::llm_settings::get_llm_user_settings,
            api::llm_settings::list_ollama_models,
            api::llm_settings::list_cloud_models,
            api::llm_settings::save_llm_user_settings,
            api::llm_settings::import_local_lora_adapter,
            api::llm_settings::activate_local_lora_adapter,
            api::llm_settings::delete_local_lora_adapter,
            api::llm_settings::get_global_ollama_model,
            api::llm_settings::set_global_ollama_model,
            api::llm_settings::probe_cloud_llm,
            api::llm_settings::scan_local_model_files,
            api::llm_settings::open_path_in_file_manager,
            api::llm_settings::import_gguf_to_ollama,
            // ?? app settings ??
            api::settings::get_remote_fallback_app_settings,
            api::settings::set_remote_fallback_to_builtin,
            // ?? chat ??
            api::chat::send_message,
            api::chat::begin_adult_stage_generation,
            api::chat::generate_adult_staged_beat,
            api::chat::commit_adult_staged_beat,
            api::chat::cancel_adult_stage_generation,
            api::chat::list_adult_staged_beats,
            api::chat::get_role_pack_path,
            api::chat::list_chat_sessions,
            api::chat::fetch_chat_messages,
            api::chat::rebuild_chat_mirror,
            api::chat::migrate_indexeddb_to_backend,
            api::chat::get_chat_storage_capabilities,
            api::chat::get_chat_storage_stats,
            api::chat::delete_role_chats,
            api::chat::delete_scene_chats,
            api::chat::export_chat_session,
            api::chat::export_role_chats,
            api::chat::search_chat_messages,
            api::chat::delete_chat_message,
            api::chat::edit_chat_message,
            api::chat::get_role_chat_storage_config,
            api::chat::save_role_chat_storage_config_cmd,
            api::chat::run_chat_auto_cleanup,
            api::chat::replay_memory_extraction,
            api::chat::get_replay_progress,
            api::chat::get_chat_storage_root,
            api::chat::set_chat_storage_root,
            // ?? role / session / slot registry ??
            api::role::load_role,
            api::role::get_role_info,
            api::role::affect::get_display_metrics,
            api::role::list_roles,
            api::role::switch_role,
            api::role::relation::set_user_relation,
            api::role::relation::set_scene_user_relation,
            api::role::relation::clear_scene_user_relation,
            api::role::identity::set_user_identity,
            api::role::identity::set_scene_user_identity,
            api::role::identity::get_user_identity_state,
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
            api::execution_plan::get_execution_plan_diagnostics,
            api::resource_coordination::get_resource_coordination_diagnostics,
            api::role::find_role_asset_path,
            api::role::read_role_asset_bytes,
            api::role::expert::list_blueprint_includes,
            api::role::expert::get_expert_routing,
            api::role::expert::save_expert_routing,
            // ?? desktop filesystem (replaces `@tauri-apps/api/fs` IPC) ??
            api::desktop_fs::write_user_text_file,
            // ?? role pack import/export ??
            api::role_pack::export_role_pack_command,
            api::portable_state::export_portable_persona,
            api::portable_state::import_portable_persona,
            api::portable_state::export_portable_memory,
            api::portable_state::import_portable_memory,
            api::role_pack::peek_role_pack_command,
            api::role_pack::import_role_pack_command,
            // ?? scene / presence ??
            api::scene::switch_scene,
            api::scene::set_user_presence_scene,
            // ?? virtual time ??
            api::time::get_time_state,
            api::time::jump_time,
            // ?? monologue ??
            api::monologue::generate_monologue,
            // ?? theater scene director ??
            api::theater::generate_theater_scene,
            // ?? chat export ??
            api::export::export_chat_logs,
            // ?? memory / events ??
            api::memory::query_memories,
            api::event::query_events,
            api::event::create_event,
            // ?? policy plugins ??
            api::policy::reload_policy_plugins,
            // ?? directory plugins (runtime + catalog) ??
            api::directory_plugin::get_directory_plugin_bootstrap,
            api::directory_plugin::read_plugin_asset_text,
            api::directory_plugin::is_host_event_subscribed,
            api::directory_plugin::get_directory_plugin_catalog,
            api::directory_plugin::get_plugin_state,
            api::directory_plugin::save_plugin_state,
            api::directory_plugin::save_global_plugin_state,
            api::directory_plugin::reset_plugin_state_to_role_default,
            api::directory_plugin::directory_plugin_invoke,
            // ?? global hotkeys ??
            api::hotkeys::get_hotkey_bindings,
            api::hotkeys::save_hotkey_bindings,
            // ?? plugin scaffold / pack ??
            api::plugin_scaffold::create_plugin_scaffold,
            api::plugin_pack::pack_plugin,
            // ?? plugin debug / test runner ??
            api::plugin_debug::spawn_plugin_for_test,
            api::plugin_debug::kill_plugin_process,
            api::plugin_debug::list_plugin_processes,
            api::plugin_debug::get_plugin_logs,
            api::plugin_debug::clear_plugin_logs,
            api::plugin_debug::test_plugin_method,
            api::plugin_debug::discover_plugin_methods,
            // ?? plugin HTML bridge ??
            api::plugin_bridge::plugin_bridge_invoke,
            // ?? plugin install / update (local zip) ??
            api::plugin_update::check_plugin_updates,
            api::plugin_update::extract_plugin_zip,
            api::plugin_update::install_plugin_from_zip,
            // ?? plugin market / index ??
            api::plugin_index::sync_plugin_index_command,
            api::plugin_index::get_cached_plugin_index,
            api::plugin_index::install_plugin_from_market,
            api::plugin_index::install_plugin_from_git,
            api::plugin_index::update_plugin_from_market,
            api::plugin_index::uninstall_plugin_from_market,
            api::plugin_index::batch_update_plugins,
            api::plugin_index::batch_uninstall_plugins,
            api::plugin_index::consume_pending_protocol_installs,
            // ?? plugin settings UI ??
            api::plugin_config::get_plugin_settings_ui,
            api::plugin_config::set_plugin_settings_config,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app_handle, event| {
            if let tauri::RunEvent::ExitRequested { .. } = event {
                if let Some(state) = app_handle.try_state::<state::SharedAppState>() {
                    state.directory_plugins.shutdown_all();
                }
                if let Some(conn) =
                    app_handle.try_state::<kernel_lifecycle::SharedKernelConnection>()
                {
                    conn.kill_spawned_child();
                }
            }
        });
}
