use crate::domain::knowledge_loader::{load_knowledge_index, should_load_knowledge};
use crate::domain::role_manifest_validate::{
    log_plugin_backends_remote_missing_env, validate_disk_manifest, validate_role_interaction_mode,
};
use crate::error::{AppError, Result};
use crate::models::role_manifest_disk::{disk_manifest_from_role, disk_manifest_to_role};
use crate::models::{
    role_settings_disk::CURRENT_SETTINGS_SCHEMA_VERSION, DiskRoleManifest, DiskRoleSettings,
    DiskSceneConfig, LlmBackend, Role,
};
use chrono::Timelike;
use oclive_validation::{
    validate_manifest_top_level_keys, validate_min_runtime_version,
    validate_settings_schema_version, validate_settings_top_level_keys,
};
use serde_json;
use std::collections::BTreeSet;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// 角色包存储管理
///
/// 负责从文件系统加载和保存角色配置
#[derive(Debug, Clone)]
pub struct RoleStorage {
    /// 角色包根目录路径
    roles_dir: PathBuf,
}

impl RoleStorage {
    /// 创建新的角色存储实例
    pub fn new(roles_dir: impl AsRef<Path>) -> Self {
        Self {
            roles_dir: roles_dir.as_ref().to_path_buf(),
        }
    }

    pub fn roles_dir(&self) -> &Path {
        &self.roles_dir
    }

    /// `roles/{role_id}/{relative}`，不检查是否存在。
    pub fn role_asset_path(&self, role_id: &str, relative: &str) -> PathBuf {
        self.roles_dir.join(role_id).join(relative)
    }

    /// 加载所有角色
    ///
    /// # Returns
    ///
    /// 返回所有可用角色的列表
    ///
    /// # Errors
    ///
    /// 如果目录不存在或读取失败，返回 `AppError::IoError`
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
            return Ok(roles);
        }

        for entry in fs::read_dir(&self.roles_dir).map_err(AppError::IoError)? {
            let entry = entry.map_err(AppError::IoError)?;
            let path = entry.path();

            if path.is_dir() {
                if let Ok(role) = self.load_role_from_dir(&path) {
                    roles.push(role);
                }
            }
        }

        Ok(roles)
    }

    /// 从目录加载单个角色
    pub fn load_role_from_dir(&self, role_dir: &Path) -> Result<Role> {
        let manifest_path = role_dir.join("manifest.json");

        if !manifest_path.exists() {
            return Err(AppError::RoleNotFound(format!(
                "manifest.json not found in {:?}",
                role_dir
            )));
        }

        let manifest_content = fs::read_to_string(&manifest_path).map_err(AppError::IoError)?;

        let manifest_value: serde_json::Value =
            serde_json::from_str(&manifest_content).map_err(AppError::SerializationError)?;
        if let serde_json::Value::Object(ref map) = manifest_value {
            validate_manifest_top_level_keys(map).map_err(AppError::InvalidParameter)?;
        }
        let mut disk: DiskRoleManifest =
            serde_json::from_value(manifest_value).map_err(AppError::SerializationError)?;

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
        if should_load_knowledge(&disk, role_dir) {
            let idx = load_knowledge_index(role_dir, &disk)?;
            role.knowledge_index = Some(Arc::new(idx));
        }
        if let Some(ref s) = settings_opt {
            role.remote_presence = s.remote_presence.clone();
            role.autonomous_scene = s.autonomous_scene.clone();
            role.interaction_mode = s.interaction_mode.clone();
            if let Some(ref pb) = s.plugin_backends {
                role.plugin_backends = pb.clone();
            }
        }
        apply_llm_backend_env_override(&mut role);
        validate_role_interaction_mode(&role).map_err(AppError::InvalidParameter)?;
        log_plugin_backends_remote_missing_env(&role);

        // 加载核心人设
        let core_personality_path = role_dir.join("core_personality.txt");
        if core_personality_path.exists() {
            role.core_personality =
                fs::read_to_string(&core_personality_path).map_err(AppError::IoError)?;
        }

        Ok(role)
    }

    /// 加载指定角色
    pub fn load_role(&self, role_id: &str) -> Result<Role> {
        let role_dir = self.roles_dir.join(role_id);
        self.load_role_from_dir(&role_dir)
    }

    /// 场景 id 列表：manifest 顶层 `scenes` 数组 + `roles/{role_id}/scenes/` 子目录名，去重排序。
    /// 若均为空则返回 `["default"]`。
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

    /// `manifest.scenes` + `scenes/` 子目录，去重排序；均空时 `["default"]`（与 [`Self::list_scene_ids`] 一致）。
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

    /// `scenes/{scene_id}/scene.json` 中 `name` 字段；缺失时用内置中文映射，再退回 id。
    pub fn scene_display_name(&self, role_id: &str, scene_id: &str) -> String {
        if let Some(cfg) = self.load_scene_config(role_id, scene_id) {
            if let Some(name) = cfg.name {
                let t = name.trim();
                if !t.is_empty() {
                    return t.to_string();
                }
            }
        }
        Self::fallback_scene_label(scene_id)
    }

    /// 场景切换欢迎语：`welcome_message` 优先；否则从 `monologues` 按 role+scene 稳定选一条。
    pub fn scene_welcome_line(&self, role_id: &str, scene_id: &str) -> Option<String> {
        let cfg = self.load_scene_config(role_id, scene_id)?;
        if let Some(w) = cfg.welcome_message {
            let t = w.trim();
            if !t.is_empty() {
                return Some(t.to_string());
            }
        }
        let templates = Self::normalize_string_vec(cfg.monologues);
        if templates.is_empty() {
            return None;
        }
        let mut h = std::collections::hash_map::DefaultHasher::new();
        role_id.hash(&mut h);
        scene_id.hash(&mut h);
        let idx = (h.finish() as usize) % templates.len();
        Some(templates[idx].clone())
    }

    /// `scenes/{scene_id}/scene.json` 中可选 `monologues: string[]`，用于独白模板或 LLM 失败兜底。
    pub fn scene_monologue_templates(&self, role_id: &str, scene_id: &str) -> Vec<String> {
        let Some(cfg) = self.load_scene_config(role_id, scene_id) else {
            return Vec::new();
        };
        Self::normalize_string_vec(cfg.monologues)
    }

    pub fn scene_keywords(&self, role_id: &str, scene_id: &str) -> Vec<String> {
        let Some(cfg) = self.load_scene_config(role_id, scene_id) else {
            return Vec::new();
        };
        Self::normalize_string_vec(cfg.keywords)
    }

    pub fn scene_events(&self, role_id: &str, scene_id: &str) -> Vec<String> {
        let Some(cfg) = self.load_scene_config(role_id, scene_id) else {
            return Vec::new();
        };
        Self::normalize_string_vec(cfg.events)
    }

    pub fn is_scene_time_allowed(
        &self,
        role_id: &str,
        scene_id: &str,
        virtual_time_ms: i64,
    ) -> bool {
        let Some(cfg) = self.load_scene_config(role_id, scene_id) else {
            return true;
        };
        if cfg.time_windows.is_empty() {
            return true;
        }
        let Some(dt) = chrono::DateTime::from_timestamp_millis(virtual_time_ms) else {
            return true;
        };
        let minute_of_day = (dt.hour() as i32) * 60 + (dt.minute() as i32);
        cfg.time_windows.iter().any(|w| {
            let Some(start_min) = Self::parse_hhmm_minutes(w.start.as_str()) else {
                return false;
            };
            let Some(end_min) = Self::parse_hhmm_minutes(w.end.as_str()) else {
                return false;
            };
            if start_min == end_min {
                return true;
            }
            if start_min < end_min {
                minute_of_day >= start_min && minute_of_day < end_min
            } else {
                minute_of_day >= start_min || minute_of_day < end_min
            }
        })
    }

    pub fn load_scene_config(&self, role_id: &str, scene_id: &str) -> Option<DiskSceneConfig> {
        let path = self.scene_json_path(role_id, scene_id);
        let raw = fs::read_to_string(path).ok()?;
        serde_json::from_str::<DiskSceneConfig>(&raw).ok()
    }

    fn scene_json_path(&self, role_id: &str, scene_id: &str) -> PathBuf {
        self.roles_dir
            .join(role_id)
            .join("scenes")
            .join(scene_id)
            .join("scene.json")
    }

    fn scene_description_path(&self, role_id: &str, scene_id: &str) -> PathBuf {
        self.roles_dir
            .join(role_id)
            .join("scenes")
            .join(scene_id)
            .join("description.txt")
    }

    fn away_life_txt_path(&self, role_id: &str, scene_id: &str) -> PathBuf {
        self.roles_dir
            .join(role_id)
            .join("scenes")
            .join(scene_id)
            .join("away_life.txt")
    }

    /// `scenes/<scene_id>/away_life.txt`（角色位于本场景时的异地生活长文素材）
    pub fn away_life_txt_file(&self, role_id: &str, scene_id: &str) -> Option<String> {
        let path = self.away_life_txt_path(role_id, scene_id);
        let raw = fs::read_to_string(path).ok()?;
        let t = raw.trim();
        if t.is_empty() {
            None
        } else {
            Some(t.to_string())
        }
    }

    /// 角色当前所在场景 `character_scene_id`、用户对话上下文场景 `user_scene_id` 不一致时注入 prompt 的素材。
    /// 优先 `away_life.txt`，其次 `away_life_by_user_scene[user_scene]`，再合并 `away_life_notes`。
    pub fn away_life_material(
        &self,
        role_id: &str,
        character_scene_id: &str,
        user_scene_id: &str,
    ) -> String {
        const MAX: usize = 8000;
        if let Some(txt) = self.away_life_txt_file(role_id, character_scene_id) {
            return Self::clamp_utf8_chars(&txt, MAX);
        }
        let Some(cfg) = self.load_scene_config(role_id, character_scene_id) else {
            return String::new();
        };
        if let Some(s) = cfg.away_life_by_user_scene.get(user_scene_id) {
            let t = s.trim();
            if !t.is_empty() {
                return Self::clamp_utf8_chars(t, MAX);
            }
        }
        let notes: Vec<String> = cfg
            .away_life_notes
            .into_iter()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        if notes.is_empty() {
            String::new()
        } else {
            Self::clamp_utf8_chars(&notes.join("\n"), MAX)
        }
    }

    /// `scenes/<scene_id>/description.txt` 全文（创作者可自行增删，无需改程序）。
    pub fn scene_description_file(&self, role_id: &str, scene_id: &str) -> Option<String> {
        let path = self.scene_description_path(role_id, scene_id);
        let raw = fs::read_to_string(path).ok()?;
        let t = raw.trim();
        if t.is_empty() {
            None
        } else {
            Some(t.to_string())
        }
    }

    /// 主对话 prompt 用的场景说明：优先长文 `description.txt`，否则从 `scene.json` 拼短说明。
    pub fn scene_prompt_enrichment(&self, role_id: &str, scene_id: &str) -> String {
        const MAX_SCENE_PROMPT_CHARS: usize = 6000;
        if let Some(desc) = self.scene_description_file(role_id, scene_id) {
            return Self::clamp_utf8_chars(&desc, MAX_SCENE_PROMPT_CHARS);
        }
        let Some(cfg) = self.load_scene_config(role_id, scene_id) else {
            return String::new();
        };
        let mut parts: Vec<String> = Vec::new();
        if let Some(n) = cfg
            .name
            .as_ref()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
        {
            parts.push(format!("场景：{}", n));
        }
        let kws = Self::normalize_string_vec(cfg.keywords);
        if !kws.is_empty() {
            parts.push(format!("常见元素：{}", kws.join("、")));
        }
        let evs = Self::normalize_string_vec(cfg.events);
        if !evs.is_empty() {
            parts.push(format!("可出现：{}", evs.join("、")));
        }
        if parts.is_empty() {
            String::new()
        } else {
            parts.join("\n")
        }
    }

    /// 场景切换 LLM 判定用的一行摘要（description 首行非空行，或 名称+首个关键词）。
    pub fn scene_switch_hint_line(&self, role_id: &str, scene_id: &str) -> String {
        const MAX_HINT: usize = 200;
        if let Some(desc) = self.scene_description_file(role_id, scene_id) {
            if let Some(line) = desc.lines().map(str::trim).find(|l| !l.is_empty()) {
                return Self::clamp_utf8_chars(line, MAX_HINT);
            }
        }
        let label = self.scene_display_name(role_id, scene_id);
        let kws = self.scene_keywords(role_id, scene_id);
        if let Some(k) = kws.first() {
            format!("{}（{}）", label, k)
        } else {
            label
        }
    }

    fn clamp_utf8_chars(s: &str, max_chars: usize) -> String {
        if s.chars().count() <= max_chars {
            s.to_string()
        } else {
            s.chars().take(max_chars).collect::<String>() + "\n…（已截断）"
        }
    }

    fn normalize_string_vec(values: Vec<String>) -> Vec<String> {
        values
            .into_iter()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }

    fn parse_hhmm_minutes(raw: &str) -> Option<i32> {
        let (h, m) = raw.trim().split_once(':')?;
        let h = h.parse::<i32>().ok()?;
        let m = m.parse::<i32>().ok()?;
        if !(0..=23).contains(&h) || !(0..=59).contains(&m) {
            return None;
        }
        Some(h * 60 + m)
    }

    fn fallback_scene_label(scene_id: &str) -> String {
        match scene_id {
            "default" => "默认".to_string(),
            "home" => "家".to_string(),
            "school" => "学校".to_string(),
            "company" => "公司".to_string(),
            "park" => "游乐园".to_string(),
            "debug_panel" => "调试".to_string(),
            "production" => "生产".to_string(),
            _ => scene_id.to_string(),
        }
    }

    /// 保存角色配置：写入 `manifest.json`（展示与关系等）与 `settings.json`（引擎字段）。
    pub fn save_role_manifest(&self, role: &Role) -> Result<()> {
        let role_dir = self.roles_dir.join(&role.id);
        fs::create_dir_all(&role_dir).map_err(AppError::IoError)?;

        let disk = disk_manifest_from_role(role);
        let manifest_path = role_dir.join("manifest.json");
        let json = serde_json::to_string_pretty(&disk).map_err(AppError::SerializationError)?;
        fs::write(&manifest_path, json).map_err(AppError::IoError)?;

        let settings = DiskRoleSettings::from_role(role);
        let settings_path = role_dir.join("settings.json");
        let settings_json =
            serde_json::to_string_pretty(&settings).map_err(AppError::SerializationError)?;
        fs::write(&settings_path, settings_json).map_err(AppError::IoError)?;

        Ok(())
    }

    /// 保存核心人设（仅创作者可改）
    pub fn save_core_personality(&self, role_id: &str, content: &str) -> Result<()> {
        let role_dir = self.roles_dir.join(role_id);
        let core_personality_path = role_dir.join("core_personality.txt");

        fs::write(&core_personality_path, content).map_err(AppError::IoError)?;

        Ok(())
    }
}

/// 与 oclive-launcher 注入的取值一致：`ollama` / `remote`（大小写不敏感）。
pub(crate) fn resolve_llm_backend_env_override() -> Option<LlmBackend> {
    let Ok(v) = std::env::var("OCLIVE_LLM_BACKEND") else {
        return None;
    };
    let t = v.trim();
    if t.is_empty() {
        return None;
    }
    if t.eq_ignore_ascii_case("ollama") {
        Some(LlmBackend::Ollama)
    } else if t.eq_ignore_ascii_case("remote") {
        Some(LlmBackend::Remote)
    } else if t.eq_ignore_ascii_case("directory") {
        Some(LlmBackend::Directory)
    } else {
        None
    }
}

/// 与 oclive-launcher 注入的取值一致：`ollama` / `remote`（大小写不敏感），覆盖磁盘 `plugin_backends.llm`。
fn apply_llm_backend_env_override(role: &mut Role) {
    if let Some(v) = resolve_llm_backend_env_override() {
        role.plugin_backends.llm = v;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::role::IdentityBinding;
    use crate::models::{
        EvolutionBounds, EvolutionConfig, MemoryConfig, PersonalityDefaults, UserRelation,
    };
    use std::collections::HashMap;
    use tempfile::TempDir;

    #[test]
    fn test_role_storage_new() {
        let storage = RoleStorage::new("./roles");
        assert_eq!(storage.roles_dir, PathBuf::from("./roles"));
    }

    #[test]
    fn test_load_all_roles_empty_dir() {
        let temp_dir = TempDir::new().unwrap();
        let storage = RoleStorage::new(temp_dir.path());
        let roles = storage.load_all_roles().unwrap();
        assert_eq!(roles.len(), 0);
    }

    #[test]
    fn test_save_and_load_role_manifest() {
        let temp_dir = TempDir::new().unwrap();
        let storage = RoleStorage::new(temp_dir.path());

        let mut topic_weights: HashMap<String, HashMap<String, f64>> = HashMap::new();
        topic_weights.insert(
            "default".to_string(),
            [("日常".to_string(), 0.5)].into_iter().collect(),
        );

        let role = Role {
            id: "test_role".to_string(),
            name: "Test Role".to_string(),
            description: "A test role".to_string(),
            version: "1.0.0".to_string(),
            author: "Test Author".to_string(),
            core_personality: "Test personality".to_string(),
            default_personality: PersonalityDefaults {
                stubbornness: 0.5,
                clinginess: 0.5,
                sensitivity: 0.5,
                assertiveness: 0.5,
                forgiveness: 0.5,
                talkativeness: 0.5,
                warmth: 0.5,
            },
            evolution_bounds: EvolutionBounds::full_01(),
            user_relations: vec![UserRelation {
                id: "friend".into(),
                name: "好友".into(),
                prompt_hint: "".into(),
                favor_multiplier: 1.0,
                initial_favorability: 50.0,
            }],
            evolution_config: EvolutionConfig {
                event_impact_factor: 1.5,
                ai_analysis_interval: 20,
                max_change_per_event: 0.1,
                max_total_change: 0.6,
                personality_source: Default::default(),
            },
            memory_config: Some(MemoryConfig {
                scene_weight_multiplier: 2.0,
                topic_weights,
            }),
            default_relation: "friend".to_string(),
            ollama_model: Some("mumu:latest".to_string()),
            identity_binding: IdentityBinding::Global,
            life_trajectory: None,
            life_schedule: None,
            remote_presence: None,
            autonomous_scene: None,
            interaction_mode: None,
            min_runtime_version: None,
            dev_only: false,
            plugin_backends: crate::models::PluginBackends::default(),
            knowledge_index: None,
        };

        storage.save_role_manifest(&role).unwrap();

        let settings_path = temp_dir.path().join("test_role").join("settings.json");
        assert!(settings_path.exists());

        let loaded_role = storage.load_role("test_role").unwrap();

        assert_eq!(loaded_role.id, "test_role");
        assert_eq!(loaded_role.name, "Test Role");
        assert_eq!(loaded_role.identity_binding, IdentityBinding::Global);
        assert_eq!(loaded_role.ollama_model.as_deref(), Some("mumu:latest"));
        assert_eq!(loaded_role.evolution_config.event_impact_factor, 1.5);
        assert_eq!(loaded_role.evolution_config.ai_analysis_interval, 20);
        assert_eq!(loaded_role.evolution_config.max_change_per_event, 0.1);
        assert_eq!(loaded_role.evolution_config.max_total_change, 0.6);
        let mem = loaded_role.memory_config.expect("memory_config");
        assert_eq!(mem.scene_weight_multiplier, 2.0);
        assert_eq!(
            mem.topic_weights.get("default").unwrap().get("日常"),
            Some(&0.5)
        );
    }
}
