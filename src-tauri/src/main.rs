// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[allow(clippy::collapsible_match, clippy::single_match)]
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut cli_port: Option<u16> = None;
    let mut api = false;
    let mut i = 1usize;
    while i < args.len() {
        match args[i].as_str() {
            "--api" => api = true,
            "--port" => {
                if i + 1 < args.len() {
                    cli_port = args[i + 1].parse().ok();
                    i += 1;
                }
            }
            _ => {}
        }
        i += 1;
    }

    if api {
        let port = oclive_kernel_runtime::resolve_api_port(cli_port);
        let app_data = std::env::temp_dir().join("oclive_api_app_data");
        oclivenewnew_tauri::init_tracing_with_log_dir(Some(app_data.as_path()));
        oclivenewnew_tauri::run_api_server(port);
    } else {
        oclivenewnew_tauri::init_tracing();
        oclivenewnew_tauri::run();
    }
}
