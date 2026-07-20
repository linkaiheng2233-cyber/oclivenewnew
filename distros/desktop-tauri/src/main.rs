// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn parse_cli(args: &[String]) -> Result<(bool, Option<u16>), String> {
    let api = args.iter().skip(1).any(|arg| arg == "--api");
    let cli_port = oclive_kernel_runtime::parse_api_port_arg(args)?;
    Ok((api, cli_port))
}

#[allow(clippy::collapsible_match, clippy::single_match)]
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let (api, cli_port) = parse_cli(&args).unwrap_or_else(|error| {
        eprintln!("[OCLIVE_CLI_INVALID_ARGUMENT] {error}");
        std::process::exit(2);
    });

    if api {
        let port = oclive_kernel_runtime::resolve_api_port(cli_port);
        let (app_data, _) = oclive_kernel_runtime::find_app_data_dir_for_api(port);
        let log_dir = match oclive_kernel_runtime::ensure_app_data_dir(&app_data) {
            Ok(path) => Some(path.join("logs")),
            Err(error) => {
                eprintln!(
                    "[OCLIVE_APP_DATA_INIT_FAILED] path={} error={error}",
                    app_data.display()
                );
                None
            }
        };
        let _log_guard = oclive_kernel_host::init_tracing_with_log_dir(log_dir.as_deref());
        oclive_kernel_host::run_api_server(port);
    } else {
        let _log_guard = oclivenewnew_tauri::init_tracing();
        oclivenewnew_tauri::run();
    }
}

#[cfg(test)]
mod tests {
    use super::parse_cli;

    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_string()).collect()
    }

    #[test]
    fn parses_api_port() {
        assert_eq!(
            parse_cli(&args(&["oclive", "--api", "--port", "9123"])).unwrap(),
            (true, Some(9123))
        );
    }

    #[test]
    fn rejects_missing_or_invalid_port() {
        assert!(parse_cli(&args(&["oclive", "--port"])).is_err());
        assert!(parse_cli(&args(&["oclive", "--port", "invalid"])).is_err());
        assert!(parse_cli(&args(&["oclive", "--port", "0"])).is_err());
    }
}
