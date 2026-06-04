//! Headless HTTP kernel entry (`kernel_server` distribution shape).
//!
//! Same orchestration as `oclivenewnew-tauri --api`; use this binary for robots,
//! gateways, and CI without pulling a GUI entrypoint.

use oclive_kernel_runtime::{resolve_api_port, KernelBinaryManifest, RUNTIME_API_VERSION};

fn print_usage() {
    eprintln!(
        "oclive-kernel-server {RUNTIME_API_VERSION}\n\
         Usage: oclive-kernel-server [--api] [--port PORT] [--version-json]\n\
         Env: OCLIVE_API_PORT, OCLIVE_HTTP_API_MOCK_LLM, OCLIVE_ROLES_DIR,\n\
         OCLIVE_APP_DATA, OCLIVE_USE_CANONICAL_APP_DATA, OCLIVE_API_USE_TEMP_APP_DATA, RUST_LOG"
    );
}

fn main() {
    let _log_guard = oclive_kernel_host::init_tracing();

    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "-h" || a == "--help") {
        print_usage();
        return;
    }

    if args.iter().any(|a| a == "--version-json") {
        let manifest = KernelBinaryManifest::from_compile_time_env();
        println!("{}", serde_json::to_string_pretty(&manifest).unwrap_or_default());
        return;
    }

    let mut cli_port: Option<u16> = None;
    let mut api = false;
    let mut i = 1usize;
    while i < args.len() {
        match args[i].as_str() {
            "--api" => api = true,
            "--port" if i + 1 < args.len() => {
                cli_port = args[i + 1].parse().ok();
                i += 1;
            }
            "--version-json" => {}
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
        oclive_kernel_host::run_api_server(port);
    } else {
        print_usage();
        std::process::exit(2);
    }
}
