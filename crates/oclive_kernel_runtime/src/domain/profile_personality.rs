//! **Persona profile** (design core) = core personality archive (manifest, human-controlled) + mutable profile (DB, model-maintained under constraints).
//! This module **derives the seven-dimension vector** from both bodies; that vector is a **view** for interpretation only, not the primary personality source.

use crate::models::PersonalityVector;
use crate::models::Role;

pub const SECTION_IMPORTANT_MEMORIES: &str = "重要记忆";
pub const SECTION_TIME_EVOLUTION: &str = "时间演化";
pub const SECTION_SOCIAL_RELATION: &str = "社交关系";
pub const SECTION_MEMORY_SHAPING: &str = "记忆塑造";

const MUTABLE_MAX_CHARS: usize = 8000;
const MIN_PROTECTED_SECTION_BULLETS: usize = 3;
const MEMORY_SNIPPET_CHARS: usize = 96;

const KW_STEP: f64 = 0.028;
const KW_CAP: f64 = 0.11;

fn dim_from_keywords(text: &str, keywords: &[&str]) -> f64 {
    let mut n: usize = 0;
    for k in keywords {
        n = n.saturating_add(text.matches(k).count());
    }
    ((n as f64) * KW_STEP).min(KW_CAP)
}

/// Derives seven-dimension deltas from keywords in core persona + mutable profile, adds to defaults, then clamps.
#[must_use]
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

/// Truncates long-term memory snippets (aligned with turn pipeline).
#[must_use]
pub fn memory_snippet_for_profile(content: &str) -> String {
    content.chars().take(MEMORY_SNIPPET_CHARS).collect()
}

/// Dedup key: summary text with the 「（首次…，强化…次）」 suffix stripped.
#[must_use]
pub fn normalize_summary_key(text: &str) -> String {
    let t = text.trim();
    let t = t.strip_prefix("- ").unwrap_or(t);
    let key = if let Some(i) = t.find('（') {
        t[..i].trim()
    } else {
        t.trim()
    };
    key.to_string()
}

/// Single-line format for an 「重要记忆」 entry.
#[must_use]
pub fn format_important_memory_line(summary: &str, first_date: &str, mention_count: i32) -> String {
    format!(
        "- {summary}（首次{first_date}，强化{mention_count}次）"
    )
}

/// Parses an 「重要记忆」 bullet; returns (summary, first date, reinforcement count).
#[must_use]
pub fn parse_important_memory_bullet(line: &str) -> Option<(String, Option<String>, Option<i32>)> {
    let rest = line.trim().strip_prefix("- ")?;
    let Some(summary_end) = rest.find('（') else {
        let summary = rest.trim();
        return (!summary.is_empty()).then(|| (summary.to_string(), None, None));
    };
    let summary = rest[..summary_end].trim().to_string();
    if summary.is_empty() {
        return None;
    }
    let tail = &rest[summary_end..];
    let first_date = tail
        .strip_prefix("（首次")
        .and_then(|t| t.split('，').next())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string);
    let mention_count = tail.split('，').find_map(|p| {
        let p = p.trim();
        let p = p.strip_prefix("强化")?;
        let digits: String = p.chars().take_while(|c| c.is_ascii_digit()).collect();
        digits.parse::<i32>().ok()
    });
    Some((summary, first_date, mention_count))
}

fn split_mutable_sections(text: &str) -> (String, Vec<(String, String)>) {
    let text = text.trim();
    if text.is_empty() {
        return (String::new(), Vec::new());
    }

    let mut preamble = String::new();
    let mut sections = Vec::new();
    let mut current_title: Option<String> = None;
    let mut current_lines: Vec<String> = Vec::new();

    for line in text.lines() {
        if let Some(title) = line.strip_prefix("## ") {
            if let Some(t) = current_title.take() {
                sections.push((t, current_lines.join("\n")));
                current_lines.clear();
            } else if !current_lines.is_empty() {
                preamble = current_lines.join("\n");
                current_lines.clear();
            }
            current_title = Some(title.trim().to_string());
        } else {
            current_lines.push(line.to_string());
        }
    }

    if let Some(t) = current_title {
        sections.push((t, current_lines.join("\n")));
    } else if !current_lines.is_empty() {
        preamble = current_lines.join("\n");
    }

    (preamble, sections)
}

fn join_mutable_sections(preamble: &str, sections: &[(String, String)]) -> String {
    let mut parts: Vec<String> = Vec::new();
    let preamble = preamble.trim();
    if !preamble.is_empty() {
        parts.push(preamble.to_string());
    }
    for (title, body) in sections {
        let mut block = format!("## {title}");
        let body = body.trim();
        if !body.is_empty() {
            block.push('\n');
            block.push_str(body);
        }
        parts.push(block);
    }
    parts.join("\n\n")
}

fn joined_char_count(preamble: &str, sections: &[(String, String)]) -> usize {
    join_mutable_sections(preamble, sections).chars().count()
}

/// Writes reinforced memory into 「## 重要记忆」; dedupes by summary and updates reinforcement count.
#[must_use]
pub fn upsert_important_memory_section(
    existing: &str,
    summary: &str,
    first_date: &str,
    mention_count: i32,
) -> String {
    let summary = summary.trim();
    if summary.is_empty() {
        return existing.to_string();
    }

    let norm_key = normalize_summary_key(summary);
    let new_line = format_important_memory_line(summary, first_date, mention_count);

    let (preamble, mut sections) = split_mutable_sections(existing);
    if let Some(idx) = sections
        .iter()
        .position(|(title, _)| title == SECTION_IMPORTANT_MEMORIES)
    {
        let body = &mut sections[idx].1;
        let mut found = false;
        let mut new_body_lines: Vec<String> = Vec::new();
        for line in body.lines() {
            if line.trim().starts_with("- ") {
                if let Some((s, fd, _)) = parse_important_memory_bullet(line) {
                    if normalize_summary_key(&s) == norm_key {
                        let keep_date = fd.as_deref().unwrap_or(first_date);
                        new_body_lines.push(format_important_memory_line(
                            &s,
                            keep_date,
                            mention_count,
                        ));
                        found = true;
                        continue;
                    }
                } else if normalize_summary_key(line) == norm_key {
                    new_body_lines.push(new_line.clone());
                    found = true;
                    continue;
                }
            }
            new_body_lines.push(line.to_string());
        }
        if !found {
            if !new_body_lines.is_empty()
                && !new_body_lines.last().is_some_and(|l| l.trim().is_empty())
            {
                new_body_lines.push(String::new());
            }
            new_body_lines.push(new_line);
        }
        sections[idx].1 = new_body_lines.join("\n");
    } else {
        sections.push((SECTION_IMPORTANT_MEMORIES.to_string(), new_line));
    }

    join_mutable_sections(&preamble, &sections)
}

fn count_bullet_lines(body: &str) -> usize {
    body.lines()
        .filter(|l| l.trim().starts_with("- "))
        .count()
}

fn remove_first_bullet_line(body: &str) -> Option<String> {
    let mut removed = false;
    let kept: Vec<&str> = body
        .lines()
        .filter(|line| {
            if !removed && line.trim().starts_with("- ") {
                removed = true;
                return false;
            }
            true
        })
        .collect();
    removed.then(|| kept.join("\n"))
}

fn trim_oldest_bullet_in_section(
    sections: &mut [(String, String)],
    title: &str,
    keep_min: usize,
) -> bool {
    let Some((_, body)) = sections.iter_mut().find(|(t, _)| t == title) else {
        return false;
    };
    if count_bullet_lines(body) <= keep_min {
        return false;
    }
    if let Some(next) = remove_first_bullet_line(body) {
        *body = next;
        return true;
    }
    false
}

fn trim_oldest_bullet_in_unprotected_sections(sections: &mut [(String, String)]) -> bool {
    const UNPROTECTED: &[&str] = &[SECTION_MEMORY_SHAPING, SECTION_SOCIAL_RELATION];
    for title in UNPROTECTED {
        if trim_oldest_bullet_in_section(sections, title, 0) {
            return true;
        }
    }
    false
}

fn trim_last_unstructured_char(preamble: &mut String) -> bool {
    if preamble.trim().is_empty() {
        return false;
    }
    if preamble.ends_with('\n') {
        preamble.pop();
        return true;
    }
    let mut chars: Vec<char> = preamble.chars().collect();
    if chars.pop().is_some() {
        *preamble = chars.into_iter().collect();
        return true;
    }
    false
}

fn trim_preamble_from_start(preamble: &mut String) -> bool {
    if preamble.trim().is_empty() {
        return false;
    }
    if let Some((_first, rest)) = preamble.split_once('\n') {
        *preamble = rest.to_string();
        return true;
    }
    preamble.clear();
    true
}

/// When over limit, trims unstructured tail first and protects sections like 「重要记忆」/「时间演化」 (keeps at least the 3 newest bullets each).
#[must_use]
pub fn trim_mutable_storage(s: &str) -> String {
    if s.chars().count() <= MUTABLE_MAX_CHARS {
        return s.to_string();
    }

    let (mut preamble, mut sections) = split_mutable_sections(s);
    let mut guard = 0usize;
    while joined_char_count(&preamble, &sections) > MUTABLE_MAX_CHARS && guard < 50_000 {
        guard += 1;
        if trim_last_unstructured_char(&mut preamble) {
            continue;
        }
        if trim_oldest_bullet_in_unprotected_sections(&mut sections) {
            continue;
        }
        if trim_oldest_bullet_in_section(
            &mut sections,
            SECTION_TIME_EVOLUTION,
            MIN_PROTECTED_SECTION_BULLETS,
        ) {
            continue;
        }
        if trim_oldest_bullet_in_section(
            &mut sections,
            SECTION_IMPORTANT_MEMORIES,
            MIN_PROTECTED_SECTION_BULLETS,
        ) {
            continue;
        }
        if trim_preamble_from_start(&mut preamble) {
            continue;
        }
        if trim_oldest_bullet_in_section(&mut sections, SECTION_TIME_EVOLUTION, 0) {
            continue;
        }
        if trim_oldest_bullet_in_section(&mut sections, SECTION_IMPORTANT_MEMORIES, 0) {
            continue;
        }
        break;
    }

    join_mutable_sections(&preamble, &sections)
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
            plugin_backends: std::sync::Arc::new(Default::default()),
            slot_registry: None,
            slot_groups: None,
            ui_config: crate::models::UiConfig::default(),
            knowledge_index: None,
            author_pack: None,
            reply_quality_anchor: None,
            time_config: Default::default(),
            pack_memory_config: Default::default(),
            pack_relation_config: Default::default(),
            pack_evolution_config: Default::default(),
            pack_chat_storage_config: Default::default(),
            runtime_config: None,
            pipeline_experimental: None,
            scene_ids: std::sync::Arc::from(Vec::<String>::new()),
            scene_config_cache: std::sync::Arc::new(parking_lot::RwLock::new(
                std::collections::HashMap::new(),
            )),
            scene_text_cache: std::sync::Arc::new(parking_lot::RwLock::new(
                std::collections::HashMap::new(),
            )),
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

    #[test]
    fn upsert_creates_section_and_dedupes_by_summary() {
        let a = upsert_important_memory_section("", "用户喜欢猫", "2026-05-01", 3);
        assert!(a.contains("## 重要记忆"));
        assert!(a.contains("用户喜欢猫"));
        assert!(a.contains("强化3次"));

        let b = upsert_important_memory_section(&a, "用户喜欢猫", "2026-05-20", 5);
        assert_eq!(b.matches("用户喜欢猫").count(), 1);
        assert!(b.contains("强化5次"));
        assert!(b.contains("首次2026-05-01"));
    }

    #[test]
    fn trim_preserves_protected_sections_over_freeform() {
        let mut body = String::from("## 重要记忆\n");
        for i in 0..5 {
            body.push_str(&format!("- 记忆条目{i}（首次2026-01-01，强化{i}次）\n"));
        }
        body.push_str("\n## 时间演化\n");
        for i in 0..5 {
            body.push_str(&format!("- 时间线{i}\n"));
        }
        let filler = "自由叙述。".repeat(1200);
        let doc = format!("{filler}\n\n{body}");
        let trimmed = trim_mutable_storage(&doc);
        assert!(trimmed.contains("## 重要记忆"));
        assert!(trimmed.contains("## 时间演化"));
        assert!(trimmed.chars().count() <= MUTABLE_MAX_CHARS);
        assert!(count_bullet_lines(
            trimmed
                .split("## 重要记忆")
                .nth(1)
                .unwrap_or("")
        ) >= MIN_PROTECTED_SECTION_BULLETS);
    }

    #[test]
    fn normalize_strips_meta_suffix() {
        let key = normalize_summary_key("- 用户怕打雷（首次2026-01-01，强化2次）");
        assert_eq!(key, "用户怕打雷");
    }
}
