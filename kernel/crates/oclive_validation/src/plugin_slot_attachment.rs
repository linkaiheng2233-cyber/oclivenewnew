//! Directory plugin `manifest.json` `slot_attachment` declarations and blueprint auto-attachment.

use std::collections::BTreeMap;

use serde::Deserialize;
use serde_json::Value;

use crate::blueprint_v2::{default_slot_key_for_module, SlotRegistryEntry};

const SLOT_ATTACHMENT_TYPES: &[&str] = &[
    "memory",
    "emotion",
    "event",
    "prompt",
    "llm",
    "agent",
    "complex_emotion",
];

/// One slot attachment declaration (`manifest.slot_attachment` object or array element).
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct SlotAttachmentDecl {
    #[serde(rename = "type")]
    pub slot_type: String,
    #[serde(default)]
    pub backend: Option<String>,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub position: Option<i64>,
}

/// Parses `slot_attachment` from manifest JSON (object or array); returns empty Vec when absent.
///
/// # Errors
///
/// Returns an error when JSON is invalid or `slot_attachment` shape/fields fail validation.
pub fn parse_slot_attachments_from_manifest_json(
    raw: &str,
) -> Result<Vec<SlotAttachmentDecl>, String> {
    let v: Value = serde_json::from_str(raw).map_err(|e| format!("manifest JSON: {e}"))?;
    let Some(att) = v.get("slot_attachment") else {
        return Ok(vec![]);
    };
    parse_slot_attachment_value(att)
}

fn parse_slot_attachment_value(v: &Value) -> Result<Vec<SlotAttachmentDecl>, String> {
    match v {
        Value::Array(arr) => {
            let mut out = Vec::with_capacity(arr.len());
            for (i, item) in arr.iter().enumerate() {
                let decl: SlotAttachmentDecl = serde_json::from_value(item.clone())
                    .map_err(|e| format!("slot_attachment[{i}]: {e}"))?;
                validate_slot_attachment_decl(&decl)
                    .map_err(|e| format!("slot_attachment[{i}]: {e}"))?;
                out.push(decl);
            }
            Ok(out)
        }
        Value::Object(_) => {
            let one: SlotAttachmentDecl =
                serde_json::from_value(v.clone()).map_err(|e| format!("slot_attachment: {e}"))?;
            validate_slot_attachment_decl(&one)?;
            Ok(vec![one])
        }
        _ => Err("slot_attachment must be an object or array".into()),
    }
}

/// Validates one `slot_attachment` declaration.
///
/// # Errors
///
/// Returns an error when `type` or `backend` is missing or not in the allowed set.
pub fn validate_slot_attachment_decl(decl: &SlotAttachmentDecl) -> Result<(), String> {
    let t = decl.slot_type.trim();
    if t.is_empty() {
        return Err("type is required".into());
    }
    if !SLOT_ATTACHMENT_TYPES.contains(&t) {
        return Err(format!(
            "unsupported type {t:?}; allowed: {}",
            SLOT_ATTACHMENT_TYPES.join(", ")
        ));
    }
    if let Some(ref b) = decl.backend {
        let b = b.trim();
        if !b.is_empty() && !is_allowed_backend(b) {
            return Err(format!(
                "unsupported backend {b:?}; allowed: builtin, remote, directory, ollama"
            ));
        }
    }
    Ok(())
}

fn is_allowed_backend(b: &str) -> bool {
    matches!(
        b,
        "builtin" | "remote" | "directory" | "ollama" | "openai_compatible"
    )
}

/// Merges plugin `slot_attachment` entries into role pack `slot_registry` (match by `type` or create instance key).
///
/// Returns human-readable notes (updated slot descriptions).
pub fn apply_slot_attachments_to_registry(
    registry: &mut BTreeMap<String, SlotRegistryEntry>,
    plugin_id: &str,
    attachments: &[SlotAttachmentDecl],
) -> Vec<String> {
    let pid = plugin_id.trim();
    let mut notes = Vec::new();
    for att in attachments {
        let slot_type = att.slot_type.trim();
        let backend = att
            .backend
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .unwrap_or("directory")
            .to_string();
        let label = att
            .label
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .unwrap_or(pid)
            .to_string();
        let position = att.position.unwrap_or(0);

        let key = find_or_create_slot_key(registry, slot_type, position);
        let entry = registry
            .entry(key.clone())
            .or_insert_with(|| SlotRegistryEntry {
                slot_type: slot_type.to_string(),
                label: label.clone(),
                backend: backend.clone(),
                position,
                plugin: None,
                plugins: None,
                model: None,
                url: None,
                local_memory_provider_id: None,
                zone: None,
                policy: None,
            });
        entry.slot_type = slot_type.to_string();
        entry.label = label.clone();
        entry.backend = backend.clone();
        entry.position = position;
        entry.plugin = Some(pid.to_string());
        notes.push(format!("{key} ({slot_type}) → directory plugin {pid}"));
    }
    notes
}

fn find_or_create_slot_key(
    registry: &BTreeMap<String, SlotRegistryEntry>,
    slot_type: &str,
    position: i64,
) -> String {
    if let Some(def) = default_slot_key_for_module(slot_type) {
        if let Some(e) = registry.get(def) {
            if e.position == position || position == 0 {
                return def.to_string();
            }
        } else {
            return def.to_string();
        }
    }
    let mut idx = 2u32;
    loop {
        let candidate = if idx == 2 {
            format!("{slot_type}_2")
        } else {
            format!("{slot_type}_{idx}")
        };
        if !registry.contains_key(&candidate) {
            return candidate;
        }
        idx += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_slot_attachment_object_and_array() {
        let obj = r#"{"slot_attachment":{"type":"llm","backend":"directory","label":"X"}}"#;
        let v = parse_slot_attachments_from_manifest_json(obj).unwrap();
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].slot_type, "llm");

        let arr = r#"{"slot_attachment":[{"type":"memory"},{"type":"llm"}]}"#;
        let v2 = parse_slot_attachments_from_manifest_json(arr).unwrap();
        assert_eq!(v2.len(), 2);
    }

    #[test]
    fn apply_attachment_updates_llm_slot() {
        let mut reg = BTreeMap::new();
        reg.insert(
            "llm".into(),
            SlotRegistryEntry {
                slot_type: "llm".into(),
                label: "LLM".into(),
                backend: "ollama".into(),
                position: 6,
                plugin: None,
                plugins: None,
                model: None,
                url: None,
                local_memory_provider_id: None,
                zone: None,
                policy: None,
            },
        );
        let att = vec![SlotAttachmentDecl {
            slot_type: "llm".into(),
            backend: Some("directory".into()),
            label: Some("llama".into()),
            position: Some(6),
        }];
        let notes = apply_slot_attachments_to_registry(&mut reg, "com.test.llm", &att);
        assert!(!notes.is_empty());
        assert_eq!(
            reg.get("llm").unwrap().plugin.as_deref(),
            Some("com.test.llm")
        );
        assert_eq!(reg.get("llm").unwrap().backend, "directory");
    }
}
