//! Slot extension envelope: opaque plugin payload keyed by `schema_id`.

use serde::{Deserialize, Serialize};

/// Opaque extension envelope for slot plugin outputs.
///
/// The kernel interprets only core contract fields; `schema_id` + `data` carry
/// plugin-specific state without expanding `PromptInput` or core DTOs per hint.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SlotExtension {
    pub schema_id: String,
    pub data: serde_json::Value,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slot_extension_json_roundtrip() {
        let ext = SlotExtension {
            schema_id: "example.v1".to_string(),
            data: serde_json::json!({
                "valence": 0.42,
                "labels": ["warm", "guarded"]
            }),
        };
        let json = serde_json::to_string(&ext).expect("serialize");
        let back: SlotExtension = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back, ext);
    }

    #[test]
    fn emotion_result_omitted_extension_deserializes() {
        let er: crate::EmotionResult = serde_json::from_str(
            r#"{"joy":0.1,"sadness":0.0,"anger":0.0,"fear":0.0,"surprise":0.0,"disgust":0.0,"neutral":0.9}"#,
        )
        .expect("deserialize legacy json");
        assert_eq!(er.extension, None);
    }
}
