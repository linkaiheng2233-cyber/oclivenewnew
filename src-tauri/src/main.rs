// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    let _ = env_logger::try_init();
    let args: Vec<String> = std::env::args().collect();
    let mut port: u16 = std::env::var("OCLIVE_API_PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(8420);
    let mut api = false;
    let mut i = 1usize;
    while i < args.len() {
        match args[i].as_str() {
            "--api" => api = true,
            "--port" => {
                if i + 1 < args.len() {
                    if let Ok(p) = args[i + 1].parse::<u16>() {
                        port = p;
                    }
                    i += 1;
                }
            }
            _ => {}
        }
        i += 1;
    }

    if api {
        oclivenewnew_tauri::run_api_server(port);
    } else {
        oclivenewnew_tauri::run();
    }
}
