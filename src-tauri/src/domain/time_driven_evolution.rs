//! 虚拟时间驱动的阶段性性格演化（持久化，与单轮事件波动互补）。

use crate::domain::personality_engine::PersonalityEngine;
use crate::domain::profile_personality::effective_vector_from_profile;
use crate::domain::relation_estrangement::append_mutable_profile_section;
use crate::error::Result;
use crate::models::{PersonalitySource, PersonalityVector, Role};
use crate::state::AppState;
/// `pre_llm` 在事件/情绪演化之前应用时间沉淀后的结果。
#[derive(Debug, Default)]
pub struct TimeEvolutionApply {
    pub personality: Option<PersonalityVector>,
    pub mutable_for_prompt: Option<String>,
    pub evolved: bool,
}

/// 计算自 `last_ms` 起可执行的完整演化阶段数（每阶段 `interval_hours` 虚拟小时）。
#[must_use]
pub fn count_evolution_stages(
    virtual_now_ms: i64,
    last_ms: i64,
    interval_hours: f64,
) -> u32 {
    let interval_h = interval_hours.max(0.25);
    let interval_ms = (interval_h * 3_600_000.0).round() as i64;
    if interval_ms <= 0 || virtual_now_ms <= last_ms {
        return 0;
    }
    let elapsed_ms = virtual_now_ms - last_ms;
    ((elapsed_ms / interval_ms) as u32).min(24)
}

#[must_use]
pub fn time_evolution_profile_line(lapse_hours: f64) -> String {
    let h = lapse_hours.clamp(0.25, 48.0);
    if h >= 12.0 {
        format!(
            "约 {h:.0} 虚拟小时未见面对话后，气质明显沉淀：更沉稳、更少外露情绪，习惯以简短回应代替长篇倾诉。"
        )
    } else {
        format!(
            "约 {h:.1} 虚拟小时的流逝里，相处气质略有沉淀：话稍少、心结稍淡，态度更稳一些。"
        )
    }
}

/// 沉浸模式下：虚拟时间每走过一个配置间隔，持久化更新性格一次。
///
/// # Errors
///
/// 数据库读写失败时返回 [`crate::error::AppError`].
pub async fn check_and_evolve_by_time(
    state: &AppState,
    role: &Role,
    role_id: &str,
    virtual_now_ms: i64,
    immersive: bool,
) -> Result<TimeEvolutionApply> {
    if !immersive || virtual_now_ms <= 0 {
        return Ok(TimeEvolutionApply::default());
    }

    let db = state.db_manager.as_ref();
    let interval_h = role
        .pack_evolution_config
        .personality_evolution_interval_hours
        .max(0.25);
    let interval_ms = (interval_h * 3_600_000.0).round() as i64;

    let mut last_ms = db.get_last_personality_evolution_virtual_ms(role_id).await?;
    if last_ms <= 0 {
        db.set_last_personality_evolution_virtual_ms(role_id, virtual_now_ms)
            .await?;
        return Ok(TimeEvolutionApply::default());
    }

    let stages = count_evolution_stages(virtual_now_ms, last_ms, interval_h);
    if stages == 0 {
        return Ok(TimeEvolutionApply::default());
    }

    let mut out = TimeEvolutionApply {
        evolved: true,
        ..Default::default()
    };

    for _ in 0..stages {
        let step_hours = interval_h;
        if role.evolution_config.personality_source == PersonalitySource::Profile {
            let existing = db.get_mutable_personality(role_id).await?;
            let line = time_evolution_profile_line(step_hours);
            let next = append_mutable_profile_section(&existing, "时间演化", &line);
            db.set_mutable_personality(role_id, &next).await?;
            out.mutable_for_prompt = Some(next);
            out.personality = Some(effective_vector_from_profile(role, out.mutable_for_prompt.as_deref().unwrap_or("")));
        } else {
            let core_v = PersonalityVector::from(&role.default_personality);
            let (_, delta_s) = db.get_core_delta_personality_json(role_id).await?;
            let mut delta = delta_s
                .and_then(|s| PersonalityVector::from_json_vec(&s).ok())
                .unwrap_or_else(PersonalityVector::zero);
            delta = PersonalityEngine::evolve_by_time_lapse(delta, step_hours);
            db.set_core_delta_personality_json(
                role_id,
                &core_v.to_json_vec(),
                &delta.to_json_vec(),
            )
            .await?;
            out.personality = Some(PersonalityVector::effective_from_core_delta(
                &role.default_personality,
                &delta,
                &role.evolution_bounds,
            ));
        }
        last_ms = last_ms.saturating_add(interval_ms);
    }

    db.set_last_personality_evolution_virtual_ms(role_id, last_ms.min(virtual_now_ms))
        .await?;
    state.invalidate_personality_cache_for_role(role_id);

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stages_every_six_virtual_hours() {
        let last = 0_i64;
        let six_h = 6 * 3_600_000_i64;
        assert_eq!(count_evolution_stages(six_h, last, 6.0), 1);
        assert_eq!(count_evolution_stages(six_h * 3 - 1, last, 6.0), 2);
        assert_eq!(count_evolution_stages(six_h * 3, last, 6.0), 3);
    }

    #[test]
    fn no_stage_before_interval() {
        let last = 1_000_000_i64;
        let now = last + 3 * 3_600_000_i64;
        assert_eq!(count_evolution_stages(now, last, 6.0), 0);
    }
}
