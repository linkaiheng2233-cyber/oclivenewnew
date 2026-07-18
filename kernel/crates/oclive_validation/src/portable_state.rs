//! Cross-distro persona and memory transfer contracts.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const PORTABLE_PERSONA_SCHEMA_VERSION: u32 = 1;
pub const PORTABLE_MEMORY_SCHEMA_VERSION: u32 = 1;
pub const MEMORY_SEED_SCHEMA_VERSION: u32 = 1;

const MAX_DOCUMENT_BYTES: usize = 8 * 1024 * 1024;
const MAX_CORE_PROFILE_CHARS: usize = 64_000;
const MAX_MUTABLE_PROFILE_CHARS: usize = 8_000;
const MAX_MEMORY_ENTRIES: usize = 10_000;
const MAX_MEMORY_CONTENT_CHARS: usize = 4_000;

fn default_importance() -> f64 {
    0.5
}

fn default_weight() -> f64 {
    1.0
}

fn default_mention_count() -> i32 {
    1
}

/// Creator-authored, read-only memory supplied by a role pack.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MemorySeedEntry {
    pub id: String,
    pub content: String,
    #[serde(default = "default_importance")]
    pub importance: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scene_id: Option<String>,
}

/// Optional role-pack `memory_seed.json`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MemorySeedFile {
    pub schema_version: u32,
    #[serde(default)]
    pub memories: Vec<MemorySeedEntry>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extensions: BTreeMap<String, serde_json::Value>,
}

/// Runtime long-term memory entry carried by an `.ocmemory` document.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PortableLongTermMemoryEntry {
    pub content: String,
    #[serde(default = "default_importance")]
    pub importance: f64,
    #[serde(default = "default_weight")]
    pub weight: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub accessed_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scene_id: Option<String>,
    #[serde(default = "default_mention_count")]
    pub mention_count: i32,
}

/// Portable persona document. Core data is an identity guard; runtime import only restores
/// `mutable_profile` and never overwrites the installed role pack.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PortablePersonaFile {
    pub schema_version: u32,
    pub role_id: String,
    pub role_name: String,
    pub role_version: String,
    pub core_profile: String,
    pub default_personality: Vec<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mutable_profile: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exported_at: Option<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extensions: BTreeMap<String, serde_json::Value>,
}

/// Portable memory document. Short-term cache, chat logs and ephemeral situation state are
/// intentionally absent.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PortableMemoryFile {
    pub schema_version: u32,
    pub role_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(default)]
    pub memory_seed: Vec<MemorySeedEntry>,
    #[serde(default)]
    pub long_term: Vec<PortableLongTermMemoryEntry>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exported_at: Option<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extensions: BTreeMap<String, serde_json::Value>,
}

fn validate_seed_entries(entries: &[MemorySeedEntry], errors: &mut Vec<String>) {
    if entries.len() > MAX_MEMORY_ENTRIES {
        errors.push(format!("memory_seed：条目数不得超过 {MAX_MEMORY_ENTRIES}"));
    }
    let mut ids = std::collections::BTreeSet::new();
    for entry in entries {
        let id = entry.id.trim();
        if id.is_empty() {
            errors.push("memory_seed：id 不得为空".into());
        } else if !ids.insert(id) {
            errors.push(format!("memory_seed：重复 id「{id}」"));
        }
        if entry.content.trim().is_empty() {
            errors.push(format!("memory_seed：条目「{id}」content 不得为空"));
        } else if entry.content.chars().count() > MAX_MEMORY_CONTENT_CHARS {
            errors.push(format!(
                "memory_seed：条目「{id}」content 不得超过 {MAX_MEMORY_CONTENT_CHARS} 字符"
            ));
        }
        if !(0.0..=1.0).contains(&entry.importance) {
            errors.push(format!("memory_seed：条目「{id}」importance 须在 0..=1"));
        }
    }
}

/// Parse and validate a role-pack `memory_seed.json` document.
///
/// # Errors
///
/// Returns validation messages when JSON or the seed contract is invalid.
pub fn parse_memory_seed(raw: &str) -> Result<MemorySeedFile, Vec<String>> {
    if raw.len() > MAX_DOCUMENT_BYTES {
        return Err(vec![format!(
            "memory_seed.json 不得超过 {MAX_DOCUMENT_BYTES} bytes"
        )]);
    }
    let file: MemorySeedFile =
        serde_json::from_str(raw).map_err(|e| vec![format!("memory_seed.json 解析失败: {e}")])?;
    let mut errors = Vec::new();
    if file.schema_version != MEMORY_SEED_SCHEMA_VERSION {
        errors.push(format!(
            "memory_seed.json schema_version 须为 {MEMORY_SEED_SCHEMA_VERSION}"
        ));
    }
    validate_seed_entries(&file.memories, &mut errors);
    if errors.is_empty() {
        Ok(file)
    } else {
        Err(errors)
    }
}

/// Parse and validate an `.ocpersona` JSON document.
///
/// # Errors
///
/// Returns validation messages when JSON or the persona contract is invalid.
pub fn parse_portable_persona(raw: &str) -> Result<PortablePersonaFile, Vec<String>> {
    if raw.len() > MAX_DOCUMENT_BYTES {
        return Err(vec![format!(
            "ocpersona 不得超过 {MAX_DOCUMENT_BYTES} bytes"
        )]);
    }
    let file: PortablePersonaFile =
        serde_json::from_str(raw).map_err(|e| vec![format!("ocpersona 解析失败: {e}")])?;
    let mut errors = Vec::new();
    if file.schema_version != PORTABLE_PERSONA_SCHEMA_VERSION {
        errors.push(format!(
            "ocpersona schema_version 须为 {PORTABLE_PERSONA_SCHEMA_VERSION}"
        ));
    }
    if file.role_id.trim().is_empty() {
        errors.push("ocpersona role_id 不得为空".into());
    }
    if file.core_profile.trim().is_empty() {
        errors.push("ocpersona core_profile 不得为空".into());
    } else if file.core_profile.chars().count() > MAX_CORE_PROFILE_CHARS {
        errors.push(format!(
            "ocpersona core_profile 不得超过 {MAX_CORE_PROFILE_CHARS} 字符"
        ));
    }
    if file
        .mutable_profile
        .as_deref()
        .is_some_and(|text| text.chars().count() > MAX_MUTABLE_PROFILE_CHARS)
    {
        errors.push(format!(
            "ocpersona mutable_profile 不得超过 {MAX_MUTABLE_PROFILE_CHARS} 字符"
        ));
    }
    if file.default_personality.len() != 7
        || file
            .default_personality
            .iter()
            .any(|value| !(0.0..=1.0).contains(value))
    {
        errors.push("ocpersona default_personality 须为 7 个 0..=1 数值".into());
    }
    if errors.is_empty() {
        Ok(file)
    } else {
        Err(errors)
    }
}

/// Parse and validate an `.ocmemory` JSON document.
///
/// # Errors
///
/// Returns validation messages when JSON or the memory contract is invalid.
pub fn parse_portable_memory(raw: &str) -> Result<PortableMemoryFile, Vec<String>> {
    if raw.len() > MAX_DOCUMENT_BYTES {
        return Err(vec![format!(
            "ocmemory 不得超过 {MAX_DOCUMENT_BYTES} bytes"
        )]);
    }
    let file: PortableMemoryFile =
        serde_json::from_str(raw).map_err(|e| vec![format!("ocmemory 解析失败: {e}")])?;
    let mut errors = Vec::new();
    if file.schema_version != PORTABLE_MEMORY_SCHEMA_VERSION {
        errors.push(format!(
            "ocmemory schema_version 须为 {PORTABLE_MEMORY_SCHEMA_VERSION}"
        ));
    }
    if file.role_id.trim().is_empty() {
        errors.push("ocmemory role_id 不得为空".into());
    }
    validate_seed_entries(&file.memory_seed, &mut errors);
    if file.long_term.len() > MAX_MEMORY_ENTRIES {
        errors.push(format!(
            "ocmemory long_term 条目数不得超过 {MAX_MEMORY_ENTRIES}"
        ));
    }
    for (index, entry) in file.long_term.iter().enumerate() {
        if entry.content.trim().is_empty() {
            errors.push(format!("ocmemory long_term[{index}].content 不得为空"));
        } else if entry.content.chars().count() > MAX_MEMORY_CONTENT_CHARS {
            errors.push(format!(
                "ocmemory long_term[{index}].content 不得超过 {MAX_MEMORY_CONTENT_CHARS} 字符"
            ));
        }
        if !(0.0..=1.0).contains(&entry.importance) {
            errors.push(format!("ocmemory long_term[{index}].importance 须在 0..=1"));
        }
        if !(0.0..=1.0).contains(&entry.weight) {
            errors.push(format!("ocmemory long_term[{index}].weight 须在 0..=1"));
        }
        if entry.mention_count < 1 {
            errors.push(format!(
                "ocmemory long_term[{index}].mention_count 须至少为 1"
            ));
        }
    }
    if errors.is_empty() {
        Ok(file)
    } else {
        Err(errors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_persona_without_seven_dimensions() {
        let raw = r#"{"schema_version":1,"role_id":"r","role_name":"R","role_version":"1","core_profile":"core","default_personality":[0.5]}"#;
        let errors = parse_portable_persona(raw).unwrap_err();
        assert!(errors.iter().any(|e| e.contains("7 个")));
    }

    #[test]
    fn rejects_duplicate_memory_seed_ids() {
        let raw = r#"{"schema_version":1,"memories":[{"id":"met","content":"a"},{"id":"met","content":"b"}]}"#;
        let errors = parse_memory_seed(raw).unwrap_err();
        assert!(errors.iter().any(|e| e.contains("重复 id")));
    }

    #[test]
    fn portable_memory_omits_transient_state_by_contract() {
        let raw = r#"{"schema_version":1,"role_id":"r","memory_seed":[],"long_term":[]}"#;
        let file = parse_portable_memory(raw).unwrap();
        let encoded = serde_json::to_string(&file).unwrap();
        assert!(!encoded.contains("short_term"));
        assert!(!encoded.contains("ephemeral"));
    }
}
