//! Read-only native loading preparation, not role activation or a canonical pack format.

use std::{fmt, path::Path};

use crate::minimal_role::{parse_minimal_role_definition, MinimalRoleDefinition};
use crate::minimal_role_local_assets::{
    canonical_asset_root, load_minimal_role_local_assets, read_local_asset,
    valid_local_byte_budgets, valid_local_reference,
};

/// A logically validated definition paired with its nonempty local file snapshots.
///
/// Private fields keep the definition/reference order and corresponding bytes
/// together. No media validity, host capability, runtime registration, identity,
/// persistence or activation is implied. Not a richer-pack round-trip model.
pub struct LocalMinimalRoleSnapshot {
    definition: MinimalRoleDefinition,
    asset_bytes: Vec<Vec<u8>>,
}

impl LocalMinimalRoleSnapshot {
    #[must_use]
    pub fn definition(&self) -> &MinimalRoleDefinition {
        &self.definition
    }

    /// References and owned snapshots in authored order, including duplicates.
    /// Consumers may invoke a media adapter on each byte slice independently;
    /// unsupported media does not mutate or invalidate the logical definition.
    #[must_use]
    pub fn assets(&self) -> impl ExactSizeIterator<Item = (&str, &[u8])> {
        self.definition
            .visual_assets
            .iter()
            .zip(&self.asset_bytes)
            .map(|(reference, bytes)| (reference.as_str(), bytes.as_slice()))
    }
}

impl fmt::Debug for LocalMinimalRoleSnapshot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LocalMinimalRoleSnapshot")
            .field("asset_count", &self.asset_bytes.len())
            .finish_non_exhaustive()
    }
}

/// Load the existing logical JSON projection and its local assets without writes.
///
/// The caller selects `definition_reference` under `asset_root`; no filename or
/// pack-version envelope is imposed. Asset references are relative to `asset_root`,
/// **not** the JSON file's containing directory. Product formats/versions must be
/// adapted separately; unknown JSON fields are dropped by the shared parser.
///
/// Reuses the same portable paths, canonical containment, regular/nonempty-file
/// checks and bounded reader as [`load_minimal_role_local_assets`]. Definition and
/// asset byte budgets are independent host inputs (not total process RAM limits).
/// All budgets/references must satisfy that reader's policy. UTF-8 and logical
/// JSON validation precede asset loading. Failures return no partial snapshot.
///
/// The caller must prevent concurrent root/path/file mutation throughout the call.
/// This is not a filesystem sandbox or an atomic snapshot of a mutable directory.
/// Returned bytes do not change when the source is later modified. No URLs, media
/// decoders, role directories, default product fields or legacy `Role` are created.
/// The result is preparation only: it cannot be passed to today's lifecycle API.
///
/// # Errors
///
/// Returns sanitized stage/field/index diagnostics, never content or paths, for
/// invalid budgets/paths, file failures, invalid UTF-8/JSON or logical content.
///
/// ```no_run
/// use std::path::Path;
/// use oclive_validation::minimal_role_local_file::load_minimal_role_local_file;
/// let snapshot = load_minimal_role_local_file(
///     Path::new("installed-pack"), "content.json", 64 * 1024,
///     4 * 1024 * 1024, 16 * 1024 * 1024,
/// )?;
/// assert!(!snapshot.definition().persona_prompt.trim().is_empty());
/// assert_eq!(snapshot.assets().len(), snapshot.definition().visual_assets.len());
/// # Ok::<(), Vec<String>>(())
/// ```
pub fn load_minimal_role_local_file(
    asset_root: &Path,
    definition_reference: &str,
    max_definition_bytes: usize,
    max_asset_bytes: usize,
    max_total_asset_bytes: usize,
) -> Result<LocalMinimalRoleSnapshot, Vec<String>> {
    if !valid_local_byte_budgets(max_definition_bytes, max_definition_bytes)
        || !valid_local_byte_budgets(max_asset_bytes, max_total_asset_bytes)
    {
        return Err(definition_error("invalid caller byte budgets"));
    }
    if !valid_local_reference(definition_reference) {
        return Err(definition_error(
            "expected a portable relative definition path",
        ));
    }
    let root = canonical_asset_root(asset_root).map_err(definition_error)?;
    let bytes = read_local_asset(&root, definition_reference, max_definition_bytes)
        .map_err(definition_error)?;
    let raw = std::str::from_utf8(&bytes)
        .map_err(|_| definition_error("definition must be valid UTF-8"))?;
    let definition = parse_minimal_role_definition(raw)?;
    // The parsed DTO owns the needed text; release the source buffer before assets.
    drop(bytes);
    let asset_bytes =
        load_minimal_role_local_assets(&root, &definition, max_asset_bytes, max_total_asset_bytes)?;
    Ok(LocalMinimalRoleSnapshot {
        definition,
        asset_bytes,
    })
}

fn definition_error(reason: &str) -> Vec<String> {
    vec![format!("minimal role local file: {reason}")]
}
