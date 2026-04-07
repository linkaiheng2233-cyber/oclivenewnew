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
use std::path::PathBuf;
use tauri::Manager;

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
        .setup(|app| {
            let app_dir = app
                .path_resolver()
                .app_data_dir()
                .expect("resolve app_data_dir");
            fs::create_dir_all(&app_dir).expect("create app_data_dir");
            let db_path = app_dir.join("app.db");
            let roles_dir = resolve_roles_dir_for_app(app);
            let app_state = tauri::async_runtime::block_on(async {
                state::AppState::new(&db_path, Some(roles_dir)).await
            })
            .expect("Failed to initialize app state");

            app.manage(app_state);
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
