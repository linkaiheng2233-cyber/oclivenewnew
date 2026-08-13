//! Multi-turn relation transition frames (SessionCache + optional Profile archive).

use crate::domain::relation_estrangement::{
    replace_mutable_profile_section, strip_mutable_profile_section,
};
use crate::error::Result;
use crate::models::{PersonalitySource, Role};
use crate::state::SessionCache;
use oclive_kernel_contracts::MutablePersonalityStore;
use oclive_kernel_runtime::domain::prompt_builder::{
    relation_rank, relation_transition_duration, relation_transition_hint,
};

const PROFILE_SECTION_TITLE: &str = "关系过渡";

/// Outcome of consuming one transition turn at pre-LLM.
pub struct RelationTransitionConsume {
    pub hint: String,
    pub profile_strip_needed: bool,
}

/// Decrement transition counter; strip Profile archive section when expired.
///
/// # Errors
///
/// Returns [`crate::error::AppError`] on mutable profile DB read/write failure.
pub async fn consume_relation_transition_at_turn_start(
    cache: &SessionCache,
    db: &(dyn MutablePersonalityStore + Send + Sync),
    role: &Role,
    srid: &str,
) -> Result<RelationTransitionConsume> {
    let Some(consumed) = cache.consume_relation_transition(srid) else {
        return Ok(RelationTransitionConsume {
            hint: String::new(),
            profile_strip_needed: false,
        });
    };
    if consumed.expired && role.evolution_config.personality_source == PersonalitySource::Profile {
        let existing = db.get_mutable_personality(srid).await?;
        let stripped = strip_mutable_profile_section(&existing, PROFILE_SECTION_TITLE);
        if stripped != existing {
            db.set_mutable_personality(srid, &stripped).await?;
        }
    }
    Ok(RelationTransitionConsume {
        hint: consumed.hint,
        profile_strip_needed: consumed.expired,
    })
}

/// Start or refresh a multi-turn transition after relation/favor movement.
///
/// # Errors
///
/// Returns [`crate::error::AppError`] on mutable profile DB read/write failure.
pub async fn maybe_start_relation_transition(
    cache: &SessionCache,
    db: &(dyn MutablePersonalityStore + Send + Sync),
    role: &Role,
    srid: &str,
    relation_before: &str,
    relation_after: &str,
    favor_delta: f64,
) -> Result<()> {
    let before_rank = relation_rank(relation_before);
    let after_rank = relation_rank(relation_after);
    if before_rank == after_rank && favor_delta.abs() < 3.0 {
        return Ok(());
    }
    let rank_delta = after_rank - before_rank;
    let hint = relation_transition_hint(relation_before, relation_after, favor_delta);
    if hint.is_empty() {
        return Ok(());
    }
    let remaining = relation_transition_duration(rank_delta, favor_delta);
    cache.set_relation_transition(srid, hint.clone(), remaining);

    if role.evolution_config.personality_source == PersonalitySource::Profile {
        let existing = db.get_mutable_personality(srid).await?;
        let next = replace_mutable_profile_section(&existing, PROFILE_SECTION_TITLE, &hint);
        db.set_mutable_personality(srid, &next).await?;
    }
    Ok(())
}
