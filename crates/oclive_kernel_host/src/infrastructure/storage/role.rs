use crate::domain::knowledge_loader::{load_knowledge_index, should_load_knowledge};
use crate::domain::role_manifest_validate::{
    log_plugin_backends_remote_missing_env, validate_disk_manifest, validate_role_interaction_mode,
};
use crate::error::{AppError, Result};
use crate::models::role_manifest_disk::disk_manifest_to_role;
use crate::models::{
    author_pack::AuthorPackFile, role_settings_disk::CURRENT_SETTINGS_SCHEMA_VERSION,
    DiskRoleManifest, DiskRoleSettings, Role, RolePackConfigFile, UiConfig,
};
use oclive_validation::{
    blueprint_schema_version_from_raw, load_blueprint_v2_for_role_dir,
    load_blueprint_v3_for_role_dir, slot_registry_to_plugin_backends, validate_min_runtime_version,
    validate_settings_schema_version, validate_settings_top_level_keys,
    BLUEPRINT_V3_SCHEMA_VERSION, PIPELINE_BLUEPRINT_FILENAME,
};
use serde_json;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use super::user_identities::load_user_identity_catalog;
use super::{apply_llm_backend_env_override, RoleStorage};

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

    /// `roles/{role_id}/{relative}`; existence is not checked.
    #[must_use]
    pub fn role_asset_path(&self, role_id: &str, relative: &str) -> PathBuf {
        self.roles_dir.join(role_id).join(relative)
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
    /// ```
    /// # use oclivenewnew_tauri::infrastructure::storage::RoleStorage;
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

            if path.is_dir() {
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
    /// Loads a single role from a directory (prefers `pipeline.ocblueprint` v2, otherwise legacy manifest/settings).
    pub fn load_role_from_dir(&self, role_dir: &Path) -> Result<Role> {
        if role_dir.join(PIPELINE_BLUEPRINT_FILENAME).is_file() {
            return self.load_role_from_blueprint_dir(role_dir);
        }
        self.load_role_from_legacy_manifest_dir(role_dir)
    }

    /// v2/v3 blueprint pack: load by `schema_version` dispatch.
    fn load_role_from_blueprint_dir(&self, role_dir: &Path) -> Result<Role> {
        let blueprint_path = role_dir.join(PIPELINE_BLUEPRINT_FILENAME);
        let raw = std::fs::read_to_string(&blueprint_path).map_err(AppError::IoError)?;
        let version = blueprint_schema_version_from_raw(&raw).unwrap_or(0);
        if version == BLUEPRINT_V3_SCHEMA_VERSION {
            return self.load_role_from_blueprint_v3_dir(role_dir);
        }
        self.load_role_from_blueprint_v2_dir(role_dir)
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
        role.plugin_backends = Arc::new(slot_registry_to_plugin_backends(&loaded.slot_registry));
        role.slot_registry = Some(loaded.slot_registry);
        role.slot_groups = if loaded.groups.is_empty() {
            None
        } else {
            Some(loaded.groups)
        };
        role.interaction_mode = loaded.interaction_mode;
        role.remote_presence = loaded.remote_presence;
        role.autonomous_scene = loaded.autonomous_scene;
        role.reply_quality_anchor = loaded.reply_quality_anchor;
        role.runtime_config = loaded.runtime_config;
        role.pipeline_experimental = if loaded.pipeline_experimental.is_empty() {
            None
        } else {
            Some(loaded.pipeline_experimental)
        };
        for entry in role.slot_registry.as_ref().into_iter().flatten() {
            if entry.1.slot_type.trim() == "llm"
                && entry.1.backend.trim() == "ollama"
                && entry.1.model.as_ref().is_some_and(|m| !m.trim().is_empty())
            {
                role.ollama_model = entry.1.model.clone();
                break;
            }
        }
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
        role.plugin_backends = Arc::new(slot_registry_to_plugin_backends(&loaded.slot_registry));
        role.slot_registry = Some(loaded.slot_registry);
        role.slot_groups = if loaded.groups.is_empty() {
            None
        } else {
            Some(loaded.groups)
        };
        role.interaction_mode = loaded.interaction_mode;
        role.remote_presence = loaded.remote_presence;
        role.autonomous_scene = loaded.autonomous_scene;
        role.reply_quality_anchor = loaded.reply_quality_anchor;

        for entry in role.slot_registry.as_ref().into_iter().flatten() {
            if entry.1.slot_type.trim() == "llm"
                && entry.1.backend.trim() == "ollama"
                && entry.1.model.as_ref().is_some_and(|m| !m.trim().is_empty())
            {
                role.ollama_model = entry.1.model.clone();
                break;
            }
        }

        self.finish_role_pack_load(role_dir, &loaded.disk, role, None)
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
        role.scene_ids = Arc::from(scene_list.into_boxed_slice());

        let core_personality_path = role_dir.join("core_personality.txt");
        if core_personality_path.exists() {
            role.core_personality =
                fs::read_to_string(&core_personality_path).map_err(AppError::IoError)?;
        }

        role.user_identity_catalog = load_user_identity_catalog(role_dir)?;

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
        let role_dir = self.roles_dir.join(rid);
        self.load_role_from_dir(&role_dir)
    }
    /// # Errors
    ///
    /// Returns [`Err`] with a human-readable message when the operation fails.
    /// Scene id list: manifest top-level `scenes` array + `roles/{role_id}/scenes/` subdirectory names, deduplicated and sorted.
    /// Returns `["default"]` when both are empty.
    pub fn list_scene_ids(&self, role_id: &str) -> Result<Vec<String>> {
        let role_dir = self.roles_dir.join(role_id);
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
