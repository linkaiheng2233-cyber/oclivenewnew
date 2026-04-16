pub mod api;
pub mod domain;
pub mod env_flags;
pub mod error;
pub mod http_api;
pub mod infrastructure;
pub mod models;
pub mod state;
pub mod utils;

use std::fs;
use std::path::{Path, PathBuf};
use tauri::http::{Request, Response, ResponseBuilder};
use tauri::{AppHandle, Manager};

use crate::infrastructure::directory_plugins::{start_plugin_fs_watcher, OclivePluginManifest};
use crate::state::AppState;

/// 向插件 HTML 注入 `window.OclivePluginBridge`（manifest 中 `bridge` + 白名单）。
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
    let script = format!(
        "<script>(function(){{\
var PLUGIN_ID={pid};var ASSET_REL={arel};var INV={inv};var EVT={ev};\
var CMD_PERM={{\"get_conversation\":\"read:conversation\",\"get_roles\":\"read:roles\",\"get_current_role\":\"read:current_role\",\"update_memory\":\"write:memory\",\"delete_memory\":\"write:memory\",\"update_emotion\":\"write:emotion\",\"update_event\":\"write:event\",\"update_prompt\":\"write:prompt\",\"export_conversation\":\"export:conversation\",\"import_role\":\"import:role\",\"delete_role\":\"delete:role\",\"update_settings\":\"write:settings\",\"get_conversation_list\":\"read:conversations\"}};\
function bridgeAllowed(n){{if(INV.indexOf(n)>=0)return true;var p=CMD_PERM[n];return p&&INV.indexOf(p)>=0;}}\
function invoke(n,p){{if(!bridgeAllowed(n))return Promise.reject(new Error('invoke denied:'+n));\
var _inv=window.__TAURI__&&(window.__TAURI__.invoke||(window.__TAURI__.tauri&&window.__TAURI__.tauri.invoke));\
if(!_inv)return Promise.reject(new Error('no invoke API'));\
return _inv('plugin_bridge_invoke',{{req:{{pluginId:PLUGIN_ID,assetRel:ASSET_REL,command:n,params:p!=null?p:{{}}}}}});}}\
function listen(e,c){{if(!EVT.includes(e))return Promise.reject(new Error('event denied:'+e));\
var T=window.__TAURI__;var t=T&&(T.event||(T.tauri&&T.tauri.event));if(!t)return Promise.reject(new Error('no event API'));\
return t.listen(e,c);}}\
window.OclivePluginBridge={{invoke:invoke,listen:listen,allowedInvoke:INV,allowedEvents:EVT}};\
}})();</script>",
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

/// 优先 `OCLIVE_ROLES_DIR`，其次打包资源目录下的 `roles/`（对应 `bundle.resources`），否则开发态 [`state::resolve_roles_dir`]。
fn resolve_roles_dir_for_app(app: &tauri::App) -> PathBuf {
    if let Ok(custom) = std::env::var("OCLIVE_ROLES_DIR") {
        let p = PathBuf::from(custom);
        if p.is_dir() {
            log::info!(
                target: "oclive_roles",
                "using OCLIVE_ROLES_DIR -> {}",
                p.display()
            );
            return p;
        }
        log::warn!(
            target: "oclive_roles",
            "OCLIVE_ROLES_DIR is set but not a directory: {}",
            p.display()
        );
    }
    match app.path_resolver().resource_dir() {
        Some(res) => {
            log::info!(target: "oclive_roles", "tauri resource_dir -> {}", res.display());
            let bundled = res.join("roles");
            if bundled.is_dir() {
                log::info!(
                    target: "oclive_roles",
                    "using bundled roles -> {}",
                    bundled.display()
                );
                return bundled;
            }
            log::warn!(
                target: "oclive_roles",
                "resource_dir/roles missing or not a directory: {}",
                bundled.display()
            );
        }
        None => log::warn!(
            target: "oclive_roles",
            "resource_dir() is None; falling back to dev resolve_roles_dir"
        ),
    }
    let dev = state::resolve_roles_dir();
    log::info!(
        target: "oclive_roles",
        "using dev fallback resolve_roles_dir -> {}",
        dev.display()
    );
    dev
}

/// 本地 HTTP API 模式（`--api`），供编写器试聊等；阻塞至进程退出。
pub fn run_api_server(port: u16) {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("tokio runtime");
    let r = rt.block_on(http_api::serve_api(port));
    if let Err(e) = r {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let _ = env_logger::try_init();
    tauri::Builder::default()
        .register_uri_scheme_protocol("ocliveplugin", |app, request| {
            serve_ocliveplugin_asset(app, request)
        })
        .setup(|app| {
            let app_dir = app
                .path_resolver()
                .app_data_dir()
                .expect("resolve app_data_dir");
            fs::create_dir_all(&app_dir).expect("create app_data_dir");
            let db_path = app_dir.join("app.db");
            let roles_dir = resolve_roles_dir_for_app(app);
            let roles_for_watcher = roles_dir.clone();
            let app_state = tauri::async_runtime::block_on(async {
                state::AppState::new(&db_path, Some(roles_dir), &app_dir).await
            })
            .expect("Failed to initialize app state");

            app.manage(app_state);
            start_plugin_fs_watcher(
                app.handle().clone(),
                &app.state::<AppState>(),
                roles_for_watcher,
            );
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            api::chat::send_message,
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
            api::role::get_plugin_resolution_debug,
            api::role::resolve_role_asset_path,
            api::role_pack::export_role_pack_command,
            api::role_pack::peek_role_pack_command,
            api::role_pack::import_role_pack_command,
            api::scene::switch_scene,
            api::scene::set_user_presence_scene,
            api::time::get_time_state,
            api::time::jump_time,
            api::monologue::generate_monologue,
            api::export::export_chat_logs,
            api::memory::query_memories,
            api::event::query_events,
            api::event::create_event,
            api::policy::reload_policy_plugins,
            api::directory_plugin::get_directory_plugin_bootstrap,
            api::directory_plugin::read_plugin_asset_text,
            api::directory_plugin::is_host_event_subscribed,
            api::directory_plugin::get_directory_plugin_catalog,
            api::directory_plugin::get_plugin_state,
            api::directory_plugin::save_plugin_state,
            api::directory_plugin::reset_plugin_state_to_role_default,
            api::directory_plugin::directory_plugin_invoke,
            api::plugin_bridge::plugin_bridge_invoke,
            api::plugin_update::check_plugin_updates,
            api::plugin_update::extract_plugin_zip,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
