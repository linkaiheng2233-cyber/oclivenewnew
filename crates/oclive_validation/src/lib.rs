//! Role pack manifest validation and on-disk DTOs (matching the oclivenewnew runtime serde).
//!
//! - **native**: `validate_disk_manifest`, etc.
//! - **wasm** (`--features wasm`, target `wasm32-unknown-unknown`): `validate_manifest_wasm`.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]

pub mod agent_backend;
pub mod blueprint_includes;
pub mod blueprint_migrate;
pub mod blueprint_v2;
pub mod blueprint_v3;
pub mod creator_profile;
pub mod disk_role_settings;
pub mod expert_actions;
pub mod expert_routing;
pub mod json_keys;
pub mod manifest;
pub mod pipeline_action;
pub mod plugin_backends;
pub mod plugin_dependencies;
pub mod plugin_permissions;
pub mod plugin_slot_attachment;
pub mod protocol_boundary;
pub mod reply_post_processor;
pub mod role_pack;
pub mod runtime_config;
pub mod user_identities;
pub mod validate;

pub use agent_backend::{
    sanitize_unimplemented_agent_backend, validate_agent_slot_backends,
    validate_implemented_agent_backend, AgentBackendSanitizeResult,
};
pub use blueprint_includes::{
    resolve_blueprint_includes_lenient, resolve_blueprint_includes_strict, validate_includes,
    BlueprintIncludeEntry,
};
pub use blueprint_migrate::{
    build_blueprint_v2_from_legacy_dir, migrate_role_pack_dir_to_blueprint_v2,
};
pub use blueprint_v2::{
    apply_slot_override, default_slot_key_for_module, effective_slot_registry,
    load_blueprint_v2_for_role_dir, merged_agent_directory_plugin_ids,
    plugin_backends_for_slot_entry, slot_registry_instances_sorted,
    slot_registry_to_plugin_backends, validate_blueprint_v2_json,
    validate_blueprint_v2_json_with_context, validate_meta_personality,
    validate_role_pack_blueprint_v2_directory, write_role_pack_blueprint_slot_registry,
    BlueprintV2LoadResult, BlueprintV2ValidateContext, SlotGroupEntry, SlotOverridePatch,
    SlotRegistryEntry, BLUEPRINT_V2_SCHEMA_VERSION, GROUP_SLOT_TYPES, PIPELINE_BLUEPRINT_FILENAME,
};
pub use blueprint_v3::{
    blueprint_schema_version_from_raw, load_blueprint_v3_for_role_dir,
    validate_blueprint_json_by_schema_version, validate_blueprint_v3_json,
    validate_role_pack_blueprint_v3_directory, BlueprintV3LoadResult, DualPipelineDef,
    PipelineStep, BLUEPRINT_V3_SCHEMA_VERSION, PLUGIN_HOST_SLOT_TYPES,
};
pub use creator_profile::validate_role_pack_creator_directory;
pub use disk_role_settings::{
    AutonomousSceneConfig, AutonomousSceneRule, DiskRoleSettings, RemotePresenceConfig,
    CURRENT_SETTINGS_SCHEMA_VERSION,
};
pub use expert_actions::{
    parse_expert_step_action, validate_expert_step_action, ExpertStepActionKind,
    EXPERT_ACTION_EXPERT_FALLBACK, EXPERT_ACTION_LORA_APPLY, EXPERT_ACTION_MEMORY_INJECT,
    EXPERT_ACTION_PERSONALITY_ADJUST, EXPERT_ACTION_PROMPT_ENHANCE,
};
pub use expert_routing::{
    load_expert_routing_from_role_dir, match_expert_route, select_expert_route, trigger_matches,
    validate_expert_routing_doc, ExpertFallback, ExpertMatchContext, ExpertRoute, ExpertRouteStep,
    ExpertRoutingDoc, ExpertTrigger, MessageLengthRange, TimeOfDayWindow, TriggerCondition,
    DEFAULT_EXPERT_ROUTING_PATH,
};
pub use json_keys::{validate_manifest_top_level_keys, validate_settings_top_level_keys};
pub use manifest::{
    DiskRoleManifest, EvolutionConfigDisk, IdentityBinding, KnowledgePackConfigDisk,
    LifeAvailability, LifeScheduleDisk, LifeScheduleEntryDisk, LifeTrajectoryDisk,
    MemoryConfigDisk, PersonalitySource, UserRelationDisk,
};
pub use pipeline_action::{
    parse_pipeline_action, parse_pipeline_action_kind, PipelineActionKind,
    PIPELINE_ACTION_EXPERT_INVOKE,
};
pub use plugin_backends::{
    AgentBackend, DirectoryPluginSlots, EmotionBackend, EventBackend, LlmBackend, MemoryBackend,
    PluginBackendSource, PluginBackends, PluginBackendsOverride, PluginBackendsSourceMap,
    PromptBackend,
};
pub use plugin_dependencies::{parse_plugin_dependencies, resolve_install_order};
pub use plugin_permissions::{
    manifest_declares_process_spawn, validate_directory_plugin_manifest_permissions,
    validate_permissions_list, ALLOWED as PLUGIN_PERMISSIONS_ALLOWED, MCP_HTTP, MCP_STDIO,
    NETWORK_GRANT_REMOTE_LLM, NETWORK_GRANT_REMOTE_PLUGIN, NETWORK_WILDCARD, PROCESS_SPAWN,
};
pub use plugin_slot_attachment::{
    apply_slot_attachments_to_registry, parse_slot_attachments_from_manifest_json,
    validate_slot_attachment_decl, SlotAttachmentDecl,
};
pub use protocol_boundary::{
    assert_layers_do_not_overlap, validate_jsonrpc_error_response, validate_kernel_error_body,
    ProtocolValidationError,
};
pub use reply_post_processor::{
    validate_reply_post_processor_config, validate_reply_post_processor_config_file,
};
pub use role_pack::{
    merge_role_pack_scene_ids, validate_default_personality_vector, validate_role_pack_directory,
    validate_role_pack_directory_with_profile, validate_role_pack_loaded,
    validate_role_pack_loaded_with_profile, validate_role_pack_manifest_settings_core,
    validate_role_pack_tail, RolePackValidationProfile,
};
pub use runtime_config::{DualCoreConfig, RuntimeConfig};
pub use user_identities::validate_user_identities_directory;
#[doc(hidden)]
pub use user_identities::validate_user_identities_index;
pub use validate::{
    parse_hhmm, validate_disk_manifest, validate_interaction_mode_pack_setting,
    validate_knowledge_manifest_disk, validate_min_runtime_version,
    validate_min_runtime_version_for_local_plugin, validate_settings_schema_version,
};

#[cfg(all(feature = "wasm", target_arch = "wasm32"))]
pub mod wasm_exports;
