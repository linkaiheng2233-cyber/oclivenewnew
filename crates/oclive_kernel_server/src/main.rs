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
    // Reuse the full runtime from `oclivenewnew-tauri` but without enabling Tauri features.
    // This provides a "Linux kernel style" standalone core service endpoint.
    if let Err(e) = oclivenewnew_tauri::http_api::serve_api(port).await {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}
