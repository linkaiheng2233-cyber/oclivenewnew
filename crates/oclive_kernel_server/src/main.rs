//! Headless HTTP kernel entry (`kernel_server` distribution shape).
//!
//! Same orchestration as `oclivenewnew-tauri --api`; use this binary for robots,
//! gateways, and CI without pulling a GUI entrypoint.

use oclive_kernel_runtime::{resolve_api_port, RUNTIME_API_VERSION};

fn print_usage() {
    eprintln!(
        "oclive-kernel-server {RUNTIME_API_VERSION}\n\
         Usage: oclive-kernel-server [--api] [--port PORT]\n\
         Env: OCLIVE_API_PORT, OCLIVE_HTTP_API_MOCK_LLM, OCLIVE_ROLES_DIR, RUST_LOG"
    );
}

fn main() {
    oclivenewnew_tauri::init_tracing();

    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "-h" || a == "--help") {
        print_usage();
        return;
    }

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

    if !api {
        api = true;
    }

    if api {
        let port = resolve_api_port(cli_port);
        tracing::info!(
            target: "oclive_kernel_server",
            version = RUNTIME_API_VERSION,
            port,
            "starting headless HTTP API"
        );
        oclivenewnew_tauri::run_api_server(port);
    } else {
        print_usage();
        std::process::exit(2);
    }
}
