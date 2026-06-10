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
        let (app_data, _) = oclive_kernel_runtime::find_app_data_dir_for_api(port);
        let _ = oclive_kernel_runtime::ensure_app_data_dir(&app_data);
        let log_dir = app_data.join("logs");
        let _log_guard = oclive_kernel_host::init_tracing_with_log_dir(Some(log_dir.as_path()));
        oclive_kernel_host::run_api_server(port);
    } else {
        let _log_guard = oclivenewnew_tauri::init_tracing();
        oclivenewnew_tauri::run();
    }
}
