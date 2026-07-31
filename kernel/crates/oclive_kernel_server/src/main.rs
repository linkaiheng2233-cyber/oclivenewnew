//! Headless HTTP kernel entry (`kernel_server` distribution shape).
//!
//! Same orchestration as `oclivenewnew-tauri --api`; use this binary for robots,
//! gateways, and CI without pulling a GUI entrypoint.
//!
//! **Dual-core**: this crate does not enable `oclive_kernel_host/dual_core`. Experimental
//! blueprint scheduling requires the desktop host built with `oclivenewnew-tauri --features dual_core`.
//! See `kernel/crates/oclive_kernel_host/src/domain/dual_pipeline.rs` and `dual_pipeline_registry.rs`.

use oclive_kernel_runtime::{
    parse_api_port_arg, resolve_api_port, KernelBinaryManifest, RUNTIME_API_VERSION,
};

fn print_usage() {
    eprintln!(
        "oclive-kernel-server {RUNTIME_API_VERSION}\n\
         Usage: oclive-kernel-server [--api] [--port PORT] [--version-json]\n\
         Env: OCLIVE_API_PORT, OCLIVE_API_TOKEN, OCLIVE_HTTP_API_MOCK_LLM, OCLIVE_ROLES_DIR,\n\
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
        println!(
            "{}",
            serde_json::to_string_pretty(&manifest).unwrap_or_default()
        );
        return;
    }

    let cli_port = parse_api_port_arg(&args).unwrap_or_else(|error| {
        tracing::error!(
            target: "oclive_kernel_server",
            error_code = "OCLIVE_CLI_INVALID_ARGUMENT",
            error = %error,
            "invalid command-line argument"
        );
        print_usage();
        std::process::exit(2);
    });
    let port = resolve_api_port(cli_port);
    tracing::info!(
        target: "oclive_kernel_server",
        version = RUNTIME_API_VERSION,
        port,
        "starting headless HTTP API"
    );
    oclive_kernel_host::run_api_server(port);
}

#[cfg(test)]
mod tests {
    use oclive_kernel_runtime::parse_api_port_arg;

    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_string()).collect()
    }

    #[test]
    fn parses_port_and_rejects_invalid_values() {
        assert_eq!(
            parse_api_port_arg(&args(&["oclive-kernel-server", "--port", "9123"])).unwrap(),
            Some(9123)
        );
        assert!(parse_api_port_arg(&args(&["oclive-kernel-server", "--port"])).is_err());
        assert!(parse_api_port_arg(&args(&["oclive-kernel-server", "--port", "invalid"])).is_err());
        assert!(parse_api_port_arg(&args(&["oclive-kernel-server", "--port", "0"])).is_err());
    }
}
