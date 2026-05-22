#![no_main]

use libfuzzer_sys::fuzz_target;
use oclive_validation::validate_blueprint_v2_json;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = validate_blueprint_v2_json(s);
    }
});
