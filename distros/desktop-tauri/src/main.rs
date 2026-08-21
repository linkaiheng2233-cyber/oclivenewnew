// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::PathBuf;

#[derive(Debug, PartialEq, Eq)]
enum LaunchMode {
    Desktop,
    Api {
        port: Option<u16>,
    },
    Repair {
        resource_dir: Option<PathBuf>,
        report_path: Option<PathBuf>,
    },
}

fn named_path_arg(args: &[String], name: &str) -> Result<Option<PathBuf>, String> {
    let Some(index) = args.iter().position(|arg| arg == name) else {
        return Ok(None);
    };
    let Some(value) = args.get(index + 1) else {
        return Err(format!("{name} requires a path value"));
    };
    if value.trim().is_empty() || value.starts_with("--") {
        return Err(format!("{name} requires a path value"));
    }
    Ok(Some(PathBuf::from(value)))
}

fn parse_cli(args: &[String]) -> Result<LaunchMode, String> {
    if args
        .iter()
        .skip(1)
        .any(|arg| arg == "--repair-installation")
    {
        return Ok(LaunchMode::Repair {
            resource_dir: named_path_arg(args, "--repair-resource-dir")?,
            report_path: named_path_arg(args, "--repair-report")?,
        });
    }
    if args.iter().skip(1).any(|arg| arg == "--api") {
        return Ok(LaunchMode::Api {
            port: oclive_kernel_runtime::parse_api_port_arg(args)?,
        });
    }
    let port = oclive_kernel_runtime::parse_api_port_arg(args)?;
    if port.is_some() {
        return Err("--port requires --api".to_string());
    }
    Ok(LaunchMode::Desktop)
}

fn default_repair_resource_dir() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(std::path::Path::to_path_buf))
        .or_else(|| std::env::current_dir().ok())
        .unwrap_or_else(|| PathBuf::from("."))
}

fn run_repair(resource_dir: Option<PathBuf>, report_path: Option<PathBuf>) -> i32 {
    let resource_dir = resource_dir.unwrap_or_else(default_repair_resource_dir);
    let roles_dir = oclive_kernel_host::state::find_roles_dir(Some(&resource_dir));
    let app_data_dir = oclive_kernel_runtime::find_app_data_dir_for_host();
    let mut report = oclivenewnew_tauri::installation_repair::run_installation_repair(
        &resource_dir,
        &roles_dir,
        &app_data_dir,
    );
    report.restart_required = report.success;
    if let Err(error) = oclivenewnew_tauri::installation_repair::write_repair_report(
        &mut report,
        report_path.as_deref(),
    ) {
        eprintln!("[OCLIVE_REPAIR_REPORT_WRITE_FAILED] {error}");
        return 2;
    }
    if report.success {
        0
    } else {
        1
    }
}

#[allow(clippy::collapsible_match, clippy::single_match)]
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mode = parse_cli(&args).unwrap_or_else(|error| {
        eprintln!("[OCLIVE_CLI_INVALID_ARGUMENT] {error}");
        std::process::exit(2);
    });

    match mode {
        LaunchMode::Api { port } => {
            let port = oclive_kernel_runtime::resolve_api_port(port);
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
        }
        LaunchMode::Repair {
            resource_dir,
            report_path,
        } => {
            std::process::exit(run_repair(resource_dir, report_path));
        }
        LaunchMode::Desktop => {
            let _log_guard = oclivenewnew_tauri::init_tracing();
            oclivenewnew_tauri::run();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{parse_cli, LaunchMode};
    use std::path::PathBuf;

    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_string()).collect()
    }

    #[test]
    fn parses_api_port() {
        assert_eq!(
            parse_cli(&args(&["oclive", "--api", "--port", "9123"])).unwrap(),
            LaunchMode::Api { port: Some(9123) }
        );
    }

    #[test]
    fn rejects_missing_or_invalid_port() {
        assert!(parse_cli(&args(&["oclive", "--port"])).is_err());
        assert!(parse_cli(&args(&["oclive", "--port", "invalid"])).is_err());
        assert!(parse_cli(&args(&["oclive", "--port", "0"])).is_err());
    }

    #[test]
    fn parses_repair_paths_without_starting_desktop() {
        assert_eq!(
            parse_cli(&args(&[
                "oclive",
                "--repair-installation",
                "--repair-resource-dir",
                "D:\\A.I.Live Chat Pro",
                "--repair-report",
                "D:\\support\\report.json",
            ]))
            .unwrap(),
            LaunchMode::Repair {
                resource_dir: Some(PathBuf::from("D:\\A.I.Live Chat Pro")),
                report_path: Some(PathBuf::from("D:\\support\\report.json")),
            }
        );
    }
}
