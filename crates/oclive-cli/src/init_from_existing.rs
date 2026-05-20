//! `init --from-existing` / `--share` — 1:1 replication helpers.

use anyhow::Result;
use std::path::Path;

use crate::init::InitArgs;
use crate::project_introspect::{analyze_project, build_init_command_line, write_share_file, ProjectSnapshot};

pub fn run_from_existing(path: &Path, args: &InitArgs) -> Result<()> {
    let snap = analyze_project(path)?;
    let out = args.output.clone();
    let cmd = build_init_command_line(&snap, &out);

    if args.share {
        let share_path = write_share_file(path, &snap, &cmd)?;
        if !args.json {
            println!("Wrote share file: {}", share_path.display());
        }
    }

    if args.json {
        let body = serde_json::json!({
            "schema_version": 1,
            "source": path.display().to_string(),
            "snapshot": snap,
            "reproduce_command": cmd,
            "suggested_output": out.display().to_string(),
        });
        println!("{}", serde_json::to_string_pretty(&body)?);
    } else {
        println!("1:1 reproduction command (run from any directory):\n");
        println!("{cmd}");
        println!();
        print_snapshot_summary(&snap);
    }
    Ok(())
}

fn print_snapshot_summary(s: &ProjectSnapshot) {
    println!("Detected configuration:");
    println!("  project_name: {}", s.project_name);
    println!("  project_type: {}", s.project_type);
    println!("  preset: {}", s.preset);
    println!(
        "  monolith: {}",
        if s.monolith_enabled {
            format!(
                "enabled ({})",
                s.monolith_preset.as_deref().unwrap_or("latency")
            )
        } else {
            "disabled".into()
        }
    );
    println!("  kernel_runtime: {} {:?}", s.kernel_dep_kind, s.kernel_source);
    if let Some(ref p) = s.pipeline {
        println!("  pipeline: {p}");
    }
}
