fn read_port() -> u16 {
    std::env::var("OOCP_API_PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(48888)
}

#[tokio::main]
async fn main() {
    let _ = env_logger::try_init();
    let port = read_port();
    // Headless OOCP API from `oclive_kernel_runtime` (no Tauri / desktop shell).
    let opt = oclive_kernel_runtime::http_api::ApiServerOptions::from_env_or_defaults(port);
    if let Err(e) = oclive_kernel_runtime::http_api::serve_api_with_options(opt).await {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}
