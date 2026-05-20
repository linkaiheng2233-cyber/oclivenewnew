#![no_main]

use libfuzzer_sys::fuzz_target;
use oclive_validation::validate_settings_top_level_keys;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(s) {
            if let Some(map) = v.as_object() {
                let _ = validate_settings_top_level_keys(map);
            }
        }
    }
});
