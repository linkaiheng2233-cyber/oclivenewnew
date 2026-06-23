#![no_main]

use libfuzzer_sys::fuzz_target;
use oclive_validation::validate_kernel_error_body;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(s) {
            if let Some(err) = v.get("error") {
                let _ = validate_kernel_error_body(err);
            }
        }
    }
});
