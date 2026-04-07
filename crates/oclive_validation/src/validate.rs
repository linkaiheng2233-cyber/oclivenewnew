//! 与运行时原 `role_manifest_validate::validate_disk_manifest` 行为一致。

use crate::manifest::{DiskRoleManifest, KnowledgePackConfigDisk, LifeScheduleDisk};
use std::collections::HashSet;

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
        }
    }
}
