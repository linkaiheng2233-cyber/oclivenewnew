//! Opt-in native file adapter for minimum role asset references.
//!
//! This is a local adapter policy, not a requirement of `MinimalRoleDefinition`.
//! The caller supplies a trusted, stable directory and explicit byte budgets. No
//! media is decoded, URL fetched, or role lifecycle started.

use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};

use crate::minimal_role::{validate_minimal_role_definition, MinimalRoleDefinition};
use crate::validate::validate_portable_path_segment;

/// Read referenced local files into bounded snapshots, in `role.visual_assets` order.
///
/// References must be slash-separated relative paths under `asset_root`. Each
/// segment uses the existing portable path policy (Unicode supported; no dot/hidden
/// segments, Windows device names, surrounding whitespace, trailing dots, forbidden
/// characters, or segments longer than 128 bytes). Links may resolve within the
/// canonical root, but not outside it. Only non-empty regular files are read.
/// Repeated references are read and charged separately; there is no cache/dedup.
///
/// Limits belong to the calling host, not the authored role contract. Both must be
/// positive, and `max_asset_bytes` must be less than `usize::MAX` so an extra byte can
/// detect growth beyond the limit. Returned payload bytes never exceed the total
/// budget; a failed read returns no partial result. A read may use one extra byte to
/// detect an overrun. These are payload limits, not a whole-process memory limit.
///
/// The caller must prevent concurrent mutation of the root, path components, and
/// files during the call. Canonicalization is not a race-free filesystem sandbox
/// and does not detect hard-link provenance. Consumers should use the returned
/// snapshots rather than reopening unchecked paths. The snapshots still need the
/// distro's media validation before decoding/rendering: arbitrary non-empty bytes
/// are accepted here. There is no HTTP/network client or implicit asset download.
///
/// # Errors
///
/// Returns field/index diagnostics for an invalid definition or budget, unsafe
/// paths, unreadable/non-regular/empty files, or per-asset/total budget overruns.
/// Diagnostics omit prompt text, asset references, and absolute paths.
///
/// ```no_run
/// use std::path::Path;
/// use oclive_validation::{load_minimal_role_local_assets, MinimalRoleDefinition};
///
/// let role = MinimalRoleDefinition {
///     persona_prompt: "A guide.".into(),
///     visual_assets: vec!["art/portrait.png".into()],
/// };
/// // Budgets chosen by this example host, not required role-pack fields.
/// let snapshots = load_minimal_role_local_assets(
///     Path::new("installed-pack"), &role, 8 * 1024 * 1024, 32 * 1024 * 1024,
/// )?;
/// assert_eq!(snapshots.len(), 1);
/// # Ok::<(), Vec<String>>(())
/// ```
pub fn load_minimal_role_local_assets(
    asset_root: &Path,
    role: &MinimalRoleDefinition,
    max_asset_bytes: usize,
    max_total_bytes: usize,
) -> Result<Vec<Vec<u8>>, Vec<String>> {
    validate_minimal_role_definition(role)?;
    if !valid_local_byte_budgets(max_asset_bytes, max_total_bytes) {
        return Err(vec![
            "minimal role local assets: invalid caller byte budgets".into(),
        ]);
    }
    // Check every reference before any filesystem access. No reference is a URL,
    // absolute path, or platform-dependent separator in this adapter.
    for (index, reference) in role.visual_assets.iter().enumerate() {
        if !valid_local_reference(reference) {
            return Err(vec![asset_error(
                index,
                "expected a portable relative path",
            )]);
        }
    }
    let root = canonical_asset_root(asset_root)
        .map_err(|reason| vec![format!("minimal role local assets: {reason}")])?;

    let mut remaining = max_total_bytes;
    let mut snapshots = Vec::new();
    for (index, reference) in role.visual_assets.iter().enumerate() {
        if remaining == 0 {
            return Err(vec![asset_error(index, "total byte budget exhausted")]);
        }
        let bytes = read_local_asset(&root, reference, max_asset_bytes.min(remaining))
            .map_err(|reason| vec![asset_error(index, reason)])?;
        remaining -= bytes.len();
        snapshots.push(bytes);
    }
    Ok(snapshots)
}

fn asset_error(index: usize, reason: &str) -> String {
    format!("minimal role local assets: visual_assets[{index}]: {reason}")
}

pub(crate) fn valid_local_byte_budgets(per_file: usize, total: usize) -> bool {
    per_file > 0 && per_file < usize::MAX && total > 0
}

pub(crate) fn valid_local_reference(reference: &str) -> bool {
    reference
        .split('/')
        .all(|segment| validate_portable_path_segment(segment).is_ok())
}

pub(crate) fn canonical_asset_root(root: &Path) -> Result<PathBuf, &'static str> {
    let reason = "root is not an accessible directory";
    let root = root.canonicalize().map_err(|_| reason)?;
    if !root.is_dir() {
        return Err(reason);
    }
    Ok(root)
}

/// Caller validates the relative reference/budget and supplies a canonical root.
pub(crate) fn read_local_asset(
    root: &Path,
    reference: &str,
    limit: usize,
) -> Result<Vec<u8>, &'static str> {
    let path = root
        .join(reference)
        .canonicalize()
        .map_err(|_| "file is not accessible")?;
    if !path.starts_with(root) {
        return Err("resolved path escapes the asset root");
    }
    // Reject devices/FIFOs before opening, then check the opened handle as well.
    let metadata = fs::metadata(&path).map_err(|_| "file metadata is not accessible")?;
    check_file_metadata(&metadata, limit)?;
    let file = File::open(&path).map_err(|_| "file is not readable")?;
    let metadata = file
        .metadata()
        .map_err(|_| "opened file metadata is not accessible")?;
    check_file_metadata(&metadata, limit)?;
    read_asset_bytes(file, limit)
}

fn read_asset_bytes(reader: impl Read, limit: usize) -> Result<Vec<u8>, &'static str> {
    let mut bytes = Vec::new();
    reader
        .take(limit as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| "file read failed")?;
    if bytes.is_empty() {
        return Err("file must not be empty");
    }
    if bytes.len() > limit {
        return Err("file exceeds the per-asset or remaining total byte budget");
    }
    Ok(bytes)
}

fn check_file_metadata(metadata: &fs::Metadata, limit: usize) -> Result<(), &'static str> {
    if !metadata.is_file() {
        return Err("expected a regular file");
    }
    if metadata.len() == 0 {
        return Err("file must not be empty");
    }
    if metadata.len() > limit as u64 {
        return Err("file exceeds the per-asset or remaining total byte budget");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{self, Cursor};

    #[test]
    fn bounded_read_detects_growth_without_reading_the_rest() {
        // Models a file that grew after its metadata checks; never silently
        // return a truncated asset or read the whole unbounded source.
        let mut source = Cursor::new(b"longer-than-budget");
        assert!(read_asset_bytes(&mut source, 4).is_err());
        assert_eq!(source.position(), 5);
        assert_eq!(read_asset_bytes(Cursor::new(b"1234"), 4).unwrap(), b"1234");
    }

    #[test]
    fn empty_or_failed_reads_cannot_yield_partial_assets() {
        struct FailingReader;
        impl Read for FailingReader {
            fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
                Err(io::Error::other("private OS detail"))
            }
        }
        assert_eq!(
            read_asset_bytes(io::empty(), 4).unwrap_err(),
            "file must not be empty"
        );
        let partial_then_error = Cursor::new(b"x").chain(FailingReader);
        assert_eq!(
            read_asset_bytes(partial_then_error, 4).unwrap_err(),
            "file read failed"
        );
    }
}
