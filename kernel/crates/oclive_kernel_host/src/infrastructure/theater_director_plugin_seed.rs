//! Seed bundled official theater director plugin into `{app_data}/plugins/` when missing.

use std::fs;
use std::io;
use std::path::Path;

pub const OFFICIAL_THEATER_DIRECTOR_PLUGIN_ID: &str = "com.oclive.theater_director_official";

/// Copy official theater director plugin from bundled `plugins/` parent when not yet in app data.
pub fn seed_official_theater_director_plugin(
    app_data: &Path,
    bundled_plugins_parent: Option<&Path>,
) {
    let dest = app_data
        .join("plugins")
        .join(OFFICIAL_THEATER_DIRECTOR_PLUGIN_ID);
    if dest.join("manifest.json").is_file() {
        return;
    }
    let Some(parent) = bundled_plugins_parent else {
        return;
    };
    let src = parent.join(OFFICIAL_THEATER_DIRECTOR_PLUGIN_ID);
    if !src.join("manifest.json").is_file() {
        return;
    }
    if let Err(e) = copy_dir_all(&src, &dest) {
        tracing::warn!(
            target: "oclive_theater",
            error = %e,
            src = %src.display(),
            dest = %dest.display(),
            "failed to seed official theater director plugin"
        );
        return;
    }
    tracing::info!(
        target: "oclive_theater",
        dest = %dest.display(),
        "seeded official theater director plugin"
    );
}

fn copy_dir_all(src: &Path, dst: &Path) -> io::Result<()> {
    fs::create_dir_all(dst)?;
    for ent in fs::read_dir(src)? {
        let ent = ent?;
        let to = dst.join(ent.file_name());
        if ent.file_type()?.is_dir() {
            copy_dir_all(&ent.path(), &to)?;
        } else {
            fs::copy(ent.path(), to)?;
        }
    }
    Ok(())
}
