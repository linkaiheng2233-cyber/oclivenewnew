#![no_main]

use libfuzzer_sys::fuzz_target;
use oclive_validation::{
    validate_blueprint_v2_json, validate_manifest_top_level_keys,
    validate_settings_top_level_keys,
};
use serde_json::Map;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = validate_blueprint_v2_json(s);
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(s) {
            if let Some(map) = v.as_object() {
                let m: Map<String, serde_json::Value> = map.clone();
                let _ = validate_manifest_top_level_keys(&m);
                let _ = validate_settings_top_level_keys(&m);
            }
        }
    }
});
