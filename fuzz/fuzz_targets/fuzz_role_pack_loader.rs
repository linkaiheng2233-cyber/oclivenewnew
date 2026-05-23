#![no_main]

use libfuzzer_sys::fuzz_target;
use oclivenewnew_tauri::infrastructure::role_pack::peek_role_pack_manifest;
use std::io::Write;

fuzz_target!(|data: &[u8]| {
    if data.is_empty() {
        return;
    }
    let tmp = std::env::temp_dir().join(format!("oclive_fuzz_role_{:x}.bin", data.len()));
    if std::fs::File::create(&tmp)
        .and_then(|mut f| f.write_all(data))
        .is_ok()
    {
        let _ = peek_role_pack_manifest(&tmp);
        let _ = std::fs::remove_file(&tmp);
    }
});
