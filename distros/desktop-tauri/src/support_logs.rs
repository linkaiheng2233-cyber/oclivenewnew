//! Read only application-owned log tails; no frontend-controlled roots or arbitrary files.

use oclive_kernel_types::models::dto::{SupportLogFile, SupportLogSnapshot};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Component, Path};
use std::time::UNIX_EPOCH;

const MAX_BYTES: u64 = 96 * 1024;
const MAX_LINES: usize = 200;
const MAX_FILES: usize = 40;

fn valid_name(name: &str) -> bool {
    let mut parts = Path::new(name).components();
    matches!(parts.next(), Some(Component::Normal(_)))
        && parts.next().is_none()
        && !name.contains(['/', '\\', ':'])
        && name.ends_with(".log")
}

pub(crate) fn read_log_tail(
    root: &Path,
    selected: Option<&str>,
) -> Result<SupportLogSnapshot, String> {
    if selected.is_some_and(|name| !valid_name(name)) {
        return Err("Invalid log file name".into());
    }
    let mut snapshot = SupportLogSnapshot {
        directory: root.to_string_lossy().into_owned(),
        files: Vec::new(),
        selected_file: None,
        text: String::new(),
        truncated: false,
    };
    if !root.exists() {
        return Ok(snapshot);
    }
    let canonical_root = root.canonicalize().map_err(|error| error.to_string())?;
    for entry in std::fs::read_dir(&canonical_root).map_err(|error| error.to_string())? {
        let entry = entry.map_err(|error| error.to_string())?;
        let name = entry.file_name().to_string_lossy().into_owned();
        let metadata =
            std::fs::symlink_metadata(entry.path()).map_err(|error| error.to_string())?;
        if !valid_name(&name) || !metadata.is_file() || metadata.file_type().is_symlink() {
            continue;
        }
        snapshot.files.push(SupportLogFile {
            name,
            size_bytes: metadata.len(),
            modified_at_epoch_ms: metadata
                .modified()
                .ok()
                .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
                .and_then(|duration| u64::try_from(duration.as_millis()).ok())
                .unwrap_or(0),
        });
    }
    snapshot.files.sort_by(|a, b| {
        b.modified_at_epoch_ms
            .cmp(&a.modified_at_epoch_ms)
            .then_with(|| a.name.cmp(&b.name))
    });
    snapshot.files.truncate(MAX_FILES);
    let name = match selected {
        Some(name) => name.to_string(),
        None => match snapshot.files.first() {
            Some(file) => file.name.clone(),
            None => return Ok(snapshot),
        },
    };
    if !snapshot.files.iter().any(|file| file.name == name) {
        return Err("Log file is unavailable or has rotated; refresh the file list".into());
    }
    let file_path = canonical_root.join(&name);
    let metadata = std::fs::symlink_metadata(&file_path).map_err(|error| error.to_string())?;
    if metadata.file_type().is_symlink()
        || !metadata.is_file()
        || file_path
            .canonicalize()
            .map_err(|error| error.to_string())?
            .parent()
            != Some(canonical_root.as_path())
    {
        return Err("Log file must stay inside the application log directory".into());
    }
    let mut file = File::open(&file_path).map_err(|error| error.to_string())?;
    let length = file.metadata().map_err(|error| error.to_string())?.len();
    let offset = length.saturating_sub(MAX_BYTES);
    file.seek(SeekFrom::Start(offset))
        .map_err(|error| error.to_string())?;
    let mut bytes = Vec::new();
    file.take(MAX_BYTES)
        .read_to_end(&mut bytes)
        .map_err(|error| error.to_string())?;
    if offset > 0 {
        // Discard the initial incomplete line, including any split UTF-8 character.
        let start = bytes
            .iter()
            .position(|byte| *byte == b'\n')
            .map_or(bytes.len(), |index| index + 1);
        bytes.drain(..start);
    }
    let text = String::from_utf8_lossy(&bytes);
    let lines: Vec<_> = text.lines().collect();
    snapshot.truncated = offset > 0 || lines.len() > MAX_LINES;
    snapshot.text = lines[lines.len().saturating_sub(MAX_LINES)..].join("\n");
    snapshot.selected_file = Some(name);
    Ok(snapshot)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_and_missing_directories_are_valid() {
        let temp = tempfile::tempdir().unwrap();
        assert!(read_log_tail(&temp.path().join("missing"), None)
            .unwrap()
            .files
            .is_empty());
        assert!(read_log_tail(temp.path(), None).unwrap().text.is_empty());
    }

    #[test]
    fn restricts_file_names_and_does_not_expose_other_files() {
        let temp = tempfile::tempdir().unwrap();
        std::fs::write(temp.path().join("token.json"), "secret").unwrap();
        std::fs::write(
            temp.path().join("oclive.2026-10-26.log"),
            "INFO ready\nERROR test",
        )
        .unwrap();
        for name in [
            "../token.json",
            "C:\\secret.log",
            "token.json",
            "../other.log",
            "a/b.log",
        ] {
            assert!(read_log_tail(temp.path(), Some(name)).is_err(), "{name}");
        }
        let preview = read_log_tail(temp.path(), None).unwrap();
        assert_eq!(preview.files.len(), 1);
        assert!(preview.text.contains("ERROR test"));
        assert!(!preview.text.contains("secret"));
        assert!(read_log_tail(temp.path(), Some("rotated.log")).is_err());
    }

    #[test]
    fn bounds_log_tail_by_bytes_and_lines() {
        let temp = tempfile::tempdir().unwrap();
        let text = (0..1500)
            .map(|i| format!("INFO {i}: {}\n", "字".repeat(50)))
            .collect::<String>();
        std::fs::write(temp.path().join("oclive.log"), text).unwrap();
        let preview = read_log_tail(temp.path(), None).unwrap();
        assert!(preview.truncated);
        assert_eq!(preview.text.lines().count(), MAX_LINES);
        assert!(preview.text.contains("1499"));
        assert!(!preview.text.contains('\u{fffd}'));
        assert!(preview.text.len() <= MAX_BYTES as usize);
    }

    #[cfg(unix)]
    #[test]
    fn skips_links_outside_the_log_directory() {
        let temp = tempfile::tempdir().unwrap();
        let outside = tempfile::NamedTempFile::new().unwrap();
        std::os::unix::fs::symlink(outside.path(), temp.path().join("escape.log")).unwrap();
        assert!(read_log_tail(temp.path(), None).unwrap().files.is_empty());
        assert!(read_log_tail(temp.path(), Some("escape.log")).is_err());
    }
}
