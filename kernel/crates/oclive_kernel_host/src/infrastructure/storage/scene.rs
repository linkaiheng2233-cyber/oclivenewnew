use super::RoleStorage;
use crate::models::{DiskSceneConfig, Role};
use chrono::Timelike;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::sync::Arc;

impl RoleStorage {
    /// Scene-switch welcome line: `welcome_message` takes priority; otherwise stably pick one from `monologues` by role+scene.
    #[must_use]
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

    /// Optional `monologues: string[]` in `scenes/{scene_id}/scene.json`, used as monologue templates or as a fallback when the LLM fails.
    #[must_use]
    pub fn scene_monologue_templates(&self, role_id: &str, scene_id: &str) -> Vec<String> {
        let Some(cfg) = self.load_scene_config(role_id, scene_id) else {
            return Vec::new();
        };
        Self::normalize_string_vec(cfg.monologues)
    }

    /// Read `scenes/{scene_id}/scene.json` from disk (no caching; for the API cold path).
    #[must_use]
    pub fn load_scene_config(&self, role_id: &str, scene_id: &str) -> Option<DiskSceneConfig> {
        let path = self.scene_json_path(role_id, scene_id)?;
        let raw = fs::read_to_string(path).ok()?;
        serde_json::from_str::<DiskSceneConfig>(&raw).ok()
    }

    fn scene_text_cache_get(role: &Role, key: &str) -> Option<Arc<str>> {
        role.scene_text_cache.read().get(key).cloned()
    }

    fn scene_text_cache_put(role: &Role, key: String, value: Arc<str>) {
        role.scene_text_cache.write().insert(key, value);
    }

    /// Scene config with an in-`Role` cache: each scene id reads from disk at most once until `invalidate_role_cache`.
    #[must_use]
    pub fn get_scene_config(&self, role: &Role, scene_id: &str) -> Option<Arc<DiskSceneConfig>> {
        {
            let cache = role.scene_config_cache.read();
            if let Some(cfg) = cache.get(scene_id) {
                return Some(Arc::clone(cfg));
            }
        }
        let disk = self.load_scene_config(role.id.as_str(), scene_id)?;
        let arc = Arc::new(disk);
        role.scene_config_cache
            .write()
            .insert(scene_id.to_string(), Arc::clone(&arc));
        Some(arc)
    }

    #[must_use]
    pub fn scene_display_name_for_role(&self, role: &Role, scene_id: &str) -> String {
        if let Some(cfg) = self.get_scene_config(role, scene_id) {
            if let Some(name) = cfg.name.as_ref() {
                let t = name.trim();
                if !t.is_empty() {
                    return t.to_string();
                }
            }
        }
        Self::fallback_scene_label(scene_id)
    }

    #[must_use]
    pub fn scene_keywords_for_role(&self, role: &Role, scene_id: &str) -> Vec<String> {
        self.get_scene_config(role, scene_id)
            .map(|cfg| Self::normalize_string_vec(cfg.keywords.clone()))
            .unwrap_or_default()
    }

    #[must_use]
    pub fn scene_events_for_role(&self, role: &Role, scene_id: &str) -> Vec<String> {
        self.get_scene_config(role, scene_id)
            .map(|cfg| Self::normalize_string_vec(cfg.events.clone()))
            .unwrap_or_default()
    }

    #[must_use]
    pub fn is_scene_time_allowed_for_role(
        &self,
        role: &Role,
        scene_id: &str,
        virtual_time_ms: i64,
    ) -> bool {
        let Some(cfg) = self.get_scene_config(role, scene_id) else {
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

    #[must_use]
    pub fn away_life_material_for_role(
        &self,
        role: &Role,
        character_scene_id: &str,
        user_scene_id: &str,
    ) -> String {
        const MAX: usize = 8000;
        let cache_key = format!("away:{character_scene_id}:{user_scene_id}");
        if let Some(cached) = Self::scene_text_cache_get(role, &cache_key) {
            return cached.to_string();
        }
        let away_key = format!("away_txt:{character_scene_id}");
        let material = if let Some(cached_txt) = Self::scene_text_cache_get(role, &away_key) {
            Self::clamp_utf8_chars(cached_txt.as_ref(), MAX)
        } else if let Some(txt) = self.away_life_txt_file(role.id.as_str(), character_scene_id) {
            let arc: Arc<str> = Arc::from(txt.as_str());
            Self::scene_text_cache_put(role, away_key, Arc::clone(&arc));
            Self::clamp_utf8_chars(&txt, MAX)
        } else {
            let Some(cfg) = self.get_scene_config(role, character_scene_id) else {
                return String::new();
            };
            if let Some(s) = cfg.away_life_by_user_scene.get(user_scene_id) {
                let t = s.trim();
                if !t.is_empty() {
                    Self::clamp_utf8_chars(t, MAX)
                } else {
                    String::new()
                }
            } else {
                let notes: Vec<String> = cfg
                    .away_life_notes
                    .iter()
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                if notes.is_empty() {
                    String::new()
                } else {
                    Self::clamp_utf8_chars(&notes.join("\n"), MAX)
                }
            }
        };
        if !material.is_empty() {
            Self::scene_text_cache_put(role, cache_key, Arc::from(material.as_str()));
        }
        material
    }

    #[must_use]
    pub fn scene_prompt_enrichment_for_role(&self, role: &Role, scene_id: &str) -> String {
        const MAX_SCENE_PROMPT_CHARS: usize = 6000;
        let cache_key = format!("desc:{scene_id}");
        if let Some(cached) = Self::scene_text_cache_get(role, &cache_key) {
            return Self::clamp_utf8_chars(cached.as_ref(), MAX_SCENE_PROMPT_CHARS);
        }
        if let Some(desc) = self.scene_description_file(role.id.as_str(), scene_id) {
            let arc: Arc<str> = Arc::from(desc.as_str());
            Self::scene_text_cache_put(role, cache_key, arc);
            return Self::clamp_utf8_chars(&desc, MAX_SCENE_PROMPT_CHARS);
        }
        let Some(cfg) = self.get_scene_config(role, scene_id) else {
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
        let kws = Self::normalize_string_vec(cfg.keywords.clone());
        if !kws.is_empty() {
            parts.push(format!("常见元素：{}", kws.join("、")));
        }
        let evs = Self::normalize_string_vec(cfg.events.clone());
        if !evs.is_empty() {
            parts.push(format!("可出现：{}", evs.join("、")));
        }
        if parts.is_empty() {
            String::new()
        } else {
            parts.join("\n")
        }
    }

    fn scene_file_path(&self, role_id: &str, scene_id: &str, filename: &str) -> Option<PathBuf> {
        oclive_validation::validate_scene_id(scene_id).ok()?;
        self.role_asset_path(role_id, &format!("scenes/{scene_id}/{filename}"))
            .ok()
    }

    fn scene_json_path(&self, role_id: &str, scene_id: &str) -> Option<PathBuf> {
        self.scene_file_path(role_id, scene_id, "scene.json")
    }

    fn scene_description_path(&self, role_id: &str, scene_id: &str) -> Option<PathBuf> {
        self.scene_file_path(role_id, scene_id, "description.txt")
    }

    fn away_life_txt_path(&self, role_id: &str, scene_id: &str) -> Option<PathBuf> {
        self.scene_file_path(role_id, scene_id, "away_life.txt")
    }

    /// `scenes/<scene_id>/away_life.txt` (long-form remote-presence life material for when the character is in this scene)
    #[must_use]
    pub fn away_life_txt_file(&self, role_id: &str, scene_id: &str) -> Option<String> {
        let path = self.away_life_txt_path(role_id, scene_id)?;
        let raw = fs::read_to_string(path).ok()?;
        let t = raw.trim();
        if t.is_empty() {
            None
        } else {
            Some(t.to_string())
        }
    }

    /// Full text of `scenes/<scene_id>/description.txt` (creators can add or remove content freely, without changing the program).
    #[must_use]
    pub fn scene_description_file(&self, role_id: &str, scene_id: &str) -> Option<String> {
        let path = self.scene_description_path(role_id, scene_id)?;
        let raw = fs::read_to_string(path).ok()?;
        let t = raw.trim();
        if t.is_empty() {
            None
        } else {
            Some(t.to_string())
        }
    }

    /// One-line summary used for the scene-switch LLM decision (first non-empty line of the description, or name + first keyword).
    pub fn scene_switch_hint_line(&self, role: &Role, scene_id: &str) -> String {
        const MAX_HINT: usize = 200;
        if let Some(desc) = self.scene_description_file(role.id.as_str(), scene_id) {
            if let Some(line) = desc.lines().map(str::trim).find(|l| !l.is_empty()) {
                return Self::clamp_utf8_chars(line, MAX_HINT);
            }
        }
        let label = self.scene_display_name_for_role(role, scene_id);
        let kws = self.scene_keywords_for_role(role, scene_id);
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
}
