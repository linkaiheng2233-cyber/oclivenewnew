#![no_main]

use libfuzzer_sys::fuzz_target;
use oclivenewnew_tauri::infrastructure::function_call_parser::parse_from_llm_response;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = parse_from_llm_response(s);
    }
});
