//! 与运行时原 `role_manifest_validate::validate_disk_manifest` 行为一致。

use crate::manifest::{DiskRoleManifest, KnowledgePackConfigDisk, LifeScheduleDisk};
use semver::Version;
use std::collections::HashSet;

#[derive(Clone, Copy)]
enum MinRuntimeVersionSource {
    RolePackManifest,
    LocalPluginDescriptor,
}

impl MinRuntimeVersionSource {
    fn invalid_parse_msg(self, req: &str, err: &semver::Error) -> String {
        match self {
            Self::RolePackManifest => format!(
                "角色包 manifest：min_runtime_version「{}」不是合法语义化版本（例如 0.2.0）：{}",
                req, err
            ),
            Self::LocalPluginDescriptor => format!(
                "本地插件：min_runtime_version「{}」不是合法语义化版本（例如 0.2.0）：{}",
                req, err
            ),
        }
    }

    fn below_minimum_msg(self, host: &Version, min_v: &Version) -> String {
        match self {
            Self::RolePackManifest => format!(
                "当前 oclive 版本为 {}，本角色包要求最低 {}（manifest.min_runtime_version）。请升级 oclive 后再加载。",
                host, min_v
            ),
            Self::LocalPluginDescriptor => format!(
                "当前 oclive 版本为 {}，该本地插件描述要求最低 {}（min_runtime_version）。请升级 oclive 后再使用。",
                host, min_v
            ),
        }
    }
}

fn validate_min_runtime_version_for_source(
    min_req: Option<&str>,
    host_version: &str,
    source: MinRuntimeVersionSource,
) -> Result<(), String> {
    let req = min_req.map(str::trim).filter(|s| !s.is_empty());
    let Some(req) = req else {
        return Ok(());
    };
    let min_v = Version::parse(req).map_err(|e| source.invalid_parse_msg(req, &e))?;
    let host = Version::parse(host_version.trim()).map_err(|e| {
        format!(
            "宿主版本「{}」解析失败：{}（此为 oclive 程序错误，请反馈）",
            host_version, e
        )
    })?;
    if host < min_v {
        return Err(source.below_minimum_msg(&host, &min_v));
    }
    Ok(())
}

/// 比较角色包要求的最低宿主版本与当前 oclive 版本（`min_req` 为 `None` 或空则跳过）。
pub fn validate_min_runtime_version(
    min_req: Option<&str>,
    host_version: &str,
) -> Result<(), String> {
    validate_min_runtime_version_for_source(
        min_req,
        host_version,
        MinRuntimeVersionSource::RolePackManifest,
    )
}

/// 本地插件描述中的 `min_runtime_version` 与宿主版本比较（`min_req` 为 `None` 或空则跳过；语义与 [`validate_min_runtime_version`] 一致，错误文案指向本地插件）。
pub fn validate_min_runtime_version_for_local_plugin(
    min_req: Option<&str>,
    host_version: &str,
) -> Result<(), String> {
    validate_min_runtime_version_for_source(
        min_req,
        host_version,
        MinRuntimeVersionSource::LocalPluginDescriptor,
    )
}

/// 校验 `settings.json.schema_version` 与宿主支持范围（当前只接受 `<= current_supported`）。
pub fn validate_settings_schema_version(
    schema_version: u32,
    current_supported: u32,
) -> Result<(), String> {
    if schema_version == 0 {
        return Err("角色包 settings：schema_version 必须为正整数".to_string());
    }
    if schema_version > current_supported {
        return Err(format!(
            "当前 oclive 仅支持 settings schema_version <= {}，而角色包为 {}。请升级 oclive 后再加载。",
            current_supported, schema_version
        ));
    }
    Ok(())
}

/// 解析 `HH:MM` 为自午夜起的分钟数 \[0, 24*60)（与 `domain/life_schedule` 一致）
pub fn parse_hhmm(s: &str) -> Option<u16> {
    let t = s.trim();
    let mut parts = t.split(':');
    let h: u32 = parts.next()?.parse().ok()?;
    let m: u32 = parts.next()?.parse().ok()?;
    if parts.next().is_some() {
        return None;
    }
    if h >= 24 || m >= 60 {
        return None;
    }
    let total = h * 60 + m;
    if total >= 24 * 60 {
        return None;
    }
    Some(total as u16)
}

/// 供校验层调用：仅校验 `KnowledgePackConfigDisk` 字段合理性（不读文件）。
pub fn validate_knowledge_manifest_disk(k: &KnowledgePackConfigDisk) -> Result<(), String> {
    if k.glob.trim().is_empty() {
        return Err("manifest / settings：knowledge.glob 不能为空".to_string());
    }
    Ok(())
}

/// 校验磁盘 manifest 与合并后的场景 id 列表（`manifest.scenes` + `scenes/` 子目录）。
pub fn validate_disk_manifest(
    disk: &DiskRoleManifest,
    merged_scene_ids: &[String],
) -> Result<(), String> {
    if disk.id.trim().is_empty() {
        return Err("角色包 manifest / settings：字段 id 不能为空".to_string());
    }
    if disk.name.trim().is_empty() {
        return Err("角色包 manifest / settings：字段 name 不能为空".to_string());
    }
    if disk.user_relations.is_empty() {
        return Err(
            "角色包 manifest / settings：user_relations 至少需要配置一种用户身份".to_string(),
        );
    }

    let relation_keys: HashSet<&str> = disk.user_relations.keys().map(|s| s.as_str()).collect();

    let dr = disk.default_relation.trim();
    if !dr.is_empty() && !relation_keys.contains(dr) {
        return Err(format!(
            "角色包 manifest / settings：default_relation「{}」在 user_relations 中不存在，请增加对应键或改正 default_relation",
            dr
        ));
    }

    let scenes: HashSet<&str> = merged_scene_ids.iter().map(|s| s.as_str()).collect();
    for scene_key in disk.memory_config.topic_weights.keys() {
        if !scenes.contains(scene_key.as_str()) {
            return Err(format!(
                "角色包 manifest / settings：memory_config.topic_weights 里出现了场景「{}」，但该场景未在 manifest.scenes 或 scenes/ 目录中声明",
                scene_key
            ));
        }
    }

    if let Some(ref ls) = disk.life_schedule {
        validate_life_schedule(ls, merged_scene_ids)?;
    }

    if let Some(ref k) = disk.knowledge {
        validate_knowledge_manifest_disk(k)?;
    }

    for (rid, ur) in &disk.user_relations {
        if !ur.initial_favorability.is_finite() {
            return Err(format!(
                "角色包 manifest / settings：身份「{}」的 initial_favorability 不是有效数字",
                rid
            ));
        }
        if ur.initial_favorability < 0.0 || ur.initial_favorability > 100.0 {
            return Err(format!(
                "角色包 manifest / settings：身份「{}」的 initial_favorability 须在 0～100 之间（当前为 {}）",
                rid, ur.initial_favorability
            ));
        }
        if !ur.favor_multiplier.is_finite() || ur.favor_multiplier <= 0.0 {
            return Err(format!(
                "角色包 manifest / settings：身份「{}」的 favor_multiplier 须为正数",
                rid
            ));
        }
    }

    Ok(())
}

fn validate_life_schedule(
    disk: &LifeScheduleDisk,
    merged_scene_ids: &[String],
) -> Result<(), String> {
    if let Some(off) = disk.timezone_offset_minutes {
        if !(-840..=840).contains(&off) {
            return Err(format!(
                "角色包 manifest / settings：life_schedule.timezone_offset_minutes 超出合理范围（当前为 {}，建议约 -840～840）",
                off
            ));
        }
    }
    let scenes: HashSet<&str> = merged_scene_ids.iter().map(|s| s.as_str()).collect();
    for (i, e) in disk.entries.iter().enumerate() {
        if !(1..=7).contains(&e.weekday) {
            return Err(format!(
                "角色包 manifest / settings：life_schedule.entries[{}] 的 weekday 须在 1～7（周一～周日）",
                i
            ));
        }
        if e.activity_id.trim().is_empty() {
            return Err(format!(
                "角色包 manifest / settings：life_schedule.entries[{}] 的 activity_id 不能为空",
                i
            ));
        }
        if e.label.trim().is_empty() {
            return Err(format!(
                "角色包 manifest / settings：life_schedule.entries[{}] 的 label 不能为空",
                i
            ));
        }
        let start = parse_hhmm(&e.time_start).ok_or_else(|| {
            format!(
                "角色包 manifest / settings：life_schedule.entries[{}] 的 time_start「{}」须为 24 小时制 HH:MM",
                i, e.time_start
            )
        })?;
        let end = parse_hhmm(&e.time_end).ok_or_else(|| {
            format!(
                "角色包 manifest / settings：life_schedule.entries[{}] 的 time_end「{}」须为 24 小时制 HH:MM",
                i, e.time_end
            )
        })?;
        if start == end {
            return Err(format!(
                "角色包 manifest / settings：life_schedule.entries[{}] 的起止时刻不能相同",
                i
            ));
        }
        if let Some(ref sid) = e.preferred_scene_id {
            let t = sid.trim();
            if !t.is_empty() && !scenes.contains(t) {
                return Err(format!(
                    "角色包 manifest / settings：life_schedule.entries[{}] 的 preferred_scene_id「{}」不在场景列表内",
                    i, t
                ));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::{
        EvolutionConfigDisk, LifeScheduleEntryDisk, MemoryConfigDisk, UserRelationDisk,
    };
    use std::collections::HashMap;

    #[test]
    fn rejects_default_relation_missing() {
        let mut disk = minimal_disk();
        disk.default_relation = "ghost".to_string();
        let r = validate_disk_manifest(&disk, &["home".to_string()]);
        assert!(r.is_err());
    }

    #[test]
    fn rejects_topic_weight_unknown_scene() {
        let mut disk = minimal_disk();
        disk.memory_config
            .topic_weights
            .insert("unknown_scene".to_string(), HashMap::new());
        let r = validate_disk_manifest(&disk, &["home".to_string()]);
        assert!(r.is_err());
    }

    #[test]
    fn accepts_matching_topic_weights() {
        let mut disk = minimal_disk();
        disk.memory_config
            .topic_weights
            .insert("home".to_string(), HashMap::new());
        let r = validate_disk_manifest(&disk, &["home".to_string()]);
        assert!(r.is_ok());
    }

    #[test]
    fn rejects_life_schedule_unknown_scene() {
        let mut disk = minimal_disk();
        disk.life_schedule = Some(crate::manifest::LifeScheduleDisk {
            timezone_offset_minutes: Some(480),
            entries: vec![LifeScheduleEntryDisk {
                weekday: 1,
                time_start: "09:00".into(),
                time_end: "12:00".into(),
                activity_id: "school".into(),
                label: "上课".into(),
                preferred_scene_id: Some("no_such_scene".into()),
                availability: None,
            }],
        });
        let r = validate_disk_manifest(&disk, &["home".to_string()]);
        assert!(r.is_err());
    }

    #[test]
    fn accepts_life_schedule_matching_scene() {
        let mut disk = minimal_disk();
        disk.scenes = vec!["home".into()];
        disk.life_schedule = Some(crate::manifest::LifeScheduleDisk {
            timezone_offset_minutes: None,
            entries: vec![LifeScheduleEntryDisk {
                weekday: 3,
                time_start: "08:00".into(),
                time_end: "22:00".into(),
                activity_id: "home".into(),
                label: "在家".into(),
                preferred_scene_id: Some("home".into()),
                availability: None,
            }],
        });
        let r = validate_disk_manifest(&disk, &["home".to_string()]);
        assert!(r.is_ok());
    }

    #[test]
    fn rejects_initial_favor_out_of_range() {
        let mut disk = minimal_disk();
        disk.user_relations
            .get_mut("friend")
            .unwrap()
            .initial_favorability = 101.0;
        let r = validate_disk_manifest(&disk, &["home".to_string()]);
        assert!(r.is_err());
    }

    fn minimal_disk() -> DiskRoleManifest {
        let mut ur = HashMap::new();
        ur.insert(
            "friend".to_string(),
            UserRelationDisk {
                prompt_hint: "x".into(),
                ..Default::default()
            },
        );
        DiskRoleManifest {
            id: "t".into(),
            name: "T".into(),
            version: "1".into(),
            author: "a".into(),
            description: "d".into(),
            ollama_model: None,
            default_personality: vec![0.5; 7],
            evolution: EvolutionConfigDisk::default(),
            scenes: vec![],
            user_relations: ur,
            default_relation: "friend".into(),
            memory_config: MemoryConfigDisk::default(),
            identity_binding: Default::default(),
            life_trajectory: None,
            life_schedule: None,
            dev_only: false,
            knowledge: None,
            min_runtime_version: None,
        }
    }

    #[test]
    fn min_runtime_accepts_equal_or_newer_host() {
        assert!(validate_min_runtime_version(Some("0.2.0"), "0.2.0").is_ok());
        assert!(validate_min_runtime_version(Some("0.1.0"), "0.2.0").is_ok());
    }

    #[test]
    fn min_runtime_rejects_older_host() {
        let e = validate_min_runtime_version(Some("0.9.0"), "0.2.0").unwrap_err();
        assert!(e.contains("0.9.0"));
    }

    #[test]
    fn min_runtime_skips_when_none() {
        assert!(validate_min_runtime_version(None, "0.1.0").is_ok());
    }

    #[test]
    fn local_plugin_min_runtime_invalid_semver_mentions_local() {
        let e = validate_min_runtime_version_for_local_plugin(Some("not-a-semver"), "0.2.0")
            .expect_err("invalid semver");
        assert!(e.contains("本地插件"));
    }

    #[test]
    fn local_plugin_min_runtime_below_host_mentions_local() {
        let e = validate_min_runtime_version_for_local_plugin(Some("99.0.0"), "0.2.0").expect_err("too new");
        assert!(e.contains("本地插件"));
        assert!(e.contains("99.0.0"));
    }

    #[test]
    fn settings_schema_version_rejects_future_version() {
        let err = validate_settings_schema_version(2, 1).expect_err("future version");
        assert!(err.contains("schema_version"));
    }

    #[test]
    fn settings_schema_version_accepts_current_or_lower() {
        assert!(validate_settings_schema_version(1, 1).is_ok());
        assert!(validate_settings_schema_version(1, 2).is_ok());
    }
}
