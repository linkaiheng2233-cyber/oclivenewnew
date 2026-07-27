//! Scene-local narrative continuity for co-present dialogue.
//!
//! Role packs own the state descriptions and explicit reply markers. The host
//! selects one initial state, injects it into the dynamic prompt suffix, and
//! advances it only after a final visible assistant reply matches a marker.

use crate::domain::role_runtime_snapshot::RoleRuntimeSnapshot;
use crate::error::Result;
use crate::models::{
    Role, SceneContinuityConfig, SceneContinuityInitialState, SceneContinuityTransition,
};
use crate::state::AppState;
use chrono::Timelike;

fn local_virtual_time_ms(virtual_time_ms: i64, timezone_offset_minutes: i32) -> Option<i64> {
    if virtual_time_ms <= 0 {
        return None;
    }
    virtual_time_ms.checked_add(i64::from(timezone_offset_minutes) * 60_000)
}

fn minute_of_day(virtual_time_ms: i64, timezone_offset_minutes: i32) -> Option<u16> {
    let local_ms = local_virtual_time_ms(virtual_time_ms, timezone_offset_minutes)?;
    let dt = chrono::DateTime::from_timestamp_millis(local_ms)?;
    Some((dt.hour() as u16) * 60 + dt.minute() as u16)
}

fn parse_hhmm(value: &str) -> Option<u16> {
    let (hour, minute) = value.trim().split_once(':')?;
    let hour = hour.parse::<u16>().ok()?;
    let minute = minute.parse::<u16>().ok()?;
    if hour > 23 || minute > 59 {
        return None;
    }
    Some(hour * 60 + minute)
}

fn window_contains(minute: u16, start: u16, end: u16) -> bool {
    if start == end {
        true
    } else if start < end {
        minute >= start && minute < end
    } else {
        minute >= start || minute < end
    }
}

fn state_matches_time(state: &SceneContinuityInitialState, minute: u16) -> bool {
    state.time_windows.iter().any(|window| {
        parse_hhmm(&window.start)
            .zip(parse_hhmm(&window.end))
            .is_some_and(|(start, end)| window_contains(minute, start, end))
    })
}

fn stable_hash(parts: &[&str]) -> u64 {
    const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
    const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;
    let mut hash = FNV_OFFSET;
    for part in parts {
        for byte in part.as_bytes().iter().copied().chain(std::iter::once(0xff)) {
            hash ^= u64::from(byte);
            hash = hash.wrapping_mul(FNV_PRIME);
        }
    }
    hash
}

fn state_by_id<'a>(
    config: &'a SceneContinuityConfig,
    state_id: &str,
) -> Option<&'a SceneContinuityInitialState> {
    config
        .initial_states
        .iter()
        .find(|state| state.id == state_id)
}

fn weighted_pick<'a>(
    candidates: &[&'a SceneContinuityInitialState],
    seed: u64,
) -> Option<&'a SceneContinuityInitialState> {
    let total = candidates.iter().fold(0_u64, |sum, state| {
        sum.saturating_add(u64::from(state.weight))
    });
    if total == 0 {
        return candidates.first().copied();
    }
    let mut cursor = seed % total;
    for state in candidates {
        let weight = u64::from(state.weight);
        if cursor < weight {
            return Some(*state);
        }
        cursor -= weight;
    }
    candidates.last().copied()
}

fn select_initial_state<'a>(
    config: &'a SceneContinuityConfig,
    srid: &str,
    scene_id: &str,
    virtual_time_ms: i64,
    timezone_offset_minutes: i32,
) -> Option<&'a SceneContinuityInitialState> {
    let minute = minute_of_day(virtual_time_ms, timezone_offset_minutes);
    let timed: Vec<_> = minute
        .map(|minute| {
            config
                .initial_states
                .iter()
                .filter(|state| !state.time_windows.is_empty() && state_matches_time(state, minute))
                .collect()
        })
        .unwrap_or_default();
    let untimed: Vec<_> = config
        .initial_states
        .iter()
        .filter(|state| state.time_windows.is_empty())
        .collect();
    let candidates = if timed.is_empty() { &untimed } else { &timed };
    if candidates.is_empty() {
        return config
            .default_state_id
            .as_deref()
            .and_then(|id| state_by_id(config, id))
            .or_else(|| config.initial_states.first());
    }

    let virtual_day = local_virtual_time_ms(virtual_time_ms, timezone_offset_minutes)
        .unwrap_or_default()
        .div_euclid(86_400_000)
        .to_string();
    weighted_pick(
        candidates,
        stable_hash(&[
            srid,
            scene_id,
            virtual_day.as_str(),
            "narrative-continuity-v1",
        ]),
    )
}

fn render_prompt(config: &SceneContinuityConfig, state: &SceneContinuityInitialState) -> String {
    let routes = config
        .transitions
        .iter()
        .filter(|transition| {
            transition.to != state.id
                && (transition.from.is_empty()
                    || transition.from.iter().any(|source| source == &state.id))
        })
        .filter_map(|transition| {
            transition
                .assistant_reply_markers
                .first()
                .zip(state_by_id(config, transition.to.as_str()))
        })
        .map(|(marker, target)| {
            format!(
                "- “{}” → {}（{}，{}）",
                marker, target.sub_location, target.posture, target.activity
            )
        })
        .collect::<Vec<_>>();
    let routes = if routes.is_empty() {
        String::new()
    } else {
        format!(
            "\n若情节确实需要移动，可自然写入以下动作之一；不要为了触发状态而移动：\n{}",
            routes.join("\n")
        )
    };
    format!(
        "当前子地点：{}\n环境锚点：{}\n当前姿态：{}\n正在做：{}\n\
默认保持这些事实。只有在本轮确实要改变地点或姿态时，先用自然动作写出过渡，再继续对话；\
不得替用户决定动作，也不要把本段规则或字段名说给用户。{}",
        state.sub_location, state.anchor, state.posture, state.activity, routes
    )
}

fn transition_for_reply<'a>(
    config: &'a SceneContinuityConfig,
    current_state_id: &str,
    reply: &str,
) -> Option<&'a SceneContinuityTransition> {
    config
        .transitions
        .iter()
        .filter(|transition| {
            (transition.from.is_empty()
                || transition
                    .from
                    .iter()
                    .any(|source| source == current_state_id))
                && transition.to != current_state_id
        })
        .filter_map(|transition| {
            transition
                .assistant_reply_markers
                .iter()
                .filter(|marker| reply.contains(marker.as_str()))
                .map(|marker| marker.chars().count())
                .max()
                .map(|matched_len| (matched_len, transition))
        })
        .max_by_key(|(matched_len, _)| *matched_len)
        .map(|(_, transition)| transition)
}

/// Resolve and, when needed, initialize the state used by this turn's prompt.
///
/// # Errors
///
/// Returns a database error when the continuity row cannot be read or initialized.
pub async fn prompt_for_turn(
    app: &AppState,
    role: &Role,
    srid: &str,
    scene_id: &str,
    virtual_time_ms: i64,
    snapshot: &RoleRuntimeSnapshot,
    initialize: bool,
) -> Result<String> {
    let Some(scene_config) = app.storage.get_scene_config(role, scene_id) else {
        return Ok(String::new());
    };
    let Some(config) = scene_config.continuity.as_ref() else {
        return Ok(String::new());
    };

    if snapshot.continuity_scene_id.as_deref() == Some(scene_id) {
        if let Some(state) = snapshot
            .continuity_state_id
            .as_deref()
            .and_then(|state_id| state_by_id(config, state_id))
        {
            return Ok(render_prompt(config, state));
        }
    }

    let timezone_offset_minutes = role
        .life_schedule
        .as_ref()
        .and_then(|schedule| schedule.timezone_offset_minutes)
        .unwrap_or(0);
    let Some(selected) = select_initial_state(
        config,
        srid,
        scene_id,
        virtual_time_ms,
        timezone_offset_minutes,
    ) else {
        return Ok(String::new());
    };
    if !initialize {
        return Ok(render_prompt(config, selected));
    }
    let initialized = app
        .db_manager
        .set_narrative_continuity_state(
            srid,
            scene_id,
            selected.id.as_str(),
            snapshot.continuity_revision,
        )
        .await?;
    if initialized.is_some() {
        tracing::debug!(
            target: "oclive_continuity",
            role_id = %srid,
            scene_id,
            state_id = %selected.id,
            "initialized narrative continuity state"
        );
        return Ok(render_prompt(config, selected));
    }

    let current = app.db_manager.get_narrative_continuity_state(srid).await?;
    Ok(current
        .and_then(|(current_scene, current_state, _)| {
            (current_scene == scene_id)
                .then(|| state_by_id(config, current_state.as_str()))
                .flatten()
        })
        .map(|state| render_prompt(config, state))
        .unwrap_or_default())
}

/// Advance the persisted state from the final user-visible assistant reply.
///
/// # Errors
///
/// Returns a database error when the persisted state cannot be read or advanced.
pub async fn update_after_reply(
    app: &AppState,
    role: &Role,
    srid: &str,
    scene_id: &str,
    display_reply: &str,
) -> Result<()> {
    let Some(scene_config) = app.storage.get_scene_config(role, scene_id) else {
        return Ok(());
    };
    let Some(config) = scene_config.continuity.as_ref() else {
        return Ok(());
    };
    let Some((stored_scene_id, current_state_id, revision)) =
        app.db_manager.get_narrative_continuity_state(srid).await?
    else {
        return Ok(());
    };
    if stored_scene_id != scene_id || state_by_id(config, current_state_id.as_str()).is_none() {
        return Ok(());
    }
    let Some(transition) = transition_for_reply(config, current_state_id.as_str(), display_reply)
    else {
        return Ok(());
    };

    let changed = app
        .db_manager
        .transition_narrative_continuity_state(
            srid,
            scene_id,
            current_state_id.as_str(),
            revision,
            transition.to.as_str(),
        )
        .await?;
    tracing::debug!(
        target: "oclive_continuity",
        role_id = %srid,
        scene_id,
        from_state_id = %current_state_id,
        to_state_id = %transition.to,
        changed,
        "evaluated narrative continuity transition"
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{SceneContinuityTimeWindow, SceneContinuityTransition};

    fn state(
        id: &str,
        time_windows: Vec<SceneContinuityTimeWindow>,
    ) -> SceneContinuityInitialState {
        SceneContinuityInitialState {
            id: id.into(),
            weight: 1,
            time_windows,
            sub_location: format!("{id}地点"),
            anchor: format!("{id}锚点"),
            posture: "坐着".into(),
            activity: "聊天".into(),
        }
    }

    #[test]
    fn timed_state_wins_over_untimed_fallback() {
        let config = SceneContinuityConfig {
            default_state_id: Some("sofa".into()),
            initial_states: vec![
                state("sofa", vec![]),
                state(
                    "breakfast",
                    vec![SceneContinuityTimeWindow {
                        start: "06:00".into(),
                        end: "10:00".into(),
                    }],
                ),
            ],
            transitions: vec![],
        };
        let at_eight_utc = 8 * 60 * 60 * 1000;
        assert_eq!(
            select_initial_state(&config, "mumu", "home", at_eight_utc, 0)
                .map(|state| state.id.as_str()),
            Some("breakfast")
        );
        let next_day_midnight_utc = 86_400_000;
        assert_eq!(
            select_initial_state(&config, "mumu", "home", next_day_midnight_utc, 480)
                .map(|state| state.id.as_str()),
            Some("breakfast")
        );
    }

    #[test]
    fn transition_uses_explicit_longest_reply_marker() {
        let config = SceneContinuityConfig {
            default_state_id: Some("sofa".into()),
            initial_states: vec![state("sofa", vec![]), state("bed", vec![])],
            transitions: vec![
                SceneContinuityTransition {
                    from: vec!["sofa".into()],
                    to: "bed".into(),
                    assistant_reply_markers: vec!["去卧室".into()],
                },
                SceneContinuityTransition {
                    from: vec!["sofa".into()],
                    to: "bed".into(),
                    assistant_reply_markers: vec!["走进卧室".into()],
                },
            ],
        };
        let transition = transition_for_reply(&config, "sofa", "沐沐站起来，走进卧室，再爬上床。");
        assert_eq!(transition.map(|item| item.to.as_str()), Some("bed"));
        assert!(transition_for_reply(&config, "sofa", "那就早点休息吧。").is_none());
    }

    #[test]
    fn overnight_window_is_supported() {
        assert!(window_contains(30, 22 * 60, 2 * 60));
        assert!(window_contains(23 * 60, 22 * 60, 2 * 60));
        assert!(!window_contains(12 * 60, 22 * 60, 2 * 60));
    }
}
