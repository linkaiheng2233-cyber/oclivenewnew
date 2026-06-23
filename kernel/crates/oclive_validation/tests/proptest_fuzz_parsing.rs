//! AB5: proptest fuzz inputs (manifest / settings / OOCP-shaped JSON); local smoke equivalent to `cargo fuzz`.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use oclive_validation::{
    validate_jsonrpc_error_response, validate_kernel_error_body, validate_manifest_top_level_keys,
    validate_settings_top_level_keys,
};
use proptest::prelude::*;
use serde_json::{json, Value};

proptest! {
    #![proptest_config(ProptestConfig::with_cases(2048))]

    #[test]
    fn fuzz_manifest_json_never_panics(bytes in prop::collection::vec(any::<u8>(), 0..4096)) {
        if let Ok(s) = std::str::from_utf8(&bytes) {
            if let Ok(v) = serde_json::from_str::<Value>(s) {
                if let Some(map) = v.as_object() {
                    let _ = validate_manifest_top_level_keys(map);
                }
            }
        }
    }

    #[test]
    fn fuzz_settings_keys_never_panics(bytes in prop::collection::vec(any::<u8>(), 0..4096)) {
        if let Ok(s) = std::str::from_utf8(&bytes) {
            if let Ok(v) = serde_json::from_str::<Value>(s) {
                if let Some(map) = v.as_object() {
                    let _ = validate_settings_top_level_keys(map);
                }
            }
        }
    }

    #[test]
    fn fuzz_oocp_chat_shape_never_panics(bytes in prop::collection::vec(any::<u8>(), 0..2048)) {
        if let Ok(s) = std::str::from_utf8(&bytes) {
            if let Ok(v) = serde_json::from_str::<Value>(s) {
                if v.get("error").is_some() {
                    if let Some(err) = v.get("error") {
                        let _ = validate_kernel_error_body(err);
                    }
                }
            }
        }
    }

    #[test]
    fn jsonrpc_int_code_never_validates_as_kernel_body(code in -32020i64..-32000i64) {
        let rpc = json!({
            "jsonrpc": "2.0",
            "error": { "code": code, "message": "x" }
        });
        prop_assert!(validate_jsonrpc_error_response(&rpc).is_ok());
        let kernel = json!({ "code": code, "message": "x" });
        prop_assert!(validate_kernel_error_body(&kernel).is_err());
    }
}
