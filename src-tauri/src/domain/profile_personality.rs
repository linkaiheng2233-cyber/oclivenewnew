//! **人设档案**（设计核心）= 核心性格档案（manifest，人控）+ 可变性格档案（DB，模型在约束下自维护）。
//! 本模块从两份正文**归纳七维向量**，该向量仅作**视图**，辅助理解，不是性格的主数据源。

use crate::models::PersonalityVector;
use crate::models::Role;

const MUTABLE_MAX_CHARS: usize = 8000;
const KW_STEP: f64 = 0.028;
const KW_CAP: f64 = 0.11;

fn dim_from_keywords(text: &str, keywords: &[&str]) -> f64 {
    let mut n: usize = 0;
    for k in keywords {
        n = n.saturating_add(text.matches(k).count());
    }
    ((n as f64) * KW_STEP).min(KW_CAP)
}

/// 由「核心人设 + 可变档案」中的关键词归纳对默认七维的增量，再与默认值相加并限幅。
pub fn effective_vector_from_profile(role: &Role, mutable_personality: &str) -> PersonalityVector {
    let mut combined = String::new();
    combined.push_str(role.core_personality.trim());
    combined.push('\n');
    combined.push_str(mutable_personality.trim());

    let d_stub = dim_from_keywords(&combined, &["倔强", "固执", "认死理", "嘴硬", "不服软"]);
    let d_cling = dim_from_keywords(&combined, &["黏人", "粘人", "撒娇", "依赖", "缠着"]);
    let d_sens = dim_from_keywords(&combined, &["敏感", "细腻", "多心", "在意", "玻璃心"]);
    let d_asrt = dim_from_keywords(&combined, &["强势", "直接", "有主见", "硬气", "不退让"]);
    let d_forg = dim_from_keywords(&combined, &["宽容", "大度", "心软", "好说话", "不计较"]);
    let d_talk = dim_from_keywords(&combined, &["话多", "健谈", "唠叨", "爱分享"]);
    let d_warm = dim_from_keywords(&combined, &["温柔", "体贴", "暖", "哄人", "安抚"]);
    let sub_warm = dim_from_keywords(&combined, &["冷淡", "疏离", "冷漠", "敷衍"]);

    let mut e = PersonalityVector::from(&role.default_personality);
    e.stubbornness += d_stub;
    e.clinginess += d_cling;
    e.sensitivity += d_sens;
    e.assertiveness += d_asrt;
    e.forgiveness += d_forg;
    e.talkativeness += d_talk;
    e.warmth += d_warm;
    e.warmth = (e.warmth - sub_warm * 0.65).max(role.evolution_bounds.warmth.0);
    e.clamp(&role.evolution_bounds);
    e
}

pub(crate) fn trim_mutable_storage(s: &str) -> String {
    let mut t = s.to_string();
    while t.chars().count() > MUTABLE_MAX_CHARS {
        if let Some(pos) = t.find('\n') {
            t = t[pos + 1..].to_string();
        } else {
            let tail: String = t.chars().rev().take(MUTABLE_MAX_CHARS).collect();
            t = tail.chars().rev().collect();
            break;
        }
    }
    t
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::EvolutionBounds;
    use crate::models::PersonalityDefaults;

    fn sample_role() -> Role {
        Role {
            id: "r".into(),
            name: "n".into(),
            description: "".into(),
            version: "1".into(),
            author: "".into(),
            core_personality: "平时温柔体贴。".into(),
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
            user_relations: vec![],
            evolution_config: Default::default(),
            memory_config: None,
            default_relation: "friend".into(),
            ollama_model: None,
            identity_binding: Default::default(),
            life_trajectory: None,
            life_schedule: None,
            remote_presence: None,
            autonomous_scene: None,
            interaction_mode: None,
            min_runtime_version: None,
            dev_only: false,
            plugin_backends: Default::default(),
            ui_config: crate::models::UiConfig::default(),
            knowledge_index: None,
            author_pack: None,
        }
    }

    #[test]
    fn keywords_raise_warmth() {
        let mut r = sample_role();
        r.core_personality = "角色".into();
        let e = effective_vector_from_profile(&r, "越来越温柔体贴会哄人");
        assert!(e.warmth > 0.5, "warmth={}", e.warmth);
    }

    #[test]
    fn trim_mutable_storage_caps_length() {
        let long = "行\n".repeat(6000);
        let t = trim_mutable_storage(&long);
        assert!(t.chars().count() <= MUTABLE_MAX_CHARS);
    }
}
