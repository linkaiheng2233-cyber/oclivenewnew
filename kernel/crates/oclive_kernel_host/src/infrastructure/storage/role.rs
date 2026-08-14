use crate::domain::knowledge_loader::{load_knowledge_index, should_load_knowledge};
use crate::domain::role_manifest_validate::{
    log_plugin_backends_remote_missing_env, validate_disk_manifest, validate_role_interaction_mode,
};
use crate::error::{AppError, Result};
use crate::models::role_manifest_disk::disk_manifest_to_role;
use crate::models::{
    author_pack::AuthorPackFile, role_settings_disk::CURRENT_SETTINGS_SCHEMA_VERSION,
    AdultRoleExtension, DiskRoleManifest, DiskRoleSettings, PortraitCatalogFile, Role,
    RolePackConfigFile, UiConfig,
};
use oclive_validation::{
    blueprint_schema_version_from_raw, load_blueprint_v2_for_role_dir,
    load_blueprint_v3_for_role_dir, load_blueprint_v4_for_role_dir,
    slot_registry_to_plugin_backends, validate_min_runtime_version,
    validate_settings_schema_version, validate_settings_top_level_keys, BlueprintExtensionDecl,
    RuntimeConfig, SlotGroupEntry, SlotRegistryEntry, BLUEPRINT_V2_SCHEMA_VERSION,
    BLUEPRINT_V3_SCHEMA_VERSION, BLUEPRINT_V4_SCHEMA_VERSION, PIPELINE_BLUEPRINT_FILENAME,
};
use serde_json;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use super::user_identities::load_user_identity_catalog;
use super::{apply_llm_backend_env_override, RoleStorage};

struct BlueprintRoleFields {
    slot_registry: BTreeMap<String, SlotRegistryEntry>,
    groups: BTreeMap<String, SlotGroupEntry>,
    runtime_config: Option<RuntimeConfig>,
    extensions: BTreeMap<String, BlueprintExtensionDecl>,
    interaction_mode: Option<String>,
    featured: bool,
    deep_capsule_enabled: bool,
    preset_order: u32,
    remote_presence: Option<oclive_validation::RemotePresenceConfig>,
    autonomous_scene: Option<oclive_validation::AutonomousSceneConfig>,
    reply_quality_anchor: Option<String>,
}

/// Non-role entries under `roles/` root (runtime data, shared docs/templates).
fn should_skip_roles_subdir(name: &str) -> bool {
    if name.starts_with('.') {
        return true;
    }
    name == "blueprint"
}

impl RoleStorage {
    /// Creates a new role storage instance.
    pub fn new(roles_dir: impl AsRef<Path>) -> Self {
        Self {
            roles_dir: roles_dir.as_ref().to_path_buf(),
        }
    }

    #[must_use]
    pub fn roles_dir(&self) -> &Path {
        &self.roles_dir
    }

    /// Resolve a validated `roles/{role_id}` path and reject symlink escapes.
    ///
    /// # Errors
    ///
    /// Returns [`AppError::InvalidParameter`] when `role_id` is invalid or an existing role
    /// directory resolves outside the configured roles root. Returns [`AppError::IoError`] when
    /// an existing path cannot be canonicalized.
    pub fn role_dir_path(&self, role_id: &str) -> Result<PathBuf> {
        oclive_validation::validate_role_id(role_id).map_err(AppError::InvalidParameter)?;
        let candidate = self.roles_dir.join(role_id);
        if candidate.exists() {
            let root = self.roles_dir.canonicalize().map_err(AppError::IoError)?;
            let resolved = candidate.canonicalize().map_err(AppError::IoError)?;
            if !resolved.starts_with(&root) {
                return Err(AppError::InvalidParameter(
                    "role directory escapes roles root".into(),
                ));
            }
            return Ok(resolved);
        }
        Ok(candidate)
    }

    /// Resolve a role asset while rejecting absolute paths, `..`, and symlink escapes.
    ///
    /// # Errors
    ///
    /// Returns [`AppError::InvalidParameter`] when the role id or relative asset path is invalid,
    /// or when an existing asset resolves outside its role directory. Returns
    /// [`AppError::IoError`] when an existing path cannot be canonicalized.
    pub fn role_asset_path(&self, role_id: &str, relative: &str) -> Result<PathBuf> {
        let role_dir = self.role_dir_path(role_id)?;
        let normalized = relative.replace('\\', "/");
        let rel = Path::new(&normalized);
        if normalized.trim().is_empty()
            || normalized.starts_with('/')
            || normalized.contains(':')
            || rel.is_absolute()
            || rel
                .components()
                .any(|part| !matches!(part, std::path::Component::Normal(_)))
        {
            return Err(AppError::InvalidParameter(
                "role asset path must be a portable relative path without roots, drive prefixes, '.' or '..'".into(),
            ));
        }
        let candidate = role_dir.join(rel);
        if candidate.exists() {
            let resolved = candidate.canonicalize().map_err(AppError::IoError)?;
            if !resolved.starts_with(&role_dir) {
                return Err(AppError::InvalidParameter(
                    "role asset escapes role directory".into(),
                ));
            }
            return Ok(resolved);
        }
        Ok(candidate)
    }

    /// Loads all roles.
    ///
    /// # Returns
    ///
    /// A list of all available roles.
    ///
    /// # Errors
    ///
    /// Returns [`AppError::IoError`] if the directory is missing or unreadable.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use oclive_kernel_host::infrastructure::RoleStorage;
    /// let storage = RoleStorage::new("./roles");
    /// let roles = storage.load_all_roles().expect("load roles");
    /// let _ = roles.len();
    /// ```
    pub fn load_all_roles(&self) -> Result<Vec<Role>> {
        let mut roles = Vec::new();

        if !self.roles_dir.exists() {
            tracing::warn!(
                target: "oclive_roles",
                "roles_dir does not exist: {} — list_roles will be empty; set OCLIVE_ROLES_DIR or fix cwd / exe-relative discovery",
                self.roles_dir.display()
            );
            return Ok(roles);
        }

        for entry in fs::read_dir(&self.roles_dir).map_err(AppError::IoError)? {
            let entry = entry.map_err(AppError::IoError)?;
            let path = entry.path();

            if path.is_dir() && !entry.file_type().map_err(AppError::IoError)?.is_symlink() {
                let dir_name = entry.file_name().to_string_lossy().into_owned();
                if should_skip_roles_subdir(&dir_name) {
                    tracing::debug!(
                        target: "oclive_role",
                        "skip non-role directory under roles/: {}",
                        path.display()
                    );
                    continue;
                }
                match self.load_role_from_dir(&path) {
                    Ok(role) => roles.push(role),
                    Err(e) => {
                        tracing::warn!(
                            target: "oclive_role",
                            "skip role directory {}: {}",
                            path.display(),
                            e
                        );
                    }
                }
            }
        }

        Ok(roles)
    }
    /// # Errors
    ///
    /// Returns [`Err`] with a human-readable message when the operation fails.
    /// Loads a single role from a directory (prefers `pipeline.ocblueprint`, otherwise legacy manifest/settings).
    pub fn load_role_from_dir(&self, role_dir: &Path) -> Result<Role> {
        if role_dir.join(PIPELINE_BLUEPRINT_FILENAME).is_file() {
            return self.load_role_from_blueprint_dir(role_dir);
        }
        self.load_role_from_legacy_manifest_dir(role_dir)
    }

    /// v2/v3/v4 blueprint pack: load by exact `schema_version` dispatch.
    fn load_role_from_blueprint_dir(&self, role_dir: &Path) -> Result<Role> {
        let blueprint_path = role_dir.join(PIPELINE_BLUEPRINT_FILENAME);
        let raw = std::fs::read_to_string(&blueprint_path).map_err(AppError::IoError)?;
        let version = blueprint_schema_version_from_raw(&raw).unwrap_or(0);
        match version {
            BLUEPRINT_V2_SCHEMA_VERSION => self.load_role_from_blueprint_v2_dir(role_dir),
            BLUEPRINT_V3_SCHEMA_VERSION => self.load_role_from_blueprint_v3_dir(role_dir),
            BLUEPRINT_V4_SCHEMA_VERSION => self.load_role_from_blueprint_v4_dir(role_dir),
            unsupported => Err(AppError::InvalidParameter(format!(
                "pipeline.ocblueprint：不支持的 schema_version {unsupported}（支持 {BLUEPRINT_V2_SCHEMA_VERSION}、{BLUEPRINT_V3_SCHEMA_VERSION} 或 {BLUEPRINT_V4_SCHEMA_VERSION}）"
            ))),
        }
    }

    /// v3 blueprint pack: loads `runtime_config` and `pipeline.experimental`.
    ///
    /// FROZEN-DRAFT (2026-06-01, see the freeze-decision section in
    /// handoff/TECHNICAL_DEBT_INVENTORY.md): v2 remains SSOT. Do not grow the v3
    /// schema / migration surface until v2 genuinely cannot express a real need.
    fn load_role_from_blueprint_v3_dir(&self, role_dir: &Path) -> Result<Role> {
        let loaded = load_blueprint_v3_for_role_dir(role_dir, env!("CARGO_PKG_VERSION")).map_err(
            |errs| {
                AppError::InvalidParameter(format!(
                    "pipeline.ocblueprint (v3) 校验失败:\n{}",
                    errs.join("\n")
                ))
            },
        )?;
        let mut role = disk_manifest_to_role(&loaded.disk);
        Self::apply_blueprint_role_fields(
            &mut role,
            BlueprintRoleFields {
                slot_registry: loaded.slot_registry,
                groups: loaded.groups,
                runtime_config: loaded.runtime_config,
                extensions: BTreeMap::new(),
                interaction_mode: loaded.interaction_mode,
                featured: loaded.featured,
                deep_capsule_enabled: loaded.deep_capsule_enabled,
                preset_order: loaded.preset_order,
                remote_presence: loaded.remote_presence,
                autonomous_scene: loaded.autonomous_scene,
                reply_quality_anchor: loaded.reply_quality_anchor,
            },
        );
        role.pipeline_experimental = if loaded.pipeline_experimental.is_empty() {
            None
        } else {
            Some(loaded.pipeline_experimental)
        };
        self.finish_role_pack_load(role_dir, &loaded.disk, role, None)
    }

    /// v2 blueprint pack: validates `pipeline.ocblueprint`, then fills `Role.slot_registry` / `plugin_backends` / `slot_groups`.
    fn load_role_from_blueprint_v2_dir(&self, role_dir: &Path) -> Result<Role> {
        // 1) Disk JSON → validated LoadedBlueprintV2 (slot_registry + groups + disk manifest fields)
        let loaded = load_blueprint_v2_for_role_dir(role_dir, env!("CARGO_PKG_VERSION")).map_err(
            |errs| {
                AppError::InvalidParameter(format!(
                    "pipeline.ocblueprint 校验失败:\n{}",
                    errs.join("\n")
                ))
            },
        )?;

        // 2) Compose runtime Role: six-slot summary (plugin_backends) + full registry for multi-instance resolution
        let mut role = disk_manifest_to_role(&loaded.disk);
        Self::apply_blueprint_role_fields(
            &mut role,
            BlueprintRoleFields {
                slot_registry: loaded.slot_registry,
                groups: loaded.groups,
                runtime_config: None,
                extensions: BTreeMap::new(),
                interaction_mode: loaded.interaction_mode,
                featured: loaded.featured,
                deep_capsule_enabled: loaded.deep_capsule_enabled,
                preset_order: loaded.preset_order,
                remote_presence: loaded.remote_presence,
                autonomous_scene: loaded.autonomous_scene,
                reply_quality_anchor: loaded.reply_quality_anchor,
            },
        );

        self.finish_role_pack_load(role_dir, &loaded.disk, role, None)
    }

    /// Stable v4 blueprint pack: activates `runtime_config` and preserves
    /// extension declarations. Required capability enforcement belongs to the
    /// host activation plan so metadata remains readable for repair.
    fn load_role_from_blueprint_v4_dir(&self, role_dir: &Path) -> Result<Role> {
        let loaded = load_blueprint_v4_for_role_dir(role_dir, env!("CARGO_PKG_VERSION")).map_err(
            |errors| {
                AppError::InvalidParameter(format!(
                    "pipeline.ocblueprint (v4) 校验失败:\n{}",
                    errors.join("\n")
                ))
            },
        )?;
        if !loaded.extensions.is_empty() {
            tracing::info!(
                target: "oclive_role",
                role_dir = %role_dir.display(),
                extension_count = loaded.extensions.len(),
                "preserving v4 extension declarations for host plan compilation"
            );
        }

        let mut role = disk_manifest_to_role(&loaded.disk);
        Self::apply_blueprint_role_fields(
            &mut role,
            BlueprintRoleFields {
                slot_registry: loaded.slot_registry,
                groups: loaded.groups,
                runtime_config: loaded.runtime_config,
                extensions: loaded.extensions,
                interaction_mode: loaded.interaction_mode,
                featured: loaded.featured,
                deep_capsule_enabled: loaded.deep_capsule_enabled,
                preset_order: loaded.preset_order,
                remote_presence: loaded.remote_presence,
                autonomous_scene: loaded.autonomous_scene,
                reply_quality_anchor: loaded.reply_quality_anchor,
            },
        );
        self.finish_role_pack_load(role_dir, &loaded.disk, role, None)
    }

    fn apply_blueprint_role_fields(role: &mut Role, fields: BlueprintRoleFields) {
        role.plugin_backends = Arc::new(slot_registry_to_plugin_backends(&fields.slot_registry));
        role.slot_registry = Some(fields.slot_registry);
        role.slot_groups = (!fields.groups.is_empty()).then_some(fields.groups);
        role.runtime_config = fields.runtime_config;
        role.blueprint_extensions = fields.extensions;
        role.interaction_mode = fields.interaction_mode;
        role.featured = fields.featured;
        role.deep_capsule_enabled = fields.deep_capsule_enabled;
        role.preset_order = fields.preset_order;
        role.remote_presence = fields.remote_presence;
        role.autonomous_scene = fields.autonomous_scene;
        role.reply_quality_anchor = fields.reply_quality_anchor;

        if let Some((_, entry)) = role.slot_registry.as_ref().and_then(|registry| {
            registry.iter().find(|(_, entry)| {
                entry.slot_type.trim() == "llm"
                    && entry.backend.trim() == "ollama"
                    && entry
                        .model
                        .as_ref()
                        .is_some_and(|model| !model.trim().is_empty())
            })
        }) {
            role.ollama_model.clone_from(&entry.model);
        }
    }

    fn load_role_from_legacy_manifest_dir(&self, role_dir: &Path) -> Result<Role> {
        let manifest_path = role_dir.join("manifest.json");

        if !manifest_path.exists() {
            return Err(AppError::RoleNotFound(format!(
                "manifest.json or {} not found in {:?} (roles_dir={})",
                PIPELINE_BLUEPRINT_FILENAME,
                role_dir,
                self.roles_dir.display()
            )));
        }

        let manifest_content = fs::read_to_string(&manifest_path).map_err(AppError::IoError)?;

        let mut disk: DiskRoleManifest = serde_json::from_str(&manifest_content)
            .map_err(|e| AppError::InvalidParameter(format!("manifest.json: {e}")))?;

        let mut settings_opt: Option<DiskRoleSettings> = None;
        let settings_path = role_dir.join("settings.json");
        if settings_path.exists() {
            let settings_content = fs::read_to_string(&settings_path).map_err(AppError::IoError)?;
            let settings_value: serde_json::Value =
                serde_json::from_str(&settings_content).map_err(AppError::SerializationError)?;
            if let serde_json::Value::Object(ref map) = settings_value {
                validate_settings_top_level_keys(map).map_err(AppError::InvalidParameter)?;
            }
            let settings: DiskRoleSettings =
                serde_json::from_value(settings_value).map_err(AppError::SerializationError)?;
            validate_settings_schema_version(
                settings.schema_version,
                CURRENT_SETTINGS_SCHEMA_VERSION,
            )
            .map_err(AppError::InvalidParameter)?;
            settings.apply_to_manifest(&mut disk);
            settings_opt = Some(settings);
        }

        let merged_scenes = Self::merge_scene_ids(role_dir, &disk.scenes)?;
        validate_disk_manifest(&disk, &merged_scenes).map_err(AppError::InvalidParameter)?;
        validate_min_runtime_version(
            disk.min_runtime_version.as_deref(),
            env!("CARGO_PKG_VERSION"),
        )
        .map_err(AppError::InvalidParameter)?;

        let mut role = disk_manifest_to_role(&disk);
        if let Some(ref s) = settings_opt {
            role.remote_presence = s.remote_presence.clone();
            role.autonomous_scene = s.autonomous_scene.clone();
            role.interaction_mode = s.interaction_mode.clone();
            if let Some(ref pb) = s.plugin_backends {
                role.plugin_backends = Arc::new(
                    oclive_validation::sanitize_unimplemented_agent_backend(pb.clone()).backends,
                );
            }
            role.reply_quality_anchor = s.reply_quality_anchor.clone();
        }
        self.finish_role_pack_load(role_dir, &disk, role, settings_opt.as_ref())
    }

    fn finish_role_pack_load(
        &self,
        role_dir: &Path,
        disk: &DiskRoleManifest,
        mut role: Role,
        _settings: Option<&DiskRoleSettings>,
    ) -> Result<Role> {
        role.ui_config = UiConfig::load_from_path(&role_dir.join("ui.json"));
        let author_path = role_dir.join("author.json");
        if author_path.is_file() {
            match std::fs::read_to_string(&author_path) {
                Ok(s) => {
                    role.author_pack = AuthorPackFile::from_json_str(&s);
                    if role.author_pack.is_none() {
                        tracing::warn!(
                            target: "oclive_role",
                            "author.json invalid JSON: {}",
                            author_path.display()
                        );
                    }
                }
                Err(e) => {
                    tracing::warn!(
                        target: "oclive_role",
                        "author.json unreadable: {} — {}",
                        author_path.display(),
                        e
                    );
                }
            }
        }
        if should_load_knowledge(disk, role_dir) {
            let idx = load_knowledge_index(role_dir, disk)?;
            role.knowledge_index = Some(Arc::new(idx));
        }
        let config_path = role_dir.join("config.json");
        if config_path.is_file() {
            match fs::read_to_string(&config_path) {
                Ok(s) => match serde_json::from_str::<RolePackConfigFile>(&s) {
                    Ok(cfg) => {
                        role.time_config = cfg.time;
                        role.pack_memory_config = cfg.memory;
                        role.pack_relation_config = cfg.relation;
                        role.pack_evolution_config = cfg.evolution;
                        role.pack_chat_storage_config = cfg.chat_storage;
                        role.pack_reply_post_processor_config = cfg.reply_post_processor;
                        role.pack_portrait_catalog = cfg.portrait_catalog;
                        role.pack_visual_presentation_config = cfg.visual_presentation;
                        role.pack_turn_thinking_config = cfg.turn_thinking;
                        role.pack_prompt_extra_sections = cfg.prompt_extra_sections;
                        if role.pack_portrait_catalog.enabled {
                            let catalog_path = role_dir.join("portrait_catalog.json");
                            if catalog_path.is_file() {
                                match fs::read_to_string(&catalog_path) {
                                    Ok(s) => {
                                        match serde_json::from_str::<PortraitCatalogFile>(&s) {
                                            Ok(catalog) => {
                                                role.portrait_catalog = Some(catalog);
                                            }
                                            Err(e) => tracing::warn!(
                                                target: "oclive_role",
                                                "portrait_catalog.json parse failed: {} — {}",
                                                catalog_path.display(),
                                                e
                                            ),
                                        }
                                    }
                                    Err(e) => tracing::warn!(
                                        target: "oclive_role",
                                        "portrait_catalog.json unreadable: {} — {}",
                                        catalog_path.display(),
                                        e
                                    ),
                                }
                            } else {
                                tracing::warn!(
                                    target: "oclive_role",
                                    "portrait_catalog.enabled but missing {}; legacy portrait path",
                                    catalog_path.display()
                                );
                            }
                        }
                    }
                    Err(e) => tracing::warn!(
                        target: "oclive_role",
                        "config.json parse failed: {} — {}",
                        config_path.display(),
                        e
                    ),
                },
                Err(e) => tracing::warn!(
                    target: "oclive_role",
                    "config.json unreadable: {} — {}",
                    config_path.display(),
                    e
                ),
            }
        }
        apply_llm_backend_env_override(&mut role);
        validate_role_interaction_mode(&role).map_err(AppError::InvalidParameter)?;
        log_plugin_backends_remote_missing_env(&role);

        let scene_list = Self::merge_scene_ids(role_dir, &disk.scenes)?;
        let adult_extension_path = role_dir.join("adult_extension.json");
        if adult_extension_path.is_file() {
            let extension = fs::read_to_string(&adult_extension_path)
                .map_err(|error| format!("adult_extension.json could not be read: {error}"))
                .and_then(|raw| {
                    serde_json::from_str::<AdultRoleExtension>(&raw)
                        .map_err(|error| format!("adult_extension.json parse failed: {error}"))
                })
                .and_then(|extension| {
                    extension
                        .validate(&scene_list)
                        .map_err(|errors| {
                            format!("adult_extension.json invalid: {}", errors.join("; "))
                        })
                        .map(|()| extension)
                });
            match extension {
                Ok(extension) => role.adult_extension = Some(extension),
                Err(error) => {
                    tracing::warn!(
                        target: "oclive_role",
                        role_id = %role.id,
                        path = %adult_extension_path.display(),
                        error = %error,
                        "adult extension disabled; ordinary role remains available"
                    );
                    role.adult_extension = None;
                    role.adult_extension_error = Some(error);
                }
            }
        }
        role.scene_ids = Arc::from(scene_list.into_boxed_slice());

        let core_personality_path = role_dir.join("core_personality.txt");
        if core_personality_path.exists() {
            role.core_personality =
                fs::read_to_string(&core_personality_path).map_err(AppError::IoError)?;
        }

        let memory_seed_path = role_dir.join("memory_seed.json");
        if memory_seed_path.is_file() {
            let raw = fs::read_to_string(&memory_seed_path).map_err(AppError::IoError)?;
            role.memory_seed = oclive_validation::parse_memory_seed(&raw)
                .map_err(|errors| AppError::InvalidParameter(errors.join("; ")))?
                .memories;
        }

        let deep_capsule_path = role_dir.join("prompts/deep_capsule.txt");
        if deep_capsule_path.is_file() {
            role.deep_capsule =
                Some(fs::read_to_string(&deep_capsule_path).map_err(AppError::IoError)?);
        }

        role.user_identity_catalog = load_user_identity_catalog(role_dir)?;
        role.source_dir = Some(role_dir.to_path_buf());

        Ok(role)
    }
    /// # Errors
    ///
    /// Returns [`Err`] with a human-readable message when the operation fails.
    /// Loads the specified role.
    pub fn load_role(&self, role_id: &str) -> Result<Role> {
        let rid = role_id.trim();
        if rid.is_empty() {
            return Err(AppError::RoleNotFound(format!(
                "role_id is empty; roles_dir={}",
                self.roles_dir.display()
            )));
        }
        let role_dir = self.role_dir_path(rid)?;
        self.load_role_from_dir(&role_dir)
    }
    /// # Errors
    ///
    /// Returns [`Err`] with a human-readable message when the operation fails.
    /// Scene id list: manifest top-level `scenes` array + `roles/{role_id}/scenes/` subdirectory names, deduplicated and sorted.
    /// Returns `["default"]` when both are empty.
    pub fn list_scene_ids(&self, role_id: &str) -> Result<Vec<String>> {
        let role_dir = self.role_dir_path(role_id)?;
        let manifest_path = role_dir.join("manifest.json");

        let manifest_scenes: Vec<String> = if manifest_path.exists() {
            let manifest_content = fs::read_to_string(&manifest_path).map_err(AppError::IoError)?;
            let disk: DiskRoleManifest =
                serde_json::from_str(&manifest_content).map_err(AppError::SerializationError)?;
            disk.scenes
        } else {
            vec![]
        };

        Self::merge_scene_ids(&role_dir, &manifest_scenes)
    }

    /// `manifest.scenes` + `scenes/` subdirectories, deduplicated and sorted; `["default"]` when both are empty (same as [`Self::list_scene_ids`]).
    fn merge_scene_ids(role_dir: &Path, manifest_scenes: &[String]) -> Result<Vec<String>> {
        let mut ids: BTreeSet<String> = BTreeSet::new();
        for s in manifest_scenes {
            if !s.trim().is_empty() {
                ids.insert(s.clone());
            }
        }

        let scenes_dir = role_dir.join("scenes");
        if scenes_dir.is_dir() {
            for entry in fs::read_dir(&scenes_dir).map_err(AppError::IoError)? {
                let entry = entry.map_err(AppError::IoError)?;
                let path = entry.path();
                if path.is_dir() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    if !name.starts_with('.') {
                        ids.insert(name);
                    }
                }
            }
        }

        if ids.is_empty() {
            ids.insert("default".to_string());
        }

        Ok(ids.into_iter().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::{should_skip_roles_subdir, RoleStorage};
    use std::fs;
    use std::path::Path;
    use tempfile::tempdir;

    fn write_v4_pack(role_dir: &Path, required: bool) {
        let extension_id = "com.example.live2d";
        let extension_dir = role_dir.join("blueprint/extensions").join(extension_id);
        fs::create_dir_all(&extension_dir).unwrap();
        fs::write(extension_dir.join("config.json"), r#"{"enabled":true}"#).unwrap();
        fs::write(
            role_dir.join(oclive_validation::PIPELINE_BLUEPRINT_FILENAME),
            serde_json::json!({
                "schema_version": 4,
                "meta": {
                    "id": role_dir.file_name().unwrap().to_string_lossy(),
                    "name": "V4",
                    "version": "1.0.0",
                    "author": "test",
                    "description": "test",
                    "relations": {
                        "friend": {
                            "initial_favorability": 50,
                            "favor_multiplier": 1
                        }
                    },
                    "default_relation": "friend"
                },
                "slot_registry": {
                    "llm": {
                        "type": "llm",
                        "label": "LLM",
                        "backend": "ollama",
                        "position": 0,
                        "model": "qwen:test"
                    }
                },
                "runtime_config": {
                    "interaction_mode": "pure_chat"
                },
                "extensions": {
                    extension_id: {
                        "capability": extension_id,
                        "required": required,
                        "config_schema_version": 1,
                        "config_ref": format!(
                            "blueprint/extensions/{extension_id}/config.json"
                        )
                    }
                }
            })
            .to_string(),
        )
        .unwrap();
    }

    #[test]
    fn skip_reserved_roles_root_dirs() {
        assert!(should_skip_roles_subdir(".oclive_directory_plugin_data"));
        assert!(should_skip_roles_subdir("blueprint"));
        assert!(!should_skip_roles_subdir("mumu"));
    }

    #[test]
    fn role_asset_path_rejects_role_and_asset_traversal() {
        let roles = tempdir().unwrap();
        fs::create_dir_all(roles.path().join("mumu")).unwrap();
        let storage = RoleStorage::new(roles.path());

        assert!(storage
            .role_asset_path("../outside", "manifest.json")
            .is_err());
        assert!(storage.role_asset_path("mumu", "../outside.json").is_err());
        assert!(storage.role_asset_path("mumu", "C:\\outside.json").is_err());
        assert!(storage.role_asset_path("mumu", "C:/outside.json").is_err());
        assert!(storage.role_asset_path("mumu", "C:outside.json").is_err());
        assert!(storage
            .role_asset_path("mumu", "\\\\server\\share.json")
            .is_err());
        assert!(storage.role_asset_path("mumu", "/outside.json").is_err());
        assert_eq!(
            storage
                .role_asset_path("mumu", "scenes/default/scene.json")
                .unwrap(),
            roles
                .path()
                .join("mumu")
                .canonicalize()
                .unwrap()
                .join("scenes/default/scene.json")
        );
    }

    #[test]
    fn v4_loads_runtime_config_and_preserves_optional_extensions() {
        let roles = tempdir().unwrap();
        let role_dir = roles.path().join("demo");
        write_v4_pack(&role_dir, false);
        let role = RoleStorage::new(roles.path())
            .load_role_from_dir(&role_dir)
            .unwrap();
        assert_eq!(role.interaction_mode.as_deref(), Some("pure_chat"));
        assert_eq!(role.ollama_model.as_deref(), Some("qwen:test"));
        assert!(role.blueprint_extensions.contains_key("com.example.live2d"));
        assert!(role.pipeline_experimental.is_none());
    }

    #[test]
    fn v4_required_extension_metadata_loads_for_plan_diagnostics() {
        let roles = tempdir().unwrap();
        let role_dir = roles.path().join("required");
        write_v4_pack(&role_dir, true);
        let role = RoleStorage::new(roles.path())
            .load_role_from_dir(&role_dir)
            .unwrap();
        assert!(role.blueprint_extensions["com.example.live2d"].required);
    }

    #[test]
    fn invalid_adult_extension_is_isolated_from_ordinary_role_loading() {
        let roles = tempdir().unwrap();
        let role_dir = roles.path().join("broken-adult-extension");
        write_v4_pack(&role_dir, false);
        fs::write(role_dir.join("adult_extension.json"), "{not-json").unwrap();

        let loaded = RoleStorage::new(roles.path()).load_all_roles().unwrap();
        let role = loaded
            .iter()
            .find(|role| role.id == "broken-adult-extension")
            .expect("ordinary role remains listed");

        assert_eq!(role.name, "V4");
        assert!(role.adult_extension.is_none());
        assert!(role
            .adult_extension_error
            .as_deref()
            .is_some_and(|error| error.contains("parse failed")));
    }

    #[test]
    fn unknown_blueprint_version_does_not_fall_back_to_v2() {
        let roles = tempdir().unwrap();
        let role_dir = roles.path().join("unknown");
        fs::create_dir_all(&role_dir).unwrap();
        fs::write(
            role_dir.join(oclive_validation::PIPELINE_BLUEPRINT_FILENAME),
            r#"{"schema_version":99}"#,
        )
        .unwrap();
        let error = RoleStorage::new(roles.path())
            .load_role_from_dir(&role_dir)
            .unwrap_err()
            .to_string();
        assert!(error.contains("schema_version 99"), "{error}");
        assert!(error.contains("2、3 或 4"), "{error}");
    }
}
