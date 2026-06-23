//! `init --check` pre-flight validation before generating a project.

use anyhow::{bail, Result};
use serde::Serialize;
use std::path::Path;
use std::process::Command;

use crate::generator;
use crate::init::{build_init_config, InitArgs, InitTemplateArg, RolePackKind};

#[derive(Serialize, Clone)]
struct CheckItem {
    id: String,
    status: String,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    hint: Option<String>,
}

pub fn run_precheck(args: &InitArgs) -> Result<()> {
    let cfg = build_init_config(args, true)?;
    let mut items = Vec::new();

    items.push(check_rust_toolchain());
    items.push(check_cargo());
    if let Some(ref ks) = cfg.kernel_source {
        items.push(check_kernel_source(ks));
    } else if args.kernel_source.is_some() {
        items.push(CheckItem {
            id: "kernel_source".into(),
            status: "fail".into(),
            message: "kernel-source path invalid".into(),
            hint: Some("Pass --kernel-source <oclivenewnew root>".into()),
        });
    }
    if cfg.monolith_enabled {
        items.push(check_monolith_toolchain());
    }
    if cfg.with_example_plugin {
        items.push(check_example_plugin_src());
    }
    if !cfg.skip_role_pack && cfg.role_pack_kind != RolePackKind::None {
        items.push(check_role_pack_assets(&cfg));
    }
    if cfg.factory_template == Some(InitTemplateArg::RobotGateway) {
        items.push(CheckItem {
            id: "template_gateway".into(),
            status: "pass".into(),
            message: "robot-gateway template will add mcp_servers/ scaffold".into(),
            hint: None,
        });
    }

    let has_fail = items.iter().any(|i| i.status == "fail");
    if args.json {
        println!("{}", serde_json::to_string_pretty(&items)?);
    } else {
        println!("oclive init --check\n");
        for it in &items {
            let icon = match it.status.as_str() {
                "pass" => "PASS",
                "warn" => "WARN",
                _ => "FAIL",
            };
            println!("  [{icon}] {} — {}", it.id, it.message);
            if let Some(h) = &it.hint {
                println!("           hint: {h}");
            }
        }
        println!(
            "\n{}",
            if has_fail {
                "Pre-check failed. Fix FAIL items before running init."
            } else {
                "Pre-check passed. Safe to run oclive init."
            }
        );
    }
    if has_fail {
        bail!("init --check failed");
    }
    Ok(())
}

fn check_rust_toolchain() -> CheckItem {
    let ok = Command::new("rustc")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    if ok {
        CheckItem {
            id: "rust".into(),
            status: "pass".into(),
            message: "Rust toolchain available".into(),
            hint: None,
        }
    } else {
        CheckItem {
            id: "rust".into(),
            status: "fail".into(),
            message: "rustc not found".into(),
            hint: Some("Install from https://rustup.rs/".into()),
        }
    }
}

fn check_cargo() -> CheckItem {
    let ok = Command::new("cargo")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    CheckItem {
        id: "cargo".into(),
        status: if ok { "pass" } else { "fail" }.into(),
        message: if ok {
            "cargo available".into()
        } else {
            "cargo not found".into()
        },
        hint: if ok {
            None
        } else {
            Some("Install Rust via rustup".into())
        },
    }
}

fn check_kernel_source(path: &Path) -> CheckItem {
    match generator::validate_kernel_source(path) {
        Ok(()) => CheckItem {
            id: "kernel_source".into(),
            status: "pass".into(),
            message: format!("kernel source OK: {}", path.display()),
            hint: None,
        },
        Err(e) => CheckItem {
            id: "kernel_source".into(),
            status: "fail".into(),
            message: format!("invalid kernel source: {e}"),
            hint: Some("Point --kernel-source at oclivenewnew repo root".into()),
        },
    }
}

fn check_monolith_toolchain() -> CheckItem {
    let cpp = if cfg!(windows) {
        Command::new("cl")
            .arg("?")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    } else {
        Command::new("cc")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    };
    CheckItem {
        id: "monolith_build".into(),
        status: if cpp { "pass" } else { "warn" }.into(),
        message: if cpp {
            "C/C++ toolchain detected for Monolith release builds".into()
        } else {
            "C/C++ toolchain not detected (Monolith release build may fail)".into()
        },
        hint: Some("Install build-essential / MSVC Build Tools".into()),
    }
}

fn check_example_plugin_src() -> CheckItem {
    let src = crate::generator::example_llamacpp_plugin_src();
    if src.is_dir() {
        CheckItem {
            id: "example_plugin".into(),
            status: "pass".into(),
            message: format!("llamacpp example found at {}", src.display()),
            hint: None,
        }
    } else {
        CheckItem {
            id: "example_plugin".into(),
            status: "fail".into(),
            message: "examples/directory-plugin-llamacpp missing in oclivenewnew".into(),
            hint: Some("Run from oclivenewnew clone or omit --with-example-plugin".into()),
        }
    }
}

fn check_role_pack_assets(cfg: &crate::init::ProjectConfig) -> CheckItem {
    match cfg.role_pack_kind {
        RolePackKind::RobotSoulMinimal => CheckItem {
            id: "role_pack".into(),
            status: "pass".into(),
            message: "robot-soul-minimal pack will be generated from templates".into(),
            hint: None,
        },
        RolePackKind::DefaultExample => CheckItem {
            id: "role_pack".into(),
            status: "pass".into(),
            message: "default example role pack will be generated".into(),
            hint: None,
        },
        RolePackKind::None => CheckItem {
            id: "role_pack".into(),
            status: "pass".into(),
            message: "no role pack".into(),
            hint: None,
        },
    }
}
