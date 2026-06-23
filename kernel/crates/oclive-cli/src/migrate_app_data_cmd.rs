//! `oclive-cli migrate-app-data` — copy legacy Tauri app data into canonical `OCLive/data`.

use anyhow::Result;
use clap::Args;
use oclive_kernel_runtime::{
    ensure_app_data_dir, ensure_canonical_app_data_ready, find_app_data_dir_for_host, find_db_path,
    tauri_legacy_app_data_dir, ENV_SKIP_APP_DATA_MIGRATION,
};
use std::path::PathBuf;

#[derive(Args, Debug)]
pub struct MigrateAppDataArgs {
    /// Target canonical directory (default: brand `OCLive/data`).
    #[arg(long)]
    pub target: Option<PathBuf>,
    /// Dry-run: print paths only.
    #[arg(long)]
    pub dry_run: bool,
}

pub fn run(args: MigrateAppDataArgs) -> Result<()> {
    let target = args.target.unwrap_or_else(find_app_data_dir_for_host);
    let legacy = tauri_legacy_app_data_dir();
    println!("legacy: {}", legacy.display());
    println!("target: {}", target.display());
    if args.dry_run {
        return Ok(());
    }
    std::env::remove_var(ENV_SKIP_APP_DATA_MIGRATION);
    ensure_app_data_dir(&target).map_err(|e| anyhow::anyhow!(e))?;
    ensure_canonical_app_data_ready(&target).map_err(|e| anyhow::anyhow!(e))?;
    let db = find_db_path(&target);
    if db.is_file() {
        println!("ok: {}", db.display());
    } else {
        println!("no migration needed (target db absent and no legacy db)");
    }
    Ok(())
}
