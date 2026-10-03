//! Preparation of developer-converted content for the reference Host's text path.

use oclive_kernel_types::{AppError, MinimalRoleDefinition, Result};
use oclive_validation::minimal_role_local_file::LocalMinimalRoleSnapshot;
use oclive_validation::validate_minimal_role_definition;

/// Immutable Host-owned minimal content and technical identity.
///
/// Developers own format conversion, asset resolution and byte budgets. This
/// handle does not activate a rich role, change selection, or write runtime state.
/// Asset snapshots establish nonempty bytes, not media validity or rendering.
pub struct PreparedMinimalRole {
    technical_id: String,
    definition: MinimalRoleDefinition,
    asset_bytes: Vec<Vec<u8>>,
}

impl PreparedMinimalRole {
    /// Prepare converter output. `asset_bytes` corresponds positionally to every
    /// authored reference, including duplicates; references need not be paths.
    ///
    /// # Errors
    /// Returns InvalidParameter for blank identity, invalid logical content,
    /// asset-count mismatch or empty asset snapshots. No partial handle is returned.
    pub fn new(
        technical_id: impl Into<String>,
        definition: MinimalRoleDefinition,
        asset_bytes: Vec<Vec<u8>>,
    ) -> Result<Self> {
        let technical_id = technical_id.into();
        if technical_id.trim().is_empty() {
            return Err(AppError::InvalidParameter(
                "minimal role technical ID must not be blank".into(),
            ));
        }
        validate_minimal_role_definition(&definition)
            .map_err(|errors| AppError::InvalidParameter(errors.join("; ")))?;
        if asset_bytes.len() != definition.visual_assets.len()
            || asset_bytes.iter().any(Vec::is_empty)
        {
            return Err(AppError::InvalidParameter(
                "minimal role needs one nonempty snapshot per asset reference".into(),
            ));
        }
        Ok(Self {
            technical_id,
            definition,
            asset_bytes,
        })
    }

    /// Adapt the optional local reader's validated snapshots, without rereading
    /// files or adopting its JSON layout as a universal distro format.
    ///
    /// # Errors
    /// Returns the same preparation errors as [`Self::new`].
    pub fn from_local_snapshot(
        technical_id: impl Into<String>,
        snapshot: &LocalMinimalRoleSnapshot,
    ) -> Result<Self> {
        Self::new(
            technical_id,
            snapshot.definition().clone(),
            snapshot.assets().map(|(_, bytes)| bytes.to_vec()).collect(),
        )
    }

    #[must_use]
    pub fn technical_id(&self) -> &str {
        &self.technical_id
    }

    #[must_use]
    pub fn definition(&self) -> &MinimalRoleDefinition {
        &self.definition
    }

    /// Readonly reference/byte pairs, preserving authored order.
    #[must_use = "iterate over the snapshots or explicitly discard the iterator"]
    pub fn assets(&self) -> impl ExactSizeIterator<Item = (&str, &[u8])> {
        self.definition
            .visual_assets
            .iter()
            .zip(&self.asset_bytes)
            .map(|(reference, bytes)| (reference.as_str(), bytes.as_slice()))
    }
}
