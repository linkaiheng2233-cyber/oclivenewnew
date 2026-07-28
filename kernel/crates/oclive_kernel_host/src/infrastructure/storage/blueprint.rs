use super::RoleStorage;
use crate::error::{AppError, Result};
use oclive_validation::{
    merge_blueprint_includes_lenient, write_role_pack_blueprint_slot_registry, SlotRegistryEntry,
    PIPELINE_BLUEPRINT_FILENAME,
};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

/// Best-effort `includes[]` merge for non-activating previews.
///
/// Production role activation uses the strict `oclive_validation::load_blueprint_v2/v3_for_role_dir`
/// paths; missing or malformed satellite files block activation.
#[must_use]
#[allow(dead_code)] // For RoleStorage / toolchain explicit calls; host default uses oclive_validation load_* path
pub fn merge_blueprint_includes_for_role_dir(role_dir: &Path, raw: &str) -> String {
    merge_blueprint_includes_lenient(role_dir, raw)
}

impl RoleStorage {
    /// # Errors
    ///
    /// Returns [`Err`] with a human-readable message when the operation fails.
    /// Write `slot_registry` back to `roles/{role_id}/pipeline.ocblueprint` (must be a v2 blueprint pack).
    pub fn save_blueprint_v2_slot_registry(
        &self,
        role_id: &str,
        registry: &BTreeMap<String, SlotRegistryEntry>,
    ) -> Result<()> {
        let role_dir = self.role_dir_path(role_id)?;
        if !role_dir.join(PIPELINE_BLUEPRINT_FILENAME).is_file() {
            return Err(AppError::InvalidParameter(format!(
                "角色 {role_id} 无 {PIPELINE_BLUEPRINT_FILENAME}，无法写 slot_registry"
            )));
        }
        write_role_pack_blueprint_slot_registry(&role_dir, registry, env!("CARGO_PKG_VERSION"))
            .map_err(|errs| {
                AppError::InvalidParameter(format!(
                    "pipeline.ocblueprint slot_registry 校验失败:\n{}",
                    errs.join("\n")
                ))
            })?;
        Ok(())
    }

    /// # Errors
    ///
    /// Returns [`Err`] with a human-readable message when the operation fails.
    /// Save core persona (creator-editable only).
    pub fn save_core_personality(&self, role_id: &str, content: &str) -> Result<()> {
        let role_dir = self.role_dir_path(role_id)?;
        let core_personality_path = role_dir.join("core_personality.txt");

        fs::write(&core_personality_path, content).map_err(AppError::IoError)?;

        Ok(())
    }
}
