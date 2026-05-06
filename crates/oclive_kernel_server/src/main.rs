fn read_port() -> u16 {
    std::env::var("OOCP_API_PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(48888)
}

/// 生产建议开启：禁止依赖 `OCLIVE_*` 的 exe/cwd 启发式默认路径。
fn require_explicit_paths_or_exit() {
    if !oclive_kernel_runtime::env_flags::env_flag_enabled("OCLIVE_REQUIRE_EXPLICIT_PATHS") {
        return;
    }
    let keys = ["OCLIVE_ROLES_DIR", "OCLIVE_DB_PATH", "OCLIVE_APP_DATA_DIR"];
    let missing: Vec<&'static str> = keys
        .into_iter()
        .filter(|k| {
            !std::env::var(k)
                .ok()
                .map(|v| !v.trim().is_empty())
                .unwrap_or(false)
        })
        .collect();
    if missing.is_empty() {
        return;
    }
    eprintln!(
        "OCLIVE_REQUIRE_EXPLICIT_PATHS=1 但以下环境变量未设置或为空：{}。\n\
         请显式设置角色目录、SQLite 路径与应用数据目录，避免生产环境落到临时目录或 cwd 探测。",
        missing.join(", ")
    );
    std::process::exit(2);
}

fn init_tracing() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .with_target(true)
        .try_init();
    let _ = tracing_log::LogTracer::init();
}

#[tokio::main]
async fn main() {
    init_tracing();
    require_explicit_paths_or_exit();
    let port = read_port();
    // Headless OOCP API from `oclive_kernel_runtime` (no Tauri / desktop shell).
    let opt = oclive_kernel_runtime::http_api::ApiServerOptions::from_env_or_defaults(port);
    if let Err(e) = oclive_kernel_runtime::http_api::serve_api_with_options(opt).await {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}
