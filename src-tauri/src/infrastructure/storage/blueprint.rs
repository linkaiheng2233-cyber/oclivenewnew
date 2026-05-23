use super::RoleStorage;
use crate::error::{AppError, Result};
use oclive_validation::{write_role_pack_blueprint_slot_registry, SlotRegistryEntry, PIPELINE_BLUEPRINT_FILENAME};
use std::collections::BTreeMap;
use std::fs;

impl RoleStorage {
    /// # Errors
    ///
    /// Returns [`Err`] with a human-readable message when the operation fails.
    /// 将 `slot_registry` 写回 `roles/{role_id}/pipeline.ocblueprint`（须为 v2 蓝图包）。
    pub fn save_blueprint_v2_slot_registry(
        &self,
        role_id: &str,
        registry: &BTreeMap<String, SlotRegistryEntry>,
    ) -> Result<()> {
        let role_dir = self.roles_dir.join(role_id);
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
    /// 保存核心人设（仅创作者可改）
    pub fn save_core_personality(&self, role_id: &str, content: &str) -> Result<()> {
        let role_dir = self.roles_dir.join(role_id);
        let core_personality_path = role_dir.join("core_personality.txt");

        fs::write(&core_personality_path, content).map_err(AppError::IoError)?;

        Ok(())
    }
}