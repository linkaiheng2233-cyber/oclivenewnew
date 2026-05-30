//! Immersive-mode virtual clock sync, jumps, and forgetting gradient.

use crate::error::Result;
use crate::infrastructure::db::DbManager;
use crate::models::{PersonalitySource, PersonalityVector, Role};
use oclive_kernel_runtime::domain::life_schedule::virtual_start_ms_from_schedule;
use crate::state::AppState;
use chrono::Utc;
use oclive_kernel_runtime::domain::time_decay::decay_personality_delta;
use oclive_kernel_runtime::domain::virtual_time::{
    compute_virtual_now_ms, round_to_minute_ms, virtual_days_between_ms,
    virtual_days_from_real_elapsed_ms,
};

/// Sync and persist virtual time; returns aligned virtual timestamp in milliseconds.
///
/// # Errors
///
/// Returns [`crate::error::AppError`] when anchor or virtual time persistence fails.
pub async fn sync_and_persist_virtual_time(
    db: &DbManager,
    role: &Role,
    immersive: bool,
) -> Result<i64> {
    let role_id = role.id.as_str();
    let time_cfg = &role.time_config;
    let real_now = round_to_minute_ms(Utc::now().timestamp_millis());
    if !immersive {
        return Ok(real_now);
    }
    let ratio = time_cfg.effective_ratio();
    let (anchor_real, anchor_virtual, stored_virtual) =
        db.get_virtual_time_anchors(role_id).await?;
    let (anchor_real, anchor_virtual) = if anchor_real <= 0 {
        let init_virtual = if stored_virtual > 0 {
            stored_virtual
        } else if let Some(ref sched) = role.life_schedule {
            virtual_start_ms_from_schedule(real_now, sched).unwrap_or(real_now)
        } else {
            real_now
        };
        db.set_virtual_time_anchors(role_id, real_now, init_virtual)
            .await?;
        (real_now, init_virtual)
    } else {
        (anchor_real, anchor_virtual)
    };
    let virtual_now =
        compute_virtual_now_ms(anchor_real, anchor_virtual, real_now, ratio);
    db.set_virtual_time_ms(role_id, virtual_now).await?;
    Ok(virtual_now)
}

/// Reset anchors after a manual jump; optionally apply forgetting over the jump interval.
///
/// # Errors
///
/// Returns [`crate::error::AppError`] when anchor persistence or jump-interval personality decay fails.
pub async fn apply_virtual_time_jump(
    state: &AppState,
    role: &Role,
    role_id: &str,
    base_ms: i64,
    target_ms: i64,
) -> Result<i64> {
    let ms = round_to_minute_ms(target_ms);
    let real_now = round_to_minute_ms(Utc::now().timestamp_millis());
    state
        .db_manager
        .set_virtual_time_anchors(role_id, real_now, ms)
        .await?;
    state.db_manager.set_virtual_time_ms(role_id, ms).await?;

    let cfg = &role.time_config;
    if cfg.decay_on_jump {
        let jump_days = virtual_days_between_ms(base_ms, ms);
        if jump_days > 0.0 {
            apply_personality_time_decay(state, role, role_id, jump_days).await?;
        }
    }
    Ok(ms)
}

/// From last interaction (real `last_interaction_at`), convert elapsed time to virtual days at flow rate and decay personality delta.
///
/// # Errors
///
/// Returns [`crate::error::AppError`] when reading interaction time or persisting personality delta fails.
pub async fn apply_idle_personality_decay(
    state: &AppState,
    role: &Role,
    role_id: &str,
) -> Result<()> {
    let Some(last) = state.db_manager.get_last_interaction_at(role_id).await? else {
        return Ok(());
    };
    let last_ms = last.timestamp_millis();
    let real_now = Utc::now().timestamp_millis();
    let virtual_days =
        virtual_days_from_real_elapsed_ms(real_now - last_ms, role.time_config.effective_ratio());
    if virtual_days > 0.0 {
        apply_personality_time_decay(state, role, role_id, virtual_days).await?;
    }
    Ok(())
}

async fn apply_personality_time_decay(
    state: &AppState,
    role: &Role,
    role_id: &str,
    virtual_days: f64,
) -> Result<()> {
    if role.evolution_config.personality_source == PersonalitySource::Profile {
        return Ok(());
    }
    let core = PersonalityVector::from(&role.default_personality);
    let (_, delta_s) = state
        .db_manager
        .get_core_delta_personality_json(role_id)
        .await?;
    let mut delta = delta_s
        .and_then(|s| PersonalityVector::from_json_vec(&s).ok())
        .unwrap_or_else(PersonalityVector::zero);
    let before = delta.clone();
    delta = decay_personality_delta(delta, virtual_days, role.time_config.decay_per_day);
    if delta == before {
        return Ok(());
    }
    state
        .db_manager
        .set_core_delta_personality_json(role_id, &core.to_json_vec(), &delta.to_json_vec())
        .await?;
    state.invalidate_personality_cache_for_role(role_id);
    Ok(())
}

