//! Long idle periods: favorability estrangement decay and relation state downgrade.

use crate::error::Result;
use crate::infrastructure::db::DbManager;
use crate::models::{PersonalitySource, Role};
use chrono::Utc;
use oclive_kernel_runtime::domain::relation_engine::{RelationEngine, RelationState};
use oclive_kernel_runtime::domain::virtual_time::virtual_days_from_real_elapsed_ms;

/// Append or update a section in the mutable personality profile (plain text, for profile mode).
#[must_use]
pub fn append_mutable_profile_section(existing: &str, title: &str, line: &str) -> String {
    let header = format!("## {title}");
    if existing.contains(&header) {
        if existing.contains(line) {
            return existing.to_string();
        }
        return format!("{existing}\n- {line}");
    }
    if existing.trim().is_empty() {
        format!("{header}\n- {line}")
    } else {
        format!("{existing}\n\n{header}\n- {line}")
    }
}

/// Before a turn starts: decay favorability by virtual days since last interaction; downgrade relation state when needed.
///
/// # Errors
///
/// Returns [`crate::error::AppError`] on database read/write or mutable profile update failure.
pub async fn apply_estrangement_at_turn_start(
    db: &DbManager,
    role: &Role,
    role_id: &str,
    user_relation_key: &str,
    immersive: bool,
) -> Result<()> {
    if !immersive {
        return Ok(());
    }
    let Some(last) = db.get_last_interaction_at(role_id).await? else {
        return Ok(());
    };
    let last_ms = last.timestamp_millis();
    let real_now = Utc::now().timestamp_millis();
    let virtual_days =
        virtual_days_from_real_elapsed_ms(real_now - last_ms, role.time_config.effective_ratio());
    if virtual_days <= 0.0 {
        return Ok(());
    }

    let rel_cfg = &role.pack_relation_config;
    let current_favor = db
        .get_favorability_for_identity(role_id, user_relation_key)
        .await?
        .unwrap_or_else(|| role.initial_favorability_for_relation(user_relation_key));
    let mut favor = RelationEngine::apply_estrangement_favor(
        current_favor,
        virtual_days,
        rel_cfg.decay_halflife_days,
    );
    favor = RelationEngine::apply_interaction_recovery(favor, rel_cfg.interaction_recovery);

    db.set_identity_favorability_value(role_id, user_relation_key, favor)
        .await?;

    if !RelationEngine::is_estranged(favor, rel_cfg.estrangement_threshold) {
        return Ok(());
    }

    let rel_s = db
        .get_relation_state_for_identity(role_id, user_relation_key)
        .await?
        .or(db.get_relation_state(role_id).await?)
        .unwrap_or_else(|| "Stranger".to_string());
    let current = RelationState::parse(rel_s.as_str());
    let downgraded = RelationEngine::estrangement_downgrade(current);
    if downgraded == current {
        return Ok(());
    }
    db.set_identity_relation_state(role_id, user_relation_key, downgraded.as_str())
        .await?;

    if role.evolution_config.personality_source == PersonalitySource::Profile {
        let existing = db.get_mutable_personality(role_id).await?;
        let line = format!(
            "与「{user_relation_key}」许久未联系，关系阶段已疏远为「{}」。",
            downgraded.as_str()
        );
        let next = append_mutable_profile_section(&existing, "社交关系", &line);
        db.set_mutable_personality(role_id, &next).await?;
    }

    Ok(())
}
