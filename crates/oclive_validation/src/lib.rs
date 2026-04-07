//! 角色包 manifest 校验与磁盘 DTO（与 oclivenewnew 运行时 serde 一致）。
//!
//! - **native**：`validate_disk_manifest` 等。
//! - **wasm**（`--features wasm`，目标 `wasm32-unknown-unknown`）：`validate_manifest_wasm`。

pub mod manifest;
pub mod validate;

pub use manifest::{
    DiskRoleManifest, EvolutionConfigDisk, IdentityBinding, KnowledgePackConfigDisk,
    LifeAvailability, LifeScheduleDisk, LifeScheduleEntryDisk, LifeTrajectoryDisk,
    MemoryConfigDisk, UserRelationDisk,
};
pub use validate::{parse_hhmm, validate_disk_manifest, validate_knowledge_manifest_disk};

#[cfg(all(feature = "wasm", target_arch = "wasm32"))]
pub mod wasm_exports;
