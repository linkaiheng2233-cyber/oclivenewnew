//! 虚拟时间 + manifest `life_schedule` → 当前活动态（不修改好感等业务数值）

use crate::models::role::{LifeAvailability, LifeScheduleDisk, LifeState};
use chrono::{DateTime, Datelike, FixedOffset, Timelike};

fn availability_busy_level(a: Option<LifeAvailability>) -> f32 {
    match a.unwrap_or(LifeAvailability::Free) {
        LifeAvailability::Busy => 0.85,
        LifeAvailability::Distracted => 0.5,
        LifeAvailability::Free => 0.15,
    }
}

/// 解析 `HH:MM` 为自午夜起的分钟数 \[0, 24*60)
pub(crate) fn parse_hhmm(s: &str) -> Option<u16> {
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

fn minute_in_window(cur: u16, start: u16, end: u16) -> bool {
    if end < start {
        cur >= start || cur < end
    } else {
        cur >= start && cur < end
    }
}

/// 由虚拟时间戳（UTC 毫秒）与创作者日程解析当前态；无匹配片段时返回 `None`（保持对话行为与未配置一致）。
#[must_use]
pub fn resolve_life_state(virtual_time_ms: i64, schedule: &LifeScheduleDisk) -> Option<LifeState> {
    if virtual_time_ms <= 0 || schedule.entries.is_empty() {
        return None;
    }
    let offset_min = schedule.timezone_offset_minutes.unwrap_or(0);
    let offset = FixedOffset::east_opt(offset_min * 60)?;
    let dt_utc = DateTime::from_timestamp_millis(virtual_time_ms)?;
    let local = dt_utc.with_timezone(&offset);
    let weekday = local.weekday().number_from_monday() as u8;
    let minute_of_day: u16 = (local.hour() * 60 + local.minute()).try_into().ok()?;

    for e in &schedule.entries {
        if e.weekday != weekday {
            continue;
        }
        let start = parse_hhmm(&e.time_start)?;
        let end = parse_hhmm(&e.time_end)?;
        if start == end {
            continue;
        }
        if !minute_in_window(minute_of_day, start, end) {
            continue;
        }
        return Some(LifeState {
            label: e.label.trim().to_string(),
            activity_key: e.activity_id.trim().to_string(),
            busy_level: availability_busy_level(e.availability),
            optional_scene_hint: e
                .preferred_scene_id
                .as_ref()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty()),
        });
    }
    None
}

/// 供主对话 / 异地心声注入的一行短约束（中文）
#[must_use]
pub fn format_life_prompt_line(state: &LifeState, user_remote_from_character: bool) -> String {
    let busy = if state.busy_level >= 0.65 {
        "回复可偏短，可带一点分心或环境声，不必长叙事。"
    } else if state.busy_level >= 0.35 {
        "语气可自然带一点琐事感，仍紧扣用户本句。"
    } else {
        "相对有空，语气可放松。"
    };
    let scene = state
        .optional_scene_hint
        .as_deref()
        .map(|sid| {
            format!(
                "（若与当前场景描写冲突，以场景为准；日程倾向场景 id：{}）",
                sid
            )
        })
        .unwrap_or_default();
    if user_remote_from_character {
        format!(
            "异地视角：对方此刻大致在「{}」（{}），心声可侧写其当下状态。{}{}",
            state.label, state.activity_key, busy, scene
        )
    } else {
        format!(
            "共景日程：角色此刻大致在「{}」（{}）。{}{}",
            state.label, state.activity_key, busy, scene
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::role::LifeScheduleEntryDisk;

    fn sched_one(entry: LifeScheduleEntryDisk, offset: Option<i32>) -> LifeScheduleDisk {
        LifeScheduleDisk {
            timezone_offset_minutes: offset,
            entries: vec![entry],
        }
    }

    #[test]
    fn resolves_weekday_window() {
        // 2024-01-01 is Monday UTC
        let ms = DateTime::parse_from_rfc3339("2024-01-01T14:30:00Z")
            .unwrap()
            .timestamp_millis();
        let s = sched_one(
            LifeScheduleEntryDisk {
                weekday: 1,
                time_start: "14:00".into(),
                time_end: "16:00".into(),
                activity_id: "study".into(),
                label: "自习".into(),
                preferred_scene_id: None,
                availability: Some(LifeAvailability::Busy),
            },
            Some(0),
        );
        let st = resolve_life_state(ms, &s).unwrap();
        assert_eq!(st.label, "自习");
        assert_eq!(st.activity_key, "study");
        assert!((st.busy_level - 0.85).abs() < 0.01);
    }

    #[test]
    fn overnight_window() {
        let ms = DateTime::parse_from_rfc3339("2024-01-02T01:30:00Z")
            .unwrap()
            .timestamp_millis();
        let s = sched_one(
            LifeScheduleEntryDisk {
                weekday: 2,
                time_start: "22:00".into(),
                time_end: "07:00".into(),
                activity_id: "rest".into(),
                label: "休息".into(),
                preferred_scene_id: None,
                availability: None,
            },
            Some(0),
        );
        assert!(resolve_life_state(ms, &s).is_some());
    }

    #[test]
    fn no_match_outside_window() {
        let ms = DateTime::parse_from_rfc3339("2024-01-01T10:00:00Z")
            .unwrap()
            .timestamp_millis();
        let s = sched_one(
            LifeScheduleEntryDisk {
                weekday: 1,
                time_start: "14:00".into(),
                time_end: "16:00".into(),
                activity_id: "x".into(),
                label: "y".into(),
                preferred_scene_id: None,
                availability: None,
            },
            Some(0),
        );
        assert!(resolve_life_state(ms, &s).is_none());
    }
}
