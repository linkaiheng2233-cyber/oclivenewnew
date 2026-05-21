//! 角色包 manifest 校验与磁盘 DTO（与 oclivenewnew 运行时 serde 一致）。
//!
//! - **native**：`validate_disk_manifest` 等。
//! - **wasm**（`--features wasm`，目标 `wasm32-unknown-unknown`）：`validate_manifest_wasm`。

#![cfg_attr(
    test,
    allow(clippy::unwrap_used, clippy::expect_used)
)]

pub mod blueprint_migrate;
pub mod blueprint_v2;
pub mod disk_role_settings;
pub mod json_keys;
pub mod manifest;
pub mod plugin_backends;
pub mod plugin_dependencies;
pub mod plugin_permissions;
pub mod protocol_boundary;
pub mod role_pack;
pub mod validate;

pub use blueprint_migrate::{
    build_blueprint_v2_from_legacy_dir, migrate_role_pack_dir_to_blueprint_v2,
};
pub use blueprint_v2::{
    apply_slot_override, default_slot_key_for_module, effective_slot_registry,
    load_blueprint_v2_for_role_dir, merged_agent_directory_plugin_ids,
    plugin_backends_for_slot_entry, slot_registry_instances_sorted,
    slot_registry_to_plugin_backends,
    validate_blueprint_v2_json, validate_blueprint_v2_json_with_context,
    validate_role_pack_blueprint_v2_directory, write_role_pack_blueprint_slot_registry,
    BlueprintV2LoadResult, BlueprintV2ValidateContext, SlotOverridePatch, SlotRegistryEntry,
    BLUEPRINT_V2_SCHEMA_VERSION, PIPELINE_BLUEPRINT_FILENAME,
};
pub use disk_role_settings::{
    AutonomousSceneConfig, AutonomousSceneRule, DiskRoleSettings, RemotePresenceConfig,
    CURRENT_SETTINGS_SCHEMA_VERSION,
};
pub use json_keys::{validate_manifest_top_level_keys, validate_settings_top_level_keys};
pub use manifest::{
    DiskRoleManifest, EvolutionConfigDisk, IdentityBinding, KnowledgePackConfigDisk,
    LifeAvailability, LifeScheduleDisk, LifeScheduleEntryDisk, LifeTrajectoryDisk,
    MemoryConfigDisk, PersonalitySource, UserRelationDisk,
};
pub use plugin_dependencies::{
    parse_plugin_dependencies, resolve_install_order,
};
pub use plugin_permissions::{
    manifest_declares_process_spawn, validate_directory_plugin_manifest_permissions,
    validate_permissions_list, ALLOWED as PLUGIN_PERMISSIONS_ALLOWED, MCP_HTTP, MCP_STDIO,
    NETWORK_GRANT_REMOTE_LLM, NETWORK_GRANT_REMOTE_PLUGIN, NETWORK_WILDCARD, PROCESS_SPAWN,
};
pub use protocol_boundary::{
    assert_layers_do_not_overlap, validate_jsonrpc_error_response, validate_kernel_error_body,
    ProtocolValidationError,
};
pub use plugin_backends::{
    AgentBackend, DirectoryPluginSlots, EmotionBackend, EventBackend, LlmBackend, MemoryBackend,
    PluginBackendSource, PluginBackends, PluginBackendsOverride, PluginBackendsSourceMap,
    PromptBackend,
};
pub use role_pack::{
    merge_role_pack_scene_ids, validate_default_personality_vector,
    validate_role_pack_directory, validate_role_pack_directory_with_profile,
    validate_role_pack_loaded, validate_role_pack_loaded_with_profile,
    validate_role_pack_manifest_settings_core, validate_role_pack_tail, RolePackValidationProfile,
};
pub use validate::{
    parse_hhmm, validate_disk_manifest, validate_interaction_mode_pack_setting,
    validate_knowledge_manifest_disk, validate_min_runtime_version,
    validate_min_runtime_version_for_local_plugin, validate_settings_schema_version,
};

#[cfg(all(feature = "wasm", target_arch = "wasm32"))]
pub mod wasm_exports;
